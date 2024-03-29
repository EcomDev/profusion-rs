use crate::metric::{Metric, MetricRecordError};
use std::time::Duration;

pub trait MetricReporterBuilder {
    type Reporter: MetricReporter;

    fn build(&self) -> Self::Reporter;
}

pub trait MetricReporter {
    type Metric: Metric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    );

    fn aggregate_into(self, other: &mut Self);
}
