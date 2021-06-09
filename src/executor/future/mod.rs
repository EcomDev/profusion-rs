use crate::report::Event;
use std::{future::Future, io::Result};

pub(super) type MeasuredOutput<T> = (Vec<Event>, Result<T>);

mod either;
mod measured;
mod noop;
mod sequence;

pub use either::EitherFuture;
pub use measured::MeasuredFuture;
pub use noop::NoopFuture;
