use std::{marker::PhantomData, time::Duration};

use crate::aggregate::{MetricAggregate, MetricAggregateBuilder};
use crate::metric::{Metric, MetricRecordError};

/// Test aggregate builder
///
/// Creates instances of aggregate for testing
pub struct TestAggregateBuilder<T>(PhantomData<T>);

/// Test aggregate
///
/// Adds reported metrics to vector for later verification in tests
pub struct TestAggregate<T> {
    values: Vec<(T, Duration, bool)>,
}

impl<T> TestAggregateBuilder<T> {
    pub fn new() -> Self {
        Self(PhantomData::<T>)
    }
}

impl<T> MetricAggregateBuilder for TestAggregateBuilder<T>
where
    T: Metric,
{
    type Reporter = TestAggregate<T>;

    fn build(&self) -> Self::Reporter {
        TestAggregate { values: Vec::new() }
    }
}

impl<T> TestAggregate<T>
where
    T: Metric,
{
    pub fn values(self) -> Vec<(T, Duration, bool)> {
        self.values
    }
}

impl<T> MetricAggregate for TestAggregate<T>
where
    T: Metric,
{
    type Metric = T;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        elapsed: Duration,
        error: Option<&MetricRecordError>,
    ) {
        self.values.push((metric, elapsed, error.is_some()))
    }

    fn merge_into(mut self, other: &mut Self) {
        other.values.append(&mut self.values);
    }
}
