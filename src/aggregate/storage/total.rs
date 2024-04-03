use std::marker::PhantomData;

pub use hdrhistogram::{CreationError, Histogram};
use tracing::error;

use crate::aggregate::AggregateStorage;
use crate::metric::Metric;

pub struct TotalAggregateStorage<T> {
    inner: Histogram<u64>,
    _metric: PhantomData<T>,
}

impl<T> Default for TotalAggregateStorage<T>
where
    T: Metric + Send,
{
    fn default() -> Self {
        Self {
            inner: match Histogram::new(3) {
                Ok(histogram) => histogram,
                Err(_) => unreachable!(),
            },
            _metric: PhantomData,
        }
    }
}

impl<T> TotalAggregateStorage<T>
where
    T: Metric + Send,
{
    pub fn with_limit(sigfig: u8, max_value: u64) -> Result<Self, CreationError> {
        let inner = Histogram::new_with_max(max_value, sigfig)?;

        Ok(Self {
            inner,
            _metric: PhantomData,
        })
    }

    pub fn with_sigfig(sigfig: u8) -> Result<Self, CreationError> {
        let inner = Histogram::new(sigfig)?;

        Ok(Self {
            inner,
            _metric: PhantomData,
        })
    }

    pub fn value(self) -> Histogram<u64> {
        self.inner
    }
}

impl<T> AggregateStorage for TotalAggregateStorage<T>
where
    T: Metric + Send,
{
    type Metric = T;

    #[inline]
    fn record(&mut self, _metric: Self::Metric, latency_value: u64) {
        match self.inner.record(latency_value) {
            Ok(_) => (),
            Err(error) => {
                error!(latency_value = ?latency_value, error = ?error, "Failed to store latency value")
            }
        }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            inner: self.inner + other.inner,
            ..self
        }
    }
}

impl<T> Clone for TotalAggregateStorage<T>
where
    T: Metric,
{
    fn clone(&self) -> Self {
        let mut inner = self.inner.clone();
        inner.clear();

        Self {
            inner,
            _metric: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Eq, PartialEq, Hash, Copy, Clone)]
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
    fn stores_values_into_histogram_on_record_as_single_metric() {
        let mut storage = TotalAggregateStorage::default();
        storage.record(TestMetric::One, 100);
        storage.record(TestMetric::Two, 20);
        storage.record(TestMetric::Two, 50);

        assert_eq!(storage.inner.len(), 3);

        assert_eq!(storage.inner.max(), 100);
        assert_eq!(storage.inner.min(), 20);
    }

    #[test]
    fn merges_multiple_storages_into_one() {
        let (mut one, mut two, three) = (
            TotalAggregateStorage::default(),
            TotalAggregateStorage::default(),
            TotalAggregateStorage::default(),
        );

        one.record(TestMetric::One, 100);
        one.record(TestMetric::Two, 20);
        one.record(TestMetric::Two, 50);

        two.record(TestMetric::One, 200);
        two.record(TestMetric::Two, 50);
        two.record(TestMetric::One, 50);

        let value = three.merge(two.merge(one)).value();

        assert_eq!(value.len(), 6);
        assert_eq!(value.max(), 200);
        assert_eq!(value.min(), 20);
        assert_eq!(value.count_at(50), 3)
    }

    #[test]
    fn modifies_proto_histogram_sigfig() {
        let storage = TotalAggregateStorage::<TestMetric>::with_sigfig(1).unwrap();
        assert_eq!(storage.inner.sigfig(), 1)
    }

    #[test]
    fn modifies_proto_histogram_high_value() {
        let storage = TotalAggregateStorage::<TestMetric>::with_limit(1, 6100).unwrap();
        assert_eq!(storage.inner.high(), 6100)
    }

    #[test]
    fn modifying_max_value_disables_auto_resize() {
        let storage = TotalAggregateStorage::<TestMetric>::with_limit(1, 6100).unwrap();
        assert_eq!(false, storage.inner.is_auto_resize())
    }
}
