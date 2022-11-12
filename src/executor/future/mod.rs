use std::{future::Future, io::Result};

pub use {
    either::EitherFuture,
    measured::MeasuredFuture,
    sequence::SequenceFuture
};

pub(crate) use either::EitherFutureKind;

use crate::report::Event;

pub(super) type MeasuredOutput<T> = (Vec<Event>, Result<T>);

mod either;
mod measured;
mod sequence;

