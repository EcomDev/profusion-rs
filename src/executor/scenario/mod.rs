/*
 * Copyright © 2022. EcomDev B.V.
 * All rights reserved.
 * See LICENSE for license details.
 */

use std::future::Future;

pub use step::{SCENARIO_INITIALIZE, SCENARIO_STEP, StepScenario, StepScenarioBuilder};

use crate::{Event, executor::future::MeasuredOutput};

mod step;

pub trait ScenarioBuilder {
    type Item: Sized;
    type Scenario: Scenario<Item=Self::Item>;

    fn build(&self) -> Self::Scenario;
}

pub trait Scenario {
    type Item: Sized;

    type InitializeOutput: Future<Output=MeasuredOutput<Self::Item>>;

    type ExecuteOutput: Future<Output=MeasuredOutput<Self::Item>>;

    fn initialize(&self, events: Vec<Event>) -> Self::InitializeOutput;

    fn execute(&self, input: Self::Item, events: Vec<Event>) -> Self::ExecuteOutput;
}
