use vector_config::configurable_component;
use vector_core::{
    config::{DataType, LogNamespace},
    schema,
};

use crate::decoding::FramingConfig;

mod prom_remote_write;

/// Mezmo Deserializers
#[configurable_component]
#[derive(Debug, Clone, Default)]
#[serde(tag = "encoding", rename_all = "snake_case")]
#[configurable(metadata(docs::enum_tag_description = "Mezmo Deserializer variants"))]
pub enum MezmoDeserializer {
    /// Prometheus Remote Write config
    #[default]
    PrometheusRemoteWrite,
}

impl MezmoDeserializer {
    /// Build the Deserializer
    pub fn build(&self) -> Box<dyn crate::decoding::format::Deserializer> {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                Box::<prom_remote_write::PrometheusRemoteWriteDeserializer>::default()
            }
        }
    }

    /// Output type of the Deserializer
    pub fn output_type(&self) -> DataType {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prom_remote_write::PrometheusRemoteWriteDeserializer::output_type()
            }
        }
    }

    /// Schema definition for the Deserializer
    pub fn schema_definition(&self, log_namespace: LogNamespace) -> schema::Definition {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prom_remote_write::PrometheusRemoteWriteDeserializer::schema_definition(
                    log_namespace,
                )
            }
        }
    }

    /// Default Stream Framing for the Deserializer
    pub fn default_stream_framing(&self) -> FramingConfig {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prom_remote_write::PrometheusRemoteWriteDeserializer::default_stream_framing()
            }
        }
    }

    /// Content Type expected by Deserializer
    pub const fn content_type(&self, framer: &FramingConfig) -> &'static str {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prom_remote_write::PrometheusRemoteWriteDeserializer::content_type(framer)
            }
        }
    }
}
