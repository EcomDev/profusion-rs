use super::*;
use pin_project_lite::pin_project;
use std::{
    io::ErrorKind,
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    /// A combined future from two future types that resolve to the same `Result<T>`.
    ///
    /// Main purpose of this future is to allows to write combinators for future builders.
    /// ```
    /// use profusion::executor::future::{EitherFuture, NoopFuture, MeasuredFuture};
    /// #[tokio::main]
    /// async fn main() {
    ///     let future = EitherFuture::<NoopFuture<usize>, _>::right(MeasuredFuture::new("right", async { Ok(2) }, Vec::new()));
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
    L: Future<Output = MeasuredOutput<T>>,
    R: Future<Output = MeasuredOutput<T>>,
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
}

#[must_use = "Futures must be awaited"]
impl<T, L, R> Future for EitherFuture<L, R>
where
    L: Future<Output = MeasuredOutput<T>>,
    R: Future<Output = MeasuredOutput<T>>,
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
    use std::{io::ErrorKind, time::Instant};

    #[tokio::test]
    async fn polls_assigned_futures() {
        let futures = vec![
            EitherFuture::left(MeasuredFuture::new("left", async { Ok(1) }, Vec::new())),
            EitherFuture::right(MeasuredFuture::new(
                "right",
                async { Ok(2) },
                Vec::new(),
            )),
        ];

        let mut results = Vec::new();
        let mut events = Vec::new();

        for future in futures {
            let (mut measurement, result) = future.await;
            results.push(result.unwrap());
            events.append(&mut measurement);
        }

        assert_eq!(
            events
                .into_iter()
                .zip(results.into_iter())
                .collect::<Vec<(Event, usize)>>(),
            vec![
                (Event::success("left", Instant::now(), Instant::now()), 1),
                (Event::success("right", Instant::now(), Instant::now()), 2)
            ]
        );
    }

    #[tokio::test]
    async fn returns_invalid_data_error_when_empty() {
        let empty: EitherFuture<NoopFuture<usize>, NoopFuture<usize>> =
            EitherFuture::empty();

        let (events, result) = empty.await;

        assert_eq!(
            (events, result.unwrap_err().kind()),
            (Vec::new(), ErrorKind::InvalidData)
        );
    }
}
