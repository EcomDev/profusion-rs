use super::{EventProcessor, Event, EventType};
use crate::{Duration, Instant};
use crate::time::instant_eq_with_delta;

static EVENT_DELTA: Duration = Duration::from_millis(1);

impl Event {
    fn new(
        name: &'static str,
        started_at: Instant,
        finished_at: Instant,
        kind: EventType,
    ) -> Self {
        Self {
            name,
            started_at,
            finished_at,
            kind,
        }
    }

    /// Creates a success event
    /// 
    /// ```rust
    /// use profusion::{Event, EventType, Instant};
    /// 
    /// let event = Event::success("default", Instant::now(), Instant::now());
    /// assert_eq!(event.kind(), EventType::Success);
    /// ```
    pub fn success(name: &'static str, started_at: Instant, finished_at: Instant) -> Self {
        Self::new(name, started_at, finished_at, EventType::Success)
    }

    /// Creates an error event
    /// 
    /// ```rust
    /// use profusion::{Event, EventType, Instant};
    /// 
    /// let (start, end) = (Instant::now(), Instant::now());
    /// let event = Event::error("default", start, end);
    /// assert_eq!(event.kind(), EventType::Error);
    /// ```
    pub fn error(name: &'static str, started_at: Instant, finished_at: Instant) -> Self {
        Self::new(name, started_at, finished_at, EventType::Error)
    }

    /// Creates a timeout event
    /// 
    /// ```rust
    /// use profusion::{Event, EventType, Instant};
    /// 
    /// let event = Event::error("default", Instant::now(), Instant::now());
    /// assert_eq!(event.kind(), EventType::Error);
    /// ```
    pub fn timeout(name: &'static str, started_at: Instant, finished_at: Instant) -> Self {
        Self::new(name, started_at, finished_at, EventType::Timeout)
    }

    pub(crate) fn process<P: EventProcessor>(&self, processor: &mut P) {
        match self.kind {
            EventType::Success => {
                processor.process_success(self.name, self.started_at, self.finished_at)
            }
            EventType::Timeout => {
                processor.process_timeout(self.name, self.started_at, self.finished_at)
            }
            EventType::Error => {
                processor.process_error(self.name, self.started_at, self.finished_at)
            }
        }
    }
    
    /// Calculates latency based on event time span
    /// 
    /// ```rust
    /// use profusion::{Event, EventType, Instant, Duration};
    /// 
    /// let start = Instant::now();
    /// let end = start + Duration::from_secs(1); 
    //
    /// let event = Event::error("default", start, end);
    /// assert_eq!(event.latency(), Duration::from_secs(1));
    /// ```
    pub fn latency(&self) -> Duration {
        self.finished_at - self.started_at
    }

    /// Type of the event that was captured
    /// 
    /// ```rust
    /// use profusion::{Event, EventType, Instant};
    ///
    /// let event = Event::error("default", Instant::now(), Instant::now());
    /// assert_eq!(event.kind(), EventType::Error);
    /// ```
    pub fn kind(&self) -> EventType {
        self.kind
    }

    /// Returns event name
    /// 
    /// ```rust
    /// use profusion::{Event, Instant};
    ///
    /// let (start, end) = (Instant::now(), Instant::now());
    /// let event = Event::success("custom_event_name", start, end);
    /// assert_eq!(event.name(), "custom_event_name");
    /// ```
    pub fn name(&self) -> &str {
        self.name
    }
}

/// Creates successfull event from tuple of name and two `Instant` objects
/// 
/// ```rust
/// use profusion::{Event, EventType, Instant};
/// 
/// let event = Event::from(("custom_event_name", Instant::now(), Instant::now()));
/// assert_eq!(event.kind(), EventType::Success)
/// ```
impl From<(&'static str, Instant, Instant)> for Event {
    fn from(value: (&'static str, Instant, Instant)) -> Self {
        Self::new(value.0, value.1, value.2, EventType::Success)
    }
}

/// Creates successfull event from tuple of name, start time and duration
/// 
/// ```rust
/// use profusion::{Event, Instant, Duration};
/// 
/// let event = Event::from(("custom_event_name", Instant::now(), Duration::from_millis(100)));
/// assert_eq!(event.latency(), Duration::from_millis(100))
/// ```
impl From<(&'static str, Instant, Duration)> for Event {
    fn from(value: (&'static str, Instant, Duration)) -> Self {
        Self::from((value.0, value.1, value.1 + value.2))
    }
}

/// Creates event based on IO error
/// 
/// ```rust
/// use profusion::{Event, EventType, Instant};
/// use std::io::{Error, ErrorKind};
/// 
/// let event_timeout = Event::from(("timeout_event", Instant::now(), Instant::now(), Error::from(ErrorKind::TimedOut)));
/// let event_error = Event::from(("error_event", Instant::now(), Instant::now(), Error::from(ErrorKind::AddrInUse)));
///
/// assert_eq!(event_timeout.kind(), EventType::Timeout);
/// assert_eq!(event_error.kind(), EventType::Error);
/// ```
impl From<(&'static str, Instant, Instant, std::io::Error)> for Event {
    fn from(value: (&'static str, Instant, Instant, std::io::Error)) -> Self {
        let kind = match value.3.kind() {
            std::io::ErrorKind::TimedOut => EventType::Timeout,
            _ => EventType::Error
        };

        Self::new(value.0, value.1, value.2, kind)
    }
}

/// Equality implementation for Event with time delta of 1ms
/// 
/// Events are going to be equal when start or end time is within 1ms of each other
///
/// # Example
/// ```
/// # use profusion::{Instant, Duration, Event};
///
/// let first_time = Instant::now();
/// let first_with_below_ms = first_time + Duration::from_micros(999);
/// 
/// assert_eq!(
///    Event::from(("delta_time", first_time, Duration::from_millis(100))),
///    Event::from(("delta_time", first_with_below_ms, Duration::from_millis(100))),
/// )
/// ```
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
            && instant_eq_with_delta(&self.started_at, &other.started_at, &EVENT_DELTA)
            && instant_eq_with_delta(&self.finished_at, &other.finished_at, &EVENT_DELTA)
            && self.kind.eq(&other.kind)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_objects::FakeProcessor;
    use std::time::{Duration, Instant};

    static NANOSECOND: Duration = Duration::from_nanos(1);

    fn create_time_pair(duration: Option<Duration>) -> (Instant, Instant) {
        let duration = match duration {
            Some(duration) => duration,
            None => Duration::from_secs(1)
        };

        let time = Instant::now();

        (time, time + duration)
    }

    #[test]
    fn calculates_latency_from_instant_difference() {
        let (start, end) = create_time_pair(Some(Duration::from_millis(100)));

        let event = Event::new(
            "something",
            start,
            end,
            EventType::Success,
        );

        assert_eq!(event.latency(), Duration::from_millis(100));
    }

    #[test]
    fn events_are_equal_when_all_properties_are_the_same() {
        let (start, end) = create_time_pair(None);

        assert_eq!(
            Event::success("event1", start, end),
            Event::success("event1", start, end)
        );
    }

    #[test]
    fn events_are_not_equal_when_name_differs() {
        let (start, end) = create_time_pair(None);

        assert_ne!(
            Event::success("event1", start, end),
            Event::success("event2", start, end)
        );
    }

    #[test]
    fn events_are_not_equal_when_start_time_drift_is_more_then_1ms() {
        let (start, end) = create_time_pair(None);

        assert_ne!(
            Event::success("event1", start, end),
            Event::success("event1", start + EVENT_DELTA + NANOSECOND, end)
        );
    }

    #[test]
    fn events_are_not_equal_when_finish_time_drift_is_more_then_1ms() {
        let (start, end) = create_time_pair(None);

        assert_ne!(
            Event::success("event1", start, end),
            Event::success("event1", start, end + EVENT_DELTA + NANOSECOND)
        );
    }

    #[test]
    fn events_are_not_equal_when_kind_differs() {
        let (start, end) = create_time_pair(None);

        assert_ne!(
            Event::success("event1", start, end),
            Event::error("event1", start, end)
        );
    }

    #[test]
    fn events_are_equal_when_time_drift_is_less_then_equals_1ms() {
        let (start, end) = create_time_pair(None);

        assert_eq!(
            Event::new("event1", start + EVENT_DELTA, end, EventType::Success),
            Event::new("event1", start, end + EVENT_DELTA - NANOSECOND, EventType::Success)
        );
    }

    #[test]
    fn reports_multiple_event_types_into_event_processor() {
        let time = Instant::now();

        let events = [
            Event::success(
                "event1",
                time.clone(),
                time.clone() + Duration::from_millis(40),
            ),
            Event::error(
                "event2",
                time.clone() + Duration::from_millis(10),
                time.clone() + Duration::from_millis(20),
            ),
            Event::success(
                "event3",
                time.clone() + Duration::from_millis(10),
                time.clone() + Duration::from_millis(30)
            ),
            Event::timeout(
                "event4",
                time.clone() + Duration::from_millis(30),
                time.clone() + Duration::from_millis(60),
            ),
        ];

        let mut aggregate = FakeProcessor::new();

        events
            .iter()
            .for_each(|event| event.process(&mut aggregate));

        aggregate.verify(vec![
            ("success:event1", Duration::from_millis(40)),
            ("error:event2", Duration::from_millis(10)),
            ("success:event3", Duration::from_millis(20)),
            ("timeout:event4", Duration::from_millis(30)),
        ]);
    }
}
