use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::MeasuredOutput;
use crate::{
    executor::{
        future::{EitherFuture, EitherFutureKind},
        step::ExecutionStep,
    },
    Event,
};

use pin_project_lite::pin_project;

pin_project! {
    pub struct SequenceFuture <T, F, S>
    where
        F: ExecutionStep<Item=T>,
        S: ExecutionStep<Item=T>
    {
        args: Option<(Vec<Event>, T)>,
        first: F,
        second: S,
        #[pin]
        future: EitherFuture<F::Output, S::Output>
    }
}

impl<T, F, S> SequenceFuture<T, F, S>
where
    F: ExecutionStep<Item = T>,
    S: ExecutionStep<Item = T>,
{
    pub fn new(events: Vec<Event>, value: T, first: F, second: S) -> Self {
        Self {
            first,
            second,
            args: Some((events, value)),
            future: EitherFuture::empty(),
        }
    }
}

impl<T, F, S> Future for SequenceFuture<T, F, S>
where
    F: ExecutionStep<Item = T>,
    S: ExecutionStep<Item = T>,
{
    type Output = MeasuredOutput<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.args.take() {
                Some((events, value)) => {
                    this.future
                        .set(EitherFuture::left(this.first.execute(events, value)));
                }
                None => {}
            };

            let kind = this.future.kind();

            let result = match this.future.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => result,
            };

            match (kind, result) {
                (EitherFutureKind::Left, (events, Ok(value))) => {
                    this.future
                        .set(EitherFuture::right(this.second.execute(events, value)));
                    continue;
                }
                (EitherFutureKind::Right, (events, Ok(value))) => {
                    return Poll::Ready((events, Ok(value)))
                }
                (_, (events, Err(error))) => return Poll::Ready((events, Err(error))),
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::ErrorKind, time::Instant, vec};

    use super::*;
    use crate::executor::step::{ClosureStep, NoopStep};
    use crate::test_util::assert_events;

    #[tokio::test]
    async fn returns_result_from_first_future() {
        let step = SequenceFuture::new(
            Vec::new(),
            40,
            ClosureStep::new("first_step", |value: usize| async move { Ok(value + 2) }),
            NoopStep::new(),
        );

        let (_, result) = step.await;

        assert_eq!(result.unwrap(), 42)
    }

    #[tokio::test]
    async fn returns_result_from_second_future() {
        let step = SequenceFuture::new(
            Vec::new(),
            40,
            NoopStep::new(),
            ClosureStep::new("second_step", |value: usize| async move { Ok(value + 2) }),
        );

        let (_, result) = step.await;

        assert_eq!(result.unwrap(), 42)
    }

    #[tokio::test]
    async fn measures_both_futures_executed() {
        let step = SequenceFuture::new(
            Vec::new(),
            40,
            ClosureStep::new("first_step", |value: usize| async move { Ok(value + 2) }),
            ClosureStep::new("second_step", |value: usize| async move { Ok(value + 2) }),
        );

        let time = Instant::now();
        let (events, _) = step.await;

        assert_events(
            events,
            vec![
                Event::success("first_step", time, time),
                Event::success("second_step", time, time),
            ]
        )
    }

    #[tokio::test]
    async fn aborts_second_future_when_first_errors_out() {
        let step = SequenceFuture::new(
            Vec::new(),
            40,
            ClosureStep::new("first_step", |_| async move {
                Err(ErrorKind::Interrupted.into())
            }),
            ClosureStep::new("second_step", |value: usize| async move { Ok(value + 2) }),
        );

        let time = Instant::now();
        let (events, _) = step.await;

        assert_events(
            events,
            vec![Event::error("first_step", time, time)]
        )
    }

    #[tokio::test]
    async fn return_error_at_the_end_of_the_chain() {
        let step = SequenceFuture::new(
            Vec::new(),
            40,
            ClosureStep::new("first_step", |value: usize| async move { Ok(value + 2) }),
            ClosureStep::new("second_step", |_| async move {
                Err(ErrorKind::ConnectionReset.into())
            }),
        );

        let (_, result) = step.await;

        assert_eq!(result.unwrap_err().kind(), ErrorKind::ConnectionReset)
    }
}
