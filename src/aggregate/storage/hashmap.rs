use hdrhistogram::{CreationError, Histogram};
pub use rustc_hash::FxHashMap;
use tracing::error;

use crate::aggregate::AggregateStorage;
use crate::metric::Metric;

pub struct MetricAggregateStorage<T> {
    inner: FxHashMap<T, Histogram<u64>>,
    proto: Histogram<u64>,
}

impl<T> Default for MetricAggregateStorage<T>
where
    T: Metric + Send,
{
    fn default() -> Self {
        match Self::with_sigfig(3) {
            Ok(storage) => storage,
            Err(_) => unreachable!(),
        }
    }
}

impl<T> MetricAggregateStorage<T>
where
    T: Metric + Send,
{
    pub fn with_limit(sigfig: u8, max_value: u64) -> Result<Self, CreationError> {
        let histogram = Histogram::new_with_max(max_value, sigfig)?;

        Ok(Self {
            proto: histogram,
            inner: FxHashMap::default(),
        })
    }

    pub fn with_sigfig(sigfig: u8) -> Result<Self, CreationError> {
        let histogram = Histogram::new(sigfig)?;

        Ok(Self {
            proto: histogram,
            inner: FxHashMap::default(),
        })
    }

    pub fn value(&self, metric: T) -> &Histogram<u64> {
        &self.inner.get(&metric).unwrap_or(&self.proto)
    }
}

impl<T> AggregateStorage for MetricAggregateStorage<T>
where
    T: Metric + Send,
{
    type Metric = T;

    #[inline]
    fn record(&mut self, metric: Self::Metric, latency_value: u64) {
        let histogram = self
            .inner
            .entry(metric)
            .or_insert_with(|| self.proto.clone());
        match histogram.record(latency_value) {
            Ok(_) => (),
            Err(error) => {
                error!(latency_value = ?latency_value, error = ?error, "Failed to store latency value")
            }
        }
    }

    fn merge(self, other: Self) -> Self {
        let mut inner = self.inner;
        for (metric, histogram) in other.inner.into_iter() {
            match inner.get_mut(&metric) {
                Some(value) => *value += histogram,
                None => drop(inner.insert(metric, histogram)),
            }
        }

        Self { inner, ..self }
    }
}

impl<T> Clone for MetricAggregateStorage<T>
where
    T: Metric,
{
    fn clone(&self) -> Self {
        Self {
            inner: FxHashMap::default(),
            proto: self.proto.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
    enum TestMetric {
        One,
        Two,
    }

    impl Metric for TestMetric {
        fn name(&self) -> &str {
            "metric"
        }
    }

    #[test]
    fn stores_values_into_histogram_on_record_per_metric() {
        let mut storage = MetricAggregateStorage::default();
        storage.record(TestMetric::One, 100);
        storage.record(TestMetric::Two, 20);
        storage.record(TestMetric::Two, 50);

        let first_histogram = storage.inner.get(&TestMetric::One).unwrap();
        let second_histogram = storage.inner.get(&TestMetric::Two).unwrap();
        assert_eq!(first_histogram.len(), 1);
        assert_eq!(second_histogram.len(), 2);

        assert_eq!(first_histogram.max(), 100);
        assert_eq!(first_histogram.min(), 100);

        assert_eq!(second_histogram.max(), 50);
        assert_eq!(second_histogram.min(), 20);
    }

    #[test]
    fn returns_stored_values_per_metric() {
        let mut storage = MetricAggregateStorage::default();
        storage.record(TestMetric::One, 100);
        storage.record(TestMetric::Two, 20);
        storage.record(TestMetric::Two, 50);

        assert_eq!(storage.value(TestMetric::One).len(), 1);
        assert_eq!(storage.value(TestMetric::Two).len(), 2);

        assert_eq!(storage.value(TestMetric::One).max(), 100);
        assert_eq!(storage.value(TestMetric::One).min(), 100);

        assert_eq!(storage.value(TestMetric::Two).max(), 50);
        assert_eq!(storage.value(TestMetric::Two).min(), 20);
    }

    #[test]
    fn modifies_proto_histogram_sigfig() {
        let storage = MetricAggregateStorage::<TestMetric>::with_sigfig(1).unwrap();
        assert_eq!(storage.proto.sigfig(), 1)
    }

    #[test]
    fn modifies_proto_histogram_high_value() {
        let storage = MetricAggregateStorage::<TestMetric>::with_limit(1, 6100).unwrap();
        assert_eq!(storage.proto.high(), 6100)
    }

    #[test]
    fn modifying_max_value_disables_auto_resize() {
        let storage = MetricAggregateStorage::<TestMetric>::with_limit(3, 6100).unwrap();
        assert_eq!(false, storage.proto.is_auto_resize())
    }

    #[test]
    fn merges_multiple_storages_into_one() {
        let (mut one, mut two, three) = (
            MetricAggregateStorage::default(),
            MetricAggregateStorage::default(),
            MetricAggregateStorage::default(),
        );

        one.record(TestMetric::One, 100);
        one.record(TestMetric::Two, 20);
        one.record(TestMetric::Two, 50);

        two.record(TestMetric::One, 200);
        two.record(TestMetric::Two, 50);
        two.record(TestMetric::One, 50);

        let merged = three.merge(two.merge(one));

        assert_eq!(merged.value(TestMetric::One).len(), 3);
        assert_eq!(merged.value(TestMetric::Two).len(), 3);
        assert_eq!(merged.value(TestMetric::One).min(), 50);
        assert_eq!(merged.value(TestMetric::Two).min(), 20);
        assert_eq!(merged.value(TestMetric::One).max(), 200);
        assert_eq!(merged.value(TestMetric::Two).max(), 50);
    }
}
