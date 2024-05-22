/*
 * Copyright © 2024. EcomDev B.V.
 * All rights reserved.
 * See LICENSE for license details.
 */

use std::cmp::max;
use std::time::Duration;

use crate::aggregate::{AggregateStorage, CombinedAggregateStorage};
use crate::metric::MetricRecordError;

#[derive(Debug)]
pub struct TimelineItem<S> {
    time: Duration,
    storage: S,
    errors: usize,
    users: usize,
}

impl<S> Eq for TimelineItem<S> {}

impl<S> PartialEq for TimelineItem<S> {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl<S> PartialOrd for TimelineItem<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Ord for TimelineItem<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl<S> TimelineItem<S>
where
    S: AggregateStorage,
{
    pub(crate) fn new(time: Duration, storage: S, errors: usize, users: usize) -> Self {
        Self {
            time,
            storage,
            errors,
            users,
        }
    }

    pub fn time(&self) -> &Duration {
        &self.time
    }

    pub(crate) fn storage(&self) -> &S {
        &self.storage
    }

    pub fn errors(&self) -> usize {
        self.errors
    }

    pub fn users(&self) -> usize {
        self.users
    }

    pub(crate) fn record(&mut self, metric: S::Metric, value: u64) {
        self.storage.record(metric, value)
    }

    pub(crate) fn update_counters(
        &mut self,
        error: Option<&MetricRecordError>,
        users: usize,
    ) {
        if error.is_some() {
            self.errors += 1;
        }

        self.users = users
    }

    pub(crate) fn merge_into(self, other: &mut Self) {
        let storage = std::mem::take(&mut other.storage);
        other.storage = storage.merge(self.storage);
        other.users = max(other.users, self.users);
        other.errors += self.errors;
    }
}

impl<L, R> TimelineItem<CombinedAggregateStorage<L, R>>
where
    L: AggregateStorage,
    R: AggregateStorage<Metric = L::Metric>,
{
    pub fn split(self) -> (TimelineItem<L>, TimelineItem<R>) {
        let (left_storage, right_storage) = self.storage.unwrap();
        (
            TimelineItem {
                time: self.time,
                storage: left_storage,
                errors: self.errors,
                users: self.users,
            },
            TimelineItem {
                storage: right_storage,
                time: self.time,
                errors: self.errors,
                users: self.users,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::*;

    #[test]
    fn splits_into_multiple_storages() {
        let mut item = TimelineItem::new(
            Duration::from_millis(10),
            MetricAggregateStorage::default().and(TotalAggregateStorage::default()),
            0,
            1,
        );

        item.record("one", 100);

        let (left, right) = item.split();

        assert_eq!(left.storage().value("one").max(), 100);
        assert_eq!(right.storage().value().max(), 100);
    }
}
