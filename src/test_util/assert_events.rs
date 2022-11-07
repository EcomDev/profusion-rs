use crate::Event;
use crate::time::Duration;

use super::assert_instant_with_delta;

pub fn assert_events(actual_events: Vec<Event>, expected_events: Vec<Event>)
{
    assert_events_with_delta(actual_events, expected_events, Duration::from_micros(900))
}

pub fn assert_events_with_delta(actual_events: Vec<Event>, expected_events: Vec<Event>, delta: Duration) {
    assert_eq!(
        actual_events.len(),
        expected_events.len(),
        "event log does not match: left {:?} and right {:?}",
        actual_events,
        expected_events
    );

    actual_events.into_iter()
        .zip(&expected_events)
        .for_each(|(left, right)| {
            assert_eq!(left.name(), right.name(), "event name does not match {} != {}", left.name(), right.name());
            assert_eq!(left.kind(), right.kind(), "event kind does not match {:?} != {:?}", left.name(), right.name());
            assert_instant_with_delta(left.at(), right.at(), delta);
            assert_instant_with_delta(left.at() + left.latency(), right.at() + right.latency(), delta);
        });
}