mod realtime;

#[cfg(test)]
mod test_objects;

#[cfg(test)]
pub(crate) use test_objects::*;

use std::time::Instant;

pub(crate) trait EventProcessor<'a> {
    fn process_success(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_error(&mut self, name: &'a str, start: Instant, end: Instant);

    fn process_timeout(&mut self, name: &'a str, start: Instant, end: Instant);
}

pub trait RealtimeReporter {
    fn operation_started(&self) {}

    fn operation_finished(&self) {}

    fn connection_started(&self) {}

    fn connection_finished(&self) {}
}

pub(crate) trait RealtimeStatus {
    fn connections(&self) -> usize;

    fn operations(&self) -> usize;
}
