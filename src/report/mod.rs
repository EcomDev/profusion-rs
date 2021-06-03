mod realtime;

pub(crate) use realtime::RealtimeReport;
use std::time::Instant;

pub(crate) trait EventProcessor<'a> {
    fn process_success(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_error(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_timeout(&mut self, name: &'a str, start: Instant, end: Instant);
}

/// Reporter used in client context
///
/// Each operation and new connection is invoking appropriate methods on
/// implementor to notify it of start and finish of the related activity.
pub trait RealtimeReporter {
    /// Callled when operation has been started
    fn operation_started(&self) {}

    /// Callled when operation has been finished
    fn operation_finished(&self) {}

    /// Callled when connection has been started
    fn connection_started(&self) {}

    /// Callled when connection has been finished
    fn connection_finished(&self) {}
}

/// Realtime status of the load test
///
/// Provides data for limiters to be able to throttle load test
/// as well as terminate it early
pub trait RealtimeStatus {
    /// Should report current active connections
    fn connections(&self) -> usize;

    /// Should report current active operations
    fn operations(&self) -> usize;

    /// Should report current total run operation until current moment
    fn total_operations(&self) -> usize;
}
