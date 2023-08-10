use vector_config::configurable_component;
use vector_core::{
    config::{DataType, LogNamespace},
    schema,
};

use crate::decoding::FramingConfig;

pub mod open_telemetry;
mod prometheus_remote_write;

/// Mezmo Deserializers
#[configurable_component]
#[derive(Debug, Clone)]
#[serde(tag = "encoding", rename_all = "snake_case")]
#[configurable(metadata(docs::enum_tag_description = "Mezmo Deserializer variants"))]
pub enum MezmoDeserializer {
    /// Prometheus Remote Write config
    PrometheusRemoteWrite,

    /// Open Telemetry Metrics config
    OpenTelemetryMetrics,

    /// Open Telemetry Logs config
    OpenTelemetryLogs,
}

impl MezmoDeserializer {
    /// Build the Deserializer
    pub fn build(&self) -> Box<dyn crate::decoding::format::Deserializer> {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                Box::<prometheus_remote_write::PrometheusRemoteWriteDeserializer>::default()
            }
            OpenTelemetryMetrics => {
                Box::<open_telemetry::OpenTelemetryMetricDeserializer>::default()
            }
            OpenTelemetryLogs => Box::<open_telemetry::OpenTelemetryLogDeserializer>::default(),
        }
    }

    /// Output type of the Deserializer
    pub fn output_type(&self) -> DataType {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prometheus_remote_write::PrometheusRemoteWriteDeserializer::output_type()
            }
            OpenTelemetryMetrics => open_telemetry::OpenTelemetryMetricDeserializer::output_type(),
            OpenTelemetryLogs => open_telemetry::OpenTelemetryLogDeserializer::output_type(),
        }
    }

    /// Schema definition for the Deserializer
    pub fn schema_definition(&self, log_namespace: LogNamespace) -> schema::Definition {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prometheus_remote_write::PrometheusRemoteWriteDeserializer::schema_definition(
                    log_namespace,
                )
            }
            OpenTelemetryMetrics => {
                open_telemetry::OpenTelemetryMetricDeserializer::schema_definition(log_namespace)
            }
            OpenTelemetryLogs => {
                open_telemetry::OpenTelemetryLogDeserializer::schema_definition(log_namespace)
            }
        }
    }

    /// Default Stream Framing for the Deserializer
    pub fn default_stream_framing(&self) -> FramingConfig {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prometheus_remote_write::PrometheusRemoteWriteDeserializer::default_stream_framing()
            }
            OpenTelemetryMetrics => {
                open_telemetry::OpenTelemetryMetricDeserializer::default_stream_framing()
            }
            OpenTelemetryLogs => {
                open_telemetry::OpenTelemetryLogDeserializer::default_stream_framing()
            }
        }
    }

    /// Content Type expected by Deserializer
    pub const fn content_type(&self, framer: &FramingConfig) -> &'static str {
        use MezmoDeserializer::*;
        match self {
            PrometheusRemoteWrite => {
                prometheus_remote_write::PrometheusRemoteWriteDeserializer::content_type(framer)
            }
            OpenTelemetryMetrics => {
                open_telemetry::OpenTelemetryMetricDeserializer::content_type(framer)
            }
            OpenTelemetryLogs => open_telemetry::OpenTelemetryLogDeserializer::content_type(framer),
        }
    }
}
