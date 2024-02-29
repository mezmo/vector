mod config;
mod request_builder;

// MEZMO: added files for azure-blob-sink file consolidation
pub mod file_consolidation_processor;
pub mod file_consolidator_async;

#[cfg(all(test, feature = "azure-blob-integration-tests"))]
pub mod integration_tests;
mod integration_tests_mezmo;
#[cfg(test)]
mod test;

pub use self::config::AzureBlobSinkConfig;
