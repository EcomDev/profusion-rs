mod error;
mod recorder;
mod reporter;

use std::hash::Hash;

pub use error::*;
pub use recorder::*;
pub use reporter::*;

#[cfg(any(feature = "test_util", test))]
pub use reporter::TestReporter;

pub trait Metric: Hash + PartialEq + Eq {
    fn name(&self) -> &'static str;
}

impl Metric for &'static str {
    fn name(&self) -> &'static str {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Hash, PartialEq, Eq, Debug)]
    enum TestMetric {
        ConnectionTime,
        RequestTime,
        WaitTime,
        DownloadTime,
    }

    impl Metric for TestMetric {
        fn name(&self) -> &'static str {
            match self {
                Self::ConnectionTime => "connection_time",
                Self::RequestTime => "request_time",
                Self::WaitTime => "wait_time",
                Self::DownloadTime => "download_time",
            }
        }
    }

    #[test]
    fn auto_applies_metric_to_static_str() {
        assert_eq!("name", "name".name());
    }

    #[test]
    fn verify_metric_can_be_used_as_hashmap_key() {
        let mut map = HashMap::new();

        *map.entry(TestMetric::ConnectionTime).or_default() += 2usize;
        *map.entry(TestMetric::RequestTime).or_default() += 3;
        *map.entry(TestMetric::RequestTime).or_default() += 4;
        *map.entry(TestMetric::DownloadTime).or_default() += 1;
        *map.entry(TestMetric::WaitTime).or_default() += 6;

        assert_eq!(
            HashMap::from([
                (TestMetric::ConnectionTime, 2),
                (TestMetric::RequestTime, 7),
                (TestMetric::DownloadTime, 1),
                (TestMetric::WaitTime, 6)
            ]),
            map
        )
    }
}
