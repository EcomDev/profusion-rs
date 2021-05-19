use std::time::Instant;

pub(crate) trait AggregateRecorder {
    fn record_success(&mut self, name: &'static str, start: Instant, end: Instant);

    fn record_error(&mut self, name: &'static str, start: Instant, end: Instant);

    fn record_timeout(&mut self, name: &'static str, start: Instant, end: Instant);
}
