use tokio::time::Instant;

/// Clock implementation with usage tokio internal library time management
///
/// You can use [`tokio::time::pause`] to stop clock and [`tokio::time::advance`] in order
/// to affect values returned by [`Clock::now`] function
pub struct Clock;

impl Clock
{
    /// Returns current monotonic
    /// Affected by tokio time library testing helpers
    ///
    /// # Example
    /// ```
    /// use tokio::time::{advance, pause, resume};
    /// use profusion::time::{Clock, Instant, Duration, InstantOffset};
    ///
    /// #[tokio::main(flavor = "current_thread", start_paused = true)]
    /// async fn main() {
    ///     let time = Clock::now();
    ///     assert_eq!(Clock::now(), time);
    ///     advance(Duration::from_millis(10)).await;
    ///     assert_eq!(Clock::now(), time.with_millis(10));
    ///     resume();
    ///     assert_ne!(Clock::now(), time.with_millis(10));
    /// }
    /// ```
    pub fn now() -> std::time::Instant {
        Instant::now().into_std()
    }
}