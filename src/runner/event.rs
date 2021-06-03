use super::EventProcessor;
use std::time::Instant;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EventType {
    Success,
    Error,
    Timeout,
}

#[derive(Debug)]
pub(super) struct Event<'a> {
    name: &'a str,
    started_at: Instant,
    finished_at: Instant,
    kind: EventType,
}

impl<'a> Event<'a> {
    fn new(
        name: &'a str,
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

    pub(super) fn process<P: EventProcessor<'a>>(&self, processor: &mut P) {
        match self.kind {
            EventType::Success => processor.process_success(
                self.name,
                self.started_at,
                self.finished_at,
            ),
            EventType::Timeout => processor.process_timeout(
                self.name,
                self.started_at,
                self.finished_at,
            ),
            EventType::Error => processor.process_error(
                self.name,
                self.started_at,
                self.finished_at,
            ),
        }
    }
}

impl<'a> From<(&'a str, Instant, Instant)> for Event<'a> {
    fn from(value: (&'a str, Instant, Instant)) -> Self {
        Self::new(value.0, value.1, value.2, EventType::Success)
    }
}

impl<'a> From<(&'a str, Instant, Instant, EventType)> for Event<'a> {
    fn from(value: (&'a str, Instant, Instant, EventType)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

#[cfg(test)]

mod tests {

    use super::*;
    use crate::FakeProcessor;
    use std::time::{Duration, Instant};

    impl Event<'_> {
        pub(in crate::runner) fn latency(&self) -> Duration {
            self.finished_at - self.started_at
        }

        pub(in crate::runner) fn kind(&self) -> EventType {
            self.kind
        }

        pub(in crate::runner) fn name(&self) -> &'_ str {
            self.name
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

    fn it_reports_multiple_event_types_into_record() {
        let time = Instant::now();

        let events = [
            Event::from((
                "event1",
                time.clone(),
                time.clone() + Duration::from_millis(40),
            )),
            Event::from((
                "event2",
                time.clone() + Duration::from_millis(10),
                time.clone() + Duration::from_millis(20),
                EventType::Error,
            )),
            Event::from((
                "event3",
                time.clone() + Duration::from_millis(10),
                time.clone() + Duration::from_millis(30),
                EventType::Success,
            )),
            Event::from((
                "event4",
                time.clone() + Duration::from_millis(30),
                time.clone() + Duration::from_millis(60),
                EventType::Timeout,
            )),
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
