use super::AggregateRecorder;

use std::time::Instant;

#[cfg(test)]
use std::time::Duration;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EventType {
    Success,
    Error,
    Timeout,
}

pub(super) struct Event {
    name: &'static str,
    started_at: Instant,
    finished_at: Instant,
    kind: EventType,
}

impl Event {
    fn new(name: &'static str, started_at: Instant, finished_at: Instant, kind: EventType) -> Self {
        Self {
            name,
            started_at,
            finished_at,
            kind,
        }
    }

    #[cfg(test)]
    pub(super) fn latency(&self) -> Duration {
        self.finished_at - self.started_at
    }

    #[cfg(test)]
    pub(super) fn kind(&self) -> EventType {
        self.kind
    }

    #[cfg(test)]
    pub(super) fn name(&self) -> &'static str {
        self.name
    }

    pub(super) fn aggregate<A: AggregateRecorder>(&self, aggregate: &mut A) {
        match self.kind {
            EventType::Success => {
                aggregate.record_success(self.name, self.started_at, self.finished_at)
            }
            EventType::Timeout => {
                aggregate.record_timeout(self.name, self.started_at, self.finished_at)
            }
            EventType::Error => {
                aggregate.record_error(self.name, self.started_at, self.finished_at)
            }
        }
    }
}

impl From<(&'static str, Instant, Instant)> for Event {
    fn from(value: (&'static str, Instant, Instant)) -> Self {
        Self::new(value.0, value.1, value.2, EventType::Success)
    }
}

impl From<(&'static str, Instant, Instant, EventType)> for Event {
    fn from(value: (&'static str, Instant, Instant, EventType)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    struct FakeAggregate {
        items: Vec<(String, Duration)>,
    }

    impl FakeAggregate {
        fn new() -> Self {
            Self { items: vec![] }
        }
    }

    impl AggregateRecorder for FakeAggregate {
        fn record_timeout(&mut self, name: &'static str, start: Instant, end: Instant) {
            self.items.push((format!("timeout:{}", name), end - start));
        }

        fn record_error(&mut self, name: &'static str, start: Instant, end: Instant) {
            self.items.push((format!("error:{}", name), end - start));
        }

        fn record_success(&mut self, name: &'static str, start: Instant, end: Instant) {
            self.items.push((format!("success:{}", name), end - start));
        }
    }

    #[test]
    fn it_calculates_latency_from_instant_difference() {
        let start = Instant::now();
        let event = Event::new(
            "something",
            start,
            start + Duration::from_millis(100),
            EventType::Success,
        );

        assert_eq!(event.latency(), Duration::from_millis(100));
    }

    #[test]
    fn it_reports_multiple_event_types_into_aggregate() {
        let time = Instant::now();

        let mut aggregate = FakeAggregate::new();

        Event::from((
            "event1",
            time.clone(),
            time.clone() + Duration::from_millis(40),
        ))
        .aggregate(&mut aggregate);

        Event::from((
            "event2",
            time.clone() + Duration::from_millis(10),
            time.clone() + Duration::from_millis(20),
            EventType::Error,
        ))
        .aggregate(&mut aggregate);

        Event::from((
            "event3",
            time.clone() + Duration::from_millis(10),
            time.clone() + Duration::from_millis(30),
            EventType::Success,
        ))
        .aggregate(&mut aggregate);

        Event::from((
            "event4",
            time.clone() + Duration::from_millis(30),
            time.clone() + Duration::from_millis(60),
            EventType::Timeout,
        ))
        .aggregate(&mut aggregate);

        assert_eq!(
            aggregate.items,
            vec![
                ("success:event1".into(), Duration::from_millis(40)),
                ("error:event2".into(), Duration::from_millis(10)),
                ("success:event3".into(), Duration::from_millis(20)),
                ("timeout:event4".into(), Duration::from_millis(30)),
            ]
        );
    }
}
