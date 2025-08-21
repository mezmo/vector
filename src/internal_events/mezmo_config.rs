use std::time::Duration;

use http::StatusCode;
use metrics::{counter, gauge, histogram};
use vector_lib::internal_event::InternalEvent;

#[derive(Debug)]
pub struct MezmoGenerateConfigError {
    pub errors: Vec<String>,
    pub pipeline_id: Option<String>,
    pub revision_id: Option<String>,
    pub toml_version: Option<u32>,
    pub incremental: bool,
    pub cache_len: usize,
}

impl InternalEvent for MezmoGenerateConfigError {
    fn emit(self) {
        if !self.incremental {
            if self.cache_len > 0 {
                error!(
                    message = format!("Error while building the initial config with {} pipelines", self.cache_len),
                    errors = ?self.errors
                );
            } else {
                error!(
                    message = "Error building the initial config without any pipeline",
                    errors = ?self.errors
                );
            }
        } else {
            match (self.pipeline_id, self.revision_id) {
                (Some(pipeline_id), Some(revision_id)) => {
                    error!(
                        message = format!("Error building config for pipeline {} with revision {}", &pipeline_id, &revision_id),
                        errors = ?self.errors
                    );
                }
                (_, _) => {
                    error!(
                        message = "Error while building the config after a diff change",
                        errors = ?self.errors
                    );
                }
            }
        }
        counter!("mezmo_generate_config_error").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoConfigReload {
    pub elapsed: Duration,
    pub success: bool,
}

impl InternalEvent for MezmoConfigReload {
    fn emit(self) {
        histogram!("mezmo_config_reload_seconds", "success" => self.success.to_string())
            .record(self.elapsed);
    }
}

pub struct MezmoConfigCompile {
    pub elapsed: Duration,
}

impl InternalEvent for MezmoConfigCompile {
    fn emit(self) {
        histogram!("mezmo_config_compile_seconds").record(self.elapsed);
    }
}

pub struct MezmoConfigVrlValidation {
    pub elapsed: Duration,
}

impl InternalEvent for MezmoConfigVrlValidation {
    fn emit(self) {
        histogram!("mezmo_config_vrl_validation_seconds").record(self.elapsed);
    }
}

pub struct MezmoConfigVrlValidationError {
    pub failure_count: u64,
}

impl InternalEvent for MezmoConfigVrlValidationError {
    fn emit(self) {
        counter!("mezmo_config_vrl_validation_error").increment(self.failure_count);
    }
}

pub struct MezmoConfigBuilderCreate {
    pub revisions: usize,
}

impl InternalEvent for MezmoConfigBuilderCreate {
    fn emit(self) {
        counter!("mezmo_config_builder_created_total").increment(1);
        gauge!("mezmo_config_builder_revisions_total").set(self.revisions as f64);
    }
}

pub struct MezmoConfigServiceResponse<'a> {
    pub elapsed: Duration,
    pub url: &'a str,
    pub status: StatusCode,
}

impl InternalEvent for MezmoConfigServiceResponse<'_> {
    fn emit(self) {
        info!(message = "Config service response received.", url = self.url, status_code = ?self.status);
        histogram!(
            "mezmo_config_service_response_seconds",
            "success" => self.status.is_success().to_string())
        .record(self.elapsed);
    }
}

pub struct MezmoConfigBuildFailure {
    pub error: String,
}

impl InternalEvent for MezmoConfigBuildFailure {
    fn emit(self) {
        error!(message = format!("Error building the config incrementally: {}", self.error));
        counter!("mezmo_config_build_failure_total").increment(1);
    }
}

pub struct MezmoConfigReloadSignalSend {}

impl InternalEvent for MezmoConfigReloadSignalSend {
    fn emit(self) {
        info!("Sending reload config signal");
        counter!("mezmo_config_reload_signal_sent_total").increment(1);
    }
}

pub struct MezmoConfigReloadSignalReceive {}

impl InternalEvent for MezmoConfigReloadSignalReceive {
    fn emit(self) {
        counter!("mezmo_config_reload_signal_received_total").increment(1);
    }
}
