use crate::aws::AwsAuthentication;
use crate::config::ProxyConfig;
use crate::tls::TlsConfig;
use aws_types::region::Region;
use vector_lib::configurable::configurable_component;

use crate::sinks::aws_s3::file_consolidation_processor::FileConsolidationProcessor;
use crate::{aws::create_client, common::s3::S3ClientBuilder};
use tokio::task::JoinHandle;

const DEFAULT_BASE_PATH: &str = "";
const DEFAULT_OUTPUT_FORMAT: &str = "ndjson";

/// File Consolidation
/// Depending on the configuration of the sink and the throughput of data,
/// S3 may receive hundreds and thousands of files. This is unmanageable from
/// the customer perspective. Instead of increasing the memory or disk footprint
/// locally, allow everything to process and later on combine all the files
///
/// Assumption(s):
/// 1. All files within the bucket directory are of the same format configured
/// to the sink
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileConsolidationConfig {
    /// boolean indicating if the consolidation process is enabled
    pub enabled: bool,

    /// Indicates the file consolidation should occur every 'X' milliseconds
    pub process_every_ms: u64,

    /// Indicates the size of the consolidation file that is produced
    pub requested_size_bytes: i64,

    /// Indicates the output format (text, json, ndjson)
    /// defaults to ndjson for backwards compatibility
    pub output_format: Option<String>,

    /// Indicates the base path to start consolidation
    pub base_path: Option<String>,
}

impl Default for FileConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            process_every_ms: 600000,        // 10 min
            requested_size_bytes: 500000000, // 500 MB
            output_format: Some(DEFAULT_BASE_PATH.to_string()),
            base_path: Some(DEFAULT_OUTPUT_FORMAT.to_string()),
        }
    }
}

// handles consolidating the small files within AWS into much larger files
#[derive(Debug, Default)]
pub struct FileConsolidatorAsync {
    auth: AwsAuthentication,
    region: Option<Region>,
    endpoint: Option<String>,
    proxy: ProxyConfig,
    tls_options: Option<TlsConfig>,
    file_consolidation_config: FileConsolidationConfig,
    bucket: String,
    join_handle: Option<JoinHandle<()>>,
}

impl AsRef<FileConsolidatorAsync> for FileConsolidatorAsync {
    fn as_ref(&self) -> &FileConsolidatorAsync {
        self
    }
}

impl FileConsolidatorAsync {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        auth: AwsAuthentication,
        region: Option<Region>,
        endpoint: Option<String>,
        proxy: ProxyConfig,
        tls_options: Option<TlsConfig>,
        file_consolidation_config: FileConsolidationConfig,
        bucket: String,
    ) -> FileConsolidatorAsync {
        FileConsolidatorAsync {
            auth,
            region,
            endpoint,
            proxy,
            tls_options,
            file_consolidation_config,
            bucket,
            join_handle: None,
        }
    }

    pub fn start(&mut self) -> bool {
        // default situation so the config isn't enabled
        if !self.file_consolidation_config.enabled {
            return false;
        }

        let base_path = self
            .file_consolidation_config
            .base_path
            .clone()
            .unwrap_or(DEFAULT_BASE_PATH.to_string())
            .clone();
        let output_format = self
            .file_consolidation_config
            .output_format
            .clone()
            .unwrap_or(DEFAULT_OUTPUT_FORMAT.to_string())
            .clone();

        if self.join_handle.is_some() {
            info!(
                message =
                    "bucket={}, base_path={}, Thread for S3 file consolidation already in progress",
                bucket = self.bucket,
                key_prefix = base_path,
            );
            return false;
        }

        info!(
            message = "bucket={}, base_path={}, Initiating thread for S3 file consolidation",
            bucket = self.bucket,
            key_prefix = base_path,
        );

        const TEN_MINUTES_MS: u64 = 10 * 60 * 1000;

        let process_every_ms = if self.file_consolidation_config.process_every_ms > 0 {
            self.file_consolidation_config.process_every_ms
        } else {
            TEN_MINUTES_MS
        };

        let box_bucket = Box::new(self.bucket.clone());
        let box_base_path = Box::new(base_path.clone());
        let box_output_format = Box::new(output_format.clone());
        let box_auth = Box::new(self.auth.clone());
        let box_region = Box::new(self.region.clone());
        let box_endpoint = Box::new(self.endpoint.clone());
        let box_proxy = Box::new(self.proxy.clone());
        let box_tls = Box::new(self.tls_options.clone());
        let box_requested_size_bytes =
            Box::new(self.file_consolidation_config.requested_size_bytes);

        let spawned = tokio::spawn(async move {
            let client = match create_client::<S3ClientBuilder>(
                &box_auth,
                (*box_region).clone(),
                (*box_endpoint).clone(),
                &box_proxy,
                &box_tls,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        ?e,
                        "bucket={}, base_path={} Failed to create s3 client for consolidation",
                        (*box_bucket).clone(),
                        (*box_base_path).clone(),
                    );
                    return;
                }
            };

            loop {
                let start_time = tokio::time::Instant::now();

                info!(
                    message = "bucket={}, base_path={}, Starting S3 file consolidation",
                    bucket = (*box_bucket).clone(),
                    base_path = (*box_base_path).clone(),
                );

                let processor = FileConsolidationProcessor::new(
                    &client,
                    (*box_bucket).clone(),
                    *box_requested_size_bytes,
                    (*box_base_path).clone(),
                    (*box_output_format).clone(),
                );

                processor.run().await;
                info!(
                    message = "bucket={}, base_path={}, Completed S3 file consolidation",
                    bucket = (*box_bucket).clone(),
                    base_path = (*box_base_path).clone(),
                );

                // determine how long this action took to complete and await
                // the duration necessary to restart on the requested interval
                let elapsed = start_time.elapsed().as_millis();
                let diff = process_every_ms - elapsed as u64;
                if diff > 0 {
                    info!(
                        message =
                            "bucket={}, base_path={}, processing time={} ms, restarting in {} ms",
                        bucket = (*box_bucket).clone(),
                        base_path = (*box_base_path).clone(),
                        elapsed,
                        diff
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(diff)).await;
                }
            }
        });

        self.join_handle = Some(spawned);
        true
    }

    pub fn stop(&mut self) -> bool {
        // default situation so the config isn't enabled
        if !self.file_consolidation_config.enabled {
            return false;
        }

        let base_path = self
            .file_consolidation_config
            .base_path
            .clone()
            .unwrap_or(DEFAULT_BASE_PATH.to_string())
            .clone();

        info!(
            message = "Triggering shutdown for S3 file consolidation",
            bucket = self.bucket,
            base_path = base_path,
        );

        if let Some(h) = self.join_handle.take() {
            h.abort();
        }

        info!(
            message = "Shutdown for S3 file consolidation complete",
            bucket = self.bucket,
            base_path = base_path,
        );

        true
    }
}
