mod config;
mod sink;

mod integration_tests;
mod integration_tests_mezmo;

pub use self::config::S3SinkConfig;

// MEZMO: added files for s3-sink file consolidation
pub mod file_consolidation_processor;
pub mod file_consolidator_async;
