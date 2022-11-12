use std::future::{ready, Ready};
use std::marker::PhantomData;

use crate::executor::future::MeasuredOutput;

use super::ExecutionStep;

pub struct NoopStep<T>(PhantomData<T>);

impl<T> Clone for NoopStep<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T> NoopStep<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> ExecutionStep for NoopStep<T> {
    type Item = T;

    type Output = Ready<MeasuredOutput<T>>;

    fn execute(&self, events: Vec<crate::Event>, input: Self::Item) -> Self::Output {
        ready((events, Ok(input)))
    }

    fn capacity(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::Event;
    use crate::time::Clock;

    use super::*;

    #[tokio::test]
    async fn creates_step_that_returns_back_passed_data() {
        let step = NoopStep::new();
        let (_, result) = step.execute(Vec::new(), 42).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test(start_paused = true)]
    async fn does_not_add_any_events() {
        let step = NoopStep::new();

        let (events, _) = step
            .execute(
                vec![Event::success("one", Clock::now(), Clock::now())],
                42,
            )
            .await;

        assert_eq!(
            events,
            vec![Event::success("one", Clock::now(), Clock::now())],
        );
    }

    #[test]
    fn noop_step_does_not_require_capacity() {
        let step = NoopStep::<usize>::new();

        assert_eq!(step.capacity(), 0);
    }
}
