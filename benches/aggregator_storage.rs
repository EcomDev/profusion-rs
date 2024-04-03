use criterion::{black_box, Criterion, criterion_group, criterion_main};

use profusion::prelude::*;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, Ord, PartialOrd)]
enum BenchMetric {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Metric for BenchMetric {
    fn name(&self) -> &'static str {
        match self {
            Self::One => "metric_one",
            Self::Two => "metric_two",
            Self::Three => "metric_three",
            Self::Four => "metric_four",
            Self::Five => "metric_five",
            Self::Six => "metric_six",
        }
    }
}

const SIXTY_METRICS: &[&str] = &[
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "twenty one",
    "twenty two",
    "twenty three",
    "twenty four",
    "twenty five",
    "twenty four",
    "twenty five",
    "twenty eight",
    "twenty nine",
    "thirty",
    "thirty one",
    "thirty two",
    "thirty three",
    "thirty four",
    "thirty five",
    "thirty four",
    "thirty five",
    "thirty eight",
    "thirty nine",
    "forty",
    "forty one",
    "forty two",
    "forty three",
    "forty four",
    "forty five",
    "forty four",
    "forty five",
    "forty eight",
    "forty nine",
    "fifty",
    "fifty one",
    "fifty two",
    "fifty three",
    "fifty four",
    "fifty five",
    "fifty four",
    "fifty five",
    "fifty eight",
    "fifty nine",
    "sixty",
];

fn populate_values_bench<T, R>(mut reporter: R, values: Vec<(T, u64)>)
where
    T: Metric + 'static,
    R: AggregateStorage<Metric = T>,
{
    for (metric, value) in values.into_iter() {
        reporter.record(metric, value)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregator_storage");
    let small_values = black_box(
        (0..=2000usize)
            .map(|index| {
                (
                    match index % 6 {
                        0 => BenchMetric::One,
                        1 => BenchMetric::Two,
                        2 => BenchMetric::Three,
                        3 => BenchMetric::Four,
                        4 => BenchMetric::Five,
                        _ => BenchMetric::Six,
                    },
                    (index % 100 * 53) as u64,
                )
            })
            .collect::<Vec<_>>(),
    );

    let large_values = black_box(
        (0..=2000usize)
            .map(|index| {
                (
                    SIXTY_METRICS[index % SIXTY_METRICS.len()],
                    (index % 100 * 53) as u64,
                )
            })
            .collect::<Vec<_>>(),
    );

    group.bench_with_input("hashmap::small", &small_values, |bench, values| {
        let storage = MetricAggregateStorage::default();
        bench.iter(move || populate_values_bench(storage.clone(), values.clone()));
    });

    group.bench_with_input("hashmap::large", &large_values, |bench, values| {
        let storage = MetricAggregateStorage::default();
        bench.iter(move || populate_values_bench(storage.clone(), values.clone()));
    });
}

criterion_group!(metric_benches, criterion_benchmark);
criterion_main!(metric_benches);
