use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::time::{Duration, Instant};

use profusion::{
    report::{
        AggregateEventProcessor, AggregateEventProcessorBuilder, Event,
        EventProcessorBuilder,
    },
    EventProcessor,
};

fn measure_record(c: &mut Criterion) {
    let time_now = Instant::now();
    let builder = AggregateEventProcessorBuilder::new()
        .with_span(Duration::from_millis(50))
        .with_time(time_now);

    let mut group = c.benchmark_group("record");

    [10u64, 100, 1000, 10000].iter().for_each(|size| {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &builder,
            |b, builder| {
                b.iter_batched(
                    || list_of_events(*size, time_now),
                    |events| populate_aggregate(builder, events),
                    BatchSize::SmallInput,
                );
            },
        );
    });
}

fn populate_aggregate(
    builder: &AggregateEventProcessorBuilder,
    events: Vec<Event>,
) -> AggregateEventProcessor {
    let mut aggregate = builder.build();
    events
        .iter()
        .for_each(|event| event.process(&mut aggregate));
    aggregate
}

fn list_of_events(size: u64, time: Instant) -> Vec<Event> {
    (0..size)
        .map(|index| {
            let (start, end) = (
                time + Duration::from_millis(index * 50),
                time + Duration::from_millis(index * 50 + index % 10),
            );
            match index % 6 {
                0..=2 => Event::success("one", start, end),
                3..=3 => Event::success("two", start, end),
                4..=4 => Event::error("two", start, end),
                5..=5 => Event::timeout("one", start, end),
                _ => Event::timeout("two", start, end),
            }
        })
        .collect()
}

fn measure_merge(c: &mut Criterion) {
    let time_now = Instant::now();
    let builder = AggregateEventProcessorBuilder::new()
        .with_span(Duration::from_millis(50))
        .with_time(time_now);

    let mut group = c.benchmark_group("merge");

    [10u64, 100, 1000, 10000].iter().for_each(|size| {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &builder,
            |b, builder| {
                b.iter_batched(
                    || {
                        (
                            populate_aggregate(builder, list_of_events(*size, time_now)),
                            populate_aggregate(builder, list_of_events(*size, time_now)),
                        )
                    },
                    |(mut left, right)| left.merge(right),
                    BatchSize::SmallInput,
                );
            },
        );
    });
}

criterion_group!(aggregate_bench, measure_record, measure_merge);
criterion_main!(aggregate_bench);
