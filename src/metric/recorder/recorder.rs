use crate::metric::{MetricRecordError, MetricReporter};
use std::{convert::Infallible, error::Error, future::Future, time::Duration};
use tokio::time::{timeout, Instant};

/// Metric recorder
///
/// Used in each test case virtual user to measure async operation time
///
pub struct MetricRecorder<T> {
    reporter: T,
    timeout: Option<Duration>,
}

impl<R> MetricRecorder<R>
where
    R: MetricReporter,
{
    pub fn with_timeout(reporter: R, timeout: Duration) -> Self {
        Self {
            reporter,
            timeout: Some(timeout),
        }
    }

    pub fn new(reporter: R) -> Self {
        Self {
            reporter,
            timeout: None,
        }
    }

    pub async fn record<T>(
        &mut self,
        metric: R::Metric,
        action: impl Future<Output = T>,
    ) -> Result<T, MetricRecordError> {
        self.try_record(metric, async { Ok::<_, Infallible>(action.await) })
            .await
    }

    pub async fn try_record<T, E>(
        &mut self,
        metric: R::Metric,
        action: impl Future<Output = Result<T, E>>,
    ) -> Result<T, MetricRecordError>
    where
        E: Error + 'static,
    {
        let start = Instant::now();

        let result = self.execute_with_timeout(action).await;

        self.reporter
            .add_entry(metric, start.elapsed(), result.as_ref().err());

        result
    }

    pub fn record_latency(
        &mut self,
        metric: R::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        self.reporter.add_entry(metric, latency, error);
    }

    async fn execute_with_timeout<T, E>(
        &self,
        action: impl Future<Output = Result<T, E>>,
    ) -> Result<T, MetricRecordError>
    where
        E: Error + 'static,
    {
        match self.timeout {
            Some(max_duration) => match timeout(max_duration, action).await {
                Ok(result) => result.map_err(|e| MetricRecordError::Dynamic(Box::new(e))),
                Err(_) => Err(MetricRecordError::Timeout(max_duration)),
            },
            None => action
                .await
                .map_err(|e| MetricRecordError::Dynamic(Box::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::{
        recorder::recorder::MetricRecorder, Metric, MetricRecordError, MetricReporter,
    };
    use std::{fmt::Debug, hash::Hash, io::ErrorKind, time::Duration};
    use tokio::{task::yield_now, time::advance};

    #[derive(Debug, PartialEq, Eq, Hash)]
    enum TestMetric {
        MetricOne,
        MetricTwo,
    }

    impl Metric for TestMetric {
        fn name(&self) -> &'static str {
            match self {
                Self::MetricOne => "metric_one",
                Self::MetricTwo => "metric_two",
            }
        }
    }

    #[derive(Default)]
    struct TestReporter {
        values: Vec<(TestMetric, Duration, bool)>,
    }

    impl MetricReporter for TestReporter {
        type Metric = TestMetric;

        fn add_entry(
            &mut self,
            metric: Self::Metric,
            elapsed: Duration,
            error: Option<&MetricRecordError>,
        ) {
            self.values.push((metric, elapsed, error.is_some()))
        }

        fn aggregate_into(mut self, other: &mut Self) {
            other.values.append(&mut self.values);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn records_measurements_from_future_without_return_value() {
        let mut recorder = MetricRecorder::new(TestReporter::default());

        recorder
            .record(TestMetric::MetricOne, async {
                advance(Duration::from_millis(30)).await;
            })
            .await
            .unwrap();

        recorder
            .record(TestMetric::MetricTwo, async {
                advance(Duration::from_millis(20)).await;
            })
            .await
            .unwrap();

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(30), false),
                (TestMetric::MetricTwo, Duration::from_millis(20), false),
            ],
            recorder.reporter.values
        )
    }

    #[tokio::test(start_paused = true)]
    async fn reports_timeout_error_when_timeout_is_specified() {
        let mut recorder = MetricRecorder::with_timeout(
            TestReporter::default(),
            Duration::from_millis(10),
        );

        let result = recorder
            .record(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                advance(Duration::from_millis(5)).await;
                yield_now().await;
                advance(Duration::from_millis(5)).await;
            })
            .await;

        assert!(result.is_err());

        assert_eq!(
            vec![(TestMetric::MetricOne, Duration::from_millis(10), true)],
            recorder.reporter.values,
            "recorded metric contains error flag and equal to timeout value"
        );
    }

    #[tokio::test]
    async fn returns_result_of_a_run() {
        let mut recorder = MetricRecorder::new(TestReporter::default());

        let result = recorder.record(TestMetric::MetricOne, async { 1 }).await;

        assert_eq!(1, result.unwrap());
    }

    #[tokio::test(start_paused = true)]
    async fn records_successful_measurement_on_each_try_record() {
        let mut recorder = MetricRecorder::new(TestReporter::default());

        recorder
            .try_record(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                Ok::<_, std::io::Error>(())
            })
            .await
            .unwrap();

        recorder
            .try_record(TestMetric::MetricTwo, async {
                advance(Duration::from_millis(5)).await;
                Ok::<_, std::io::Error>(())
            })
            .await
            .unwrap();

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(5), false),
                (TestMetric::MetricTwo, Duration::from_millis(5), false),
            ],
            recorder.reporter.values
        )
    }

    #[tokio::test(start_paused = true)]
    async fn records_errors_when_try_records() {
        let mut recorder = MetricRecorder::new(TestReporter::default());

        recorder
            .try_record(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                Err::<(), _>(std::io::Error::from(ErrorKind::TimedOut))
            })
            .await
            .unwrap_err();

        recorder
            .try_record(TestMetric::MetricTwo, async {
                advance(Duration::from_millis(5)).await;
                Err::<(), _>(std::io::Error::from(ErrorKind::TimedOut))
            })
            .await
            .unwrap_err();

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(5), true),
                (TestMetric::MetricTwo, Duration::from_millis(5), true),
            ],
            recorder.reporter.values
        )
    }

    #[tokio::test(start_paused = true)]
    async fn records_latencies_passed_manually() {
        let mut recorder = MetricRecorder::new(TestReporter::default());

        recorder.record_latency(
            TestMetric::MetricOne,
            Duration::from_millis(6),
            Some(&MetricRecordError::Timeout(Duration::from_millis(2))),
        );

        recorder.record_latency(TestMetric::MetricTwo, Duration::from_millis(1), None);

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(6), true),
                (TestMetric::MetricTwo, Duration::from_millis(1), false),
            ],
            recorder.reporter.values
        )
    }
}
