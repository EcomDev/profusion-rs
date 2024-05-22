use crate::aggregate::MetricAggregateStorage;
use crate::metric::Metric;

use super::TimelineItem;

impl<T> TimelineItem<MetricAggregateStorage<T>>
where
    T: Metric + Send,
{
    pub fn min_value(&self, metric: T) -> u64 {
        self.storage().value(metric).min()
    }

    pub fn max_value(&self, metric: T) -> u64 {
        self.storage().value(metric).max()
    }

    pub fn mean_value(&self, metric: T) -> f64 {
        self.storage().value(metric).mean()
    }

    pub fn percentile_value<P: Into<f64>>(&self, metric: T, percentile: P) -> u64 {
        self.storage().value(metric).value_at_percentile(percentile.into())
    }

    pub fn histogram(&self, metric: T) -> Vec<(u64, f64, u64)> {
        let histogram = self.storage().value(metric);
        let total_counts = histogram.len();
        histogram
            .iter_log(1, 2.0)
            .map(|value| {
                (
                    value.value_iterated_to() + 1,
                    (value.count_since_last_iteration() as f64 / total_counts as f64) * 100.0,
                    value.count_since_last_iteration(),
                )
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn populate_timeline_item() -> TimelineItem<MetricAggregateStorage<&'static str>> {
        let mut item = TimelineItem::new(
            Duration::from_millis(10),
            MetricAggregateStorage::default(),
            0,
            1,
        );

        item.record("one", 50);
        item.record("two", 100);
        item.record("one", 200);
        item.record("one", 400);
        item.record("one", 800);
        item.record("two", 1600);
        item.record("one", 3200);
        item.record("one", 6400);
        item.record("one", 6400);
        item.record("one", 6400);
        item.record("one", 6400);
        item.record("one", 12800);
        item
    }

    #[test]
    fn calculates_minimum_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(item.min_value("one"), 50);
        assert_eq!(item.min_value("two"), 100);
    }

    #[test]
    fn calculates_maximum_per_metric() {
        let item = populate_timeline_item();

        assert_eq!(item.max_value("one"), 12807);
        assert_eq!(item.max_value("two"), 1600);
    }

    #[test]
    fn calculates_mean_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(item.mean_value("one"), 4306.3);
        assert_eq!(item.mean_value("two"), 850.0);
    }

    #[test]
    fn calculates_percentiles_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(item.percentile_value("one", 90), 6403);
        assert_eq!(item.percentile_value("one", 50), 3201);
        assert_eq!(item.percentile_value("two", 90), 1600);
        assert_eq!(item.percentile_value("two", 50), 100);
    }

    #[test]
    fn returns_log_histogram_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(
            item.histogram("one"),
            vec![
                (1, 0.0, 0),
                (2, 0.0, 0),
                (4, 0.0, 0),
                (8, 0.0, 0),
                (16, 0.0, 0),
                (32, 0.0, 0),
                (64, 10.0, 1),
                (128, 0.0, 0),
                (256, 10.0, 1),
                (512, 10.0, 1),
                (1024, 10.0, 1),
                (2048, 0.0, 0),
                (4096, 10.0, 1),
                (8192, 40.0, 4),
                (16384, 10.0, 1)
            ]
        );
        assert_eq!(
            item.histogram("two"),
            vec![
                (1, 0.0, 0),
                (2, 0.0, 0),
                (4, 0.0, 0),
                (8, 0.0, 0),
                (16, 0.0, 0),
                (32, 0.0, 0),
                (64, 0.0, 0),
                (128, 50.0, 1),
                (256, 0.0, 0),
                (512, 0.0, 0),
                (1024, 0.0, 0),
                (2048, 50.0, 1)
            ]
        );
    }
}
