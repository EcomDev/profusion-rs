use crate::metric::{Metric, MetricRecordError, MetricReporter, MetricReporterBuilder};
use std::{marker::PhantomData, time::Duration};

/// Test reporter builder
///
/// Creates instances of reporter for testing
pub struct TestReporterBuilder<T>(PhantomData<T>);

/// Test reporter
///
/// Adds reported metrics to vector for later verification in tests
pub struct TestReporter<T> {
    values: Vec<(T, Duration, bool)>,
}

impl<T> TestReporterBuilder<T> {
    pub fn new() -> Self {
        Self(PhantomData::<T>)
    }
}

impl<T> MetricReporterBuilder for TestReporterBuilder<T>
where
    T: Metric,
{
    type Reporter = TestReporter<T>;

    fn build(&self) -> Self::Reporter {
        TestReporter { values: Vec::new() }
    }
}

impl<T> TestReporter<T>
where
    T: Metric,
{
    pub fn values(self) -> Vec<(T, Duration, bool)> {
        self.values
    }
}

impl<T> MetricReporter for TestReporter<T>
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

    fn aggregate_into(mut self, other: &mut Self) {
        other.values.append(&mut self.values);
    }
}
