use std::future::Future;
use std::io::Result;

use super::{MeasuredOutput, EitherFuture};
use crate::executor::step::ExecutionStep;
use pin_project_lite::pin_project;

pin_project! {

    #[project = SequenceFutureProj]
    pub enum SequenceFuture<F: ExecutionStep, S: ExecutionStep>
    {
        Ready {
            first: F,
            second: S
        },
        FirstStep {
            #[pin]
            inner: F::Output, 
            second: S
        },
        SecondStep {
            #[pin]
            inner: S::Output
        },
        Done
    }

}

impl <T, F, S> SequenceFuture<F, S> 
    where T: Unpin,
          F: ExecutionStep<Item=T>,
          S: ExecutionStep<Item=T>
{

}