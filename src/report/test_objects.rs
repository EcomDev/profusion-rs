use super::EventProcessor;
use std::time::{Duration, Instant};

pub(crate) struct FakeProcessor {
    items: Vec<(String, Duration)>,
}

impl FakeProcessor {
    pub(crate) fn new() -> Self {
        Self { items: vec![] }
    }

    pub(crate) fn verify(self, items: Vec<(&str, Duration)>) {
        assert_eq!(
            self.items.iter().map(|(text, duration)| (text.as_str(), duration.to_owned())).collect::<Vec<_>>(),
            items,
        );
    }

    pub(crate) fn verify_names(self, items: Vec<&str>) {
        assert_eq!(
            self.items
                .iter()
                .map(move |event| event.0.as_str())
                .collect::<Vec<_>>(),
            items
        );
    }
}

impl EventProcessor<'_> for FakeProcessor {
    fn process_success(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("success:{}", name), end - start));
    }

    fn process_error(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("error:{}", name), end - start));
    }

    fn process_timeout(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("timeout:{}", name), end - start));
    }
}
