use std::{collections::HashMap, time::{Duration, Instant}};

use hdrhistogram::Histogram;
use std::collections::hash_map::Iter;


use crate::Event;

pub struct AggregateBuilder {
    span: Duration,
    time_reference: Instant
}

#[derive(Debug, PartialEq)]
pub struct AggregateEvent {
    started: usize,
    success: usize,
    timeout: usize,
    error: usize
}

impl AggregateEvent {
    pub fn new(started: usize, success: usize, timeout: usize, error: usize) -> Self {
        Self {
            started,
            success,
            timeout,
            error
        }
    }
}

pub struct Aggregate {
    latencies: HashMap<&'static str, Histogram<u64>>,
    events: HashMap<Duration, AggregateEvent>,
    span: Duration
}

impl Aggregate {
    pub fn events(&self) -> Iter<'_, Duration, AggregateEvent>
    {
        self.events.iter()
    }
}

impl Default for AggregateBuilder {
    fn default() -> Self {
        Self {
            span: Duration::from_millis(50),
            time_reference: Instant::now()
        }
    }
}

impl AggregateBuilder 
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> Aggregate
    {
        Aggregate {
            latencies: HashMap::new(),
            events: HashMap::new(),
            span: self.span
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_list_of_empty_events() {
        let aggregate = AggregateBuilder::new()
            .build();

        assert_eq!(
            aggregate.events().collect::<Vec<_>>(),
            vec![]
        )
    }
}