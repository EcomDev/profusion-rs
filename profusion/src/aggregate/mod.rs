use std::time::Duration;

pub use scale::AggregateScale;
pub use settings::AggregateSettings;
pub use storage::*;
#[cfg(any(feature = "test_util", test))]
pub use test_aggregate::*;
pub use timeline::*;

use crate::metric::{Metric, MetricRecordError};
pub use crate::start_time::StartTime;

mod scale;
mod settings;
mod storage;
#[cfg(any(feature = "test_util", test))]
mod test_aggregate;
mod timeline;

pub trait MetricAggregateBuilder {
    type Reporter: MetricAggregate;

    fn build(&self) -> Self::Reporter;
}

pub trait MetricAggregate {
    type Metric: Metric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    );

    fn merge_into(self, other: &mut Self);
}
