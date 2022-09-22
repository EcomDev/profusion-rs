use super::{report::EventProcessor, RealtimeStatus};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct RealtimeStatusStub {
    connections: usize,
    operations: usize,
    total_operations: usize,
}

impl Default for RealtimeStatusStub {
    fn default() -> Self {
        Self {
            connections: 0,
            operations: 0,
            total_operations: 0,
        }
    }
}

impl RealtimeStatusStub {
    pub fn with_connections(value: usize) -> Self {
        Self {
            connections: value,
            ..Self::default()
        }
    }

    pub fn with_operations(value: usize) -> Self {
        Self {
            operations: value,
            ..Self::default()
        }
    }

    pub fn with_total(value: usize) -> Self {
        Self {
            total_operations: value,
            ..Self::default()
        }
    }
}

impl RealtimeStatus for RealtimeStatusStub {
    fn connections(&self) -> usize {
        self.connections
    }

    fn operations(&self) -> usize {
        self.operations
    }

    fn total_operations(&self) -> usize {
        self.total_operations
    }
}

pub struct FakeProcessor {
    items: Vec<(String, Duration)>,
}

impl FakeProcessor {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn verify(self, items: Vec<(&str, Duration)>) {
        assert_eq!(
            self.items
                .iter()
                .map(|(text, duration)| (text.as_str(), duration.to_owned()))
                .collect::<Vec<_>>(),
            items,
        );
    }

    pub fn verify_names(self, items: Vec<&str>) {
        assert_eq!(
            self.items
                .iter()
                .map(move |event| event.0.as_str())
                .collect::<Vec<_>>(),
            items
        );
    }
}

impl EventProcessor for FakeProcessor {
    fn process_success(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("success:{}", name), end - start));
    }

    fn process_error(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("error:{}", name), end - start));
    }

    fn process_timeout(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("timeout:{}", name), end - start));
    }

    fn merge(&mut self, mut other: Self) {
        self.items.append(&mut other.items);
    }
}
