//! Load test executor

pub mod limit;
mod measured_future;

use std::future::Future;
use std::io::Result;

use crate::report::Event;

pub use measured_future::MeasuredFuture;
