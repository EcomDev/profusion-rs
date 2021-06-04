mod instant;

pub use std::time::{Instant, Duration};
use std::cmp::Ordering;

pub trait DurationBucket {
    fn as_duration_bucket(&self, origin: &Instant, bucket_size: &Duration) -> Duration;
}

pub fn instant_eq_with_delta(left: &Instant, right: &Instant, delta: &Duration) -> bool {
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

        assert!(instant_eq_with_delta(&time, &time, &ZERO))
    }

    #[test]
    fn not_equal_when_not_the_same() {
        let left = Instant::now();
        let right = left + Duration::from_nanos(1);

        assert!(!instant_eq_with_delta(&left, &right, &ZERO))
    }

    #[test]
    fn equal_when_difference_with_right_is_below_delta() {
        let left = Instant::now();
        let right = left + Duration::from_micros(999);

        assert!(instant_eq_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_left_is_below_delta() {
        let right = Instant::now();
        let left = right + Duration::from_micros(999);

        assert!(instant_eq_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_right_equal_delta() {
        let left = Instant::now();
        let right = left + MILLISECOND;

        assert!(instant_eq_with_delta(&left, &right, &MILLISECOND))
    }

    #[test]
    fn equal_when_difference_with_left_equal_delta() {
        let right = Instant::now();
        let left = right + MILLISECOND;

        assert!(instant_eq_with_delta(&left, &right, &MILLISECOND))
    }
}