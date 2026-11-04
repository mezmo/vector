use vector_lib::{
    config::log_schema,
    event::{Event, Value},
    lookup::PathPrefix,
};

use opentelemetry::{
    logs::{AnyValue as OtlpAnyValue, LogRecord, Severity},
    trace::{SpanContext, TraceState},
};
use opentelemetry_proto::tonic::logs::v1::SeverityNumber;
use opentelemetry_sdk::export::logs::LogData;
use std::borrow::Cow;

use crate::sinks::mezmo_opentelemetry::{
    models::{
        OpentelemetryModelMatch, OpentelemetryModelType, OpentelemetryResource, OpentelemetryScope,
        OpentelemetrySpanId, OpentelemetryTraceFlags, OpentelemetryTraceId, native_enum_number,
        native_envelope, native_time, value_to_otlp_any_value, value_to_system_time,
    },
    sink::OpentelemetrySinkError,
};

#[derive(Debug)]
pub struct OpentelemetryLogsModel(pub LogData);

impl OpentelemetryModelMatch for OpentelemetryLogsModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        if let Some(log) = event.maybe_as_log() {
            if native_envelope(log, "log").is_some() {
                return Some(OpentelemetryModelType::Logs);
            }

            let message = log.get_message();
            let metadata = log.get((PathPrefix::Event, log_schema().user_metadata_key()));

            if let Some(metadata) = metadata {
                let scope = metadata.get("scope");
                let trace_id = metadata.get("trace_id");
                let span_id = metadata.get("span_id");
                let severity_number = metadata.get("severity_number");
                let flags = metadata.get("flags");

                if message
                    .and(scope)
                    .and(trace_id)
                    .and(span_id)
                    .and(severity_number)
                    .and(flags)
                    .is_some()
                {
                    return Some(OpentelemetryModelType::Logs);
                }
            }
        }

        None
    }
}

impl OpentelemetryLogsModel {
    pub const fn new(log: LogData) -> Self {
        Self(log)
    }

    fn from_native(message: &Value, record: &Value) -> Self {
        let mut record_builder = LogRecord::builder();

        if let Some(body) = record.get("body") {
            record_builder = record_builder.with_body(value_to_otlp_any_value(body.clone()));
        }

        if let Some(value) = record.get("attributes")
            && let OtlpAnyValue::Map(attrs) = value_to_otlp_any_value(value.clone())
        {
            record_builder = record_builder.with_attributes(attrs.into_iter().collect::<Vec<_>>());
        }

        if let Some(timestamp) = native_time(record.get("timeUnixNano")) {
            record_builder = record_builder.with_timestamp(timestamp);
        }

        if let Some(timestamp) = native_time(record.get("observedTimeUnixNano")) {
            record_builder = record_builder.with_observed_timestamp(timestamp);
        }

        let raw_trace_id = record.get("traceId");
        let raw_span_id = record.get("spanId");

        if raw_trace_id.or(raw_span_id).is_some() {
            let trace_id: OpentelemetryTraceId = raw_trace_id.into();
            let span_id: OpentelemetrySpanId = raw_span_id.into();
            let trace_flags: OpentelemetryTraceFlags = record.get("flags").into();
            let context = SpanContext::new(
                trace_id.into(),
                span_id.into(),
                trace_flags.into(),
                false,
                TraceState::NONE,
            );

            record_builder = record_builder.with_span_context(&context);
        }

        if let Some(Value::Bytes(bytes)) = record.get("severityText") {
            let severity_text = String::from_utf8_lossy(bytes).into_owned();
            record_builder = record_builder.with_severity_text(Cow::from(severity_text));
        }

        // droppedAttributesCount is not read: the opentelemetry 0.22 LogRecord has no field for
        // it, so it is always encoded as 0.
        let mut log_record = record_builder.build();

        let severity_number = native_enum_number(record.get("severityNumber"), |name| {
            SeverityNumber::from_str_name(name).map(i32::from)
        });
        log_record.severity_number = severity_number.and_then(severity_from_number);

        Self::new(LogData {
            record: log_record,
            resource: Cow::Owned(OpentelemetryResource::from_native(message).into()),
            instrumentation: OpentelemetryScope::from_native(message).into(),
        })
    }
}

const fn severity_from_number(number: i64) -> Option<Severity> {
    match number {
        1 => Some(Severity::Trace),
        2 => Some(Severity::Trace2),
        3 => Some(Severity::Trace3),
        4 => Some(Severity::Trace4),
        5 => Some(Severity::Debug),
        6 => Some(Severity::Debug2),
        7 => Some(Severity::Debug3),
        8 => Some(Severity::Debug4),
        9 => Some(Severity::Info),
        10 => Some(Severity::Info2),
        11 => Some(Severity::Info3),
        12 => Some(Severity::Info4),
        13 => Some(Severity::Warn),
        14 => Some(Severity::Warn2),
        15 => Some(Severity::Warn3),
        16 => Some(Severity::Warn4),
        17 => Some(Severity::Error),
        18 => Some(Severity::Error2),
        19 => Some(Severity::Error3),
        20 => Some(Severity::Error4),
        21 => Some(Severity::Fatal),
        22 => Some(Severity::Fatal2),
        23 => Some(Severity::Fatal3),
        24 => Some(Severity::Fatal4),
        _ => None,
    }
}

impl TryFrom<Event> for OpentelemetryLogsModel {
    type Error = OpentelemetrySinkError;

    fn try_from(buf_event: Event) -> Result<Self, Self::Error> {
        let log = &buf_event.into_log();

        if let Some((message, record)) = native_envelope(log, "log") {
            return Ok(Self::from_native(message, record));
        }

        let mut record_builder = LogRecord::builder();

        if let Some(Value::Bytes(message)) = log.get_message() {
            let body = OtlpAnyValue::from(String::from_utf8_lossy(message).into_owned());
            record_builder = record_builder.with_body(body);
        }

        let mut severity_number = None;

        if let Some(metadata) = log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
            if let Some(value) = metadata.get("attributes")
                && let OtlpAnyValue::Map(attrs) = value_to_otlp_any_value(value.clone())
            {
                let attributes = attrs.clone().into_iter().collect::<Vec<_>>();

                record_builder = record_builder.with_attributes(attributes);
            }

            if let Some(timestamp) = metadata.get("time") {
                record_builder = record_builder.with_timestamp(value_to_system_time(timestamp));
            }

            if let Some(timestamp) = metadata.get("observed_timestamp") {
                record_builder =
                    record_builder.with_observed_timestamp(value_to_system_time(timestamp));
            }

            let raw_trace_id = metadata.get("trace_id");
            let raw_span_id = metadata.get("span_id");

            if raw_trace_id.or(raw_span_id).is_some() {
                let trace_id: OpentelemetryTraceId = raw_trace_id.into();
                let span_id: OpentelemetrySpanId = raw_span_id.into();
                let trace_flags: OpentelemetryTraceFlags = metadata.get("flags").into();
                let context = SpanContext::new(
                    trace_id.into(),
                    span_id.into(),
                    trace_flags.into(),
                    false,
                    TraceState::NONE,
                );

                record_builder = record_builder.with_span_context(&context);
            }

            if let Some(value) = metadata.get("severity_text") {
                let severity_text = if let Value::Bytes(bytes) = value {
                    Cow::from(String::from_utf8_lossy(bytes).into_owned())
                } else {
                    Cow::from("")
                };

                record_builder = record_builder.with_severity_text(severity_text);
            };

            if let Some(Value::Integer(number)) = metadata.get("severity_number") {
                severity_number = severity_from_number(*number);
            };
        }

        let resource = OpentelemetryResource::from(log);
        let scope = OpentelemetryScope::from(log);

        let mut log_record = record_builder.build();

        log_record.severity_number = severity_number;

        Ok(Self::new(LogData {
            record: log_record,
            resource: Cow::Owned(resource.into()),
            instrumentation: scope.into(),
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::event::Value;
    use chrono::DateTime;
    use std::collections::{BTreeMap, HashMap};
    use std::time::SystemTime;
    use vector_lib::event::{Event, EventMetadata, LogEvent, ObjectMap};

    use opentelemetry::{
        InstrumentationLibrary, Key, KeyValue,
        logs::{AnyValue as OtlpAnyValue, LogRecord, Severity, TraceContext},
        trace::{SpanId, TraceFlags, TraceId},
    };
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::export::logs::LogData;

    fn line_generator(index: usize) -> String {
        format!("opentelemetry test log index {index}")
    }

    fn event_generator(index: usize) -> Event {
        Event::Log(LogEvent::from(line_generator(index)))
    }

    pub fn generate_events<Gen: FnMut(usize) -> Event>(generator: Gen, count: usize) -> Vec<Event> {
        (0..count).map(generator).collect::<Vec<_>>()
    }

    #[test]
    fn test_otlp_sink_event_to_log_model() {
        let trace_id = [
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ];
        let span_id = [76, 114, 27, 243, 62, 60, 175, 143];

        let expected_resource_attribute = Value::Object(BTreeMap::from([
            ("str".into(), "bar".into()),
            ("int".into(), Value::from(100)),
            ("flt".into(), Value::from(100.123_f64)),
            ("bool".into(), Value::from(false)),
            ("empty".into(), Value::Null),
            (
                "list".into(),
                Value::Array(vec![
                    "bar".into(),
                    Value::from(100),
                    Value::from(100.123_f64),
                    Value::from(false),
                    Value::Null,
                ]),
            ),
        ]));
        let expected_scope_attributes = expected_resource_attribute.clone();
        let expected_log_attributes = Value::Object(BTreeMap::from([
            ("str".into(), "bar".into()),
            ("int".into(), Value::from(100)),
            ("flt".into(), Value::from(100.123_f64)),
            ("bool".into(), Value::from(false)),
            ("empty".into(), Value::Null),
            (
                "attributes".into(),
                Value::Object(BTreeMap::from([
                    ("str".into(), "bar".into()),
                    ("int".into(), Value::from(100)),
                    ("flt".into(), Value::from(100.123_f64)),
                    ("bool".into(), Value::from(false)),
                    ("empty".into(), Value::Null),
                ])),
            ),
        ]));

        let generator = |idx| {
            let mut event = event_generator(idx);
            let log = event.as_mut_log();

            log.insert(
                "metadata",
                Value::Object(BTreeMap::from([
                    ("attributes".into(), expected_log_attributes.clone()),
                    ("flags".into(), Value::from(1)),
                    (
                        "observed_timestamp".into(),
                        Value::from(
                            DateTime::from_timestamp(1_579_134_612_i64, 0o11_u32)
                                .expect("timestamp should be a valid timestamp"),
                        ),
                    ),
                    (
                        "time".into(),
                        Value::from(
                            DateTime::from_timestamp(1_579_134_612_i64, 0o11_u32)
                                .expect("timestamp should be a valid timestamp"),
                        ),
                    ),
                    ("severity_number".into(), 17.into()),
                    ("severity_text".into(), "ERROR".into()),
                    ("level".into(), "ERROR".into()),
                    (
                        "trace_id".into(),
                        Value::from(faster_hex::hex_string(&trace_id)),
                    ),
                    (
                        "span_id".into(),
                        Value::from(faster_hex::hex_string(&span_id)),
                    ),
                    ("resource".into(), expected_resource_attribute.clone()),
                    (
                        "scope".into(),
                        Value::Object(BTreeMap::from([
                            ("attributes".into(), expected_scope_attributes.clone()),
                            ("name".into(), "sone_scope_name".into()),
                            ("version".into(), "1.0.0".into()),
                        ])),
                    ),
                ])),
            );
            event
        };

        let mut logs: Vec<OpentelemetryLogsModel> = vec![];
        for event in generate_events(generator, 1) {
            match OpentelemetryLogsModel::try_from(event.clone()) {
                Ok(m) => logs.push(m),
                Err(err) => panic!("Log event cannot be converted to a model: {err:#?}"),
            }
        }

        let log_data: LogData = logs[0].0.clone();
        let record: LogRecord = log_data.record;
        let _resource: Resource = log_data.resource.into_owned();
        let _instrumentation: InstrumentationLibrary = log_data.instrumentation;

        assert!(record.event_name.is_none());
        assert!(record.timestamp.is_some());
        assert!(record.severity_text.is_some());
        assert!(record.severity_number.is_some());
        assert!(record.body.is_some());
        assert!(record.trace_context.is_some());
        assert!(record.attributes.is_some());

        let timestamp_duration = record
            .timestamp
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        assert_eq!(timestamp_duration.as_millis(), 1_579_134_612_000_u128);

        let observed_timestamp_duration = record
            .observed_timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        assert_eq!(
            observed_timestamp_duration.as_millis(),
            1_579_134_612_000_u128
        );

        assert_eq!(record.severity_text.unwrap().into_owned(), "ERROR");

        assert_eq!(record.severity_number.unwrap(), Severity::Error);

        assert_eq!(
            record.body.unwrap(),
            OtlpAnyValue::from("opentelemetry test log index 0".to_string())
        );

        let trace_context: TraceContext = record.trace_context.unwrap();
        assert_eq!(trace_context.trace_id, TraceId::from_bytes(trace_id));
        assert_eq!(trace_context.span_id, SpanId::from_bytes(span_id));
        assert!(trace_context.trace_flags.is_some());
        assert_eq!(trace_context.trace_flags.unwrap(), TraceFlags::SAMPLED);
    }

    const NATIVE_TRACE_ID: &str = "5f467fe7bf42676c05e20ba4a90e448e";
    const NATIVE_SPAN_ID: &str = "4c721bf33e3caf8f";

    fn native_log_event(record: ObjectMap) -> Event {
        Event::Log(LogEvent::from_map(
            btreemap! {
                "message" => btreemap! {
                    "type" => "log",
                    "resource" => btreemap! {
                        "attributes" => btreemap! { "service.name" => "checkout" },
                    },
                    "scope" => btreemap! {
                        "name" => "checkout.logger",
                        "version" => "1.4.0",
                        "attributes" => btreemap! { "scope.attr" => "scoped" },
                    },
                    "record" => record,
                },
            },
            EventMetadata::default(),
        ))
    }

    fn native_log_attributes() -> Value {
        Value::from(btreemap! {
            "str" => "bar",
            "int" => 100,
            "flt" => 1.5,
            "bool" => true,
            "list" => vec![Value::from("a"), Value::from(1)],
            "map" => btreemap! { "k" => "x" },
        })
    }

    fn native_log_record(record: ObjectMap) -> LogData {
        OpentelemetryLogsModel::try_from(native_log_event(record))
            .expect("native log cannot be converted to a model")
            .0
    }

    fn assert_native_log_attributes(record: &LogRecord) {
        let attributes = record
            .attributes
            .clone()
            .expect("attributes should be set")
            .into_iter()
            .collect::<HashMap<Key, OtlpAnyValue>>();
        let list = vec![OtlpAnyValue::from("a".to_string()), OtlpAnyValue::Int(1)];
        let map = HashMap::from([(Key::new("k"), OtlpAnyValue::from("x".to_string()))]);

        assert_eq!(
            attributes,
            HashMap::from([
                (Key::new("str"), OtlpAnyValue::from("bar".to_string())),
                (Key::new("int"), OtlpAnyValue::Int(100)),
                (Key::new("flt"), OtlpAnyValue::Double(1.5)),
                (Key::new("bool"), OtlpAnyValue::Boolean(true)),
                (Key::new("list"), OtlpAnyValue::ListAny(list)),
                (Key::new("map"), OtlpAnyValue::Map(map)),
            ])
        );
    }

    fn nanos(nanos: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)
    }

    #[test]
    fn test_otlp_sink_log_model_matcher_matches_native_log() {
        let event = native_log_event(btreemap! { "body" => "hello" });

        assert_eq!(
            OpentelemetryLogsModel::maybe_match(&event),
            Some(OpentelemetryModelType::Logs)
        );
    }

    #[test]
    fn test_otlp_sink_native_log_to_log_model() {
        let log_data = native_log_record(btreemap! {
            "body" => "payment accepted",
            "attributes" => native_log_attributes(),
            "timeUnixNano" => "1792065600000000000",
            "observedTimeUnixNano" => 1_792_065_600_000_000_123_i64,
            "severityNumber" => 17,
            "severityText" => "ERROR",
            "traceId" => NATIVE_TRACE_ID,
            "spanId" => NATIVE_SPAN_ID,
            "flags" => 1,
            "droppedAttributesCount" => 2,
        });
        let record = &log_data.record;

        assert_eq!(
            record.body,
            Some(OtlpAnyValue::from("payment accepted".to_string()))
        );
        assert_native_log_attributes(record);
        assert_eq!(record.timestamp, Some(nanos(1_792_065_600_000_000_000)));
        assert_eq!(record.observed_timestamp, nanos(1_792_065_600_000_000_123));
        assert_eq!(record.severity_number, Some(Severity::Error));
        assert_eq!(record.severity_text, Some(Cow::from("ERROR")));

        let trace_context = record
            .trace_context
            .as_ref()
            .expect("trace context should be set");
        assert_eq!(
            trace_context.trace_id,
            TraceId::from_hex(NATIVE_TRACE_ID).unwrap()
        );
        assert_eq!(
            trace_context.span_id,
            SpanId::from_hex(NATIVE_SPAN_ID).unwrap()
        );
        assert_eq!(trace_context.trace_flags, Some(TraceFlags::SAMPLED));

        assert_eq!(
            log_data.resource.get(Key::new("service.name")),
            Some("checkout".into())
        );
        assert_eq!(log_data.instrumentation.name, "checkout.logger");
        assert_eq!(log_data.instrumentation.version, Some(Cow::from("1.4.0")));
        assert_eq!(
            log_data.instrumentation.attributes,
            vec![KeyValue::new("scope.attr", "scoped")]
        );
    }

    #[test]
    fn test_otlp_sink_native_log_without_severity_or_trace_context() {
        let log_data = native_log_record(btreemap! {
            "body" => "payment accepted",
            "attributes" => native_log_attributes(),
            "timeUnixNano" => "1792065600000000000",
            "observedTimeUnixNano" => "1792065600000000123",
        });
        let record = &log_data.record;

        assert_eq!(
            record.body,
            Some(OtlpAnyValue::from("payment accepted".to_string()))
        );
        assert_native_log_attributes(record);
        assert_eq!(record.timestamp, Some(nanos(1_792_065_600_000_000_000)));
        assert_eq!(record.observed_timestamp, nanos(1_792_065_600_000_000_123));
        assert!(record.severity_number.is_none());
        assert!(record.severity_text.is_none());
        assert!(record.trace_context.is_none());
    }

    #[test]
    fn test_otlp_sink_native_log_severity_number() {
        for (severity_number, expected) in [
            (Value::from("SEVERITY_NUMBER_TRACE"), Some(Severity::Trace)),
            (
                Value::from("SEVERITY_NUMBER_TRACE2"),
                Some(Severity::Trace2),
            ),
            (Value::from("SEVERITY_NUMBER_DEBUG"), Some(Severity::Debug)),
            (Value::from("SEVERITY_NUMBER_INFO"), Some(Severity::Info)),
            (Value::from("SEVERITY_NUMBER_WARN3"), Some(Severity::Warn3)),
            (Value::from("SEVERITY_NUMBER_ERROR"), Some(Severity::Error)),
            (
                Value::from("SEVERITY_NUMBER_FATAL4"),
                Some(Severity::Fatal4),
            ),
            (Value::from("SEVERITY_NUMBER_UNSPECIFIED"), None),
            (Value::from("SEVERITY_NUMBER_LOUD"), None),
            (Value::from(9), Some(Severity::Info)),
            (Value::from(25), None),
        ] {
            let record = native_log_record(btreemap! {
                "body" => "hello",
                "severityNumber" => severity_number.clone(),
            })
            .record;

            assert_eq!(
                record.severity_number, expected,
                "severityNumber {severity_number}"
            );
        }
    }
}
