use std::time::Duration;

use crate::start_time::StartTime;

use super::AggregateScale;

#[derive(Clone, Copy, Debug)]
pub struct AggregateSettings {
    window: Duration,
    scale: AggregateScale,
    zero: StartTime,
}

impl AggregateSettings {
    /// Changes window for aggregation
    ///
    /// # Arguments
    ///
    /// * `window`: size of the window to storage data
    pub fn with_window(self, window: Duration) -> Self {
        Self { window, ..self }
    }

    /// Modifies zero point time for starting data collection
    ///
    /// # Arguments
    ///
    /// * `zero`: [`StartTime`](crate::metric::StartTime) to use as a starting point
    pub fn with_zero(self, zero: StartTime) -> Self {
        Self { zero, ..self }
    }

    pub fn with_scale(self, scale: AggregateScale) -> Self {
        Self { scale, ..self }
    }

    /// Returns current zero point
    pub fn zero(&self) -> &StartTime {
        &self.zero
    }

    /// Returns current window
    pub fn window(&self) -> &Duration {
        &self.window
    }

    /// Reporter
    pub fn scale(&self) -> AggregateScale {
        self.scale
    }
}

impl Default for AggregateSettings {
    fn default() -> Self {
        Self {
            window: Duration::from_millis(100),
            scale: AggregateScale::default(),
            zero: StartTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn defaults_to_microseconds_scale_and_hundred_milliseconds_window() {
        let settings = AggregateSettings::default();
        assert_eq!(settings.scale, AggregateScale::Microseconds);
        assert_eq!(settings.window, Duration::from_millis(100));
    }

    #[test]
    fn allows_modifying_scale() {
        let settings =
            AggregateSettings::default().with_window(Duration::from_millis(500));

        assert_eq!(settings.window, Duration::from_millis(500));
    }

    #[test]
    fn allows_modifying_zero_point_from_standard_instant() {
        let now = Instant::now();

        let settings = AggregateSettings::default()
            .with_window(Duration::from_millis(20))
            .with_zero(StartTime::new(
                Duration::new(0, 0),
                now - Duration::from_millis(60),
            ));

        assert_eq!(
            settings.zero().window(settings.window()),
            Duration::from_millis(60)
        );
    }
}
