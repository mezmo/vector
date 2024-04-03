use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::SystemTime;
use vector_core::event::metric::mezmo::IntoValue;

use smallvec::SmallVec;

use opentelemetry_rs::opentelemetry::logs::{ExportLogsServiceRequest, Validate};

use vector_core::{
    config::log_schema,
    event::{Event, EventMetadata, LogEvent, Value},
};

use crate::decoding::format::mezmo::open_telemetry::{
    DeserializerError, OpenTelemetryAnyValue, OpenTelemetryKeyValue,
};

pub fn parse_logs_request(bytes: &[u8]) -> vector_common::Result<smallvec::SmallVec<[Event; 1]>> {
    let parsed_logs = ExportLogsServiceRequest::try_from(bytes)
        .map_err(|e| DeserializerError::ProtobufParseError { source: e })?;

    parsed_logs
        .validate()
        .map_err(|e| DeserializerError::ProtobufValidationError { source: e })?;

    Ok(to_events(parsed_logs))
}

const MAX_LOG_LEVEL_LEN: usize = 80;
const NANOS_IN_MILLIS: u64 = 1_000_000;

#[allow(clippy::too_many_lines)]
pub fn to_events(log_request: ExportLogsServiceRequest) -> SmallVec<[Event; 1]> {
    let log_count = log_request.resource_logs.iter().fold(0, |acc, rlgs| {
        rlgs.scope_logs
            .iter()
            .fold(acc, |acc, slgs| acc + slgs.log_records.len())
    });
    let mut i = 0;
    log_request.resource_logs.into_iter().fold(
        SmallVec::with_capacity(log_count),
        |mut acc, resource_logs| {
            // process resource
            let resource = resource_logs.resource;

            let mut resource_attrs = Value::from(BTreeMap::new());
            if let Some(resource) = &resource {
                resource_attrs = (OpenTelemetryKeyValue {
                    attributes: resource.attributes.clone(),
                })
                .to_value();
            }

            // done changing resource_attrs
            let resource_attrs = resource_attrs;

            for scope_logs in resource_logs.scope_logs.into_iter() {
                // Scope attributes
                let mut scope_attrs = Value::from(BTreeMap::new());
                if let Some(scope) = &scope_logs.scope {
                    scope_attrs = (OpenTelemetryKeyValue {
                        attributes: scope.attributes.clone(),
                    })
                    .to_value();
                }

                // done changing scope_attrs
                let scope_attrs = scope_attrs;

                for log_record in scope_logs.log_records.into_iter() {
                    // Assemble metadata
                    let mut metadata = BTreeMap::new();

                    metadata.insert("resource".into(), Value::from(resource_attrs.clone()));
                    metadata.insert("scope".into(), Value::from(scope_attrs.clone()));

                    // "time":"2023-10-31T13:32:42.240772879-04:00",
                    let time_unix_millis = Value::from(if log_record.time_unix_nano == 0 {
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|t| t.as_millis())
                            .unwrap_or(0)
                            .try_into()
                            .unwrap_or(u64::MAX)
                    } else {
                        log_record.time_unix_nano / NANOS_IN_MILLIS
                    });
                    metadata.insert("time".into(), Value::from(time_unix_millis.clone()));
                    // "observed_timestamp": "2023-10-31T13:32:42.240772879-04:00",
                    if log_record.observed_time_unix_nano != 0 {
                        metadata.insert(
                            "observed_timestamp".into(),
                            Value::from(log_record.observed_time_unix_nano / NANOS_IN_MILLIS),
                        );
                    }
                    // "severity_text": "ERROR",
                    let sev = log_record.severity_text;
                    if !sev.is_empty() {
                        metadata.insert("severity_text".into(), Value::from(sev.clone()));
                        metadata.insert(
                            "level".into(),
                            Value::from(Cow::from(
                                &sev[..std::cmp::min(sev.len(), MAX_LOG_LEVEL_LEN)],
                            )),
                        );
                    }
                    // "severity_number": 17,
                    metadata.insert(
                        "severity_number".into(),
                        Value::from(log_record.severity_number as i32),
                    );
                    // "trace_id": "0x5b8aa5a2d2c872e8321cf37308d69df2",
                    metadata.insert(
                        "trace_id".into(),
                        Value::from(faster_hex::hex_string(&log_record.trace_id)),
                    );
                    // "span_id": "0x051581bf3cb55c13",
                    metadata.insert(
                        "span_id".into(),
                        Value::from(faster_hex::hex_string(&log_record.span_id)),
                    );
                    // "trace_flags": "00",
                    metadata.insert("flags".into(), Value::from(log_record.flags));

                    // LogRecord attributes
                    let attributes = OpenTelemetryKeyValue {
                        attributes: log_record.attributes,
                    };
                    metadata.insert("attributes".into(), attributes.to_value());

                    let line = match log_record.body {
                        Some(av) => OpenTelemetryAnyValue { value: av }.to_value(),
                        None => Value::Null,
                    };

                    let log_line = BTreeMap::from_iter([
                        // Add the user metadata
                        (
                            log_schema().user_metadata_key().into(),
                            Value::from(metadata),
                        ),
                        // Add the actual line
                        (log_schema().message_key().unwrap().to_string().into(), line),
                    ]);

                    // Wrap line in mezmo format
                    let mut log_event = LogEvent::from_map(log_line, EventMetadata::default());

                    if let Some(timestamp_key) = log_schema().timestamp_key() {
                        log_event
                            .insert((lookup::PathPrefix::Event, timestamp_key), time_unix_millis);
                    }

                    acc.insert(i, Event::Log(log_event));
                    i += 1;
                }
            }
            acc
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::ops::Deref;

    use opentelemetry_rs::opentelemetry::{
        common::{AnyValue, AnyValueOneOfvalue, InstrumentationScope, KeyValue},
        logs::{ExportLogsServiceRequest, LogRecord, Resource, ResourceLogs, ScopeLogs},
    };

    #[test]
    fn otlp_logs_deserialize_to_events() {
        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let logs_data = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: AnyValueOneOfvalue::string_value(Cow::from("asdf")),
                        }),
                        time_unix_nano: 1_579_134_612_000_000_011,
                        observed_time_unix_nano: 1_579_134_612_000_000_011,
                        span_id: Cow::from("test".as_bytes()),
                        trace_id: Cow::from("test".as_bytes()),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        flags: 1,
                        severity_number: 1.into(),
                        severity_text: Cow::Borrowed("ERROR"),
                        dropped_attributes_count: 0,
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };
        let log_events = to_events(logs_data);
        assert_eq!(
            *log_events[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
                ("flags".into(), Value::from(1)),
                (
                    "observed_timestamp".into(),
                    Value::from(1_579_134_612_000_000_011_i64 / 1_000_000)
                ),
                (
                    "time".into(),
                    Value::from(1_579_134_612_000_000_011_i64 / 1_000_000)
                ),
                ("severity_number".into(), 1.into()),
                ("severity_text".into(), "ERROR".into()),
                ("level".into(), "ERROR".into()),
                ("span_id".into(), "74657374".into()),
                ("trace_id".into(), "74657374".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *log_events[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            "asdf".into(),
        );
    }
}
