mod realtime;

pub(crate) use realtime::RealtimeReport;

use std::time::Instant;

pub(crate) trait EventProcessor<'a> {
    fn process_success(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_error(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_timeout(&mut self, name: &'a str, start: Instant, end: Instant);
}

// Used to report current status
// 
//
pub trait RealtimeReporter {
    fn operation_started(&self) {}

    fn operation_finished(&self) {}

    fn connection_started(&self) {}

    fn connection_finished(&self) {}
}

pub trait RealtimeStatus {
    fn connections(&self) -> usize;

    fn operations(&self) -> usize;

    fn total_operations(&self) -> usize;
}
