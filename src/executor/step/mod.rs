use super::future::MeasuredOutput;
use crate::report::Event;
use std::{future::Future, io::Result};

mod closure;
mod noop;
mod sequence;

pub use closure::ClosureStep;
pub use noop::NoopStep;
pub use sequence::SequenceStep;

pub trait ExecutionStep: Clone {
    type Item: Sized;
    type Output: Future<Output = MeasuredOutput<Self::Item>>;

    /// Executes a step by creating a future with input as an argument
    fn execute(&self, events: Vec<Event>, input: Self::Item) -> Self::Output;

    /// Expected capacity of the execution step
    fn capacity(&self) -> usize;

    /// Chains with current step as first and passed one as second.
    ///
    /// ```rust
    /// use profusion::prelude::*;
    /// use std::time::Instant;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     use profusion::test_util::assert_events;
    ///     let step = NoopStep::new()
    ///         .step("first", |value| async move { Ok(value + 10) } )        
    ///         .step("second", |value| async move { Ok(value + 10) } )        
    ///         .step("third", |value| async move { Ok(value + 10) } )        
    ///         .step("last", |value| async move { Ok(value + 2) } )        
    ///     ;
    ///     let data: usize = 10;
    ///     let time = Instant::now();
    ///     let (events, result) = step.execute(Vec::with_capacity(step.capacity()), data).await;
    ///
    ///     assert_events(events, vec![
    ///         Event::success("first", time, time),
    ///         Event::success("second", time, time),
    ///         Event::success("third", time, time),
    ///         Event::success("last", time, time),
    ///     ]);
    ///     assert_eq!(result.unwrap(), 42)
    /// }
    /// ```
    fn step<F, Fut>(
        self,
        name: &'static str,
        closure: F,
    ) -> SequenceStep<Self, ClosureStep<Self::Item, F, Fut>>
    where
        F: Fn(Self::Item) -> Fut + Clone,
        Fut: Future<Output = Result<Self::Item>>,
    {
        SequenceStep::new(self, ClosureStep::new(name, closure))
    }
}
