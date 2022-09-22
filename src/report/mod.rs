//! Tools to process load test results and recieve realtime information
//! on its current status.

mod event;
mod realtime;
mod aggregate;

pub use aggregate::{AggregateBuilder, AggregateEvent, AggregateBucket, AggregateEventProcessor};

/// A type of event result during load test operation execution.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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

use crate::time::Instant;

/// Processor event
pub trait EventProcessor {
    fn process_success(&mut self, name: &'static str, start: Instant, end: Instant);

    fn process_error(&mut self, name: &'static str, start: Instant, end: Instant);

    fn process_timeout(&mut self, name: &'static str, start: Instant, end: Instant);

    fn merge(&mut self, other: Self);
}

pub trait EventProcessorBuilder<T: EventProcessor>
{
    fn build(&self) -> T;
}

/// Notification trait for implementing dispatcher of current load test progress.
///
/// Receiver of notifications must be as light weight as possible, otherwise it will introduce bottlenecks into the load test and scew your results.
///
/// Here is a simple example of notifier trait implementation that outputs
/// a message every time something happens
/// # Example
/// ```rust
/// use profusion::report::RealtimeReporter;
///
/// struct ReportLogger;
///
/// impl RealtimeReporter for ReportLogger {
///     fn operation_started(&self) { println!("started operation"); }
///     fn operation_finished(&self) { println!("finished operation"); }
///     fn connection_created(&self) { println!("created connection"); }
///     fn connection_closed(&self) { println!("closed connection"); }
/// }
/// ```
pub trait RealtimeReporter {
    /// Invoked when load test starts new operation iteration.
    fn operation_started(&self) {}

    /// Invoked when load test finished or fails to complete operation iteration.
    fn operation_finished(&self) {}

    /// Invoked when load test creates a new client/connection.
    fn connection_created(&self) {}

    /// Invoked when load test closes a new client/connection.
    fn connection_closed(&self) {}
}

/// Realtime status of the load test.
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