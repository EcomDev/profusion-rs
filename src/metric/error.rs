/*
 * Copyright © 2024. EcomDev B.V.
 * All rights reserved.
 * See LICENSE for license details.
 */
use std::{error::Error, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricRecordError {
    #[error("Operation has reached maximum time limit {0:?}")]
    Timeout(Duration),

    // Allows returning any error from that supports Error trait
    #[error(transparent)]
    Dynamic(#[from] Box<dyn Error>),
}

impl From<std::io::Error> for MetricRecordError {
    fn from(value: std::io::Error) -> Self {
        MetricRecordError::Dynamic(Box::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn can_be_created_from_io_error() {
        let _error: MetricRecordError = Error::from(ErrorKind::InvalidData).into();
    }
}
