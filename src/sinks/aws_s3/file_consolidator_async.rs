use vector_config::configurable_component;

use crate::aws::AwsAuthentication;
use crate::config::ProxyConfig;
use crate::tls::TlsConfig;
use aws_types::region::Region;

use crate::sinks::aws_s3::file_consolidation_processor::FileConsolidationProcessor;
use crate::{aws::create_client, common::s3::S3ClientBuilder};
use tokio::task::JoinHandle;

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
#[derive(Clone, Debug, Copy)]
#[serde(deny_unknown_fields)]
pub struct FileConsolidationConfig {
    /// boolean indicating if the consolidation process is enabled
    pub enabled: bool,

    /// Indicates the file consolidation should occur every 'X' milliseconds
    pub process_every_ms: u64,

    /// Indicates the size of the consolidation file that is produced
    pub requested_size_bytes: i64,
}

impl Default for FileConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            process_every_ms: 600000,        // 10 min
            requested_size_bytes: 500000000, // 500 MB
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
    key_prefix: String,
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
        key_prefix: String,
    ) -> FileConsolidatorAsync {
        FileConsolidatorAsync {
            auth,
            region,
            endpoint,
            proxy,
            tls_options,
            file_consolidation_config,
            bucket,
            key_prefix,
            join_handle: None,
        }
    }

    pub fn start(&mut self) -> bool {
        // default situation so the config isn't enabled
        if !self.file_consolidation_config.enabled {
            return false;
        }

        if self.join_handle.is_some() {
            info!(
                message =
                    "bucket={}, prefix={}, Thread for S3 file consolidation already in progress",
                bucket = self.bucket,
                key_prefix = self.key_prefix,
            );
            return false;
        }

        info!(
            message = "bucket={}, prefix={}, Initiating thread for S3 file consolidation",
            bucket = self.bucket,
            key_prefix = self.key_prefix,
        );

        const TEN_MINUTES_MS: u64 = 10 * 60 * 1000;

        let process_every_ms = if self.file_consolidation_config.process_every_ms > 0 {
            self.file_consolidation_config.process_every_ms
        } else {
            TEN_MINUTES_MS
        };

        let box_bucket = Box::new(self.bucket.clone());
        let box_key_prefix = Box::new(self.key_prefix.clone());
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
                true,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        ?e,
                        "bucket={}, key_prefix={} Failed to create s3 client for consolidation",
                        (*box_bucket).clone(),
                        (*box_key_prefix).clone()
                    );
                    return;
                }
            };

            loop {
                let start_time = tokio::time::Instant::now();

                info!(
                    message = "bucket={}, prefix={}, Starting S3 file consolidation",
                    bucket = (*box_bucket).clone(),
                    key_prefix = (*box_key_prefix).clone(),
                );

                let processor = FileConsolidationProcessor::new(
                    &client,
                    (*box_bucket).clone(),
                    (*box_key_prefix).clone(),
                    *box_requested_size_bytes,
                );

                processor.run().await;
                info!(
                    message = "bucket={}, prefix={}, Completed S3 file consolidation",
                    bucket = (*box_bucket).clone(),
                    key_prefix = (*box_key_prefix).clone(),
                );

                // determine how long this action took to complete and await
                // the duration necessary to restart on the requested interval
                let elapsed = start_time.elapsed().as_millis();
                let diff = process_every_ms - elapsed as u64;
                if diff > 0 {
                    info!(
                        message =
                            "bucket={}, prefix={}, processing time={} ms, restarting in {} ms",
                        bucket = (*box_bucket).clone(),
                        key_prefix = (*box_key_prefix).clone(),
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

        info!(
            message = "Triggering shutdown for S3 file consolidation",
            bucket = self.bucket,
            key_prefix = self.key_prefix,
        );

        if let Some(h) = self.join_handle.take() {
            h.abort();
        }

        info!(
            message = "Shutdown for S3 file consolidation complete",
            bucket = self.bucket,
            key_prefix = self.key_prefix,
        );

        true
    }
}
