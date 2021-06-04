use super::{Instant, DurationBucket, Duration};

impl DurationBucket for Instant {
    fn as_duration_bucket(&self, origin: &Instant, bucket_size: &Duration) -> Duration {
        let latency_nanos = (*self - *origin).as_nanos() as u64;
        let bucket_nanos = bucket_size.as_nanos() as u64;
        let buckets = latency_nanos / bucket_nanos;
        let remainder = latency_nanos % bucket_nanos;
        
        Duration::from_nanos(buckets * bucket_nanos + match remainder.cmp(&(bucket_nanos / 2)) {
            std::cmp::Ordering::Less => 0,
            _ => bucket_nanos
        })
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
                (time + Duration::from_micros(1499)).as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(1500)).as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(1999)).as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(2499)).as_duration_bucket(&time, &MILLISECOND),
                (time + Duration::from_micros(2560)).as_duration_bucket(&time, &MILLISECOND),
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
                (time + Duration::from_millis(999)).as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(1000)).as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(2999)).as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(3000)).as_duration_bucket(&time, &two_seconds),
                (time + Duration::from_millis(7199)).as_duration_bucket(&time, &two_seconds),
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
}