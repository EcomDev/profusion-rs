use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hdrhistogram::Histogram;
use profusion::metric::{Metric, MetricRecordError, MetricReporter};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
enum BenchMetric {
    MetricOne,
    MetricTwo,
    MetricThree,
    MetricFour,
    MetricFive,
    MetricSix,
}

impl Metric for BenchMetric {
    fn name(&self) -> &'static str {
        match self {
            Self::MetricOne => "metric_one",
            Self::MetricTwo => "metric_two",
            Self::MetricThree => "metric_three",
            Self::MetricFour => "metric_four",
            Self::MetricFive => "metric_five",
            Self::MetricSix => "metric_six",
        }
    }
}

struct ReportVec {
    storage: Vec<(BenchMetric, Histogram<u64>)>,
}

struct ReportSmallVec {
    storage: SmallVec<[(BenchMetric, Histogram<u64>); 10]>,
}

struct ReportHashMap {
    storage: HashMap<BenchMetric, Histogram<u64>>,
}

impl MetricReporter for ReportHashMap {
    type Metric = BenchMetric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        let entry = self
            .storage
            .entry(metric)
            .or_insert_with(|| Histogram::new(3).unwrap());

        entry.record(latency.as_nanos() as u64).unwrap_or_default();
    }

    fn aggregate_into(self, other: &mut Self) {
        todo!()
    }
}

impl MetricReporter for ReportVec {
    type Metric = BenchMetric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        let position = self.storage.iter().position(|(metric, _)| metric == metric);
        let (_, histogram) = match position {
            Some(value) => &mut self.storage[value],
            None => {
                self.storage.push((metric, Histogram::new(3).unwrap()));
                let index = self.storage.len() - 1;
                &mut self.storage[index]
            }
        };

        histogram
            .record(latency.as_nanos() as u64)
            .unwrap_or_default();
    }

    fn aggregate_into(self, other: &mut Self) {
        todo!()
    }
}

impl MetricReporter for ReportSmallVec {
    type Metric = BenchMetric;

    fn add_entry(
        &mut self,
        metric: Self::Metric,
        latency: Duration,
        error: Option<&MetricRecordError>,
    ) {
        let position = self.storage.iter().position(|(metric, _)| metric == metric);
        let (_, histogram) = match position {
            Some(value) => &mut self.storage[value],
            None => {
                self.storage.push((metric, Histogram::new(3).unwrap()));
                let index = self.storage.len() - 1;
                &mut self.storage[index]
            }
        };

        histogram
            .record(latency.as_nanos() as u64)
            .unwrap_or_default();
    }

    fn aggregate_into(self, other: &mut Self) {
        todo!()
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_on_collection");

    let values = black_box(
        (0..=2000usize)
            .into_iter()
            .map(|index| {
                (
                    match index % 6 {
                        0 => BenchMetric::MetricOne,
                        1 => BenchMetric::MetricTwo,
                        2 => BenchMetric::MetricThree,
                        3 => BenchMetric::MetricFour,
                        4 => BenchMetric::MetricFive,
                        _ => BenchMetric::MetricSix,
                    },
                    Duration::from_micros((index % 100 * 1000) as u64),
                )
            })
            .collect::<Vec<_>>(),
    );

    group.bench_with_input("hashmap", &values, |bench, values| {
        bench.iter(move || {
            let mut reporter = ReportHashMap {
                storage: HashMap::new(),
            };

            values
                .iter()
                .for_each(|(metric, value)| reporter.add_entry(*metric, *value, None));
        });
    });

    group.bench_with_input("vec", &values, |bench, values| {
        bench.iter(move || {
            let mut reporter = ReportVec {
                storage: Vec::new(),
            };

            values
                .iter()
                .for_each(|(metric, value)| reporter.add_entry(*metric, *value, None));
        });
    });

    group.bench_with_input("small_vec", &values, |bench, values| {
        bench.iter(move || {
            let mut reporter = ReportSmallVec {
                storage: SmallVec::new(),
            };

            values
                .iter()
                .for_each(|(metric, value)| reporter.add_entry(*metric, *value, None));
        });
    });
}

criterion_group!(metric_benches, criterion_benchmark);
criterion_main!(metric_benches);
