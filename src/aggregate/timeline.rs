use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::aggregate::{
    AggregateSettings, AggregateStorage, MetricAggregate, MetricAggregateBuilder,
};
use crate::metric::MetricRecordError;

pub struct TimelineItem<S> {
    time: Duration,
    storage: S,
    errors: usize,
    users: usize,
}

impl<S> Eq for TimelineItem<S> {}

impl<S> PartialEq for TimelineItem<S> {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl<S> PartialOrd for TimelineItem<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl<S> Ord for TimelineItem<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

pub struct TimelineAggregateBuilder<S> {
    settings: AggregateSettings,
    storage: S,
    users: Arc<AtomicUsize>,
}

pub struct TimelineAggregate<S> {
    settings: AggregateSettings,
    timeline: Vec<TimelineItem<S>>,
    storage: S,
    users: Arc<AtomicUsize>,
}

impl<S> TimelineItem<S>
where
    S: AggregateStorage,
{
    fn new(time: Duration, storage: S, errors: usize, users: usize) -> Self {
        Self {
            time,
            storage,
            errors,
            users,
        }
    }

    pub fn time(&self) -> &Duration {
        &self.time
    }

    pub fn storage(&self) -> &S {
        &self.storage
    }

    pub fn errors(&self) -> usize {
        self.errors
    }

    pub fn users(&self) -> usize {
        self.users
    }

    fn record(&mut self, metric: S::Metric, value: u64) {
        self.storage.record(metric, value)
    }

    fn update_counters(
        &mut self,
        error: Option<&MetricRecordError>,
        users: &Arc<AtomicUsize>,
    ) {
        if error.is_some() {
            self.errors += 1;
        }

        self.users = users.load(Ordering::Relaxed)
    }

    fn merge_into(self, other: &mut Self) {
        let storage = std::mem::take(&mut other.storage);
        other.storage = storage.merge(self.storage);
        other.users = max(other.users, self.users);
        other.errors += self.errors;
    }
}

impl<S> TimelineAggregateBuilder<S>
where
    S: AggregateStorage,
    S::Metric: Sync,
{
    pub fn new(storage: S) -> Self {
        Self::with_settings(storage, AggregateSettings::default())
    }

    pub fn with_settings(storage: S, settings: AggregateSettings) -> Self {
        Self {
            settings,
            storage,
            users: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl<S> MetricAggregateBuilder for TimelineAggregateBuilder<S>
where
    S: AggregateStorage,
    S::Metric: Sync,
{
    type Reporter = TimelineAggregate<S>;

    fn build(&self) -> Self::Reporter {
        self.users.fetch_add(1, Ordering::Relaxed);
        TimelineAggregate {
            timeline: Vec::new(),
            storage: self.storage.clone(),
            settings: self.settings,
            users: self.users.clone(),
        }
    }
}

impl<S> MetricAggregate for TimelineAggregate<S>
where
    S: AggregateStorage,
{
    type Metric = S::Metric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        let time_window = self.settings.zero().window(self.settings.window());
        let item = match self.timeline.last_mut() {
            Some(item) if item.time().eq(&time_window) => item,
            _ => {
                let position = self.timeline.len();
                self.timeline.push(TimelineItem::new(
                    time_window,
                    self.storage.clone(),
                    0,
                    0,
                ));
                &mut self.timeline[position]
            }
        };

        item.record(metric, self.settings.scale().duration_to_value(latency));
        item.update_counters(error, &self.users);
    }

    fn merge_into(self, other: &mut Self) {
        for item in self.timeline.into_iter() {
            match other.timeline.binary_search(&item) {
                Ok(position) => {
                    item.merge_into(&mut other.timeline[position]);
                }
                Err(position) => other.timeline.insert(position, item),
            }
        }
    }
}

impl<S> TimelineAggregate<S>
where
    S: AggregateStorage,
    S::Metric: Sync,
{
    pub fn flush(self) -> Vec<TimelineItem<S>> {
        self.timeline
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::advance;

    use crate::aggregate::{AggregateScale, MetricAggregateStorage};
    use crate::metric::Metric;

    use super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
    enum ReportMetric {
        One,
        Two,
    }

    impl Metric for ReportMetric {
        fn name(&self) -> &str {
            "report_metric"
        }
    }

    #[tokio::test(start_paused = true)]
    async fn aggregates_values_per_each_time_window() {
        let mut reporter = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Milliseconds),
        )
        .build();

        reporter.add_entry(ReportMetric::One, Duration::from_millis(10), None);
        reporter.add_entry(
            ReportMetric::Two,
            Duration::from_millis(10),
            Some(MetricRecordError::Timeout(Duration::from_millis(10))).as_ref(),
        );
        advance(Duration::from_millis(200)).await;
        reporter.add_entry(ReportMetric::Two, Duration::from_millis(20), None);
        advance(Duration::from_millis(51)).await;
        reporter.add_entry(ReportMetric::One, Duration::from_millis(40), None);

        verify_timeline(
            vec![
                (Duration::from_millis(0), (10, 10), 1, 1),
                (Duration::from_millis(200), (0, 20), 0, 1),
                (Duration::from_millis(300), (40, 0), 0, 1),
            ],
            reporter.flush(),
        );
    }

    #[tokio::test(start_paused = true)]
    async fn merges_aggregated_values_per_each_time_window() {
        let builder = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Milliseconds),
        );

        let mut reporter_one = builder.build();
        let mut reporter_two = builder.build();

        reporter_one.add_entry(ReportMetric::One, Duration::from_millis(10), None);
        reporter_two.add_entry(
            ReportMetric::Two,
            Duration::from_millis(10),
            Some(MetricRecordError::Timeout(Duration::from_millis(10))).as_ref(),
        );
        advance(Duration::from_millis(200)).await;
        reporter_one.add_entry(ReportMetric::Two, Duration::from_millis(20), None);
        advance(Duration::from_millis(51)).await;
        reporter_two.add_entry(ReportMetric::One, Duration::from_millis(40), None);

        let mut aggregated = builder.build();
        reporter_one.merge_into(&mut aggregated);
        reporter_two.merge_into(&mut aggregated);

        verify_timeline(
            vec![
                (Duration::from_millis(0), (10, 10), 1, 2),
                (Duration::from_millis(200), (0, 20), 0, 2),
                (Duration::from_millis(300), (40, 0), 0, 2),
            ],
            aggregated.flush(),
        );
    }

    fn verify_timeline(
        expected_values: Vec<(Duration, (u64, u64), usize, usize)>,
        result: Vec<TimelineItem<MetricAggregateStorage<ReportMetric>>>,
    ) {
        assert_eq!(
            result.len(),
            expected_values.len(),
            "Number of timeline values does not match"
        );
        for (
            item,
            (
                expected_time,
                (expected_metric_one, expected_metric_two),
                expected_errors,
                expected_users,
            ),
        ) in result.into_iter().zip(expected_values)
        {
            assert_eq!(*item.time(), expected_time, "Time window does not match");
            assert_eq!(
                item.storage().value(ReportMetric::One).min(),
                expected_metric_one,
                "Minimum metric one does not match"
            );
            assert_eq!(
                item.storage().value(ReportMetric::Two).min(),
                expected_metric_two,
                "Minimum metric two does not match"
            );
            assert_eq!(
                item.errors(),
                expected_errors,
                "Number of errors does not match"
            );
            assert_eq!(item.users(), expected_users, "Number of users does match");
        }
    }
}
