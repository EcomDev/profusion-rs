use super::*;
use pin_project_lite::pin_project;
use std::{
    io::ErrorKind,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, PartialEq, Eq)]
pub enum EitherFutureKind {
    Empty,
    Left,
    Right,
}

pin_project! {
    /// A combined future from two future types that resolve to the same [`MeasuredOutput<T>`][`crate::executor::future::MeasuredOutput`].
    ///
    /// Main purpose of this future is to allows to write combinators for future builders.
    /// ```
    /// use profusion::executor::{EitherFuture, MeasuredFuture};
    /// use profusion::report::Event;
    /// use std::{future::Ready, io::Result};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let future = EitherFuture::<Ready<(Vec<Event>, Result<usize>)>, _>::right(MeasuredFuture::new("right", async { Ok(2) }, Vec::new()));
    ///     let (_, result) = future.await;
    ///     assert_eq!(result.unwrap(), 2);
    /// }
    /// ```
    #[project = EitherFutureProj]
    pub enum EitherFuture<L,R> {
        Empty,
        Left {
            #[pin]
            inner: L
        },
        Right {
            #[pin]
            inner: R
        }
    }
}

impl<T, L, R> EitherFuture<L, R>
    where
        L: Future<Output=MeasuredOutput<T>>,
        R: Future<Output=MeasuredOutput<T>>,
{
    /// Creates empty variant of the future
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Creates left hand variant of the future
    pub fn left(inner: L) -> Self {
        Self::Left { inner }
    }

    /// Creates right hand variant of the future
    pub fn right(inner: R) -> Self {
        Self::Right { inner }
    }

    pub fn kind(&self) -> EitherFutureKind {
        match self {
            Self::Left { .. } => EitherFutureKind::Left,
            Self::Right { .. } => EitherFutureKind::Right,
            Self::Empty => EitherFutureKind::Empty,
        }
    }
}

impl<T, L, R> Future for EitherFuture<L, R>
    where
        L: Future<Output=MeasuredOutput<T>>,
        R: Future<Output=MeasuredOutput<T>>,
{
    type Output = MeasuredOutput<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            EitherFutureProj::Empty => {
                Poll::Ready((Vec::new(), Err(ErrorKind::InvalidData.into())))
            }
            EitherFutureProj::Left { inner } => {
                let future: Pin<&mut L> = inner;
                future.poll(cx)
            }
            EitherFutureProj::Right { inner } => {
                let future: Pin<&mut R> = inner;
                future.poll(cx)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::{ready, Ready},
        io::ErrorKind,
        time::Instant,
    };
    use std::time::Duration;

    use tokio::join;
    use crate::test_util::assert_events;

    #[tokio::test]
    async fn polls_assigned_futures() {
        let time = Instant::now();

        let (events, first_result) = EitherFuture::<_, Ready<(Vec<Event>, Result<usize>)>>::left(
            MeasuredFuture::new("left", async { Ok(1) }, vec![])
        ).await;
        let (events, second_result) = EitherFuture::<Ready<(Vec<Event>, Result<usize>)>, _>::right(
            MeasuredFuture::new("right", async { Ok(2) }, events)
        ).await;

        let results: Vec<usize> = vec![first_result.unwrap(), second_result.unwrap()];

        assert_events(
            events,
            vec![
                Event::success("left", time, time),
                Event::success("right", time, time)
            ]
        );

        assert_eq!(
            results,
            vec![1, 2]
        );
    }

    #[tokio::test]
    async fn returns_invalid_data_error_when_empty() {
        let empty: EitherFuture<
            Ready<MeasuredOutput<usize>>,
            Ready<MeasuredOutput<usize>>,
        > = EitherFuture::empty();

        let (events, result) = empty.await;

        assert_eq!(
            (events, result.unwrap_err().kind()),
            (Vec::new(), ErrorKind::InvalidData)
        );
    }

    #[test]
    fn allows_to_detect_empty_future() {
        let empty: EitherFuture<
            Ready<MeasuredOutput<usize>>,
            Ready<MeasuredOutput<usize>>,
        > = EitherFuture::empty();

        assert_eq!(empty.kind(), EitherFutureKind::Empty);
    }

    #[test]
    fn allows_to_detect_left_future() {
        let empty: EitherFuture<_, Ready<MeasuredOutput<usize>>> =
            EitherFuture::left(ready((Vec::new(), Ok(1))));

        assert_eq!(empty.kind(), EitherFutureKind::Left);
    }

    #[test]
    fn allows_to_detect_right_future() {
        let empty: EitherFuture<Ready<MeasuredOutput<usize>>, _> =
            EitherFuture::right(ready((Vec::new(), Ok(1))));

        assert_eq!(empty.kind(), EitherFutureKind::Right);
    }
}
