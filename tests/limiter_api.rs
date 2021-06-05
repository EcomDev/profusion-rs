use profusion::{
    executor::limit::{ConcurrencyLimiter, Limit, Limiter},
    report::RealtimeStatus,
};

use std::time::Duration;

#[derive(Debug, Clone, Copy)]
struct LimiterStub(Limit);
struct StatusStub(usize);

impl RealtimeStatus for StatusStub {
    fn connections(&self) -> usize {
        0
    }

    fn operations(&self) -> usize {
        self.0
    }

    fn total_operations(&self) -> usize {
        0
    }
}

impl LimiterStub {
    fn new(limit: Limit) -> Self {
        Self(limit)
    }
}

impl Limiter for LimiterStub {
    fn apply<S: RealtimeStatus>(&self, _status: &S) -> Limit {
        self.0
    }
}

#[test]
fn every_limit_is_checked_to_get_final_result_when_nothing_happens_on_left() {
    let limiter = LimiterStub::new(Limit::None)
        .with(ConcurrencyLimiter::new(10, Duration::from_millis(10)))
        .with(LimiterStub::new(Limit::None))
        .with(LimiterStub::new(Limit::Wait(Duration::from_millis(20))));

    assert_eq!(
        limiter.apply(&StatusStub(5)),
        Limit::Wait(Duration::from_millis(20))
    );
}

#[test]
fn breaks_chain_on_first_limit_that_is_not_none() {
    let limiter = LimiterStub::new(Limit::None)
        .with(ConcurrencyLimiter::new(10, Duration::from_millis(10)))
        .with(LimiterStub::new(Limit::None))
        .with(LimiterStub::new(Limit::Wait(Duration::from_millis(20))));

    assert_eq!(
        limiter.apply(&StatusStub(10)),
        Limit::Wait(Duration::from_millis(10))
    );
}
