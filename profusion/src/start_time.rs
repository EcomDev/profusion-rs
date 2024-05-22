use std::time::{Duration, SystemTime};

use tokio::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StartTime {
    timestamp: Duration,
    instant: Instant,
}

impl StartTime {
    pub fn now() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default(),
            instant: Instant::now(),
        }
    }

    pub fn new(timestamp: Duration, instant: std::time::Instant) -> Self {
        Self {
            timestamp,
            instant: Instant::from_std(instant),
        }
    }

    #[inline]
    pub fn window(&self, window: &Duration) -> Duration {
        let elapsed = self.instant.elapsed();
        let latency_nanos = elapsed.as_nanos();
        let bucket_nanos = window.as_nanos();

        if bucket_nanos == 0 {
            return elapsed;
        }

        let buckets = latency_nanos / bucket_nanos;
        let remainder = latency_nanos % bucket_nanos;
        let target_duration = buckets * bucket_nanos
            + match remainder.cmp(&(bucket_nanos / 2)) {
                std::cmp::Ordering::Less => 0,
                _ => bucket_nanos,
            };

        Duration::from_nanos(target_duration as u64)
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::advance;

    use super::*;

    const MILLISECOND: &'static Duration = &Duration::from_millis(1);
    const TWO_SECONDS: &'static Duration = &Duration::from_secs(2);
    const FIFTY_MILLISECONDS: &'static Duration = &Duration::from_millis(50);

    #[tokio::test(start_paused = true)]
    async fn returns_zero_for_not_passed_window_yet() {
        let time = StartTime::now();

        assert_eq!(time.window(MILLISECOND), Duration::default());
    }

    #[tokio::test(start_paused = true)]
    async fn puts_into_lower_millisecond_window() {
        let time = StartTime::now();
        advance(Duration::from_micros(499)).await;

        assert_eq!(time.window(MILLISECOND), Duration::from_millis(0));
    }

    #[tokio::test(start_paused = true)]
    async fn puts_into_higher_millisecond_window() {
        let time = StartTime::now();
        advance(Duration::from_micros(500)).await;

        assert_eq!(time.window(MILLISECOND), *MILLISECOND);
    }

    #[tokio::test(start_paused = true)]
    async fn puts_into_first_millisecond_windows() {
        let time = StartTime::now();
        advance(Duration::from_micros(1499)).await;
        assert_eq!(time.window(MILLISECOND), *MILLISECOND);
    }

    #[tokio::test(start_paused = true)]
    async fn spreads_millisecond_after_match() {
        let time = StartTime::now();

        advance(Duration::from_micros(1500)).await;
        assert_eq!(time.window(MILLISECOND), Duration::from_millis(2));

        advance(Duration::from_micros(499)).await;
        assert_eq!(time.window(MILLISECOND), Duration::from_millis(2));

        advance(Duration::from_micros(500)).await;
        assert_eq!(time.window(MILLISECOND), Duration::from_millis(2));

        advance(Duration::from_micros(61)).await;
        assert_eq!(time.window(MILLISECOND), Duration::from_millis(3));
    }

    #[tokio::test(start_paused = true)]
    async fn spreads_second_buckets() {
        let time = StartTime::now();

        advance(Duration::from_millis(999)).await;
        assert_eq!(time.window(TWO_SECONDS), Duration::from_secs(0));

        advance(Duration::from_millis(1)).await;
        assert_eq!(time.window(TWO_SECONDS), Duration::from_secs(2));

        advance(Duration::from_millis(1999)).await;
        assert_eq!(time.window(TWO_SECONDS), Duration::from_secs(2));

        advance(Duration::from_millis(1)).await;
        assert_eq!(time.window(TWO_SECONDS), Duration::from_secs(4));

        advance(Duration::from_millis(4199)).await;
        assert_eq!(time.window(TWO_SECONDS), Duration::from_secs(8));
    }

    #[tokio::test(start_paused = true)]
    async fn spreads_50ms_buckets() {
        let time = StartTime::now();
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(0));

        advance(Duration::from_millis(24)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(0));

        advance(Duration::from_millis(1)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(50));

        advance(Duration::from_millis(24)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(50));

        advance(Duration::from_millis(1)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(50));

        advance(Duration::from_millis(24)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(50));

        advance(Duration::from_millis(1)).await;
        assert_eq!(time.window(FIFTY_MILLISECONDS), Duration::from_millis(100));
    }

    #[tokio::test(start_paused = true)]
    async fn returns_duration_if_window_is_zero() {
        let time = StartTime::now();

        advance(Duration::from_millis(411)).await;
        assert_eq!(
            time.window(&Duration::new(0, 0)),
            Duration::from_millis(411)
        );
    }
}
