mod fake_processor;
mod assert_instant;
mod assert_events;

use super::{report::EventProcessor, RealtimeStatus};
use std::time::{Duration, Instant};
use crate::time::cmp_instant_with_delta;

pub use fake_processor::*;
pub use assert_instant::*;
pub use assert_events::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct RealtimeStatusStub {
    connections: usize,
    operations: usize,
    total_operations: usize,
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