use metrics::counter;
use vector_core::internal_event::InternalEvent;

#[derive(Debug)]
pub struct MezmoGenerateConfigError {
    pub errors: Vec<String>,
    pub pipeline_id: Option<String>,
    pub revision_id: Option<String>,
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
        counter!("generate_config_error", 1);
    }
}
