use smallvec::SmallVec;
use vector_config::configurable_component;
use vector_core::{config::LogNamespace, event::Value};

use codecs::decoding::MezmoDeserializer;

use crate::mezmo::user_trace::handle_deserializer_error;

use crate::{
    config::{DataType, GenerateConfig, Input, Output, TransformConfig, TransformContext},
    event::{Event, LogEvent},
    mezmo::MezmoContext,
    schema,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

use lookup::PathPrefix;
use vector_core::config::log_schema;

/// The Enum to choose a protobuf vendor.
#[configurable_component]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtobufVendors {
    /// This is a description
    #[default]
    OpenTelemetryLogs,
}

/// Configuration for the `protobuf_to_metric` transform.
#[configurable_component(transform("protobuf_to_metric"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ProtobufToLogConfig {
    /// This is a description
    #[serde(default)]
    pub vendor: ProtobufVendors,
}

#[derive(Debug, Clone)]
pub struct ProtobufToLog {
    #[allow(dead_code)]
    config: ProtobufToLogConfig,

    /// The mezmo context used to surface errors
    mezmo_ctx: Option<MezmoContext>,
}

impl GenerateConfig for ProtobufToLogConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            vendor: ProtobufVendors::default(),
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "protobuf_to_metric")]
impl TransformConfig for ProtobufToLogConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(ProtobufToLog::new(
            self.clone(),
            context.mezmo_ctx.clone(),
        )))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(&self, _: &schema::Definition, _: LogNamespace) -> Vec<Output> {
        vec![Output::default(DataType::Log)]
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}

impl ProtobufToLog {
    pub const fn new(config: ProtobufToLogConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        ProtobufToLog { config, mezmo_ctx }
    }
}

impl FunctionTransform for ProtobufToLog {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        let log = &event.into_log();
        let mut buffer: Option<SmallVec<[Event; 1]>> = None;

        let message = log
            .get_message()
            .and_then(Value::as_bytes)
            .expect("Log event has no message");

        let deserializer = match self.config.vendor {
            ProtobufVendors::OpenTelemetryLogs => {
                MezmoDeserializer::build(&MezmoDeserializer::OpenTelemetryLogs)
            }
        };

        match deserializer.parse(message.clone(), LogNamespace::Legacy) {
            Ok(metrics) => {
                buffer = Some(metrics);
            }
            Err(err) => {
                handle_deserializer_error(&self.mezmo_ctx, err);
            }
        }

        // Log generation was successful, publish it
        if let Some(mut events) = buffer {
            while let Some(event) = events.pop() {
                unimplemented!();
            }
        }
    }
}
