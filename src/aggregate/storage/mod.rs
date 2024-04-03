pub use combined::*;
pub use hashmap::*;
pub use total::*;

use crate::metric::Metric;

mod combined;
mod hashmap;
mod total;

/// Storage for aggregation of metric values
pub trait AggregateStorage: Clone + Default + Send {
    type Metric: Metric;

    /// Records aggregate into histogram to be used later
    ///
    /// # Arguments
    ///
    /// * `metric`: metric to associate recorded value with
    /// * `latency_value`: latency value to be recorde in histogram
    fn record(&mut self, metric: Self::Metric, latency_value: u64);

    /// Creates a new storage by merging together both storages
    ///
    /// # Arguments
    ///
    /// * `other`: other storage of the same type
    fn merge(self, other: Self) -> Self;

    fn and<O>(self, other: O) -> CombinedAggregateStorage<Self, O>
    where
        O: AggregateStorage<Metric = Self::Metric>,
    {
        CombinedAggregateStorage::new(self, other)
    }
}
