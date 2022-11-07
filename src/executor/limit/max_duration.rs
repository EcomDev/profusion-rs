use std::time::{Duration, Instant};

use crate::executor::limit::{Limit, Limiter};

///
#[derive(Clone, Copy)]
pub struct MaxDurationLimiter {
    start: Instant,
    max_duration: Duration,
}

impl MaxDurationLimiter {
    /// Creates a `MaxDurationLimiter` instance
    ///
    /// # Arguments
    ///
    /// * `max_duration`: time delay for internal limiter timer
    ///
    /// returns: MaxDurationLimiter
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread::sleep;
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    /// use std::time::Duration;
    ///
    /// let limiter = MaxDurationLimiter::new(Duration::from_millis(10));
    ///
    /// sleep(Duration::from_millis(9));
    /// assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::None);
    /// sleep(Duration::from_millis(2));
    /// assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::Shutdown);
    /// ```
    pub fn new(max_duration: Duration) -> Self {
        Self {
            max_duration,
            start: Instant::now(),
        }
    }

    /// Adds delay to max duration limiter
    ///
    /// Shifts internal timer by provided delay
    ///
    /// # Arguments
    ///
    /// * `delay`: time delay for internal limiter timer
    ///
    /// returns: MaxDurationLimiter
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread::sleep;
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    /// use std::time::Duration;
    ///
    /// let limiter = MaxDurationLimiter::new(Duration::from_millis(10))
    ///     .with_delay(Duration::from_millis(5));
    ///
    /// assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::None);
    /// sleep(Duration::from_millis(11));
    /// assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::None);
    /// sleep(Duration::from_millis(5));
    /// assert_eq!(limiter.apply(&RealtimeStatusStub::default()), Limit::Shutdown);
    /// ```
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
    use crate::test_util::RealtimeStatusStub;

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
