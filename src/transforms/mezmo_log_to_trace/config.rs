use std::collections::HashMap;

use vector_lib::config::{LogNamespace, TransformOutput};
use vector_lib::configurable::configurable_component;

use crate::config::{DataType, GenerateConfig, Input, OutputId, TransformConfig, TransformContext};
use crate::transforms::Transform;

use super::LogToTrace;

/// Configuration for the `mezmo_log_to_trace` transform.
#[configurable_component(transform("mezmo_log_to_trace"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LogToTraceConfig {
    /// When true, drop all metadata from converted trace events.
    #[serde(default)]
    pub exclude_metadata: bool,
}

impl GenerateConfig for LogToTraceConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self::default()).unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_log_to_trace")]
impl TransformConfig for LogToTraceConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(LogToTrace::new(
            self.clone(),
            context.mezmo_ctx.clone(),
        )))
    }

    fn input(&self) -> Input {
        Input::new(DataType::Log | DataType::Trace)
    }

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        _: &[(OutputId, crate::schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Trace, HashMap::new())]
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}
