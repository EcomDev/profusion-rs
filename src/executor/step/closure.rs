use std::{future::Future, io::Result, marker::PhantomData};

use super::ExecutionStep;
use crate::executor::future::MeasuredFuture;

pub struct ClosureStep<T, F, Fut> {
    name: &'static str,
    inner: F,
    _type: PhantomData<T>,
    _future: PhantomData<Fut>,
}

impl <T, F, Fut> Clone for ClosureStep<T, F, Fut> 
    where F: Clone
{
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            inner: self.inner.clone(),
            _type: PhantomData,
            _future: PhantomData
        }
    }
}

impl<T, F, Fut> ClosureStep<T, F, Fut>
where
    F: Fn(T) -> Fut + Clone,
    Fut: Future<Output = Result<T>>,
{
    pub fn new(name: &'static str, closure: F) -> Self {
        Self {
            inner: closure,
            name,
            _future: PhantomData,
            _type: PhantomData,
        }
    }
}

impl<T, F, Fut> ExecutionStep for ClosureStep<T, F, Fut>
where
    F: Fn(T) -> Fut + Clone,
    Fut: Future<Output = Result<T>>,
{
    type Item = T;

    type Output = MeasuredFuture<Fut>;

    fn execute(&self, events: Vec<crate::Event>, input: Self::Item) -> Self::Output {
        MeasuredFuture::new(self.name, (self.inner)(input), events)
    }

    fn capacity(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::Event;

    use super::*;

    #[tokio::test]
    async fn executes_closure_and_returns_its_result() {
        let step =
            ClosureStep::new("event", |counter: usize| async move { Ok(counter + 1) });

        let (_, result) = step.execute(Vec::new(), 8).await;

        assert_eq!(result.unwrap(), 9);
    }

    #[tokio::test]
    async fn executes_closure_and_reports_success_event() {
        let step =
            ClosureStep::new(
                "success_event",
                |counter: usize| async move { Ok(counter) },
            );

        let (mut events, _) = step.execute(Vec::new(), 42).await;

        assert_eq!(
            events.pop().unwrap(),
            Event::success("success_event", Instant::now(), Instant::now())
        )
    }

    #[test]
    fn capacity_is_set_to_single_operation() {
        let step =
            ClosureStep::new(
                "success_event",
                |counter: usize| async move { Ok(counter) },
            );

        assert_eq!(step.capacity(), 1);
    }

    #[tokio::test]
    async fn allows_to_set_function_pointer_as_closure_step() {
        async fn calculate_answer(_: usize) -> Result<usize> {
            Ok(42)
        }

        let step = ClosureStep::new("success_event", calculate_answer);

        let (mut events, _) = step.execute(Vec::new(), 42).await;

        assert_eq!(
            events.pop().unwrap(),
            Event::success("success_event", Instant::now(), Instant::now())
        )
    }
}
