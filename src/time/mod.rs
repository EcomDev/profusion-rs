//! Time utilities used in the load test

#[doc(hidden)]
pub use std::time::{Duration, Instant};

mod instant;
mod clock;

pub use clock::Clock;

/// Extents [Instant][`std::time::Instant`] with [as_duration_bucket][`DurationBucket::as_duration_bucket`] method.
pub trait DurationBucket {
    /// Returns a bucket for the duration based on [bucket_size][`std::time::Duration`]
    ///
    /// Invaluable functionality to aggregate events by specific [Duration][`std::time::Duration`] as time span.
    /// When moment of [Instant][`std::time::Instant`] is in excatly in between time spans it moves it to a higher a level time span.
    ///
    /// # Arguments
    /// 
    ///
    /// # Example
    /// ```
    /// use profusion::time::DurationBucket;
    /// use std::time::{Instant, Duration};
    ///
    /// let start_time = Instant::now();
    /// let spans = vec![
    ///    start_time + Duration::from_secs(2), // +2s
    ///    start_time + Duration::from_secs(5), // +5s
    ///    start_time + Duration::from_secs(11), // +11s
    ///    start_time + Duration::from_secs(19), // +19s
    ///    start_time + Duration::from_secs(23) // +23s
    /// ];
    ///
    /// assert_eq!(
    ///     spans
    ///        .into_iter()
    ///        .map(|time| time.as_duration_bucket(&start_time, &Duration::from_secs(10)))
    ///        .collect::<Vec<_>>(),
    ///     vec![
    ///         Duration::from_secs(0),   // 2s ~> 0s bucket
    ///         Duration::from_secs(10),  // 5s ~> 10s bucket
    ///         Duration::from_secs(10),  // 11s ~> 10s bucket
    ///         Duration::from_secs(20),  // 19s ~> 20s bucket
    ///         Duration::from_secs(20)   // 23s ~> 20s bucket
    ///     ]
    /// )
    /// ```
    fn as_duration_bucket(&self, origin: &Instant, bucket_size: &Duration) -> Duration;
}

/// Extents [Instant][`std::time::Instant`] with addition methods
pub trait InstantOffset {
    /// Creates new instance with offset in milliseconds
    ///
    /// # Arguments
    ///
    /// * `value`: milliseconds to add into new [`Instant`][`std::time::Instant`]
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    /// use profusion::time::InstantOffset;
    ///
    /// let time = Instant::now();
    /// assert_eq!(time.with_millis(100), time + Duration::from_millis(100));
    /// ```
    fn with_millis(&self, value: u64) -> Self;
    /// Creates new instance with offset in micros
    ///
    /// # Arguments
    ///
    /// * `value`: microseconds to add into new [`Instant`][`std::time::Instant`]
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    /// use profusion::time::InstantOffset;
    ///
    /// let time = Instant::now();
    /// assert_eq!(time.with_micros(112), time + Duration::from_micros(112));
    /// ```
    fn with_micros(&self, value: u64) -> Self;
    /// Creates new instance with offset in nanos
    ///
    /// # Arguments
    ///
    /// * `value`: nanoseconds to add into new [`Instant`][`std::time::Instant`]
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    /// use profusion::time::InstantOffset;
    ///
    /// let time = Instant::now();
    /// assert_eq!(time.with_nanos(499), time + Duration::from_nanos(499));
    /// ```
    fn with_nanos(&self, value: u64) -> Self;
}