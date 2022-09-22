use std::{collections::HashMap, time::{Duration, Instant}};
use std::cmp::Ordering;

use hdrhistogram::Histogram;
use std::collections::hash_map::{Entry, Iter};


use crate::report::{EventProcessor, EventProcessorBuilder};
use crate::time::DurationBucket;

pub struct AggregateBuilder {
    span: Duration,
    time_reference: Instant,
    max_latency: Duration
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct AggregateEvent {
    started: usize,
    success: usize,
    timeout: usize,
    error: usize,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AggregateBucket {
    name: &'static str,
    offset: Duration
}

impl PartialOrd for AggregateBucket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(other.name) {
            Some(Ordering::Equal) => {
                self.offset.partial_cmp(&other.offset)
            },
            value => value
        }
    }
}

impl Ord for AggregateBucket {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name.cmp(other.name) {
            Ordering::Equal => {
                self.offset.cmp(&other.offset)
            },
            value => value
        }
    }
}

impl AggregateEvent {
    pub fn new(started: usize, success: usize, timeout: usize, error: usize) -> Self {
        Self {
            started,
            success,
            timeout,
            error,
        }
    }

    fn record_started(&mut self) {
        self.started += 1;
    }

    fn record_error(&mut self) {
        self.error += 1;
    }

    fn record_success(&mut self) {
        self.success += 1;
    }

    fn record_timeout(&mut self) {
        self.timeout += 1;
    }

    fn merge(&mut self, other: Self) {
        self.started += other.started;
        self.success += other.success;
        self.error += other.error;
        self.timeout += other.timeout;
    }
}

impl AggregateBucket {
    pub fn new(name: &'static str, offset: Duration) -> Self {
        AggregateBucket {
            name,
            offset
        }
    }
}

pub struct AggregateEventProcessor {
    latencies: HashMap<&'static str, Histogram<u64>>,
    events: HashMap<AggregateBucket, AggregateEvent>,
    span: Duration,
    time_reference: Instant,
    max_latency: u64
}

impl AggregateEventProcessor {
    pub fn events(&self) -> Iter<'_, AggregateBucket, AggregateEvent>
    {
        self.events.iter()
    }

    pub fn latencies(&self) -> Iter<'_, &'static str, Histogram<u64>>
    {
        self.latencies.iter()
    }

    fn store_latency(&mut self, name: &'static str, start: &Instant, end: &Instant)
    {
        let duration = *end - *start;
        let max_latency = self.max_latency;
        let latency_histogram = self.latencies.entry(name).or_insert_with(|| Histogram::new_with_bounds(1, max_latency, 4).unwrap());
        latency_histogram.record(duration.as_micros() as u64).unwrap();
    }

    fn find_event_by_instant(&mut self, name: &'static str, time: &Instant) -> &mut AggregateEvent {
        self.events
            .entry(
                AggregateBucket::new(
                        name,
                        time.as_duration_bucket(&self.time_reference, &self.span)
                )
            )
            .or_default()
    }
}

impl EventProcessor for AggregateEventProcessor {
    fn process_success(&mut self, name: &'static str, start: Instant, end: Instant) {
        self.find_event_by_instant(name, &start).record_started();
        self.find_event_by_instant(name, &end).record_success();
        self.store_latency(name, &start, &end);
    }

    fn process_error(&mut self, name: &'static str, start: Instant, end: Instant) {
        self.find_event_by_instant(name, &start).record_started();
        self.find_event_by_instant(name, &end).record_error();
        self.store_latency(name, &start, &end);
    }

    fn process_timeout(&mut self, name: &'static str, start: Instant, end: Instant) {
        self.find_event_by_instant(name, &start).record_started();
        self.find_event_by_instant(name, &end).record_timeout();
        self.store_latency(name, &start, &end);
    }

    fn merge(&mut self, other: Self) {
        other.events.into_iter().for_each(|(key, value)| {
            self.events.entry(key).or_default().merge(value);
        });
        other.latencies.into_iter().for_each(|(key, value)| {
            match self.latencies.entry(key) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().add(value).unwrap()
                }
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        });
    }
}

impl Default for AggregateBuilder {
    fn default() -> Self {
        Self {
            span: Duration::from_millis(50),
            time_reference: Instant::now(),
            max_latency: Duration::from_secs(60)
        }
    }
}

impl AggregateBuilder
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time(self, time_reference: Instant) -> Self {
        Self { time_reference, ..self }
    }

    pub fn with_span(self, span: Duration) -> Self {
        Self { span, ..self }
    }

    pub fn with_max_latency(self, max_latency: Duration) -> Self {
        Self { max_latency, ..self }
    }
}

impl EventProcessorBuilder<AggregateEventProcessor> for AggregateBuilder {
    fn build(&self) -> AggregateEventProcessor
    {
        AggregateEventProcessor {
            latencies: HashMap::new(),
            events: HashMap::new(),
            span: self.span,
            time_reference: self.time_reference,
            max_latency: self.max_latency.as_millis() as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait InstantMod {
        fn after_ms(&self, offset: u64) -> Self;
    }

    impl InstantMod for Instant {
        fn after_ms(&self, offset: u64) -> Self {
            *self + Duration::from_millis(offset)
        }
    }

    impl AggregateBucket {
        fn create(name: &'static str, duration: u64) -> Self {
            AggregateBucket::new(name, Duration::from_millis(duration))
        }
    }

    #[test]
    fn returns_list_of_empty_events() {
        let aggregate = AggregateBuilder::new()
            .build();

        assert_eq!(
            aggregate.events().collect::<Vec<_>>(),
            vec![]
        )
    }

    #[test]
    fn aggregates_events_by_default_time_span() {
        let reference = Instant::now();
        let mut aggregate = AggregateBuilder::new()
            .with_time(reference)
            .build();


        generate_events(reference, &mut aggregate);

        itertools::assert_equal(
            sorted_events(aggregate),
            vec![
                (AggregateBucket::create("user1", 0), AggregateEvent::new(1, 0, 0, 0)),
                (AggregateBucket::create("user1", 50), AggregateEvent::new(1, 0, 0, 2)),
                (AggregateBucket::create("user1", 100), AggregateEvent::new(1, 1, 0, 0)),
                (AggregateBucket::create("user1", 150), AggregateEvent::new(1, 0, 0, 0)),
                (AggregateBucket::create("user1", 200), AggregateEvent::new(0, 0, 0, 1)),
                (AggregateBucket::create("user2", 250), AggregateEvent::new(2, 0, 0, 0)),
                (AggregateBucket::create("user2", 300), AggregateEvent::new(0, 0, 1, 1)),
            ]
        );
    }

    #[test]
    fn aggregates_events_by_custom_time_span() {
        let reference = Instant::now();
        let mut aggregate = AggregateBuilder::new()
            .with_time(reference)
            .with_span(Duration::from_millis(100))
            .build();


        generate_events(reference, &mut aggregate);

        itertools::assert_equal(
            sorted_events(aggregate),
            vec![
                (AggregateBucket::create("user1", 0), AggregateEvent::new(2, 0, 0, 1)),
                (AggregateBucket::create("user1", 100), AggregateEvent::new(1, 1, 0, 1)),
                (AggregateBucket::create("user1", 200), AggregateEvent::new(1, 0, 0, 1)),
                (AggregateBucket::create("user2", 300), AggregateEvent::new(2, 0, 1, 1)),
            ]
        );
    }

    #[test]
    fn aggregates_latencies_from_all_events_for_each_user() {
        let reference = Instant::now();
        let mut aggregate = AggregateBuilder::new()
            .with_time(reference)
            .build();

        generate_events(reference, &mut aggregate);

        itertools::assert_equal(
            sorted_latencies(&mut aggregate),
            vec![
                ("user1", (1000, 28000, 28000)),
                ("user2", (25000, 28000, 28000))
            ]
        )
    }

    #[test]
    fn merges_aggregates() {
        let reference = Instant::now();
        let builder = AggregateBuilder::new()
            .with_time(reference)
            .with_span(Duration::from_millis(100));

        let mut root_aggregate = builder.build();
        let mut first_aggregate = builder.build();
        let mut second_aggregate = builder.build();
        let mut third_aggregate = builder.build();

        generate_events(reference, &mut first_aggregate);
        generate_events(reference, &mut second_aggregate);
        generate_events(reference, &mut third_aggregate);

        root_aggregate.merge(first_aggregate);
        root_aggregate.merge(second_aggregate);
        root_aggregate.merge(third_aggregate);


        itertools::assert_equal(
            sorted_latencies(&mut root_aggregate),
            vec![
                ("user1", (1000, 28000, 28000)),
                ("user2", (25000, 28000, 28000))
            ]
        );

        itertools::assert_equal(
            sorted_events(root_aggregate),
            vec![
                (AggregateBucket::create("user1", 0), AggregateEvent::new(6, 0, 0, 3)),
                (AggregateBucket::create("user1", 100), AggregateEvent::new(3, 3, 0, 3)),
                (AggregateBucket::create("user1", 200), AggregateEvent::new(3, 0, 0, 3)),
                (AggregateBucket::create("user2", 300), AggregateEvent::new(6, 0, 3, 3)),
            ]
        );
    }

    fn sorted_latencies(aggregate: &mut AggregateEventProcessor) -> Vec<(&str, (u64, u64, u64))> {
        let mut latencies: Vec<(_, (_, _, _))> = aggregate.latencies().map(
            |(key, value)|
                (*key, (value.min(), value.max(), value.value_at_percentile(95f64)))
        ).collect();

        latencies.sort_by(|(keyLeft, _), (keyRight, _)| keyLeft.cmp(keyRight));
        latencies
    }

    fn sorted_events(aggregate: AggregateEventProcessor) -> Vec<(AggregateBucket, AggregateEvent)> {
        let mut events: Vec<(_, _)> = aggregate.events.into_iter().collect();
        events.sort_by(|(bucketLeft, _), (bucketRight, _ )| bucketLeft.cmp(bucketRight));
        events
    }

    fn generate_events(reference: Instant, aggregate: &mut AggregateEventProcessor) {
        aggregate.process_error("user1", reference.after_ms(10), reference.after_ms(25));
        aggregate.process_error("user1", reference.after_ms(26), reference.after_ms(51));
        aggregate.process_success("user1", reference.after_ms(75), reference.after_ms(76));
        aggregate.process_error("user1", reference.after_ms(150), reference.after_ms(178));
        aggregate.process_timeout("user2", reference.after_ms(250), reference.after_ms(278));
        aggregate.process_error("user2", reference.after_ms(251), reference.after_ms(276));
    }
}