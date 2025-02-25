use vector_lib::{
    config::log_schema,
    event::{Event, Value},
    lookup::PathPrefix,
};

use opentelemetry::{
    logs::{AnyValue as OtlpAnyValue, LogRecord, Severity},
    trace::{SpanContext, TraceState},
};
use opentelemetry_sdk::export::logs::LogData;
use std::borrow::Cow;

use crate::sinks::opentelemetry::{
    models::{
        value_to_otlp_any_value, value_to_system_time, OpentelemetryModelMatch,
        OpentelemetryModelType, OpentelemetryResource, OpentelemetryScope, OpentelemetrySpanId,
        OpentelemetryTraceFlags, OpentelemetryTraceId,
    },
    sink::OpentelemetrySinkError,
};

#[derive(Debug)]
pub struct OpentelemetryLogsModel(pub LogData);

impl OpentelemetryModelMatch for OpentelemetryLogsModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        if let Some(log) = event.maybe_as_log() {
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
}

impl TryFrom<Event> for OpentelemetryLogsModel {
    type Error = OpentelemetrySinkError;

    fn try_from(buf_event: Event) -> Result<Self, Self::Error> {
        let log = &buf_event.into_log();

        let mut record_builder = LogRecord::builder();

        if let Some(Value::Bytes(message)) = log.get_message() {
            let body = OtlpAnyValue::from(String::from_utf8_lossy(message).into_owned());
            record_builder = record_builder.with_body(body);
        }

        let mut severity_number = None;

        if let Some(metadata) = log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
            if let Some(value) = metadata.get("attributes") {
                if let OtlpAnyValue::Map(attrs) = value_to_otlp_any_value(value.clone()) {
                    let attributes = attrs.clone().into_iter().collect::<Vec<_>>();

                    record_builder = record_builder.with_attributes(attributes);
                }
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
                severity_number = match *number {
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
                };
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
    use std::collections::BTreeMap;
    use std::time::SystemTime;
    use vector_lib::event::{Event, LogEvent};

    use opentelemetry::{
        logs::{AnyValue as OtlpAnyValue, LogRecord, Severity, TraceContext},
        trace::{SpanId, TraceFlags, TraceId},
        InstrumentationLibrary,
    };
    use opentelemetry_sdk::export::logs::LogData;
    use opentelemetry_sdk::Resource;

    fn line_generator(index: usize) -> String {
        format!("opentelemetry test log index {}", index)
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
                Err(err) => panic!("Log event cannot be converted to a model: {:#?}", err),
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
}
