use super::{Limit, Limiter};

#[derive(Clone, Copy)]
pub struct MaxOperationsLimiter {
    max_operations: usize,
}

impl MaxOperationsLimiter {
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
    use super::{Limit, Limiter, MaxOperationsLimiter};
    use crate::test_util::RealtimeStatusStub;

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
