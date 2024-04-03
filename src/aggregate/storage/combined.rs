use crate::aggregate::AggregateStorage;
use crate::metric::Metric;

pub struct CombinedAggregateStorage<L, R>(L, R);

impl<L, R, T> Clone for CombinedAggregateStorage<L, R>
where
    L: AggregateStorage<Metric = T>,
    R: AggregateStorage<Metric = T>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<L, R, T> CombinedAggregateStorage<L, R>
where
    L: AggregateStorage<Metric = T>,
    R: AggregateStorage<Metric = T>,
    T: Metric,
{
    pub fn new(left: L, right: R) -> Self {
        Self(left, right)
    }

    pub fn unwrap(self) -> (L, R) {
        (self.0, self.1)
    }
}

impl<L, R, T> Default for CombinedAggregateStorage<L, R>
where
    L: AggregateStorage<Metric = T> + Default,
    R: AggregateStorage<Metric = T> + Default,
    T: Metric,
{
    fn default() -> Self {
        Self(L::default(), R::default())
    }
}

impl<L, R, T> AggregateStorage for CombinedAggregateStorage<L, R>
where
    L: AggregateStorage<Metric = T>,
    R: AggregateStorage<Metric = T>,
    T: Metric,
{
    type Metric = T;

    #[inline]
    fn record(&mut self, metric: Self::Metric, latency_value: u64) {
        self.0.record(metric.clone(), latency_value);
        self.1.record(metric.clone(), latency_value)
    }

    fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0), self.1.merge(other.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::storage::{AggregateStorage, TotalAggregateStorage};

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
    fn stores_values_into_each_storage() {
        let mut storage =
            TotalAggregateStorage::default().and(TotalAggregateStorage::default());

        storage.record(TestMetric::One, 100);
        storage.record(TestMetric::Two, 20);
        storage.record(TestMetric::Two, 50);

        let (left, right) = storage.unwrap();

        let (left, right) = (left.value(), right.value());

        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 3);

        assert_eq!(left.min(), 20);
        assert_eq!(right.min(), 20);

        assert_eq!(left.max(), 100);
        assert_eq!(right.max(), 100);
    }

    #[test]
    fn merges_from_all_storages() {
        let mut one =
            TotalAggregateStorage::default().and(TotalAggregateStorage::default());

        let (mut two, three) = (one.clone(), one.clone());

        one.record(TestMetric::One, 100);
        one.record(TestMetric::Two, 20);
        one.record(TestMetric::Two, 50);

        two.record(TestMetric::One, 200);
        two.record(TestMetric::Two, 50);
        two.record(TestMetric::One, 50);

        let (left, right) = three.merge(two.merge(one)).unwrap();
        let (left, right) = (left.value(), right.value());

        assert_eq!(left.len(), 6);
        assert_eq!(right.len(), 6);

        assert_eq!(left.min(), 20);
        assert_eq!(right.min(), 20);

        assert_eq!(left.max(), 200);
        assert_eq!(right.max(), 200);
    }
}
