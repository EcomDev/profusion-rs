use super::{Duration, DurationBucket, Instant, InstantOffset};

impl DurationBucket for Instant {
    #[inline]
    fn as_duration_bucket(&self, origin: &Instant, bucket_size: &Duration) -> Duration {
        let latency_nanos = (*self - *origin).as_nanos() as u64;
        let bucket_nanos = bucket_size.as_nanos() as u64;
        let buckets = latency_nanos / bucket_nanos;
        let remainder = latency_nanos % bucket_nanos;

        Duration::from_nanos(
            buckets * bucket_nanos
                + match remainder.cmp(&(bucket_nanos / 2)) {
                std::cmp::Ordering::Less => 0,
                _ => bucket_nanos,
            },
        )
    }
}

impl InstantOffset for Instant {
    #[inline]
    fn with_millis(&self, value: u64) -> Self {
        *self + Duration::from_millis(value)
    }

    #[inline]
    fn with_micros(&self, value: u64) -> Self {
        *self + Duration::from_micros(value)
    }

    #[inline]
    fn with_nanos(&self, value: u64) -> Self {
        *self + Duration::from_nanos(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MILLISECOND: Duration = Duration::from_millis(1);
    const SECOND: Duration = Duration::from_secs(1);

    #[test]
    fn returns_zero_when_no_time_elapsed() {
        let time = Instant::now();
        assert_eq!(
            time.as_duration_bucket(&time, &MILLISECOND),
            Duration::default()
        );
    }

    #[test]
    fn puts_into_lower_millisecond_bucket() {
        let time = Instant::now();
        assert_eq!(
            (time + Duration::from_micros(499)).as_duration_bucket(&time, &MILLISECOND),
            Duration::from_millis(0)
        );
    }

    #[test]
    fn puts_into_higher_millisecond_bucket() {
        let time = Instant::now();
        assert_eq!(
            (time + Duration::from_micros(500)).as_duration_bucket(&time, &MILLISECOND),
            MILLISECOND
        );
    }

    #[test]
    fn puts_into_next_millisecond_bucket() {
        let time = Instant::now();
        assert_eq!(
            (time + Duration::from_micros(1499)).as_duration_bucket(&time, &MILLISECOND),
            Duration::from_millis(1)
        );
    }

    #[test]
    fn spreads_millisecond_buckets() {
        let time = Instant::now();

        assert_eq!(
            vec![
                (time + Duration::from_micros(1499))
                    .as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(1500))
                    .as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(1999))
                    .as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(2499))
                    .as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(2560))
                    .as_duration_bucket(&time, &MILLISECOND),
            ],
            vec![
                Duration::from_millis(1),
                Duration::from_millis(2),
                Duration::from_millis(2),
                Duration::from_millis(2),
                Duration::from_millis(3),
            ]
        );
    }

    #[test]
    fn spreads_second_buckets() {
        let time = Instant::now();
        let two_seconds: Duration = SECOND * 2;
        assert_eq!(
            vec![
                (time + Duration::from_millis(999))
                    .as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(1000))
                    .as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(2999))
                    .as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(3000))
                    .as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(7199))
                    .as_duration_bucket(&time, &two_seconds),
            ],
            vec![
                Duration::from_secs(0),
                Duration::from_secs(2),
                Duration::from_secs(2),
                Duration::from_secs(4),
                Duration::from_secs(8),
            ]
        );
    }

    #[test]
    fn spreads_50ms_buckets() {
        let time = Instant::now();
        let bucket: Duration = Duration::from_millis(50);
        assert_eq!(
            vec![
                (time + Duration::from_millis(0)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(24)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(25)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(49)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(50)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(74)).as_duration_bucket(&time, &bucket),
                (time + Duration::from_millis(75)).as_duration_bucket(&time, &bucket),
            ],
            vec![
                Duration::from_millis(0),
                Duration::from_millis(0),
                Duration::from_millis(50),
                Duration::from_millis(50),
                Duration::from_millis(50),
                Duration::from_millis(50),
                Duration::from_millis(100),
            ]
        );
    }

    #[test]
    fn adds_milliseconds() {
        let time = Instant::now();
        assert_eq!(
            time.with_millis(430),
            time + Duration::new(0, 430_000_000)
        )
    }

    #[test]
    fn adds_microseconds() {
        let time = Instant::now();
        assert_eq!(
            time.with_micros(430),
            time + Duration::new(0, 430_000)
        )
    }

    #[test]
    fn adds_nanoseconds() {
        let time = Instant::now();
        assert_eq!(
            time.with_nanos(120),
            time + Duration::new(0, 120)
        )
    }
}
