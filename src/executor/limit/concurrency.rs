use super::{Duration, Limit, Limiter};

/// Limiter that applies delay for operations to throttle execution
#[derive(Clone, Copy)]
pub struct ConcurrencyLimiter {
    max_concurrency: usize,
    wait_for: Duration,
}

impl ConcurrencyLimiter {
    /// Creates [`ConcurrencyLimiter`] with provided attributes
    ///
    /// # Arguments
    ///
    /// * `max_concurrency`: Maximum concurrent operations after which new operations will be delayed
    /// * `wait_for`: Delay for new operation when concurrency is reached
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    ///
    /// let limiter = ConcurrencyLimiter::new(20, Duration::from_millis(5));
    /// let status = RealtimeStatusStub::with_operations(25);
    ///
    /// assert_eq!(limiter.apply(&status), Limit::Wait(Duration::from_millis(5)));
    /// ```
    pub fn new(max_concurrency: usize, wait_for: Duration) -> Self {
        Self {
            max_concurrency,
            wait_for,
        }
    }
}

impl Limiter for ConcurrencyLimiter {
    fn apply<S: crate::RealtimeStatus>(&self, status: &S) -> Limit {
        match status.operations() >= self.max_concurrency {
            true => Limit::Wait(self.wait_for),
            false => Limit::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::RealtimeStatusStub;

    use super::{ConcurrencyLimiter, Duration, Limit, Limiter};

    #[test]
    fn no_limit_when_enough_when_operation_limit_is_not_reached() {
        let limiter = ConcurrencyLimiter::new(20, Duration::from_millis(1));

        assert_eq!(
            limiter.apply(&RealtimeStatusStub::with_connections(19)),
            Limit::None
        );
    }

    #[test]
    fn applies_wait_limit_when_operations_reach_max_value() {
        let limiter = ConcurrencyLimiter::new(20, Duration::from_millis(5));

        assert_eq!(
            limiter.apply(&RealtimeStatusStub::with_operations(20)),
            Limit::Wait(Duration::from_millis(5))
        );
    }

    #[test]
    fn applies_wait_limit_when_operations_exceeds_max_value() {
        let limiter = ConcurrencyLimiter::new(20, Duration::from_millis(5));

        let status = RealtimeStatusStub::with_operations(25);

        assert_eq!(
            limiter.apply(&status),
            Limit::Wait(Duration::from_millis(5))
        );
    }
}
