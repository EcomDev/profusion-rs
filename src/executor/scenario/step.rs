use std::{future::Future, io::Result, marker::PhantomData};

use crate::{Event, executor::{
    future::{MeasuredFuture, MeasuredOutput},
    scenario::{Scenario, ScenarioBuilder},
}, prelude::{ClosureStep, ExecutionStep, NoopStep, SequenceStep}};

pub const SCENARIO_INITIALIZE: &'static str = "scenario::initialize";
pub const SCENARIO_STEP: &'static str = "scenario::step";

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
        InitFut: Future<Output=Result<T>>,
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
        Step: ExecutionStep<Item=T, Output=StepFut>,
        StepFut: Future<Output=MeasuredOutput<T>>,
{
    pub fn with_step<F, Fut>(
        self,
        step: F,
    ) -> StepScenarioBuilder<T, SequenceStep<Step, ClosureStep<T, F, Fut>>, Init, InitFut>
        where
            F: Fn(T) -> Fut + Clone,
            Fut: Future<Output=Result<T>>,
    {
        self.with_named_step(SCENARIO_STEP, step)
    }

    pub fn with_named_step<F, Fut>(
        self,
        name: &'static str,
        step: F,
    ) -> StepScenarioBuilder<T, SequenceStep<Step, ClosureStep<T, F, Fut>>, Init, InitFut>
        where
            F: Fn(T) -> Fut + Clone,
            Fut: Future<Output=Result<T>>,
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
        InitFut: Future<Output=Result<T>>,
        Step: ExecutionStep<Item=T, Output=StepFut>,
        StepFut: Future<Output=MeasuredOutput<T>>,
{
    type Item = T;
    type Scenario = StepScenario<T, Step, Init, InitFut>;

    fn build(&self) -> Self::Scenario {
        StepScenario {
            initialize: self.initialize.clone(),
            step: self.step.clone(),
            _type: self._type,
            _init_future: self._init_future,
        }
    }
}


impl<T, Step, Init, InitFut, StepFut> Scenario for StepScenario<T, Step, Init, InitFut>
    where
        Init: Fn() -> InitFut + Clone,
        InitFut: Future<Output=Result<T>>,
        Step: ExecutionStep<Item=T, Output=StepFut>,
        StepFut: Future<Output=MeasuredOutput<T>>,
{
    type Item = T;
    type InitializeOutput = MeasuredFuture<InitFut>;
    type ExecuteOutput = Step::Output;

    fn initialize(&self, events: Vec<Event>) -> Self::InitializeOutput {
        MeasuredFuture::new(SCENARIO_INITIALIZE, (self.initialize)(), events)
    }

    fn execute(&self, input: Self::Item, events: Vec<Event>) -> Self::ExecuteOutput {
        self.step.execute(events, input)
    }
}

#[cfg(test)]
mod tests {
    use std::future;
    use tokio::time;

    use crate::{
        report::Event,
        time::{Duration, Instant},
    };
    use crate::time::{Clock, InstantOffset};

    use super::*;

    #[tokio::test]
    async fn executes_sequence_scenario() {
        let builder = StepScenarioBuilder::new(|| future::ready(Ok(1)))
            .with_step(|value| future::ready(Ok(value + 1)))
            .with_step(|value| future::ready(Ok(value + 2)))
            .with_step(|value| future::ready(Ok(value + 1)));

        let scenario = builder.build();

        let (_, result) = scenario.initialize(vec![]).await;
        let (_, result) = scenario.execute(result.unwrap(), vec![]).await;

        assert_eq!(result.unwrap(), 5)
    }

    #[tokio::test(start_paused=true)]
    async fn accumulates_events_passed_argument() {
        let builder = StepScenarioBuilder::new(|| async move {
                time::advance(Duration::from_millis(5)).await;
                Ok(1)
            })
            .with_step(|item| async move {
                time::advance(Duration::from_millis(3)).await;
                Ok(item)
            })
            .with_step(|item| async move {
                time::advance(Duration::from_millis(5)).await;
                Ok(item)
            })
            ;

        let scenario = builder.build();
        let events = vec![];
        let time_reference = Clock::now();

        let (events, _) = scenario.initialize(events).await;
        let (events, _) = scenario.execute(1, events).await;

        assert_eq!(
            events,
            vec![
                Event::success(SCENARIO_INITIALIZE, time_reference, time_reference.with_millis(5)),
                Event::success(SCENARIO_STEP, time_reference.with_millis(5), time_reference.with_millis(8)),
                Event::success(SCENARIO_STEP, time_reference.with_millis(8), time_reference.with_millis(13)),
            ],
        )
    }

    #[tokio::test(start_paused=true)]
    async fn measures_connection_timing() {
        let builder = StepScenarioBuilder::new(|| async move {
            time::advance(Duration::from_millis(5)).await;
            Ok(0)
        });

        let scenario = builder.build();
        let time = Clock::now();
        let events = Vec::with_capacity(1);
        let (events, _) = scenario.initialize(events).await;

        assert_eq!(
            events,
            vec![Event::success(
                SCENARIO_INITIALIZE,
                time,
                time.with_millis(5),
            )],
        );
    }
}
