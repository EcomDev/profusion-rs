use std::time::Duration;
use crate::RealtimeStatus;

pub trait LimiterBuilder {
    type Item: Limiter + Clone;

    fn build(&self) -> Self::Item;
}

pub trait Limiter {
    fn apply<S: RealtimeStatus>(status: &S) -> Limit;
}

#[derive(Debug, PartialEq)]
pub enum Limit {
    None,
    Wait(Duration),
    Shutdown
}