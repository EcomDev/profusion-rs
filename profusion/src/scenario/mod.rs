use crate::aggregate::MetricAggregate;
use crate::measurer::MetricMeasurer;
use crate::metric::MetricRecordError;
use crate::prelude::Metric;

pub trait ScenarioBuilder<T>
where
    T: Metric,
{
    type Scenario: Scenario<T>;

    fn build(&self) -> Self::Scenario;
}

#[allow(async_fn_in_trait)]
pub trait Scenario<T>
where
    T: Metric,
{
    async fn execute(
        &mut self,
        aggregate: &mut MetricMeasurer<impl MetricAggregate<Metric = T>>,
    ) -> Result<(), MetricRecordError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestScenario;

    #[derive(Eq, PartialEq, Hash, Clone, Copy)]
    struct TestMetric;

    impl Metric for TestMetric {
        fn name(&self) -> &str {
            "test"
        }
    }

    impl Scenario<TestMetric> for TestScenario {
        async fn execute(
            &mut self,
            aggregate: &mut MetricMeasurer<impl MetricAggregate<Metric = TestMetric>>,
        ) -> Result<(), MetricRecordError> {
            aggregate.measure(TestMetric, async {}).await
        }
    }
}
