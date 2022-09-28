mod step;

use crate::{executor::future::MeasuredOutput};
use std::future::Future;

pub use step::{StepScenario, StepScenarioBuilder};

pub trait ScenarioBuilder {
    type Item: Sized;
    type Scenario: Scenario<Item = Self::Item>;

    fn build(&self) -> Self::Scenario;
}

pub trait Scenario {
    type Item: Sized;

    type InitializeOutput: Future<Output = MeasuredOutput<Self::Item>>;

    type ExecuteOutput: Future<Output = MeasuredOutput<Self::Item>>;

    fn initialize(&self) -> Self::InitializeOutput;

    fn execute(&self, input: Self::Item) -> Self::ExecuteOutput;
}
