use std::{
    pin::Pin,
    task::{Context, Poll},
};

use super::*;

/// A future that directly resolves to the provided value
///
/// It is used internally to provide default state of future builders that might implement it as part of own combinators.
#[derive(Debug)]
pub struct NoopFuture<T> {
    state: Option<(T, Vec<Event>)>,
}

impl<T> NoopFuture<T> {
    /// Creates `NoopFuture` with provided vector of Events
    pub fn new(value: T, events: Vec<Event>) -> Self {
        Self {
            state: Some((value, events)),
        }
    }
}

#[must_use = "Futures must be awaited"]
impl<T> Future for NoopFuture<T>
where
    T: Unpin,
{
    type Output = MeasuredOutput<T>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.state.take();

        match state {
            None => panic!("NoopFuture is polled more then once"),
            Some((value, events)) => Poll::Ready((events, Ok(value))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn resolves_to_a_provided_argument() {
        let future = NoopFuture::new(123, Vec::new());

        let (_, result) = future.await;

        assert_eq!(result.unwrap(), 123);
    }

    #[tokio::test]
    async fn does_not_modify_passed_events() {
        let future = NoopFuture::new(
            123,
            vec![Event::success("one", Instant::now(), Instant::now())],
        );

        let (events, _) = future.await;

        assert_eq!(
            events,
            vec![Event::success("one", Instant::now(), Instant::now())]
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn should_panic_if_polled_empty() {
        let future = NoopFuture::<u32> {
            state: None
        };

        future.await;
    }
}
