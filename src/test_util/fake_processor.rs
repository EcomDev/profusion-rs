use crate::time::*;
use crate::EventProcessor;

#[derive(Default)]
pub struct FakeProcessor {
    items: Vec<(String, Duration)>,
}

impl FakeProcessor {
    pub fn verify(self, items: Vec<(&str, Duration)>) {
        assert_eq!(
            self.items
                .iter()
                .map(|(text, duration)| (text.as_str(), *duration))
                .collect::<Vec<_>>(),
            items,
        );
    }
}

impl EventProcessor for FakeProcessor {
    fn process_success(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("success:{}", name), end - start));
    }

    fn process_error(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("error:{}", name), end - start));
    }

    fn process_timeout(&mut self, name: &str, start: Instant, end: Instant) {
        self.items.push((format!("timeout:{}", name), end - start));
    }

    fn merge(&mut self, mut other: Self) {
        self.items.append(&mut other.items);
    }
}

#[cfg(test)]
mod tests
{
    use std::time::{Duration, Instant};
    use crate::EventProcessor;
    use crate::test_util::FakeProcessor;
    use crate::time::InstantOffset;

    #[test]
    fn records_time_difference_of_different_events() {
        let mut processor = FakeProcessor::default();
        let time = Instant::now();

        processor.process_success("one", time.with_millis(10), time.with_millis(30));
        processor.process_success("two", time.with_millis(30), time.with_millis(40));
        processor.process_error("three", time.with_millis(40), time.with_millis(60));
        processor.process_timeout("four", time.with_millis(140), time.with_millis(190));

        assert_eq!(
            processor.items,
            vec![
                ("success:one".into(), Duration::from_millis(20)),
                ("success:two".into(), Duration::from_millis(10)),
                ("error:three".into(), Duration::from_millis(20)),
                ("timeout:four".into(), Duration::from_millis(50)),
            ]
        );
    }
}