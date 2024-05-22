use std::time::Duration;

use profusion::prelude::*;
use profusion::scenario;

#[derive(Default)]
struct State(usize);

impl State {
    fn incr(&mut self) {
        self.0 += 1;
    }

    fn value(&self) -> u64 {
        self.0 as u64
    }
}

#[scenario(LoadTestOne)]
async fn load_test_items(
    reporter: &mut MetricMeasurer<impl MetricAggregate<Metric = &'static str>>,
) -> Result<(), MetricRecordError> {
    reporter.measure("load_test_one", async { Ok(()) }).await?
}

#[scenario(LoadTestTwo)]
async fn load_test_items_with_state(
    reporter: &mut MetricMeasurer<impl MetricAggregate<Metric = &'static str>>,
    state_one: &mut State,
) -> Result<(), MetricRecordError> {
    reporter.add_measurement(
        "load_test_one::one",
        Duration::from_millis(state_one.value()),
        None,
    );
    reporter.measure("load_test_two", async { Ok(state_one.incr()) }).await?
}

#[tokio::test]
async fn stateless_scenario() -> Result<(), MetricRecordError> {
    let item = LoadTestOne;

    Ok(())
}
