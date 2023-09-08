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
use vrl::value::kind::Collection;
use vrl::value::Kind;

use vector_core::event::Event;

#[derive(Debug, snafu::Snafu)]
pub enum DeserializerError {
    Parse {
        #[snafu(source)]
        source: parser::ParseError,
    },
    Protobuf {
        #[snafu(source)]
        source: prometheus_remote_write::Error,
    },
}

#[derive(Clone, Debug, Default)]
pub struct PrometheusRemoteWriteDeserializer {
    metadata_cache: std::sync::Arc<std::sync::RwLock<parser::MetricMetadataGroups>>,
}

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
impl Deserializer for PrometheusRemoteWriteDeserializer {
    fn parse(
        &self,
        bytes: Bytes,
        _log_namespace: LogNamespace,
    ) -> vector_common::Result<SmallVec<[Event; 1]>> {
        // Convert Prometheus write request metrics into vector_core::event::LogEvent
        // according to our format
        // See lib/vector-core/src/event/metric/mezmo.rs from_metric for an
        // example of converting the internal vector metric format

        use prometheus_remote_write::{prometheus::WriteRequest, validation::StaticValidate};

        let bytes = Decoder::new().decompress_vec(&bytes[..])?;

        let mut write_req = WriteRequest::try_from(&bytes[..])
            .map_err(|source| DeserializerError::Protobuf { source })?;

        write_req
            .validate()
            .map_err(|source| DeserializerError::Protobuf { source })?;

        let WriteRequest {
            ref metadata,
            ref mut timeseries,
        } = write_req;

        if !metadata.is_empty() {
            self.metadata_cache
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .update_from_iter(metadata.iter());
        }

        let metric_types_lookup = self
            .metadata_cache
            .read()
            .unwrap_or_else(|e| e.into_inner());

        Ok(parser::parse_write_req(timeseries, &metric_types_lookup)
            .map_err(|source| DeserializerError::Parse { source })?)
    }
}

#[cfg(test)]
mod test {

    use super::PrometheusRemoteWriteDeserializer;
    use crate::decoding::format::Deserializer;

    use quick_protobuf::serialize_into_vec;

    use prometheus_remote_write::prometheus::{
        Label, MetricMetadata, MetricType, Sample, TimeSeries, WriteRequest,
    };

    use std::borrow::Cow;

    #[test]
    fn test_count() {
        let test_label = Label {
            name: Cow::Borrowed("__name__"),
            value: Cow::Borrowed("unknown"),
        };

        // Split metadata from timeseries
        let message_wr = WriteRequest {
            timeseries: vec![TimeSeries {
                exemplars: vec![],
                histograms: vec![],
                labels: vec![
                    test_label.clone(),
                    Label {
                        name: Cow::Borrowed("test_label"),
                        value: Cow::Borrowed("test_value"),
                    },
                ],
                samples: vec![Sample::default()],
            }],
            metadata: vec![],
        };

        let message_md = WriteRequest {
            timeseries: vec![],
            metadata: vec![MetricMetadata {
                help: Cow::from("help"),
                metric_family_name: Cow::from("unknown"),
                type_pb: MetricType::COUNTER,
                unit: Cow::from("unit"),
            }],
        };

        let deser = PrometheusRemoteWriteDeserializer::default();

        let mut compressor = snap::raw::Encoder::new();

        let out = compressor
            .compress_vec(&serialize_into_vec(&message_md).unwrap()[1..])
            .unwrap();
        let ret = deser
            .parse(out.into(), vector_core::config::LogNamespace::Legacy)
            .expect("Failed to parse");
        assert_eq!(ret.len(), 0);

        let out = compressor
            .compress_vec(&serialize_into_vec(&message_wr).unwrap()[1..])
            .unwrap();
        let ret = deser
            .parse(out.into(), vector_core::config::LogNamespace::Legacy)
            .expect("Failed to parse");

        assert_eq!(ret.len(), 1);
        assert_eq!(
            ret[0]
                .as_log()
                .get("message")
                .unwrap()
                .get("value")
                .unwrap()
                .get("type")
                .unwrap()
                .as_str()
                .unwrap(),
            "count"
        );
    }
}
