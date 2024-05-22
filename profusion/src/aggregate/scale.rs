use std::time::Duration;

/// Scale for reports
///
/// Defaults to microseconds
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum AggregateScale {
    Nanoseconds,
    #[default]
    Microseconds,
    Milliseconds,
    Seconds,
}

const NANOS_PER_SEC: u64 = 1_000_000_000;
const MICROS_PER_SEC: u64 = 1_000_000;
const MILLIS_PER_SEC: u64 = 1_000;

impl AggregateScale {
    /// Converts duration into single u64 value
    ///
    /// It is used by report aggregator to calculate stats based on chosen scale
    pub fn duration_to_value(&self, duration: Duration) -> u64 {
        match self {
            Self::Nanoseconds => {
                duration.as_secs() * NANOS_PER_SEC + duration.subsec_nanos() as u64
            }
            Self::Microseconds => {
                duration.as_secs() * MICROS_PER_SEC + duration.subsec_micros() as u64
            }
            Self::Milliseconds => {
                duration.as_secs() * MILLIS_PER_SEC + duration.subsec_millis() as u64
            }
            Self::Seconds => duration.as_secs(),
        }
    }

    /// Converts aggregated value into duration
    ///
    /// Can be used to generate duration based on storage report
    pub fn value_to_duration(&self, value: u64) -> Duration {
        match self {
            AggregateScale::Nanoseconds => Duration::from_nanos(value),
            AggregateScale::Microseconds => Duration::from_micros(value),
            AggregateScale::Milliseconds => Duration::from_millis(value),
            AggregateScale::Seconds => Duration::from_secs(value),
        }
    }

    /// Converts aggregated value of latency to duration
    ///
    /// Can be used to generate duration based on storage report
    pub fn aggregate_to_duration(&self, value: f64) -> Duration {
        let (seconds, nanos) = match self {
            AggregateScale::Nanoseconds => (
                value as u64 / NANOS_PER_SEC,
                (value as u64 % NANOS_PER_SEC) as u32,
            ),
            AggregateScale::Microseconds => (
                value as u64 / MICROS_PER_SEC,
                ((value % MICROS_PER_SEC as f64) * 1_000f64) as u32,
            ),
            AggregateScale::Milliseconds => (
                value as u64 / MILLIS_PER_SEC,
                ((value % MILLIS_PER_SEC as f64) * 1_000_000f64) as u32,
            ),
            AggregateScale::Seconds => (
                value as u64,
                (value * NANOS_PER_SEC as f64 % NANOS_PER_SEC as f64) as u32,
            ),
        };

        Duration::new(seconds, nanos)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::aggregate::scale::AggregateScale;

    #[test]
    fn defaults_to_microseconds() {
        assert_eq!(AggregateScale::default(), AggregateScale::Microseconds)
    }

    #[test]
    fn converts_duration_to_value() {
        assert_eq!(
            AggregateScale::Nanoseconds.duration_to_value(Duration::new(2, 100_000_000)),
            2_100_000_000
        );

        assert_eq!(
            AggregateScale::Microseconds.duration_to_value(Duration::new(29, 20_000)),
            29_000_020
        );

        assert_eq!(
            AggregateScale::Milliseconds
                .duration_to_value(Duration::new(25, 100_000_000)),
            25_100
        );

        assert_eq!(
            AggregateScale::Seconds.duration_to_value(Duration::new(25, 20)),
            25
        );
    }

    #[test]
    fn converts_value_to_duration() {
        assert_eq!(
            AggregateScale::Nanoseconds.value_to_duration(2_100_000_000),
            Duration::new(2, 100_000_000)
        );

        assert_eq!(
            AggregateScale::Microseconds.value_to_duration(29_000_020),
            Duration::new(29, 20_000)
        );

        assert_eq!(
            AggregateScale::Milliseconds.value_to_duration(25_100),
            Duration::new(25, 100_000_000)
        );

        assert_eq!(
            AggregateScale::Seconds.value_to_duration(25),
            Duration::new(25, 0)
        );
    }

    #[test]
    fn converts_aggregate_to_duration() {
        assert_eq!(
            AggregateScale::Nanoseconds.aggregate_to_duration(1_000_002_100.001),
            Duration::new(1, 2_100)
        );

        assert_eq!(
            AggregateScale::Microseconds.aggregate_to_duration(29_000_020.01),
            Duration::new(29, 20_010)
        );

        assert_eq!(
            AggregateScale::Milliseconds.aggregate_to_duration(25_100.04),
            Duration::new(25, 100_040_000)
        );

        assert_eq!(
            AggregateScale::Seconds.aggregate_to_duration(25.0122),
            Duration::new(25, 12_200_000)
        );
    }
}
