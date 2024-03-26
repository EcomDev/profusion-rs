use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hdrhistogram::Histogram;
use smallvec::SmallVec;
use std::{collections::HashMap, fmt::Debug, future::Future, hash::Hash, time::Duration};
use tokio::{runtime::Builder, time::Instant};

#[derive(Hash, PartialEq, Eq, Debug, Default)]
enum TestMetric {
    #[default]
    OneValue,
    TwoValue,
}

trait Metric: Hash + PartialEq + Eq + Debug {
    fn name(&self) -> &'static str;
}

impl Metric for TestMetric {
    fn name(&self) -> &'static str {
        match self {
            Self::OneValue => "one_value",
            Self::TwoValue => "two_value",
        }
    }
}

trait MetricRecorder {
    type Metric: Metric;

    async fn record<T>(
        &mut self,
        metric: Self::Metric,
        action: impl Future<Output = T>,
    ) -> T;
}

async fn async_action(recorder: &mut impl MetricRecorder<Metric = TestMetric>) {
    recorder.record(TestMetric::OneValue, async { 1 + 2 }).await;
    recorder.record(TestMetric::TwoValue, async { 2 + 2 }).await;
    recorder.record(TestMetric::OneValue, async { 4 + 2 }).await;
    recorder.record(TestMetric::TwoValue, async { 5 + 2 }).await;
}

#[derive(Default)]
struct MetricStorage<M: Metric> {
    values: HashMap<M, Histogram<u64>>,
}

#[derive(Default)]
struct AccumulatorRecorder<M: Metric> {
    accumulator: MetricStorage<M>,
}

#[derive(Default)]
struct FlusherRecorder<M: Metric> {
    events: SmallVec<[(M, Duration); 32]>,
}

impl<M: Metric> MetricRecorder for AccumulatorRecorder<M> {
    type Metric = M;

    async fn record<T>(&mut self, metric: M, action: impl Future<Output = T>) -> T {
        let started = Instant::now();
        let result = action.await;
        let end = started.elapsed();
        let values = self
            .accumulator
            .values
            .entry(metric)
            .or_insert_with(|| Histogram::new(3).unwrap());

        values
            .record(end.as_secs() * 1_000_000_000 + end.subsec_nanos() as u64)
            .unwrap();
        result
    }
}

impl<M: Metric> MetricRecorder for FlusherRecorder<M> {
    type Metric = M;

    async fn record<T>(&mut self, metric: M, action: impl Future<Output = T>) -> T {
        let started = Instant::now();
        let result = action.await;
        let end = started.elapsed();
        self.events.push((metric, end));
        result
    }
}

impl<M: Metric> FlusherRecorder<M> {
    pub fn flush(&mut self, storage: &mut MetricStorage<M>) {
        self.events.drain(..).for_each(|(metric, duration)| {
            let values = storage
                .values
                .entry(metric)
                .or_insert_with(|| Histogram::new(3).unwrap());

            values
                .record(
                    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64,
                )
                .unwrap();
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("measure_performance");
    let values = black_box(0..2000);

    group.bench_with_input("accumulator", &values, |bench, values| {
        let runtime = Builder::new_current_thread()
            .start_paused(true)
            .build()
            .unwrap();

        bench.to_async(runtime).iter(|| async move {
            let mut recorder = AccumulatorRecorder::default();
            for _ in values.clone().into_iter() {
                async_action(&mut recorder).await;
            }
        });
    });

    group.bench_with_input("flusher", &values, |bench, values| {
        let runtime = Builder::new_current_thread()
            .start_paused(true)
            .build()
            .unwrap();

        bench.to_async(runtime).iter(|| async move {
            let mut recorder = FlusherRecorder::default();
            for _ in values.clone().into_iter() {
                async_action(&mut recorder).await;
            }

            let mut storage = MetricStorage::default();
            recorder.flush(&mut storage);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
