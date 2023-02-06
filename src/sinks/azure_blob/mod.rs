mod config;
mod request_builder;

#[cfg(all(test, feature = "azure-blob-integration-tests"))]
pub mod integration_tests;
mod mezmo_integration_tests;
#[cfg(test)]
mod test;

pub use self::config::AzureBlobSinkConfig;
