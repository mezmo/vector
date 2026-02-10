mod log_parser;
mod metric_parser;
mod trace_parser;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use opentelemetry_rs::opentelemetry::common::{AnyValue, AnyValueOneOfvalue, KeyValue};
use std::borrow::Cow;
use std::cell::Cell;

use smallvec::SmallVec;

use crate::decoding::FramingConfig;
use crate::decoding::format::Deserializer;

use vector_core::{
    config::{DataType, LogNamespace},
    event::{
        Value,
        metric::mezmo::{IntoValue, from_f64_or_zero},
    },
    schema,
};

use vector_core::event::Event;

use vrl::value::Kind;
use vrl::value::kind::Collection;

use opentelemetry_rs::Error as OpenTelemetryError;

thread_local! {
    static OTLP_REQUEST_COUNTER: Cell<u64> = const { Cell::new(0) };
}

const MAX_METADATA_SIZE: usize = 32 * 1024;
const NANO_RATIO: u64 = 1_000_000_000;

pub fn nano_to_timestamp(time_unix_nano: u64) -> Value {
    Value::Timestamp(if time_unix_nano > 0 {
        let ms: i64 = (time_unix_nano / NANO_RATIO).try_into().unwrap();
        let nanos: u32 = (time_unix_nano % NANO_RATIO) as u32;
        DateTime::from_timestamp(ms, nanos).expect("timestamp should be a valid timestamp")
    } else {
        Utc::now()
    })
}

// WORNING: This function cannot be used within async/await context
pub fn get_uniq_request_id() -> [u8; 8] {
    OTLP_REQUEST_COUNTER.with(|cell| {
        let next_value = cell.get().wrapping_add(1);
        cell.set(next_value);
        next_value.to_be_bytes()
    })
}

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
        FramingConfig::NewlineDelimited(Default::default())
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

/// The OpenTelemetry logs deserializer
#[derive(Clone, Debug, Default)]
pub struct OpenTelemetryLogDeserializer;

impl OpenTelemetryLogDeserializer {
    /// Output type of the Deserializer
    ///
    /// OpenTelemetryLogDeserializer returns vector Log types encoding
    /// Logs in the standard Mezmo format
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
        FramingConfig::NewlineDelimited(Default::default())
    }

    /// Content Type expected by Deserializer
    pub const fn content_type(_framer: &FramingConfig) -> &'static str {
        "text/plain"
    }
}

// import prometheus remote write types
impl Deserializer for OpenTelemetryLogDeserializer {
    fn parse(
        &self,
        bytes: Bytes,
        _log_namespace: LogNamespace,
    ) -> vector_common::Result<SmallVec<[Event; 1]>> {
        // Convert Open Telemetry write request logs into vector_core::event::LogEvent

        log_parser::parse_logs_request(&bytes[..])
    }
}

/// The OpenTelemetry traces deserializer
#[derive(Clone, Debug, Default)]
pub struct OpenTelemetryTraceDeserializer;

impl OpenTelemetryTraceDeserializer {
    /// Output type of the Deserializer
    ///
    /// OpenTelemetryTraceDeserializer returns vector Trace types encoding
    /// Traces in the standard Mezmo format
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
        FramingConfig::NewlineDelimited(Default::default())
    }

    /// Content Type expected by Deserializer
    pub const fn content_type(_framer: &FramingConfig) -> &'static str {
        "text/plain"
    }
}

// import prometheus remote write types
impl Deserializer for OpenTelemetryTraceDeserializer {
    fn parse(
        &self,
        bytes: Bytes,
        _log_namespace: LogNamespace,
    ) -> vector_common::Result<SmallVec<[Event; 1]>> {
        // Convert Open Telemetry write request traces into vector_core::event::LogEvent

        trace_parser::parse_traces_request(&bytes[..])
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct OpenTelemetryKeyValue<'a> {
    pub attributes: Vec<KeyValue<'a>>,
}

impl IntoValue for OpenTelemetryKeyValue<'_> {
    fn to_value(&self) -> Value {
        self.attributes
            .iter()
            .filter_map(|key_value| match &key_value.value {
                Some(any_value) => {
                    let value = OpenTelemetryAnyValue {
                        value: any_value.clone(),
                    };
                    Some((key_value.key.to_string(), value.to_value()))
                }
                None => None,
            })
            .collect::<Value>()
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct OpenTelemetryAnyValue<'a> {
    pub value: AnyValue<'a>,
}

impl IntoValue for OpenTelemetryAnyValue<'_> {
    fn to_value(&self) -> Value {
        match &self.value.value {
            AnyValueOneOfvalue::string_value(val) if val.is_empty() => Value::Null,
            AnyValueOneOfvalue::string_value(val) => {
                let length = std::cmp::min(val.len(), MAX_METADATA_SIZE);
                Value::from(Cow::Borrowed(&val.as_ref()[..length]))
            }
            AnyValueOneOfvalue::bool_value(val) => Value::Boolean(*val),
            AnyValueOneOfvalue::int_value(val) => Value::Integer(*val),
            AnyValueOneOfvalue::double_value(val) => from_f64_or_zero(*val),
            AnyValueOneOfvalue::bytes_value(value) => Value::from(&value[..]),
            AnyValueOneOfvalue::array_value(val_list) => Value::Array(
                val_list
                    .values
                    .iter()
                    .map(|any_value| {
                        let metric_any_value = OpenTelemetryAnyValue {
                            value: any_value.clone(),
                        };
                        metric_any_value.to_value()
                    })
                    .collect(),
            ),
            AnyValueOneOfvalue::kvlist_value(kv_list) => kv_list
                .values
                .iter()
                .filter_map(|key_value| match &key_value.value {
                    Some(any_value) => {
                        let metric_any_value = OpenTelemetryAnyValue {
                            value: any_value.clone(),
                        };
                        Some((key_value.key.to_string(), metric_any_value.to_value()))
                    }
                    None => None,
                })
                .collect::<Value>(),
            AnyValueOneOfvalue::None => Value::Null,
        }
    }
}
