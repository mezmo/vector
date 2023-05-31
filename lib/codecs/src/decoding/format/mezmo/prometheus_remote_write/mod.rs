mod metric_sample_types;
mod parser;

use bytes::Bytes;

use smallvec::SmallVec;
use snap::raw::Decoder;

use crate::decoding::format::Deserializer;
use crate::decoding::FramingConfig;

use vector_core::{
    config::{DataType, LogNamespace},
    schema,
};

use vector_core::event::Event;

#[derive(Clone, Debug, Default)]
pub struct PrometheusRemoteWriteDeserializer;

impl PrometheusRemoteWriteDeserializer {
    /// Output type of the Deserializer
    ///
    /// PrometheusRemoteWriteDeserializer returns vector Log types encoding
    /// Metrics in the standard Mezmo format
    pub fn output_type() -> DataType {
        DataType::Log
    }

    /// Schema definition for the Deserializer
    pub fn schema_definition(log_namespace: LogNamespace) -> schema::Definition {
        schema::Definition::new_with_default_metadata(
            value::Kind::object(value::kind::Collection::empty()),
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
impl Deserializer for PrometheusRemoteWriteDeserializer {
    fn parse(
        &self,
        bytes: Bytes,
        _log_namespace: LogNamespace,
    ) -> vector_common::Result<SmallVec<[Event; 1]>> {
        // stub
        let bytes = Decoder::new().decompress_vec(&bytes[..])?;

        // Convert Prometheus write request metrics into vector_core::event::LogEvent
        // according to our format
        // See lib/vector-core/src/event/metric/mezmo.rs from_metric for an
        // example of converting the internal vector metric format

        let write_req = parser::parse_write_req(&bytes[..], _log_namespace);

        write_req
    }
}
