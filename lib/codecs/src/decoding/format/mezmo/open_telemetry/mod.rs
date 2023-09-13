mod metric_parser;

use bytes::Bytes;

use smallvec::SmallVec;

use crate::decoding::format::Deserializer;
use crate::decoding::FramingConfig;

use vector_core::{
    config::{DataType, LogNamespace},
    schema,
};

use vector_core::event::Event;

use vrl::value::kind::Collection;
use vrl::value::Kind;

use opentelemetry_rs::Error as OpenTelemetryError;

/// OpenTelemetry protobuf deserializer error list
#[derive(Debug, snafu::Snafu)]
pub enum DeserializerError {
    /// Protobuf parser error
    ProtobufParseError {
        /// The original error
        source: OpenTelemetryError,
    },
    /// Protobuf validation error
    ProtobufValidationError {
        /// The original error
        source: OpenTelemetryError,
    },
}

/// The OpenTelemetry metrics deserializer
#[derive(Clone, Debug, Default)]
pub struct OpenTelemetryMetricDeserializer;

impl OpenTelemetryMetricDeserializer {
    /// Output type of the Deserializer
    ///
    /// OpenTelemetryMetricDeserializer returns vector Log types encoding
    /// Metrics in the standard Mezmo format
    pub fn output_type() -> DataType {
        DataType::Log
    }

    /// Schema definition for the Deserializer
    pub fn schema_definition(log_namespace: LogNamespace) -> schema::Definition {
        schema::Definition::new_with_default_metadata(
            Kind::object(Collection::empty()),
            [log_namespace],
        )
    }

    /// Default Stream Framing for the Deserializer
    pub fn default_stream_framing() -> FramingConfig {
        FramingConfig::NewlineDelimited {
            newline_delimited: Default::default(),
        }
    }

    /// Content Type expected by Deserializer
    pub const fn content_type(_framer: &FramingConfig) -> &'static str {
        "text/plain"
    }
}

// import prometheus remote write types
impl Deserializer for OpenTelemetryMetricDeserializer {
    fn parse(
        &self,
        bytes: Bytes,
        _log_namespace: LogNamespace,
    ) -> vector_common::Result<SmallVec<[Event; 1]>> {
        // Convert Open Telemetry write request metrics into vector_core::event::LogEvent
        // according to our format
        // See lib/vector-core/src/event/metric/mezmo.rs from_metric for an
        // example of converting the internal vector metric format

        metric_parser::parse_metrics_request(&bytes[..])
    }
}
