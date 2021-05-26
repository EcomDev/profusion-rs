use std::future::Future;
use std::io::{Error, ErrorKind, Result};

use std::time::{Duration, Instant};
use tokio::time::timeout;

use super::*;
use crate::report::RealtimeReporter;

#[derive(Debug)]
pub struct Runner<'a, R: RealtimeReporter> {
    events: Vec<Event<'a>>,
    timeout: Duration,
    reporter: R,
}

impl <R: RealtimeReporter + Clone> Clone for Runner<'_, R> {
    fn clone(&self) -> Self {
        Self::new(
            self.timeout.clone(),
            self.events.capacity(),
            self.reporter.clone(),
        )
    }
}

impl<'a, R: RealtimeReporter> Runner<'a, R> {
    fn new(timeout: Duration, capacity: usize, reporter: R) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            timeout,
            reporter,
        }
    }

    pub(crate) async fn init<T, TFut>(
        &mut self,
        name: &'static str,
        activity: fn() -> TFut,
    ) -> Result<T>
    where
        T: Sized,
        TFut: Future<Output = Result<T>>,
    {
        let start = Instant::now();

        match activity().await {
            Ok(init) => {
                self.events.push((name, start, Instant::now()).into());
                Ok(init)
            }
            Err(error) => {
                self.events
                    .push((name, start, Instant::now(), EventType::Error).into());
                Err(error)
            }
        }
    }

    pub async fn run<T, TFut>(
        &mut self,
        name: &'a str,
        activity: fn(T) -> TFut,
        state: T,
    ) -> Result<T>
    where
        T: Sized,
        TFut: Future<Output = Result<T>>,
    {
        let start = Instant::now();

        self.reporter.operation_started();

        let result = match timeout(self.timeout, activity(state)).await {
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
        };

        self.reporter.operation_finished();
        result
    }

    pub(crate) fn append_to(&mut self, target: &mut Self) {
        target.events.append(self.events.as_mut())
    }

    pub(crate) fn process<P: EventProcessor<'a>>(&mut self, process: &mut P) {
        self.events
            .drain(..)
            .for_each(|event| event.processor(process));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::FakeProcessor;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::sleep;

    #[derive(Clone)]
    struct FakeReporter {
        operations: Arc<AtomicUsize>,
    }

    impl RealtimeReporter for FakeReporter {
        fn operation_started(&self) {
            self.operations.fetch_add(1, Ordering::Relaxed);
        }

        fn operation_finished(&self) {
            self.operations.fetch_sub(1, Ordering::Relaxed);
        }
    }

    impl Default for FakeReporter {
        fn default() -> Self {
            Self {
                operations: Arc::new(0.into()),
            }
        }
    }

    impl FakeReporter {
        fn current_runners(&self) -> usize {
            self.operations.load(Ordering::Relaxed)
        }
    }

    fn create_runner<'a>() -> Runner<'a, FakeReporter> {
        return Runner::new(Duration::from_secs(3600), 5, FakeReporter::default());
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

    async fn init_value() -> Result<usize> {
        sleep(Duration::from_millis(4)).await;
        Ok(4)
    }

    async fn failed_init_value() -> Result<usize> {
        sleep(Duration::from_millis(4)).await;
        Err(Error::from(ErrorKind::InvalidData))
    }

    #[tokio::test]
    async fn returns_result_from_invoked_function() {
        let mut runner = create_runner();

        assert_eq!(runner.run("add_one", add_one, 123).await.unwrap(), 124);
    }

    #[tokio::test]
    async fn records_time_spend_on_on_each_call() {
        let mut runner = create_runner();

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
        let mut runner = Runner::new(Duration::from_millis(5), 1, FakeReporter::default());

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
        let mut runner = Runner::new(Duration::from_millis(5), 1, FakeReporter::default());

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
        let mut runner = create_runner();

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
        let mut runner_one = create_runner();
        let mut runner_two = create_runner();

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
    async fn reports_internal_events_to_a_recorder() {
        let mut runner = create_runner();

        let mut aggregate = FakeProcessor::new();

        runner.run("event1", add_one, 1).await.unwrap();
        runner.run("event2", add_one, 1).await.unwrap();
        runner.run("event3", add_one, 1).await.unwrap();
        runner.run("event4", add_one, 1).await.unwrap();

        runner.process(&mut aggregate);

        aggregate.verify_names(vec![
            "success:event1",
            "success:event2",
            "success:event3",
            "success:event4",
        ])
    }

    #[tokio::test]
    async fn keeps_list_of_events_empty_after_aggregation() {
        let mut runner = create_runner();

        let mut processor = FakeProcessor::new();

        runner.run("event1", add_one, 1).await.unwrap();
        runner.run("event2", add_one, 1).await.unwrap();
        runner.run("event3", add_one, 1).await.unwrap();
        runner.run("event4", add_one, 1).await.unwrap();

        runner.process(&mut processor);

        assert_eq!((runner.events.len(), runner.events.capacity()), (0, 5));
    }

    #[tokio::test]
    async fn increments_at_realtime_number_of_running_operations() {
        let runner = create_runner();

        let reporter = runner.reporter.clone();

        for n in 0..5 {
            let mut local_runner = runner.clone();
            tokio::spawn(
                async move { local_runner.run("default", add_one_and_wait_10ms, n).await },
            );
        }

        sleep(Duration::from_millis(5)).await;

        assert_eq!(reporter.current_runners(), 5);
    }

    #[tokio::test]
    async fn decrements_at_realtime_number_of_running_operations() {
        let runner = create_runner();

        let reporter = runner.reporter.clone();

        for n in 0..5 {
            let mut local_runner = runner.clone();
            tokio::spawn(
                async move { local_runner.run("default", add_one_and_wait_10ms, n).await },
            );
        }

        sleep(Duration::from_millis(11)).await;

        assert_eq!(reporter.current_runners(), 0);
    }

    #[tokio::test]
    async fn runs_async_function_on_init() {
        let mut runner = create_runner();

        assert_eq!(
            runner
                .init("one", || futures::future::ok(123))
                .await
                .unwrap(),
            123
        );
    }

    #[tokio::test]
    async fn records_time_spend_on_init_events() {
        let mut runner = create_runner();

        runner.init("one", init_value).await.unwrap();
        runner.init("two", init_value).await.unwrap();
        runner.init("three", failed_init_value).await.unwrap_err();

        assert_eq!(
            runner
                .events
                .iter()
                .map(|event| event.latency().as_millis() >= 4 && event.latency().as_millis() <= 6)
                .collect::<Vec<_>>(),
            vec![true, true, true]
        )
    }

    #[tokio::test]
    async fn records_event_names() {
        let mut runner = create_runner();

        runner.init("one", init_value).await.unwrap();
        runner.init("two", init_value).await.unwrap();
        runner.init("three", failed_init_value).await.unwrap_err();

        let mut processor = FakeProcessor::new();

        runner.process(&mut processor);

        processor.verify_names(vec!["success:one", "success:two", "error:three"]);
    }
}
