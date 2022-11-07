//! Collections of limiters for executor.

use std::{
    io::{Error, ErrorKind, Result},
    time::Duration,
};

use tokio::time::sleep;

use crate::RealtimeStatus;

pub use self::{
    concurrency::ConcurrencyLimiter, max_duration::MaxDurationLimiter,
    max_operations::MaxOperationsLimiter,
};

mod concurrency;
mod max_duration;
mod max_operations;

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

/// Limits execution task
///
/// During execution of load test it's [`apply`] function called to impose a limit if [`RealtimeStatus`] reaches some defined limit for its implementation
///
/// It is possible to combine multiple limiters with the help of [`with`] function
///
/// [`apply`]: Limiter::apply
/// [`with`]: Limiter::with
pub trait Limiter: Sized {
    ///
    ///
    /// # Arguments
    ///
    /// * `status`:
    ///
    /// returns: Limit
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn apply<S: RealtimeStatus>(&self, status: &S) -> Limit;

    ///
    ///
    /// # Arguments
    ///
    /// * `another`:
    ///
    /// returns: CompoundLimiter<Self, L>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn with<L: Limiter>(self, another: L) -> CompoundLimiter<Self, L> {
        CompoundLimiter::new(self, another)
    }
}

/// Result of [`Limiter::apply`] call
///
/// Used by load test runner in order to control throughput and duration of execution
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Limit {
    /// Indicates that no limit has been applied by [`Limiter`]
    None,
    /// Indicates that load test runner should impose waiting of [`Duration`]
    Wait(Duration),
    /// Indicates that load test runner task should be shutdown
    Shutdown,
}

impl Limit {
    pub(crate) async fn process(self) -> Result<()> {
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
    use std::time::{Duration, Instant};

    use super::{ErrorKind, Limit};

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
    async fn immediately_continues_execution_when_no_limit_is_set() {
        let limit = Limit::None;

        let start = Instant::now();

        limit.process().await.unwrap();

        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(1));
    }
}
