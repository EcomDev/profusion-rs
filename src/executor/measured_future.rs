use super::*;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use pin_project_lite::pin_project;

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
                *self = Self::Running(name, std::mem::take(events), Instant::now());
            }
            _ => unreachable!(),
        }
    }

    fn finish_timer<T>(&mut self, result: &Result<T>) -> Vec<Event> {
        match self {
            Self::Running(name, events, start) => {
                events.push((*name, *start, Instant::now(), result).into());
                let events = std::mem::take(events);
                *self = Self::Complete;
                events
            }
            _ => unreachable!(),
        }
    }
}

/// Measures execution time and result type of underlying [inner][`std::future::Future`] future.
///
/// Result of the measurement is as an [Event][`crate::report::Event`] appeneded to a vector passed as an argument.
/// ```
/// use profusion::{report::Event, executor::MeasuredFuture};
/// use std::time::Instant;
///
/// #[tokio::main]
/// async fn main() {
///    let (events, _) = MeasuredFuture::with_events(
///        "one_plus_one",
///        Box::pin(async { Ok(1 + 1) }),
///        vec![Event::success("another_event", Instant::now(), Instant::now())]
///    ).await;
///
///    assert_eq!(
///        events,
///        vec![
///            Event::success("another_event", Instant::now(), Instant::now()),
///            Event::success("one_plus_one", Instant::now(), Instant::now())
///       ]
///    );
/// }
/// ```


pin_project! {
    pub struct MeasuredFuture<F> {
        #[pin]
        inner: F,
        state: MeasuredFutureState,
    }
}

impl<T, F> MeasuredFuture<F>
where
    F: Future<Output = Result<T>>
{
    pub fn new(name: &'static str, inner: F) -> Self {
        Self::with_events(name, inner, Vec::new())
    }

    pub fn with_events(name: &'static str, inner: F, events: Vec<Event>) -> Self {
        Self {
            inner: inner,
            state: MeasuredFutureState::Ready(name, events),
        }
    }
}

impl<T, F> Future for MeasuredFuture<F>
where
    F: Future<Output = Result<T>>
{
    type Output = (Vec<Event>, Result<T>);

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

    use super::*;

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

    #[test]
    fn records_event_with_running_state_on_finishing() {
        let mut state = MeasuredFutureState::Running("one", Vec::new(), Instant::now());
        let events = state.finish_timer(&Ok(1));

        assert_eq!(
            events,
            vec![Event::success("one", Instant::now(), Instant::now())]
        );
    }

    #[tokio::test]
    async fn executes_underlying_future() {
        let (_, result) =
            MeasuredFuture::new("fast_future", Box::pin(async { Ok(1 + 1) })).await;

        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn returns_event_based_on_underlying_future_execution() {
        let (events, _) =
            MeasuredFuture::new("fast_future", async { Ok(1 + 1) }).await;

        assert_eq!(
            events,
            vec![Event::success(
                "fast_future",
                Instant::now(),
                Instant::now()
            )]
        );
    }

    #[tokio::test]
    async fn appends_to_existings_events_after_execution() {
        let future = || async { Ok(1 + 1) };
        let (events, _) = MeasuredFuture::with_events(
            "fast_future",
            future(),
            vec![Event::success(
                "another_event",
                Instant::now(),
                Instant::now(),
            )],
        )
        .await;

        assert_eq!(
            events,
            vec![
                Event::success("another_event", Instant::now(), Instant::now()),
                Event::success("fast_future", Instant::now(), Instant::now())
            ]
        );
    }

    #[tokio::test]
    async fn propagates_io_error() {
        let (_, result) = MeasuredFuture::new(
            "fast_future",
            async { Result::<u32>::Err(ErrorKind::InvalidInput.into()) },
        )
        .await;

        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput.into());
    }

    #[tokio::test]
    async fn reports_error_events() {
        let (events, _) = MeasuredFuture::new(
            "timer_out",
            async { Result::<u32>::Err(ErrorKind::TimedOut.into()) },
        )
        .await;

        let (events, _) = MeasuredFuture::with_events(
            "errored_out",
            async { Result::<u32>::Err(ErrorKind::InvalidData.into()) },
            events,
        )
        .await;

        assert_eq!(
            events,
            vec![
                Event::timeout("timer_out", Instant::now(), Instant::now()),
                Event::error("errored_out", Instant::now(), Instant::now()),
            ]
        );
    }
}
