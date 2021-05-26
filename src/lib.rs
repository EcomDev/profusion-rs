#[warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub
)]

mod runner;
mod report;

pub use runner::{EventType, Runner};
