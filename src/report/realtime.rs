use super::{RealtimeReporter, RealtimeStatus};
use crate::{Arc, AtomicUsize, Ordering};

#[derive(Debug)]
struct Counter(Arc<AtomicUsize>);

impl Default for Counter {
    fn default() -> Self {
        Self(Arc::from(AtomicUsize::new(0)))
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

const COUNTER_STEP: usize = 1;

impl Counter {
    fn increment(&self) {
        match self.0.fetch_add(COUNTER_STEP, Ordering::Relaxed) {
            usize::MAX => {
                self.0.fetch_sub(COUNTER_STEP, Ordering::Relaxed);
            }
            _ => (),
        }
    }

    fn decrement(&self) {
        match self.0.fetch_sub(COUNTER_STEP, Ordering::Relaxed) {
            usize::MIN => {
                self.0.fetch_add(COUNTER_STEP, Ordering::Relaxed);
            }
            _ => (),
        }
    }

    fn value(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeReport {
    operations: Counter,
    connections: Counter,
    total_operations: Counter,
}

impl Default for RealtimeReport {
    fn default() -> Self {
        Self {
            operations: Counter::default(),
            connections: Counter::default(),
            total_operations: Counter::default(),
        }
    }
}

impl RealtimeReporter for RealtimeReport {
    fn operation_started(&self) {
        self.operations.increment();
        self.total_operations.increment();
    }

    fn operation_finished(&self) {
        self.operations.decrement();
    }

    fn connection_created(&self) {
        self.connections.increment();
    }

    fn connection_closed(&self) {
        self.connections.decrement();
    }
}

impl RealtimeStatus for RealtimeReport {
    fn connections(&self) -> usize {
        self.connections.value()
    }

    fn operations(&self) -> usize {
        self.operations.value()
    }

    fn total_operations(&self) -> usize {
        self.total_operations.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loom::thread::{spawn, yield_now, JoinHandle};

    impl From<usize> for Counter {
        fn from(value: usize) -> Self {
            Self(Arc::from(AtomicUsize::from(value)))
        }
    }

    fn run_on_threads<T: 'static + Clone + Send>(
        value: T,
        operation: fn(T) -> (),
    ) -> JoinHandle<()> {
        spawn(move || operation(value))
    }

    fn repeat<T>(times: usize, value: &T, operation: fn(&T) -> ()) {
        for _ in 0..times {
            operation(value);

            yield_now();
        }
    }

    #[test]
    fn counts_when_operation_starts() {
        loom::model(|| {
            let report = RealtimeReport::default();

            run_on_threads(report.clone(), |report| {
                repeat(7, &report, |report| report.operation_started());
            })
            .join()
            .unwrap();

            assert_eq!(report.operations(), 7);
        });
    }

    #[test]
    fn counts_when_operations_finish() {
        loom::model(|| {
            let report = RealtimeReport::default();

            run_on_threads(report.clone(), |report| {
                repeat(7, &report, |report| report.operation_started());

                repeat(4, &report, |report| report.operation_finished());
            })
            .join()
            .unwrap();

            assert_eq!(report.operations(), 3);
        });
    }

    #[test]
    fn counts_when_connection_starts() {
        loom::model(|| {
            let report = RealtimeReport::default();

            run_on_threads(report.clone(), |report| {
                repeat(99, &report, |report| report.connection_created());
            })
            .join()
            .unwrap();

            assert_eq!(report.connections(), 99);
        });
    }

    #[test]
    fn counts_when_connection_finishes() {
        loom::model(|| {
            let report = RealtimeReport::default();

            run_on_threads(report.clone(), |report| {
                repeat(99, &report, |report| report.connection_created());

                repeat(80, &report, |report| report.connection_closed());
            })
            .join()
            .unwrap();

            assert_eq!(report.connections(), 19);
        });
    }

    #[test]
    fn counts_total_operations_started() {
        loom::model(|| {
            let report = RealtimeReport::default();

            run_on_threads(report.clone(), |report| {
                repeat(99, &report, |report| report.operation_started());

                repeat(80, &report, |report| report.operation_finished());
            })
            .join()
            .unwrap();

            assert_eq!(report.total_operations(), 99);
        });
    }

    #[test]
    fn counter_does_not_underflow() {
        loom::model(|| {
            let counter = Counter::default();

            let increment_handle = run_on_threads(counter.clone(), |counter| {
                repeat(100, &counter, |counter| counter.increment());
            });

            let decrement_handle = run_on_threads(counter.clone(), |counter| {
                repeat(200, &counter, |counter| counter.decrement());
            });

            increment_handle.join().unwrap();

            decrement_handle.join().unwrap();

            assert_eq!(counter.value(), 0);
        });
    }

    #[test]
    fn counter_does_not_overflow() {
        loom::model(|| {
            let counter = Counter::from(usize::MAX - 1);

            run_on_threads(counter.clone(), |counter| {
                repeat(100, &counter, |counter| counter.increment());
            })
            .join()
            .unwrap();

            assert_eq!(counter.value(), usize::MAX);
        });
    }
}
