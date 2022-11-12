use super::{Limit, Limiter};

/// Limiter that shuts down connections when limit of operations is reached
#[derive(Clone, Copy)]
pub struct MaxOperationsLimiter {
    max_operations: usize,
}

impl MaxOperationsLimiter {
    /// Creates an instance of [`MaxOperationsLimiter`]
    ///
    /// # Arguments
    ///
    /// * `max_operations`: maximum number of operations after which to shutdown load test
    ///
    /// # Examples
    ///
    /// ```
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    ///
    /// let limit = MaxOperationsLimiter::new(200);
    ///
    /// assert_eq!(limit.apply(&RealtimeStatusStub::with_total(199)), Limit::None);
    /// assert_eq!(limit.apply(&RealtimeStatusStub::with_total(200)), Limit::Shutdown);
    /// ```
    pub fn new(max_operations: usize) -> Self {
        Self { max_operations }
    }
}

impl Limiter for MaxOperationsLimiter {
    fn apply<S: crate::RealtimeStatus>(&self, status: &S) -> Limit {
        match status.total_operations() >= self.max_operations {
            true => Limit::Shutdown,
            false => Limit::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::RealtimeStatusStub;

    use super::{Limit, Limiter, MaxOperationsLimiter};

    #[test]
    fn does_not_terminate_when_condition_is_not_reached() {
        let limit = MaxOperationsLimiter::new(200);

        assert_eq!(
            limit.apply(&RealtimeStatusStub::with_total(199)),
            Limit::None
        );
    }

    #[test]
    fn terminates_execution_when_max_operations_is_reached() {
        let limit = MaxOperationsLimiter::new(200);

        assert_eq!(
            limit.apply(&RealtimeStatusStub::with_total(200)),
            Limit::Shutdown
        );
    }

    #[test]
    fn terminates_execution_when_total_operations_exceeds_limit() {
        let limit = MaxOperationsLimiter::new(200);

        assert_eq!(
            limit.apply(&RealtimeStatusStub::with_total(220)),
            Limit::Shutdown
        );
    }
}
