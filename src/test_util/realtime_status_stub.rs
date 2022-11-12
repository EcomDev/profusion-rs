use crate::RealtimeStatus;

/// Stub for [`RealtimeStatus`]
///
/// Allows testing custom implementation of [`Limiter`]
///
/// # Examples
///
/// ```
/// use profusion::prelude::*;
/// use profusion::test_util::RealtimeStatusStub;
///
/// let limiter = MaxOperationsLimiter::new(200);
///
/// assert_eq!(Limit::None, limiter.apply(&RealtimeStatusStub::with_total(100)));
/// assert_eq!(Limit::Shutdown, limiter.apply(&RealtimeStatusStub::with_total(201)));
/// ```
///
/// [`Limiter`]: crate::Limiter
#[derive(Debug, Clone, Copy, Default)]
pub struct RealtimeStatusStub {
    connections: usize,
    operations: usize,
    total_operations: usize,
}

impl RealtimeStatusStub {
    /// Creates stub with provided number of connections
    ///
    /// # Arguments
    ///
    /// * `value`: number of connections to report
    ///
    /// returns: RealtimeStatusStub
    ///
    /// # Examples
    ///
    /// ```
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    /// let status = RealtimeStatusStub::with_connections(99);
    ///
    /// assert_eq!(status.connections(), 99);
    /// ```
    pub fn with_connections(value: usize) -> Self {
        Self {
            connections: value,
            ..Self::default()
        }
    }

    /// Creates stub with provided number current concurrent operations
    ///
    /// # Arguments
    ///
    /// * `value`: number of concurrent operations to set
    ///
    /// returns: RealtimeStatusStub
    ///
    /// # Examples
    ///
    /// ```
    /// use profusion::prelude::*;
    /// use profusion::test_util::RealtimeStatusStub;
    ///
    /// let status = RealtimeStatusStub::with_operations(394);
    ///
    /// assert_eq!(status.operations(), 394);
    /// ```
    pub fn with_operations(value: usize) -> Self {
        Self {
            operations: value,
            ..Self::default()
        }
    }

    /// Creates stub with provided number total completed operations
    ///
    /// # Arguments
    ///
    /// * `value`: number of total operations to set
    ///
    /// returns: RealtimeStatusStub
    ///
    /// # Examples
    ///
    /// ```
    /// use profusion::prelude::*;
    /// use profusion::RealtimeStatus;
    /// use profusion::test_util::RealtimeStatusStub;
    ///
    /// let status = RealtimeStatusStub::with_total(13001);
    ///
    /// assert_eq!(status.total_operations(), 13001);
    /// ```
    pub fn with_total(value: usize) -> Self {
        Self {
            total_operations: value,
            ..Self::default()
        }
    }
}

impl RealtimeStatus for RealtimeStatusStub {
    fn connections(&self) -> usize {
        self.connections
    }

    fn operations(&self) -> usize {
        self.operations
    }

    fn total_operations(&self) -> usize {
        self.total_operations
    }
}