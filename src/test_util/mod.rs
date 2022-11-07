use std::time::{Duration, Instant};

pub use assert_events::*;
pub use assert_instant::*;
pub use fake_processor::*;

use super::RealtimeStatus;

mod fake_processor;
mod assert_instant;
mod assert_events;

/// Stub for [`RealtimeStatus`]
///
/// Allows testing custom implementation of [`Limiter`]
///
/// # Examples
///
///
///
/// [`Limiter`]: crate::Limiter
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