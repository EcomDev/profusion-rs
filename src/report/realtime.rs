use super::{RealtimeReporter, RealtimeStatus};
use std::sync::{Arc, atomic::AtomicUsize, atomic::Ordering};

#[derive(Debug)]
struct Counter(Arc<AtomicUsize>);

impl Default for Counter
{
    fn default() -> Self {
        Self(Arc::from(AtomicUsize::new(0)))
    }
}

impl Clone for Counter
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Counter
{
    fn increment(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement(&self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }

    fn value(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RealtimeReport
{
    operations: Counter,
    connections: Counter,
    total_operations: Counter
}

impl Default for RealtimeReport {
    fn default() -> Self {
        Self {
            operations: Counter::default(),
            connections: Counter::default(),
            total_operations: Counter::default()
        }
    }
}

impl RealtimeReporter for RealtimeReport
{
    fn operation_started(&self) {
        self.operations.increment();
        self.total_operations.increment();
    }

    fn operation_finished(&self) {
        self.operations.decrement();
    }

    fn connection_started(&self) {
        self.connections.increment();
    }

    fn connection_finished(&self) {
        self.connections.decrement();
    }
}

impl RealtimeStatus for RealtimeReport
{
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
mod tests
{
    use super::*;
    use std::thread::spawn;

    fn run_on_threads<T: 'static + Clone + Send>(threads: usize, value: T, operation: fn(T) -> ())
    {
        let handles: Vec<_> = (0..threads).into_iter()
            .map(|_| {
                let passed = value.clone();
                spawn(move || operation(passed))
            }).collect();

        for thread in handles.into_iter() {
            thread.join().unwrap();
        }
    }

    #[test]
    fn counts_when_operation_starts() {
        let report = RealtimeReport::default();


        run_on_threads(7, report.clone(), |report| report.operation_started() );

        assert_eq!(report.operations(), 7);
    }

    #[test]
    fn counts_when_operations_finish() {
        let report = RealtimeReport::default();

        run_on_threads(7, report.clone(), |report| report.operation_started() );
        run_on_threads(4, report.clone(), |report| report.operation_finished() );

        assert_eq!(report.operations(), 3);
    }

    #[test]
    fn counts_when_connection_starts() {
        let report = RealtimeReport::default();

        run_on_threads(99, report.clone(), |report| report.connection_started() );

        assert_eq!(report.connections(), 99);
    }

    #[test]
    fn counts_when_connection_finishes() {
        let report = RealtimeReport::default();

        run_on_threads(99, report.clone(), |report| report.connection_started() );
        run_on_threads(80, report.clone(), |report| report.connection_finished() );

        assert_eq!(report.connections(), 19);
    }

    #[test]
    fn counts_total_operations_started() {
        let report = RealtimeReport::default();

        run_on_threads(99, report.clone(), |report| report.operation_started() );
        run_on_threads(80, report.clone(), |report| report.operation_finished() );

        assert_eq!(report.total_operations(), 99);
    }
}