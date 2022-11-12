use crate::executor::future::SequenceFuture;

use super::ExecutionStep;

/// Sequence execution step
///
/// Allows to chain multiple execution steps together. It is
/// implicitly created when [`ExecutionStep::step`] is invoked
///
/// # Example
/// ```
/// use profusion::prelude::*;
///
/// let step = SequenceStep::new(
///    SequenceStep::new(
///         ClosureStep::new("first_call", |item: usize| async move { Ok(item) }),
///         ClosureStep::new("second_call", |item: usize| async move { Ok(item) }),
///    ),
///    ClosureStep::new("third_call", |item: usize| async move { Ok(item) }),
/// );
///
/// assert_eq!(step.capacity(), 3);
/// ```
///
#[derive(Clone)]
pub struct SequenceStep<F, S> {
    first: F,
    second: S,
}

impl<T, F, S> SequenceStep<F, S>
    where
        F: ExecutionStep<Item=T>,
        S: ExecutionStep<Item=T>,
{
    pub fn new(first: F, second: S) -> Self {
        Self { first, second }
    }
}

impl<T, F, S> ExecutionStep for SequenceStep<F, S>
    where
        F: ExecutionStep<Item=T>,
        S: ExecutionStep<Item=T>,
        T: Sized,
{
    type Item = T;

    type Output = SequenceFuture<T, F, S>;

    fn execute(&self, events: Vec<crate::Event>, input: Self::Item) -> Self::Output {
        SequenceFuture::new(events, input, self.first.clone(), self.second.clone())
    }

    fn capacity(&self) -> usize {
        self.first.capacity().saturating_add(self.second.capacity())
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Ready, time::Instant};

    use crate::{
        Event,
        executor::{
            future::MeasuredOutput,
            step::{ClosureStep, NoopStep},
        },
    };
    use crate::time::Clock;

    use super::*;

    #[tokio::test]
    async fn returns_result_from_first_step() {
        let step = SequenceStep::new(
            ClosureStep::new("some_call", |item: usize| async move { Ok(item + 3) }),
            NoopStep::new(),
        );

        let (_, result) = step.execute(Vec::new(), 30).await;

        assert_eq!(result.unwrap(), 33);
    }

    #[tokio::test]
    async fn returns_result_from_second_combined_with_second_step() {
        let step = SequenceStep::new(
            ClosureStep::new("some_call", |item: usize| async move { Ok(item + 3) }),
            ClosureStep::new("some_call", |item: usize| async move { Ok(item + 4) }),
        );

        let (_, result) = step.execute(Vec::new(), 3).await;

        assert_eq!(result.unwrap(), 10);
    }

    #[tokio::test(start_paused = true)]
    async fn report_completed_steps_as_events() {
        let step = SequenceStep::new(
            ClosureStep::new("first", |item: usize| async move { Ok(item) }),
            ClosureStep::new("second", |item: usize| async move { Ok(item) }),
        );

        let (events, _) = step.execute(Vec::new(), 1).await;

        assert_eq!(
            events,
            vec![
                Event::success("first", Clock::now(), Clock::now()),
                Event::success("second", Clock::now(), Clock::now()),
            ],
        );
    }

    #[test]
    fn combines_capacity_of_multiple_sequence_steps() {
        let step = SequenceStep::new(
            SequenceStep::new(
                ClosureStep::new("first_call", |item: usize| async move { Ok(item) }),
                ClosureStep::new("second_call", |item: usize| async move { Ok(item) }),
            ),
            ClosureStep::new("third_call", |item: usize| async move { Ok(item) }),
        );

        assert_eq!(step.capacity(), 3);
    }

    #[test]
    fn does_not_exceed_max_usize_for_capacity() {
        #[derive(Clone)]
        struct OverflowStep;

        impl ExecutionStep for OverflowStep {
            type Item = usize;

            type Output = Ready<MeasuredOutput<usize>>;

            fn execute(&self, _events: Vec<Event>, _input: Self::Item) -> Self::Output {
                unreachable!()
            }

            fn capacity(&self) -> usize {
                usize::MAX
            }
        }

        let step = SequenceStep::new(
            OverflowStep,
            ClosureStep::new("third_call", |item: usize| async move { Ok(item) }),
        );

        assert_eq!(step.capacity(), usize::MAX);
    }
}
