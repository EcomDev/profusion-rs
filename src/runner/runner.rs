use std::future::Future;
use std::io::{Error, ErrorKind, Result};

use std::time::{Duration, Instant};
use tokio::time::timeout;

use super::*;

pub struct Runner {
    events: Vec<Event>,
    timeout: Duration,
}

impl Runner {
    fn new(timeout: Duration, capacity: usize) -> Self {
        Runner {
            events: Vec::with_capacity(capacity),
            timeout: timeout,
        }
    }

    pub async fn run<T, TFut>(
        &mut self,
        name: &'static str,
        activity: fn(T) -> TFut,
        state: T,
    ) -> Result<T>
    where
        T: Sized,
        TFut: Future<Output = Result<T>>,
    {
        let start = Instant::now();

        match timeout(self.timeout, activity(state)).await {
            Ok(result) => {
                match &result {
                    Ok(_) => self.events.push((name, start, Instant::now()).into()),
                    Err(_) => self
                        .events
                        .push((name, start, Instant::now(), EventType::Error).into()),
                };
                result
            }
            Err(_) => {
                self.events
                    .push((name, start, Instant::now(), EventType::Timeout).into());
                Err(Error::from(ErrorKind::TimedOut))
            }
        }
    }

    pub(crate) fn append_to(&mut self, target: &mut Self) {
        target.events.append(self.events.as_mut())
    }

    pub(crate) fn aggregate<A: AggregateRecorder>(&mut self, aggregate: &mut A) {
        self.events
            .drain(..)
            .for_each(|event| event.aggregate(aggregate));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    struct FakeAggregate {
        names: Vec<&'static str>,
    }

    impl FakeAggregate {
        fn new() -> Self {
            Self { names: vec![] }
        }
    }

    impl AggregateRecorder for FakeAggregate {
        fn record_timeout(&mut self, name: &'static str, _: Instant, _: Instant) {
            self.names.push(name);
        }
        fn record_error(&mut self, name: &'static str, _: Instant, _: Instant) {
            self.names.push(name);
        }
        fn record_success(&mut self, name: &'static str, _: Instant, _: Instant) {
            self.names.push(name);
        }
    }

    impl Default for Runner {
        fn default() -> Self {
            Self {
                timeout: Duration::from_secs(24 * 60 * 60),
                events: Vec::new(),
            }
        }
    }

    async fn add_one(value: usize) -> Result<usize> {
        Ok(value.saturating_add(1))
    }

    async fn fail_operation(_: usize) -> Result<usize> {
        Err(Error::from(ErrorKind::InvalidData))
    }

    async fn add_one_and_wait_10ms(value: usize) -> Result<usize> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        add_one(value).await
    }

    #[tokio::test]
    async fn returns_result_from_invoked_function() {
        let mut runner = Runner::default();

        assert_eq!(runner.run("add_one", add_one, 123).await.unwrap(), 124);
    }

    #[tokio::test]
    async fn records_time_spend_on_on_each_call() {
        let mut runner = Runner::default();

        runner.run("one", add_one_and_wait_10ms, 123).await.unwrap();
        runner.run("two", add_one_and_wait_10ms, 123).await.unwrap();
        runner
            .run("three", add_one_and_wait_10ms, 123)
            .await
            .unwrap();

        assert_eq!(
            runner
                .events
                .iter()
                .map(|event| event.latency().as_millis() >= 10 && event.latency().as_millis() <= 12)
                .collect::<Vec<_>>(),
            vec![true, true, true]
        )
    }

    #[tokio::test]
    async fn returns_timeout_error_when_task_is_too_long() {
        let mut runner = Runner::new(Duration::from_millis(5), 1);

        assert_eq!(
            format!(
                "{}",
                runner
                    .run("too_long", add_one_and_wait_10ms, 123)
                    .await
                    .unwrap_err()
            ),
            "timed out"
        );
    }

    #[tokio::test]
    async fn timeouts_task_when_it_takes_too_long() {
        let mut runner = Runner::new(Duration::from_millis(5), 1);

        runner
            .run("too_long", add_one_and_wait_10ms, 123)
            .await
            .unwrap_err();

        assert_eq!(
            runner
                .events
                .iter()
                .map(|event| event.kind())
                .collect::<Vec<_>>(),
            vec![EventType::Timeout]
        )
    }

    #[tokio::test]
    async fn reports_on_errors_during_execution() {
        let mut runner = Runner::new(Duration::from_millis(5), 1);

        runner
            .run("error_one", fail_operation, 123)
            .await
            .unwrap_err();
        runner
            .run("error_two", fail_operation, 123)
            .await
            .unwrap_err();
        runner
            .run("error_three", fail_operation, 123)
            .await
            .unwrap_err();

        assert_eq!(
            runner
                .events
                .iter()
                .map(|event| event.kind())
                .collect::<Vec<_>>(),
            vec![EventType::Error, EventType::Error, EventType::Error]
        )
    }

    #[tokio::test]
    async fn merges_runner_events_into_enother_one() {
        let mut runner_one = Runner::default();
        let mut runner_two = Runner::default();

        let value = runner_one.run("task_one", add_one, 1).await.unwrap();
        let value = runner_two.run("task_two", add_one, value).await.unwrap();
        let value = runner_one.run("task_three", add_one, value).await.unwrap();
        runner_two.run("task_four", add_one, value).await.unwrap();
        runner_two.append_to(&mut runner_one);

        assert_eq!(
            runner_one
                .events
                .iter()
                .map(|event| event.name())
                .collect::<Vec<_>>(),
            vec!["task_one", "task_three", "task_two", "task_four"]
        )
    }

    #[tokio::test]
    async fn reports_internal_events_to_an_aggregate() {
        let mut runner = Runner::new(Duration::from_millis(10), 5);

        let mut aggregate = FakeAggregate::new();

        runner.run("event1", add_one, 1).await.unwrap();
        runner.run("event2", add_one, 1).await.unwrap();
        runner.run("event3", add_one, 1).await.unwrap();
        runner.run("event4", add_one, 1).await.unwrap();

        runner.aggregate(&mut aggregate);

        assert_eq!(
            aggregate.names,
            vec!["event1", "event2", "event3", "event4"]
        );
    }

    #[tokio::test]
    async fn keeps_list_of_events_empty_after_aggregate() {
        let mut runner = Runner::new(Duration::from_millis(10), 5);

        let mut aggregate = FakeAggregate::new();

        runner.run("event1", add_one, 1).await.unwrap();
        runner.run("event2", add_one, 1).await.unwrap();
        runner.run("event3", add_one, 1).await.unwrap();
        runner.run("event4", add_one, 1).await.unwrap();

        runner.aggregate(&mut aggregate);

        assert_eq!((runner.events.len(), runner.events.capacity()), (0, 5));
    }
}
