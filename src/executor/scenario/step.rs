use crate::{
    executor::{
        future::{MeasuredFuture, MeasuredOutput},
        scenario::{Scenario, ScenarioBuilder},
    },
    prelude::{ClosureStep, ExecutionStep, NoopStep, SequenceStep},
    RealtimeReporter,
};
use std::{future::Future, io::Result, marker::PhantomData};

pub struct StepScenarioBuilder<T, Step, Init, InitFut> {
    initialize: Init,
    step: Step,
    _type: PhantomData<T>,
    _init_future: PhantomData<InitFut>,
}

pub struct StepScenario<T, Step, Init, InitFut> {
    initialize: Init,
    step: Step,
    _type: PhantomData<T>,
    _init_future: PhantomData<InitFut>,
}

impl<T, Init, InitFut> StepScenarioBuilder<T, NoopStep<T>, Init, InitFut>
where
    Init: Fn() -> InitFut + Clone,
    InitFut: Future<Output = Result<T>>,
{
    pub fn new(initialize: Init) -> Self {
        Self {
            initialize,
            step: NoopStep::new(),
            _type: PhantomData,
            _init_future: PhantomData,
        }
    }
}

impl<T, Step, Init, StepFut, InitFut> StepScenarioBuilder<T, Step, Init, InitFut>
where
    Step: ExecutionStep<Item = T, Output = StepFut>,
    StepFut: Future<Output = MeasuredOutput<T>>,
{
    pub fn with_step<F, Fut>(
        self,
        step: F,
    ) -> StepScenarioBuilder<T, SequenceStep<Step, ClosureStep<T, F, Fut>>, Init, InitFut>
    where
        F: Fn(T) -> Fut + Clone,
        Fut: Future<Output = Result<T>>,
    {
        self.with_named_step("", step)
    }

    pub fn with_named_step<F, Fut>(
        self,
        name: &'static str,
        step: F,
    ) -> StepScenarioBuilder<T, SequenceStep<Step, ClosureStep<T, F, Fut>>, Init, InitFut>
    where
        F: Fn(T) -> Fut + Clone,
        Fut: Future<Output = Result<T>>,
    {
        let step = self.step.step(name, step);

        StepScenarioBuilder {
            step,
            initialize: self.initialize,
            _type: self._type,
            _init_future: self._init_future,
        }
    }
}

impl<T, Step, Init, InitFut, StepFut> ScenarioBuilder
    for StepScenarioBuilder<T, Step, Init, InitFut>
where
    Init: Fn() -> InitFut + Clone,
    InitFut: Future<Output = Result<T>>,
    Step: ExecutionStep<Item = T, Output = StepFut>,
    StepFut: Future<Output = MeasuredOutput<T>>,
{
    type Item = T;
    type Scenario = StepScenario<T, Step, Init, InitFut>;

    fn build(&self) -> Self::Scenario {
        StepScenario {
            initialize: self.initialize.clone(),
            step: self.step.clone(),
            _type: self._type.clone(),
            _init_future: self._init_future.clone(),
        }
    }
}

impl<T, Step, Init, InitFut, StepFut> Scenario for StepScenario<T, Step, Init, InitFut>
where
    Init: Fn() -> InitFut + Clone,
    InitFut: Future<Output = Result<T>>,
    Step: ExecutionStep<Item = T, Output = StepFut>,
    StepFut: Future<Output = MeasuredOutput<T>>,
{
    type Item = T;
    type InitializeOutput = MeasuredFuture<InitFut>;
    type ExecuteOutput = Step::Output;

    fn initialize(&self) -> Self::InitializeOutput {
        MeasuredFuture::new("initialize", (&self.initialize)(), Vec::with_capacity(1))
    }

    fn execute(&self, input: Self::Item) -> Self::ExecuteOutput {
        self.step
            .execute(Vec::with_capacity(self.step.capacity()), input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        report::Event,
        time::{Duration, Instant},
    };

    async fn init() -> Result<usize> {
        Ok(1)
    }

    async fn add_one(value: usize) -> Result<usize> {
        Ok(value + 1)
    }

    async fn add_two(value: usize) -> Result<usize> {
        Ok(value + 2)
    }

    async fn init_wait() -> Result<usize> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok(1)
    }

    #[tokio::test]
    async fn executes_sequence_scenario() {
        let builder = StepScenarioBuilder::new(init)
            .with_step(add_one)
            .with_step(add_two)
            .with_step(add_one);

        let scenario = builder.build();

        let (_, result) = scenario.initialize().await;
        let (_, result) = scenario.execute(result.unwrap()).await;

        assert_eq!(result.unwrap(), 5)
    }

    #[tokio::test]
    async fn measures_connection_timing() {
        let builder = StepScenarioBuilder::new(init_wait);

        let scenario = builder.build();
        let time = Instant::now();
        let (events, _) = scenario.initialize().await;
        assert_eq!(
            events,
            vec![Event::success(
                "initialize",
                time,
                time + Duration::from_millis(5)
            )]
        );
    }
}
