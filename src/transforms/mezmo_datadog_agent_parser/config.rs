use vector_lib::config::{clone_input_definitions, LogNamespace};
use vector_lib::configurable::configurable_component;
use vector_lib::lookup::lookup_v2::ConfigTargetPath;

use crate::{
    config::{
        log_schema, DataType, GenerateConfig, Input, OutputId, TransformConfig, TransformContext,
        TransformOutput,
    },
    schema,
    transforms::Transform,
};

use super::{
    MezmoDatadogAgentParser, LOGS_OUTPUT, METRICS_OUTPUT, TRACES_OUTPUT, UNMATCHED_OUTPUT,
};

fn default_event_type_path() -> ConfigTargetPath {
    ConfigTargetPath(
        log_schema()
            .metadata_key_target_path()
            .expect("metadata key must exist")
            .clone()
            .with_field_appended("x-mezmo-dd-event-type"),
    )
}

fn default_payload_version_path() -> ConfigTargetPath {
    ConfigTargetPath(
        log_schema()
            .message_key_target_path()
            .expect("message key must exist")
            .clone()
            .with_field_appended("mezmo_payload_version"),
    )
}

/// Mapping of event type string values to internal types.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(default)]
pub struct EventTypeValues {
    /// Value that identifies a log event.
    pub log: String,

    /// Value that identifies a metric event.
    pub metric: String,

    /// Value that identifies a trace event.
    pub trace: String,

    /// Value that identifies a sketch event.
    pub sketch: String,
}

impl Default for EventTypeValues {
    fn default() -> Self {
        Self {
            log: "log".into(),
            metric: "metric".into(),
            trace: "trace".into(),
            sketch: "sketch".into(),
        }
    }
}

/// Configuration for the `mezmo_datadog_agent_parser` transform.
#[configurable_component(transform(
    "mezmo_datadog_agent_parser",
    "Parse and normalize Datadog agent payloads into structured log events."
))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MezmoDatadogAgentParserConfig {
    /// Path to the event type field in the incoming data.
    #[serde(default = "default_event_type_path")]
    pub event_type_path: ConfigTargetPath,

    /// Mapping of event type string values to internal types.
    #[serde(default)]
    pub event_type_values: EventTypeValues,

    /// Path to the payload version field in the incoming data.
    #[serde(default = "default_payload_version_path")]
    pub payload_version_path: ConfigTargetPath,

    /// Remove the event type field after identifying the event.
    #[serde(default = "crate::serde::default_true")]
    pub strip_event_type: bool,

    /// Remove the payload version field before sending to output.
    #[serde(default = "crate::serde::default_true")]
    pub strip_payload_version: bool,

    /// Route unrecognized event types to the _unmatched output instead of dropping.
    #[serde(default = "crate::serde::default_true")]
    pub reroute_unmatched: bool,
}

impl Default for MezmoDatadogAgentParserConfig {
    fn default() -> Self {
        Self {
            event_type_path: default_event_type_path(),
            event_type_values: EventTypeValues::default(),
            payload_version_path: default_payload_version_path(),
            strip_event_type: false,
            strip_payload_version: false,
            reroute_unmatched: true,
        }
    }
}

impl GenerateConfig for MezmoDatadogAgentParserConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self::default()).unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_datadog_agent_parser")]
impl TransformConfig for MezmoDatadogAgentParserConfig {
    async fn build(&self, _context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::synchronous(MezmoDatadogAgentParser::new(self)))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        input_definitions: &[(OutputId, schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        let mut outputs = vec![
            TransformOutput::new(DataType::Log, clone_input_definitions(input_definitions))
                .with_port(LOGS_OUTPUT),
            TransformOutput::new(DataType::Log, clone_input_definitions(input_definitions))
                .with_port(METRICS_OUTPUT),
            TransformOutput::new(DataType::Log, clone_input_definitions(input_definitions))
                .with_port(TRACES_OUTPUT),
        ];

        if self.reroute_unmatched {
            outputs.push(
                TransformOutput::new(DataType::Log, clone_input_definitions(input_definitions))
                    .with_port(UNMATCHED_OUTPUT),
            );
        }

        outputs
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<MezmoDatadogAgentParserConfig>();
    }
}
