use std::time::{Duration, Instant};

use crate::{Limit, Limiter};

#[derive(Debug, Clone, Copy)]
pub struct GradualConcurrencyLimiter {
    target_connections: usize,
    start: Instant,
    over: Duration,
}
