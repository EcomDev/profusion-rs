//! Extensions of [`std::time::Instant`] and time calculation utilities.

use std::cmp::Ordering;
#[doc(hidden)]
pub use std::time::{Duration, Instant};

mod instant;

/// Extents [Instant][`std::time::Instant`] with [as_duration_bucket][`DurationBucket::as_duration_bucket`] method.
pub trait DurationBucket {
    /// Returns a bucket for the duration based on [bucket_size][`std::time::Duration`]
    ///
    /// Invaluable functionality to aggregate events by specific [Duration][`std::time::Duration`] as time span.
    /// When moment of [Instant][`std::time::Instant`] is in excatly in between time spans it moves it to a higher a level time span.
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

/// Extents [Instant][`std::time::Instant`] with addition and substitution method
pub trait InstantOffset {
    fn with_millis(&self, value: u64) -> Self;
    fn with_micros(&self, value: u64) -> Self;
    fn with_nanos(&self, value: u64) -> Self;
}

/// Compares two [Instant][`std::time::Instant`] instances with [delta][`std::time::Duration`] offset to allow time drift.
///
/// # Example
/// ```
/// use profusion::time::cmp_instant_with_delta;
/// use std::time::Instant;
/// use std::time::Duration;
///
/// assert!(
///    cmp_instant_with_delta(
///        &Instant::now(),
///        &Instant::now(),
///        &Duration::from_micros(10)
///    )
/// );
/// assert!(
///    !cmp_instant_with_delta(
///        &Instant::now(),
///        &(Instant::now() + Duration::from_millis(11)),
///        &Duration::from_millis(10)
///    )
/// )
/// ```
pub fn cmp_instant_with_delta(left: &Instant, right: &Instant, delta: &Duration) -> bool {
    match left.cmp(right) {
        Ordering::Equal => true,
        Ordering::Less => *right - *left <= *delta,
        Ordering::Greater => *left - *right <= *delta,
    }
}

#[cfg(test)]
mod delta_eq_test {
    use super::*;

    static ZERO: Duration = Duration::from_nanos(0);
    static MILLISECOND: Duration = Duration::from_millis(1);

    #[test]
    fn equal_when_the_same() {
        let time = Instant::now();

        assert!(cmp_instant_with_delta(&time, &time, &ZERO))
    }

    #[test]
    fn not_equal_when_not_the_same() {
        let left = Instant::now();
        let right = left + Duration::from_nanos(1);

        assert!(!cmp_instant_with_delta(&left, &right, &ZERO))
    }

    #[test]
    fn equal_when_difference_with_right_is_below_delta() {
        let left = Instant::now();
        let right = left + Duration::from_micros(999);

        assert!(cmp_instant_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_left_is_below_delta() {
        let right = Instant::now();
        let left = right + Duration::from_micros(999);

        assert!(cmp_instant_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_right_equal_delta() {
        let left = Instant::now();
        let right = left + MILLISECOND;

        assert!(cmp_instant_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_left_equal_delta() {
        let right = Instant::now();
        let left = right + MILLISECOND;

        assert!(cmp_instant_with_delta(&left, &right, &MILLISECOND))
    }
}
