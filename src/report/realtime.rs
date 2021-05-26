use super::{RealtimeReporter, RealtimeStatus};
use std::sync::{Arc, atomic::AtomicUsize, atomic::Ordering};

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

pub(crate) struct RealtimeReport
{
    operations: Counter,
    connections: Counter
}

impl Clone for RealtimeReport {
    fn clone(&self) -> Self {
        Self {
            operations: self.operations.clone(),
            connections: self.connections.clone()
        }
    }
}

impl Default for RealtimeReport {
    fn default() -> Self {
        Self {
            operations: Counter::default(),
            connections: Counter::default()
        }
    }
}

impl RealtimeReporter for RealtimeReport
{
    fn operation_started(&self) {
        self.operations.increment()
    }

    fn operation_finished(&self) {
        self.operations.decrement()
    }
}

impl RealtimeStatus for RealtimeReport
{
    fn connections(&self) -> usize {
        todo!()
    }

    fn operations(&self) -> usize {
        self.operations.value()
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
    fn each_clone_increments_number_of_operations() {
        let report = RealtimeReport::default();


        run_on_threads(7, report.clone(), |report| report.operation_started() );

        assert_eq!(report.operations(), 7);
    }

    #[test]
    fn operations_get_reduced_back_to_when_they_are_over() {
        let report = RealtimeReport::default();

        run_on_threads(7, report.clone(), |report| report.operation_started() );
        run_on_threads(4, report.clone(), |report| report.operation_finished() );

        assert_eq!(report.operations(), 3);
    }
}