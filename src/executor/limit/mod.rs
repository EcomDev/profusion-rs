//! Collections of limiters for executor.

mod concurrency;
mod max_duration;
mod max_operations;

pub use self::{
    concurrency::ConcurrencyLimiter, max_duration::MaxDurationLimiter,
    max_operations::MaxOperationsLimiter,
};

use crate::RealtimeStatus;
use std::io::{Error, ErrorKind, Result};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Copy, Clone)]
pub struct CompoundLimiter<L, R>(L, R);

impl<L, R> CompoundLimiter<L, R>
where
    L: Limiter,
    R: Limiter,
{
    fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<L, R> Limiter for CompoundLimiter<L, R>
where
    L: Limiter,
    R: Limiter,
{
    fn apply<S: RealtimeStatus>(&self, status: &S) -> Limit {
        match self.0.apply(status) {
            Limit::None => self.1.apply(status),
            limit => limit,
        }
    }
}

pub trait Limiter: Sized {
    fn apply<S: RealtimeStatus>(&self, status: &S) -> Limit;

    fn with<L: Limiter>(self, another: L) -> CompoundLimiter<Self, L> {
        CompoundLimiter::new(self, another)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Limit {
    None,
    Wait(Duration),
    Shutdown,
}

impl Limit {
    pub async fn process(self) -> Result<()> {
        match self {
            Self::Wait(duration) => sleep(duration).await,
            Self::Shutdown => return Err(Error::from(ErrorKind::Interrupted)),
            Self::None => {}
        };

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use super::{ErrorKind, Limit};
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn delays_for_specified_duration_when_is_wait_limit() {
        let limit = Limit::Wait(Duration::from_millis(5));

        let start = Instant::now();

        limit.process().await.unwrap();

        let spend_time = start.elapsed();

        assert!(spend_time >= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn terminates_process_execution_by_returning_error() {
        let limit = Limit::Shutdown;

        let result = limit.process().await.unwrap_err();

        assert_eq!(result.kind(), ErrorKind::Interrupted)
    }

    #[tokio::test]
    async fn imidiatelly_continues_execution_when_no_limit_is_set() {
        let limit = Limit::None;

        let start = Instant::now();

        limit.process().await.unwrap();

        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(1));
    }
}
