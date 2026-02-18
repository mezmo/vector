use std::borrow::Cow;
use std::collections::BTreeMap;
use vector_core::event::metric::mezmo::IntoValue;

use smallvec::SmallVec;

use opentelemetry_rs::opentelemetry::logs::{ExportLogsServiceRequest, Validate};

use vector_core::{
    config::log_schema,
    event::{Event, EventMetadata, KeyString, LogEvent, Value},
};

use vector_common::btreemap;

use crate::decoding::format::mezmo::open_telemetry::{
    DeserializerError, OpenTelemetryAnyValue, OpenTelemetryKeyValue, nano_to_timestamp,
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
            let resource = resource_logs.resource;

            let mut resource_attrs = Value::from(BTreeMap::new());
            if let Some(resource) = &resource {
                resource_attrs = (OpenTelemetryKeyValue {
                    attributes: resource.attributes.clone(),
                })
                .to_value();
            }

            let resource_attrs = resource_attrs;

            for scope_logs in resource_logs.scope_logs.into_iter() {
                let mut scope: BTreeMap<KeyString, Value> = BTreeMap::new();
                if let Some(s) = &scope_logs.scope {
                    let attributes = OpenTelemetryKeyValue {
                        attributes: s.attributes.clone(),
                    };

                    scope = btreemap! {
                        KeyString::from("name") => Value::from(s.name.clone()),
                        KeyString::from("version") => Value::from(s.version.clone()),
                        KeyString::from("attributes") => attributes.to_value(),
                    }
                }
                scope.insert("schema_url".into(), Value::from(scope_logs.schema_url));

                let scope = Value::from(scope);

                for log_record in scope_logs.log_records.into_iter() {
                    let attributes = OpenTelemetryKeyValue {
                        attributes: log_record.attributes,
                    };

                    let time = nano_to_timestamp(log_record.time_unix_nano);

                    let mut metadata: BTreeMap<KeyString, _> = btreemap! {
                        KeyString::from("resource") => resource_attrs.clone(),
                        KeyString::from("scope") => scope.clone(),
                        KeyString::from("time") => time.clone(),
                        KeyString::from("observed_timestamp") => nano_to_timestamp(log_record.observed_time_unix_nano),
                        KeyString::from("severity_number") => log_record.severity_number as i32,
                        KeyString::from("trace_id") => faster_hex::hex_string(&log_record.trace_id),
                        KeyString::from("span_id") => faster_hex::hex_string(&log_record.span_id),
                        KeyString::from("flags") => log_record.flags,
                        KeyString::from("attributes") => attributes.to_value(),
                    };

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

                    let line = match log_record.body {
                        Some(av) => OpenTelemetryAnyValue { value: av }.to_value(),
                        None => Value::Null,
                    };

                    let message_key = log_schema().message_key().unwrap();

                    let mut log_event = LogEvent::from_map(btreemap! {
                        log_schema().user_metadata_key() => Value::Object(metadata),
                        KeyString::from(message_key.to_string().as_str()) => line,
                    }, EventMetadata::default());

                    if let Some(timestamp_key) = log_schema().timestamp_key() {
                        log_event.insert((lookup::PathPrefix::Event, timestamp_key), time);
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

    use chrono::DateTime;
    use std::borrow::Cow;
    use std::collections::BTreeMap;

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
                .unwrap(),
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
                    Value::from(
                        DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                            .expect("timestamp should be a valid timestamp")
                    )
                ),
                (
                    "time".into(),
                    Value::from(
                        DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                            .expect("timestamp should be a valid timestamp")
                    )
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
                        ("name".into(), "test_name".into()),
                        ("schema_url".into(), "https://some_url.com".into()),
                        ("version".into(), "1.2.3".into()),
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        )
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
                .unwrap(),
            "asdf".into(),
        );
    }
}
