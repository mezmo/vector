mod config;
#[cfg(feature = "aws-kinesis-firehose-integration-tests")]
#[cfg(test)]
mod integration_tests;
mod record;
mod tests;

use crate::mezmo::user_trace::UserLoggingError;
use aws_sdk_firehose::{
    Client, error::ProvideErrorMetadata, operation::put_record_batch::PutRecordBatchError,
    types::Record as FRecord,
};
use aws_smithy_runtime_api::client::{orchestrator::HttpResponse, result::SdkError};

use vrl::value::Value;

pub use self::config::KinesisFirehoseSinkConfig;
pub use super::{
    config::{KinesisSinkBaseConfig, build_sink},
    record::{Record, SendRecord},
    request_builder,
    service::{KinesisResponse, KinesisService},
    sink,
};

pub type KinesisError = PutRecordBatchError;
pub type KinesisRecord = FRecord;
pub type KinesisClient = Client;

impl UserLoggingError for SdkError<KinesisError, HttpResponse> {
    fn log_msg(&self) -> Option<Value> {
        match &self {
            SdkError::ServiceError(inner) => inner.err().message().map(Into::into),
            _ => None, // Other errors are not user-facing
        }
    }
}
