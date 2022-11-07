use std::{future::Future, io::Result};

pub use either::{EitherFuture, EitherFutureKind};
pub use measured::MeasuredFuture;
pub use sequence::SequenceFuture;

use crate::report::Event;

pub(super) type MeasuredOutput<T> = (Vec<Event>, Result<T>);

mod either;
mod measured;
mod sequence;

