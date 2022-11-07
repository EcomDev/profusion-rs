# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `report::Event` and `report::EventType` to measure the result of async operation execution
- `report::RealtimeStatus` trait for monitoring in realtime how many active operations and connections are concurrently
  utilized.
- `report::RealtimeReporter` trait for building own listener on open connections and operations
- `report::RealtimeReport` default concurrent lightweight implementation for `report::RealtimeReporter`
  and `report::RealtimeStatus` traits
- `report::EventProcessorBuilder` and `report::EventProcessor` traits for processing timings of operations
- `report::AggregagteEventProcessorBuilder` and `report::AggregagteEventProcessor` implementation that aggregates
  timings into `HdrHistogram` and `AggregateEvent` timeline
- `executor::ExecutionStep` trait for implementing async load test scenario steps with combinators like
  `executor::NoopStep`, `executor::ClosureStep`, `executor::SequenceStep` that allows to build scenarios in simple dsl
- `executor::ScenarioBuilder` and `executor::Scenario` trait for creating load testing scenarios
  with `executor::StepScenarioBuilder` and `executor::StepScenario` based on `ExecutionStep` functionality.
- `executor::Limiter` trait for creating control structures that can throttle or terminate test run based
  on `report::RealtimeStatus` results.
- `executor::MaxDurationLimiter` limiter that terminates load test after specified duration.
- `executor::MaxOperationsLimiter` limiter that terminates load test after specified number of operations finished.
- `executor::ConcurrencyLimiter` limiter that throttles operations when too concurrent operations reach limit.
- `executor::MeasuredFuture` for capturing execution events of the async tasks.

[Unreleased]: https://github.com/EcomDev/profusion-rs/compare/3077010...HEAD