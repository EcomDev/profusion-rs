use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::metric::MetricRecordError;
use crate::prelude::*;

struct Counter(Arc<AtomicUsize>);

impl Counter {
    fn new() -> Self {
        Self(Arc::new(AtomicUsize::new(0)))
    }

    fn increment(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement(&self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }

    fn current(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        self.increment();
        Self(self.0.clone())
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        self.decrement()
    }
}

pub struct TimelineAggregateBuilder<S> {
    settings: AggregateSettings,
    storage: S,
    users: Counter,
}

pub struct TimelineAggregate<S> {
    settings: AggregateSettings,
    timeline: Vec<TimelineItem<S>>,
    storage: S,
    total: TimelineItem<S>,
    users: Counter,
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
            users: Counter::new(),
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
        TimelineAggregate {
            timeline: Vec::new(),
            storage: self.storage.clone(),
            total: TimelineItem::new(
                self.settings.zero().window(self.settings.window()),
                self.storage.clone(),
                0,
                0,
            ),
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

        let latency = self.settings.scale().duration_to_value(latency);
        item.record(metric, latency);
        item.update_counters(error, self.users.current());
        self.total.record(metric, latency);
        self.total.update_counters(error, self.users.current());
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
{
    pub fn flush(self) -> (TimelineItem<S>, Vec<TimelineItem<S>>) {
        (self.total, self.timeline)
    }
}

impl<L, R> TimelineAggregate<CombinedAggregateStorage<L, R>>
where
    L: AggregateStorage,
    R: AggregateStorage<Metric = L::Metric>,
{
    pub fn split(self) -> (TimelineAggregate<L>, TimelineAggregate<R>) {
        let (left_storage, right_storage) = self.storage.unwrap();
        let (left_total, right_total) = self.total.split();

        let (left_timeline, right_timeline) =
            self.timeline.into_iter().map(|item| item.split()).unzip();

        (
            TimelineAggregate {
                users: self.users.clone(),
                settings: self.settings,
                total: left_total,
                storage: left_storage,
                timeline: left_timeline,
            },
            TimelineAggregate {
                users: self.users.clone(),
                settings: self.settings,
                total: right_total,
                storage: right_storage,
                timeline: right_timeline,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::advance;

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

    enum Action {
        Wait(Duration),
        Add(ReportMetric, Duration),
        Error(ReportMetric, Duration, MetricRecordError),
    }

    async fn populate_test_metric<S>(
        reporter: &mut TimelineAggregate<S>,
        metrics: Vec<Action>,
    ) where
        S: AggregateStorage<Metric = ReportMetric>,
    {
        for action in metrics {
            match action {
                Action::Wait(duration) => advance(duration).await,
                Action::Add(metric, latency) => reporter.add_entry(metric, latency, None),
                Action::Error(metric, latency, error) => {
                    reporter.add_entry(metric, latency, Some(&error))
                }
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn aggregates_values_per_each_time_window() {
        let builder = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Milliseconds),
        );
        let mut reporter = builder.build();

        populate_test_metric(
            &mut reporter,
            vec![
                Action::Add(ReportMetric::One, Duration::from_millis(10)),
                Action::Error(
                    ReportMetric::Two,
                    Duration::from_millis(10),
                    MetricRecordError::Timeout(Duration::from_millis(10)),
                ),
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::Two, Duration::from_millis(20)),
                Action::Wait(Duration::from_millis(51)),
                Action::Add(ReportMetric::One, Duration::from_millis(40)),
                Action::Wait(Duration::from_millis(151)),
                Action::Add(ReportMetric::One, Duration::from_millis(60)),
            ],
        )
        .await;

        verify_timeline(
            vec![
                (Duration::from_millis(0), (10, 10), 1, 1),
                (Duration::from_millis(200), (0, 20), 0, 1),
                (Duration::from_millis(300), (40, 0), 0, 1),
                (Duration::from_millis(400), (60, 0), 0, 1),
            ],
            reporter.flush().1,
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

        populate_test_metric(
            &mut reporter_one,
            vec![Action::Add(ReportMetric::One, Duration::from_millis(10))],
        )
        .await;

        populate_test_metric(
            &mut reporter_two,
            vec![Action::Error(
                ReportMetric::Two,
                Duration::from_millis(10),
                MetricRecordError::Timeout(Duration::from_millis(10)),
            )],
        )
        .await;

        populate_test_metric(
            &mut reporter_one,
            vec![
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::Two, Duration::from_millis(20)),
            ],
        )
        .await;

        populate_test_metric(
            &mut reporter_one,
            vec![
                Action::Wait(Duration::from_millis(51)),
                Action::Add(ReportMetric::One, Duration::from_millis(40)),
            ],
        )
        .await;

        let mut aggregated = builder.build();
        reporter_one.merge_into(&mut aggregated);
        reporter_two.merge_into(&mut aggregated);

        verify_timeline(
            vec![
                (Duration::from_millis(0), (10, 10), 1, 2),
                (Duration::from_millis(200), (0, 20), 0, 2),
                (Duration::from_millis(300), (40, 0), 0, 2),
            ],
            aggregated.flush().1,
        );
    }

    #[tokio::test(start_paused = true)]
    async fn allows_splitting_chained_storage_in_timeline() {
        let builder = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default()
                .and(MetricAggregateStorage::with_sigfig(1).unwrap()),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Microseconds),
        );

        let mut aggregate = builder.build();

        populate_test_metric(
            &mut aggregate,
            vec![
                Action::Add(ReportMetric::One, Duration::from_millis(10)),
                Action::Add(ReportMetric::Two, Duration::from_millis(20)),
            ],
        )
        .await;

        let (one, two) = aggregate.split();

        verify_timeline(
            vec![(Duration::new(0, 0), (10000, 20000), 0, 1)],
            one.flush().1,
        );
        verify_timeline(
            vec![(Duration::new(0, 0), (9728, 19456), 0, 1)],
            two.flush().1,
        );
    }

    #[tokio::test(start_paused = true)]
    async fn collects_total_values_for_all_timeline() {
        let mut aggregate = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Microseconds),
        )
        .build();

        populate_test_metric(
            &mut aggregate,
            vec![
                Action::Add(ReportMetric::One, Duration::from_millis(10)),
                Action::Add(ReportMetric::Two, Duration::from_millis(20)),
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::One, Duration::from_millis(100)),
                Action::Add(ReportMetric::Two, Duration::from_millis(230)),
                Action::Wait(Duration::from_millis(100)),
                Action::Add(ReportMetric::One, Duration::from_millis(400)),
                Action::Add(ReportMetric::Two, Duration::from_millis(230)),
                Action::Wait(Duration::from_millis(100)),
                Action::Add(ReportMetric::One, Duration::from_millis(100)),
                Action::Add(ReportMetric::Two, Duration::from_millis(2030)),
            ],
        )
        .await;

        let storage = aggregate.flush().0;
        assert_eq!(storage.storage().value(ReportMetric::One).len(), 4);
        assert_eq!(storage.storage().value(ReportMetric::Two).len(), 4);

        assert_eq!(storage.storage().value(ReportMetric::One).min(), 10000);
        assert_eq!(storage.storage().value(ReportMetric::Two).min(), 20000);
    }

    #[tokio::test(start_paused = true)]
    async fn counts_errors_and_users_in_total() {
        let builder = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Microseconds),
        );
        let mut aggregate = builder.build();
        let (_other, _another) = (builder.build(), builder.build());

        populate_test_metric(
            &mut aggregate,
            vec![
                Action::Error(
                    ReportMetric::One,
                    Duration::from_millis(10),
                    MetricRecordError::Timeout(Duration::from_millis(10)),
                ),
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::Two, Duration::from_millis(230)),
            ],
        )
        .await;

        let total = aggregate.flush().0;
        assert_eq!(total.users(), 3);
        assert_eq!(total.errors(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn reduces_users_count_on_removal_of_aggregators() {
        let builder = TimelineAggregateBuilder::with_settings(
            MetricAggregateStorage::default(),
            AggregateSettings::default()
                .with_window(Duration::from_millis(100))
                .with_scale(AggregateScale::Milliseconds),
        );
        let mut aggregate = builder.build();
        let (other, _another) = (builder.build(), builder.build());

        populate_test_metric(
            &mut aggregate,
            vec![
                Action::Add(ReportMetric::One, Duration::from_millis(10)),
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::Two, Duration::from_millis(230)),
            ],
        )
        .await;

        drop(other);
        populate_test_metric(
            &mut aggregate,
            vec![
                Action::Wait(Duration::from_millis(200)),
                Action::Add(ReportMetric::Two, Duration::from_millis(230)),
            ],
        )
        .await;

        verify_timeline(
            vec![
                (Duration::from_millis(0), (10, 0), 0, 3),
                (Duration::from_millis(200), (0, 230), 0, 3),
                (Duration::from_millis(400), (0, 230), 0, 2),
            ],
            aggregate.flush().1,
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
            assert_eq!(
                item.users(),
                expected_users,
                "Number of users does not match"
            );
        }
    }
}
