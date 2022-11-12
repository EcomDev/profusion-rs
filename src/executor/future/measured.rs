use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::time::{Instant, Clock};

use super::*;

#[derive(Debug)]
enum MeasuredFutureState {
    Ready(&'static str, Vec<Event>),
    Running(&'static str, Vec<Event>, Instant),
    Complete,
}

impl MeasuredFutureState {
    fn start_timer(&mut self) {
        match self {
            Self::Ready(name, events) => {
                *self = Self::Running(name, std::mem::take(events), Clock::now());
            }
            _ => unreachable!(),
        }
    }

    fn finish_timer<T>(&mut self, result: &Result<T>) -> Vec<Event> {
        match self {
            Self::Running(name, events, start) => {
                events.push((*name, *start, Clock::now(), result).into());
                let events = std::mem::take(events);
                *self = Self::Complete;
                events
            }
            _ => unreachable!(),
        }
    }
}

pin_project! {
    /// Wraps async code into measured code block
    #[doc(hidden)]
    pub struct MeasuredFuture<F> {
        #[pin]
        inner: F,
        state: MeasuredFutureState,
    }
}

impl<T, F> MeasuredFuture<F>
    where
        F: Future<Output=Result<T>>,
{
    /// Creates `MeasuredFuture` with provided vector of Events
    pub(crate) fn new(name: &'static str, inner: F, events: Vec<Event>) -> Self {
        Self {
            inner,
            state: MeasuredFutureState::Ready(name, events),
        }
    }
}

impl<T, F> Future for MeasuredFuture<F>
    where
        F: Future<Output=Result<T>>,
{
    type Output = MeasuredOutput<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            let result = match &this.state {
                MeasuredFutureState::Ready(..) => {
                    this.state.start_timer();
                    continue;
                }
                MeasuredFutureState::Running(..) => this.inner.poll(cx),
                MeasuredFutureState::Complete => unreachable!(),
            };

            match result {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => {
                    let events = this.state.finish_timer(&result);
                    return Poll::Ready((events, result));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::time::Duration;
    use crate::time::InstantOffset;

    use super::*;

    impl<T, F> MeasuredFuture<F>
        where
            F: Future<Output=Result<T>>,
    {
        fn empty(name: &'static str, inner: F) -> Self {
            Self::new(name, inner, Vec::new())
        }
    }

    #[test]
    fn transitions_into_running_state_on_starting_timer() {
        let mut state = MeasuredFutureState::Ready("name", Vec::new());

        state.start_timer();

        assert!(matches!(state, MeasuredFutureState::Running(..)));
    }

    #[test]
    fn transitions_into_complete_state_on_finished_timer() {
        let mut state = MeasuredFutureState::Running("name", Vec::new(), Instant::now());

        state.finish_timer(&Ok(1));

        assert!(matches!(state, MeasuredFutureState::Complete));
    }

    #[tokio::test(start_paused = true)]
    async fn records_event_with_running_state_on_finishing() {
        let mut state = MeasuredFutureState::Running("one", Vec::new(), Clock::now());
        let events = state.finish_timer(&Ok(1));

        assert_eq!(
            events,
            vec![Event::success("one", Clock::now(), Clock::now())],
        );
    }

    #[tokio::test]
    async fn executes_underlying_future() {
        let (_, result) =
            MeasuredFuture::empty("fast_future", Box::pin(async { Ok(1 + 1) })).await;

        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn returns_event_based_on_underlying_future_execution() {
        let (events, _) = MeasuredFuture::empty("fast_future", async { Ok(1 + 1) }).await;

        assert_eq!(
            events,
            vec![Event::success(
                "fast_future",
                Clock::now(),
                Clock::now(),
            )],
        );
    }

    #[tokio::test(start_paused = true)]
    async fn appends_to_existings_events_after_execution() {
        let future = || async { Ok(1 + 1) };

        let (events, _) = MeasuredFuture::new(
            "fast_future",
            future(),
            vec![Event::success(
                "another_event",
                Clock::now(),
                Clock::now(),
            )],
        ).await;

        assert_eq!(
            events,
            vec![
                Event::success("another_event", Clock::now(), Clock::now()),
                Event::success("fast_future", Clock::now(), Clock::now()),
            ],
        );
    }

    #[tokio::test]
    async fn propagates_io_error() {
        let (_, result) = MeasuredFuture::empty("fast_future", async {
            Result::<u32>::Err(ErrorKind::InvalidInput.into())
        })
            .await;

        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }

    #[tokio::test(start_paused = true)]
    async fn reports_error_events() {
        let (events, _) = MeasuredFuture::empty("timer_out", async {
            Result::<u32>::Err(ErrorKind::TimedOut.into())
        })
            .await;

        let (events, _) = MeasuredFuture::new(
            "error_out",
            async { Result::<u32>::Err(ErrorKind::InvalidData.into()) },
            events,
        )
            .await;

        assert_eq!(
            events,
            vec![
                Event::timeout("timer_out", Clock::now(), Clock::now()),
                Event::error("error_out", Clock::now(), Clock::now()),
            ],
        );
    }

    #[tokio::test(start_paused = true)]
    async fn measures_time_spend_by_feature() {
        let reference = Clock::now();
        let (events, _) = MeasuredFuture::new(
            "answer_to_everything",
            async {
                tokio::time::advance(Duration::from_secs(7_500_000)).await;
                Ok(42)
            },
            vec![],
        ).await;

        assert_eq!(
            events,
            vec![
                Event::success(
                    "answer_to_everything",
                    reference,
                    reference.with_millis(7_500_000_000)
                )
            ]
        )
    }
}
