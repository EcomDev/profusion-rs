/*
 * Copyright © 2024. EcomDev B.V.
 * All rights reserved.
 * See LICENSE for license details.
 */

use std::{convert::Infallible, error::Error, future::Future, time::Duration};

use tokio::time::{Instant, timeout};

use crate::aggregate::MetricAggregate;
use crate::metric::MetricRecordError;

/// Metric measurer
///
/// Used in each test case virtual user to measure async operation time
///
pub struct MetricMeasurer<T> {
    aggregate: T,
    timeout: Option<Duration>,
}

impl<M> MetricMeasurer<M>
where
    M: MetricAggregate,
{
    pub fn with_timeout(aggregate: M, timeout: Duration) -> Self {
        Self {
            aggregate,
            timeout: Some(timeout),
        }
    }

    pub fn new(aggregate: M) -> Self {
        Self {
            aggregate,
            timeout: None,
        }
    }

    pub async fn measure<T>(
        &mut self,
        metric: M::Metric,
        action: impl Future<Output = T>,
    ) -> Result<T, MetricRecordError> {
        self.try_measure(metric, async { Ok::<_, Infallible>(action.await) })
            .await
    }

    pub async fn try_measure<T, E>(
        &mut self,
        metric: M::Metric,
        action: impl Future<Output = Result<T, E>>,
    ) -> Result<T, MetricRecordError>
    where
        E: Error + 'static,
    {
        let start = Instant::now();

        let result = self.execute_with_timeout(action).await;

        self.aggregate
            .add_entry(metric, start.elapsed(), result.as_ref().err());

        result
    }

    pub fn add_measurement(
        &mut self,
        metric: M::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        self.aggregate.add_entry(metric, latency, error);
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
    use std::{fmt::Debug, hash::Hash, io::ErrorKind, time::Duration};

    use tokio::{task::yield_now, time::advance};

    use crate::aggregate::{MetricAggregateBuilder, TestAggregateBuilder};
    use crate::measurer::MetricMeasurer;
    use crate::metric::{Metric, MetricRecordError};

    #[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Copy, Clone)]
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

    #[tokio::test(start_paused = true)]
    async fn records_measurements_from_future_without_return_value() {
        let mut recorder = MetricMeasurer::new(TestAggregateBuilder::new().build());

        recorder
            .measure(TestMetric::MetricOne, async {
                advance(Duration::from_millis(30)).await;
            })
            .await
            .unwrap();

        recorder
            .measure(TestMetric::MetricTwo, async {
                advance(Duration::from_millis(20)).await;
            })
            .await
            .unwrap();

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(30), false),
                (TestMetric::MetricTwo, Duration::from_millis(20), false),
            ],
            recorder.aggregate.values()
        )
    }

    #[tokio::test(start_paused = true)]
    async fn reports_timeout_error_when_timeout_is_specified() {
        let mut recorder = MetricMeasurer::with_timeout(
            TestAggregateBuilder::new().build(),
            Duration::from_millis(10),
        );

        let result = recorder
            .measure(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                advance(Duration::from_millis(5)).await;
                yield_now().await;
                advance(Duration::from_millis(5)).await;
            })
            .await;

        assert!(result.is_err());

        assert_eq!(
            vec![(TestMetric::MetricOne, Duration::from_millis(10), true)],
            recorder.aggregate.values(),
            "recorded metric contains error flag and equal to timeout value"
        );
    }

    #[tokio::test]
    async fn returns_result_of_a_run() {
        let mut recorder = MetricMeasurer::new(TestAggregateBuilder::new().build());

        let result = recorder.measure(TestMetric::MetricOne, async { 1 }).await;

        assert_eq!(1, result.unwrap());
    }

    #[tokio::test(start_paused = true)]
    async fn records_successful_measurement_on_each_try_record() {
        let mut recorder = MetricMeasurer::new(TestAggregateBuilder::new().build());

        recorder
            .try_measure(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                Ok::<_, std::io::Error>(())
            })
            .await
            .unwrap();

        recorder
            .try_measure(TestMetric::MetricTwo, async {
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
            recorder.aggregate.values()
        )
    }

    #[tokio::test(start_paused = true)]
    async fn records_errors_when_try_records() {
        let mut recorder = MetricMeasurer::new(TestAggregateBuilder::new().build());

        recorder
            .try_measure(TestMetric::MetricOne, async {
                advance(Duration::from_millis(5)).await;
                Err::<(), _>(std::io::Error::from(ErrorKind::TimedOut))
            })
            .await
            .unwrap_err();

        recorder
            .try_measure(TestMetric::MetricTwo, async {
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
            recorder.aggregate.values()
        )
    }

    #[tokio::test(start_paused = true)]
    async fn records_latencies_passed_manually() {
        let mut recorder = MetricMeasurer::new(TestAggregateBuilder::new().build());

        recorder.add_measurement(
            TestMetric::MetricOne,
            Duration::from_millis(6),
            Some(&MetricRecordError::Timeout(Duration::from_millis(2))),
        );

        recorder.add_measurement(TestMetric::MetricTwo, Duration::from_millis(1), None);

        assert_eq!(
            vec![
                (TestMetric::MetricOne, Duration::from_millis(6), true),
                (TestMetric::MetricTwo, Duration::from_millis(1), false),
            ],
            recorder.aggregate.values()
        )
    }
}
