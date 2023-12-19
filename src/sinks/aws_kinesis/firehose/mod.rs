mod config;
mod integration_tests;
mod record;
mod tests;

use crate::mezmo::user_trace::UserLoggingError;
use aws_sdk_firehose::{
    error::PutRecordBatchError, model::Record as FRecord, types::SdkError, Client,
};
use vrl::value::Value;

pub use super::{
    config::{build_sink, KinesisSinkBaseConfig},
    record::{Record, SendRecord},
    request_builder,
    service::{KinesisResponse, KinesisService},
    sink,
};

pub use self::config::KinesisFirehoseSinkConfig;

pub type KinesisError = PutRecordBatchError;
pub type KinesisRecord = FRecord;
pub type KinesisClient = Client;

impl UserLoggingError for SdkError<KinesisError> {
    fn log_msg(&self) -> Option<Value> {
        match &self {
            SdkError::ServiceError(inner) => inner.err().message().map(Into::into),
            _ => None, // Other errors are not user-facing
        }
    }
}
