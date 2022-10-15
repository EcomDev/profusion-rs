use std::cmp::Ordering;
use super::*;
use more_asserts::assert_le;

/// Asserts that both instances have difference not more then 200us
pub fn assert_instant(actual: Instant, expected: Instant)
{
    assert_instant_with_delta(actual, expected, Duration::from_micros(200));
}

/// Asserts that both instances have difference not more then specified delta
pub fn assert_instant_with_delta(actual: Instant, expected: Instant, delta: Duration) {
    let difference = match actual.cmp(&expected) {
        Ordering::Equal => Duration::default(),
        Ordering::Less => expected - actual,
        Ordering::Greater => actual - expected
    };

    assert_le!(difference, delta, "instance difference is larger than {:?}", delta)
}

#[cfg(test)]
mod tests
{
    use std::time::Duration;
    use crate::time::{Instant, InstantOffset};
    use crate::test_util::assert_instant;
    use crate::test_util::assert_instant::assert_instant_with_delta;

    #[test]
    fn does_not_throw_any_panic_on_same_instants() {
        let left = Instant::now();
        let right = left;

        assert_instant(left, right);
    }

    #[test]
    fn does_not_throw_any_panic_when_difference_is_less_than_default_delta() {
        let left = Instant::now();
        let right = left.with_micros(199);

        assert_instant(left, right);
    }

    #[test]
    fn does_not_throw_any_panic_when_difference_is_less_than_custom_delta() {
        let left = Instant::now();
        let right = left.with_micros(60);

        assert_instant_with_delta(left, right, Duration::from_micros(60));
    }

    #[test]
    #[should_panic(expected = "instance difference is larger than 200µs")]
    fn panics_with_default_delta_being_exceeded() {
        let left = Instant::now();
        let right = left.with_millis(30);

        assert_instant(left, right);
    }

    #[test]
    #[should_panic(expected = "instance difference is larger than 60µs")]
    fn panics_with_custom_delta_being_exceeded() {
        let left = Instant::now();
        let right = left.with_micros(62);

        assert_instant_with_delta(left, right, Duration::from_micros(60));
    }
}