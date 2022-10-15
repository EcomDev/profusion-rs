mod step;

use crate::{Event, executor::future::MeasuredOutput};
use std::future::Future;

pub use step::{StepScenario, StepScenarioBuilder, SCENARIO_INITIALIZE, SCENARIO_STEP};

pub trait ScenarioBuilder {
    type Item: Sized;
    type Scenario: Scenario<Item = Self::Item>;

    fn build(&self) -> Self::Scenario;
}

pub trait Scenario {
    type Item: Sized;

    type InitializeOutput: Future<Output = MeasuredOutput<Self::Item>>;

    type ExecuteOutput: Future<Output = MeasuredOutput<Self::Item>>;

    fn initialize(&self, events: Vec<Event>) -> Self::InitializeOutput;

    fn execute(&self, input: Self::Item, events: Vec<Event>) -> Self::ExecuteOutput;
}
