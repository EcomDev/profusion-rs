//! Module that automatically chooses which sync structs to import
//!
//! Used to facilitate [loom][`loom::model`] model tests

#[cfg(loom)]
pub(crate) use loom::sync::atomic::AtomicUsize;

#[cfg(loom)]
pub(crate) use loom::sync::Arc;

#[cfg(not(loom))]
pub(crate) use std::sync::atomic::AtomicUsize;

#[cfg(not(loom))]
pub(crate) use std::sync::Arc;

#[cfg(loom)]
pub(crate) use std::sync::atomic::AtomicUsize;

#[cfg(loom)]
pub(crate) use loom::sync::atomic::Ordering;

#[cfg(not(loom))]
pub(crate) use std::sync::atomic::Ordering;
