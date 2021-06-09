use super::future::MeasuredOutput;
use crate::report::Event;
use std::future::Future;

mod closure;
mod noop;
mod combined;

pub use noop::NoopStep;
pub use closure::ClosureStep;

pub trait ExecutionStep: Clone {
    type Item;
    type Output: Future<Output = MeasuredOutput<Self::Item>>;

    /// Executes a step by creating a future with input as an argument
    fn execute(&self, events: Vec<Event>, input: Self::Item) -> Self::Output;

    /// Expected capacity of the execution step
    fn capacity(&self) -> usize;
}

pub trait WeightedExecutionStep: ExecutionStep {
    /// Executes weighted step with target
    fn execute_with_target(
        &self,
        _target: usize,
        events: Vec<Event>,
        input: Self::Item,
    ) -> Self::Output {
        self.execute(events, input)
    }

    /// Weight of the execution step
    ///
    /// It is only used when test step is selected with another step
    fn weight(&self) -> usize;
}
