//! Tools to process load test results and recieve realtime information 
//! on its current status.

mod realtime;
mod event;

/// A type of event result during load test operation execution.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EventType {
    /// A successfully finished operation.
    /// 
    /// When operation succesfully finish, 
    /// client/connection stays open and ready to execute the next operation by scheduler.
    Success,
    /// An operation that failed with an I/O error.
    /// 
    /// When operation fails with error client/connection is considered corrupted, 
    /// so it is closed to prevent un-expected test interactions.
    Error,
    /// An operation that failed by exceeding predefined timeout.
    /// 
    /// When operation timeouts executor is going to cancel the scheduled operation
    /// and close the underlying client/connection.
    Timeout,
}

/// An `Event` type to represent the result of executing 
/// a single operation during load test.
#[derive(Debug)]
pub struct Event {
    name: &'static str,
    started_at: Instant,
    finished_at: Instant,
    kind: EventType,
}

pub(crate) use realtime::RealtimeReport;

use crate::time::Instant;

/// Processor event 
pub(crate) trait EventProcessor {
    fn process_success(&mut self, name: &'static str, start: Instant, end: Instant);

    fn process_error(&mut self, name: &'static str, start: Instant, end: Instant);

    fn process_timeout(&mut self, name: &'static str, start: Instant, end: Instant);
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