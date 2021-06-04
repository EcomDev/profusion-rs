use std::time::{Duration, Instant};

use crate::executor::limit::{Limit, Limiter};

#[derive(Debug, Clone, Copy)]
pub struct MaxDurationLimiter {
    start: Instant,
    max_duration: Duration,
}

impl MaxDurationLimiter {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            max_duration,
            start: Instant::now(),
        }
    }

    pub fn with_delay(self, delay: Duration) -> Self {
        Self {
            start: self.start + delay,
            ..self
        }
    }
}

impl Limiter for MaxDurationLimiter {
    fn apply<S: crate::RealtimeStatus>(&self, _status: &S) -> Limit {
        match self.start.elapsed() > self.max_duration {
            true => Limit::Shutdown,
            false => Limit::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_objects::RealtimeStatusStub;

    use super::{Duration, Limit, Limiter, MaxDurationLimiter};

    #[test]
    fn does_not_limit_when_enough_time_is_left() {
        let limiter = MaxDurationLimiter::new(Duration::from_secs(60));

        assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::None)
    }

    #[test]
    fn terminates_execution_when_duration_is_longer_then_start_time() {
        let mut limit = MaxDurationLimiter::new(Duration::from_secs(40));

        limit.start -= Duration::from_secs(40);

        assert_eq!(limit.apply(&RealtimeStatusStub::default()), Limit::Shutdown);
    }

    #[test]
    fn keeps_running_past_initial_duration_when_delay_is_provided() {
        let mut limit = MaxDurationLimiter::new(Duration::from_secs(40))
            .with_delay(Duration::from_secs(10));

        limit.start -= Duration::from_secs(40);

        assert_eq!(limit.apply(&RealtimeStatusStub::default()), Limit::None);
    }
}
