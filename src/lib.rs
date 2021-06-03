#[warn(missing_debug_implementations, missing_docs, unreachable_pub)]
mod report;
mod runner;
mod sync;

pub(crate) use sync::{Arc, AtomicUsize, Ordering};

#[cfg(any(test, feature = "test"))]
mod test_objects;

#[cfg(any(test, feature = "test"))]
pub use test_objects::{FakeProcessor, RealtimeStatusStub};

pub(crate) use report::{EventProcessor, RealtimeReport};
pub use report::{RealtimeReporter, RealtimeStatus};
pub use runner::{
    ConcurrencyLimiter, EventType, Limit, Limiter, MaxDurationLimiter,
    MaxOperationsLimiter, Runner,
};
