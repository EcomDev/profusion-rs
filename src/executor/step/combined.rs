use std::{future::Future, io::Result, marker::PhantomData};

use super::{ExecutionStep, WeightedExecutionStep};

use crate::executor::future::MeasuredFuture;