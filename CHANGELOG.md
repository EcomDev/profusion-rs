# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added 
- `report::Event` and `report::EventType` to measure the result of execution of the load test.
- `report::RealtimeStatus` trait for monitoring at realtime how many active operations and connections are concurrently utilized.
- `executor::limit::Limiter` trait for creating control structures that can throttle or terminate test run based on `report::RealtimeStatus` results.
- `executor::limit::MaxDurationLimiter` limiter that terminates load test after specified duration.
- `executor::limit::MaxOperationsLimiter` limiter that terminates load test after specified number of operations finished.
- `executor::limit::ConcurrencyLimiter` limiter that throttles operations when too concurrent operations reach limit.

[Unreleased]: https://github.com/EcomDev/profusion-rs/compare/3077010...HEAD