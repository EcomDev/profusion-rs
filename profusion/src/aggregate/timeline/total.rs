use crate::metric::Metric;
use crate::prelude::TotalAggregateStorage;

use super::TimelineItem;

impl<T> TimelineItem<TotalAggregateStorage<T>>
where
    T: Metric + Send,
{
    pub fn min_value(&self) -> u64 {
        self.storage().value().min()
    }

    pub fn max_value(&self) -> u64 {
        self.storage().value().max()
    }

    pub fn mean_value(&self) -> f64 {
        self.storage().value().mean()
    }

    pub fn percentile_value<P: Into<f64>>(&self, percentile: P) -> u64 {
        self.storage().value().value_at_percentile(percentile.into())
    }

    pub fn histogram(&self) -> Vec<(u64, f64, u64)> {
        let histogram = self.storage().value();
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

    fn populate_timeline_item() -> TimelineItem<TotalAggregateStorage<&'static str>> {
        let mut item = TimelineItem::new(
            Duration::from_millis(10),
            TotalAggregateStorage::default(),
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
        assert_eq!(item.min_value(), 50);
    }

    #[test]
    fn calculates_maximum_per_metric() {
        let item = populate_timeline_item();

        assert_eq!(item.max_value(), 12807);
    }

    #[test]
    fn calculates_mean_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(item.mean_value(), 3730.25);
    }

    #[test]
    fn calculates_percentiles_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(item.percentile_value(90), 6403);
        assert_eq!(item.percentile_value(50), 1600);
    }

    #[test]
    fn returns_log_histogram_per_metric() {
        let item = populate_timeline_item();
        assert_eq!(
            item.histogram(),
            vec![
                (1, 0.0, 0),
                (2, 0.0, 0),
                (4, 0.0, 0),
                (8, 0.0, 0),
                (16, 0.0, 0),
                (32, 0.0, 0),
                (64, 8.333333333333332, 1),
                (128, 8.333333333333332, 1),
                (256, 8.333333333333332, 1),
                (512, 8.333333333333332, 1),
                (1024, 8.333333333333332, 1),
                (2048, 8.333333333333332, 1),
                (4096, 8.333333333333332, 1),
                (8192, 33.33333333333333, 4),
                (16384, 8.333333333333332, 1)
            ]
        );
    }
}
