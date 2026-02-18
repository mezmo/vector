mod config;
mod integration_tests;
mod record;

use crate::mezmo::user_trace::UserLoggingError;
use aws_sdk_kinesis::{
    Client, error::ProvideErrorMetadata, operation::put_records::PutRecordsError,
    types::PutRecordsRequestEntry,
};
use aws_smithy_runtime_api::client::{orchestrator::HttpResponse, result::SdkError};

use vrl::value::Value;

pub use self::config::KinesisStreamsSinkConfig;
pub use super::{
    config::{KinesisSinkBaseConfig, build_sink},
    record::{Record, SendRecord},
    request_builder,
    service::{KinesisResponse, KinesisService},
    sink,
};

pub type KinesisError = PutRecordsError;
pub type KinesisRecord = PutRecordsRequestEntry;
pub type KinesisClient = Client;

impl UserLoggingError for SdkError<KinesisError, HttpResponse> {
    fn log_msg(&self) -> Option<Value> {
        match &self {
            SdkError::ServiceError(inner) => inner.err().message().map(Into::into),
            _ => None, // Other errors are not user-facing
        }
    }
}
