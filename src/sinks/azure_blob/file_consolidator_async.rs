use crate::sinks::azure_blob::file_consolidation_processor::FileConsolidationProcessor;
use tokio::task::JoinHandle;
use vector_lib::configurable::configurable_component;
use vector_lib::sensitive_string::SensitiveString;

/// File Consolidation
/// Depending on the configuration of the sink and the throughput of data,
/// Azure may receive hundreds and thousands of files. This is unmanageable from
/// the customer perspective. Instead of increasing the memory or disk footprint
/// locally, allow everything to process and later on combine all the files
///
/// Assumption(s):
/// 1. All files within the blob directory are of the same format configured
/// to the sink
#[configurable_component]
// #[derive(Clone, Debug, Copy)]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileConsolidationConfig {
    /// boolean indicating if the consolidation process is enabled
    pub enabled: bool,

    /// Indicates the file consolidation should occur every 'X' milliseconds
    pub process_every_ms: u64,

    /// Indicates the size of the consolidation file that is produced
    pub requested_size_bytes: u64,

    /// Indicates the output format (text, json, ndjson)
    pub output_format: String,

    /// Indicates the base path to start consolidation
    pub base_path: String,
}

impl Default for FileConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            process_every_ms: 600000,        // 10 min
            requested_size_bytes: 500000000, // 500 MB
            output_format: "ndjson".to_owned(),
            base_path: "".to_owned(),
        }
    }
}

// handles consolidating the small files within AWS into much larger files
#[derive(Debug, Default)]
pub struct FileConsolidatorAsync {
    connection_string: Option<SensitiveString>,
    container_name: String,
    file_consolidation_config: FileConsolidationConfig,
    join_handle: Option<JoinHandle<()>>,
}

impl AsRef<FileConsolidatorAsync> for FileConsolidatorAsync {
    fn as_ref(&self) -> &FileConsolidatorAsync {
        self
    }
}

impl FileConsolidatorAsync {
    pub const fn new(
        connection_string: Option<SensitiveString>,
        container_name: String,
        file_consolidation_config: FileConsolidationConfig,
    ) -> FileConsolidatorAsync {
        FileConsolidatorAsync {
            connection_string,
            container_name,
            file_consolidation_config,
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
                message = "container_name={}, base_path={} Thread for azure-blob file consolidation already in progress",
                container_name = self.container_name,
                base_path = self.file_consolidation_config.base_path.clone(),
            );
            return false;
        }

        info!(
            message = "container_name={}, base_path={}, Initiating thread for azure-blob file consolidation",
            container_name = self.container_name,
            base_path = self.file_consolidation_config.base_path.clone(),
        );

        const TEN_MINUTES_MS: u64 = 10 * 60 * 1000;

        let process_every_ms = if self.file_consolidation_config.process_every_ms > 0 {
            self.file_consolidation_config.process_every_ms
        } else {
            TEN_MINUTES_MS
        };

        let box_connection_string = Box::new(self.connection_string.as_ref().unwrap().clone());
        let box_container_name = Box::new(self.container_name.clone());
        let box_base_path = Box::new(self.file_consolidation_config.base_path.clone());
        let box_requested_size_bytes =
            Box::new(self.file_consolidation_config.requested_size_bytes);
        let box_output_format = Box::new(self.file_consolidation_config.output_format.clone());

        let build_client = crate::sinks::azure_common::config::build_client(
            box_connection_string.inner().to_string(),
            *box_container_name.clone(),
        );

        // double check that we were able to build the client
        if let Err(e) = build_client {
            error!(
                ?e,
                message = "container_name={}, base_path={}, Failed to build client for azure-blob file consolidation",
                bucket = *box_container_name.clone(),
                base_path = *box_base_path.clone(),
            );
            return false;
        }

        let client = build_client.unwrap();
        let spawned = tokio::spawn(async move {
            loop {
                let start_time = tokio::time::Instant::now();

                info!(
                    message =
                        "container_name={}, base_path={}, Starting azure-blob file consolidation",
                    bucket = *box_container_name.clone(),
                    base_path = *box_base_path.clone(),
                );

                let processor = FileConsolidationProcessor::new(
                    &client,
                    *box_container_name.clone(),
                    *box_base_path.clone(),
                    *box_requested_size_bytes,
                    *box_output_format.clone(),
                );

                processor.run().await;
                info!(
                    message =
                        "container_name={}, base_path={}, Completed azure-blob file consolidation",
                    container_name = *box_container_name.clone(),
                    base_path = *box_base_path.clone(),
                );

                // determine how long this action took to complete and await
                // the duration necessary to restart on the requested interval
                let elapsed = start_time.elapsed().as_millis();
                let diff = process_every_ms - elapsed as u64;
                if diff > 0 {
                    info!(
                        message = "container_name={}, base_path={}, processing time={} ms, restarting in {} ms",
                        container_name = *box_container_name.clone(),
                        base_path = *box_base_path.clone(),
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
            message = "Triggering shutdown for azure-blob file consolidation",
            container_name = self.container_name.clone(),
            base_path = self.file_consolidation_config.base_path.clone()
        );

        if let Some(h) = self.join_handle.take() {
            h.abort();
        }

        info!(
            message = "Shutdown for azure-blob file consolidation complete",
            container_name = self.container_name.clone(),
            base_path = self.file_consolidation_config.base_path.clone()
        );

        true
    }
}
