use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::SystemTime;
use vector_core::event::metric::mezmo::IntoValue;

use smallvec::SmallVec;

use opentelemetry_rs::opentelemetry::{
    common::AnyValueOneOfvalue,
    logs::{AnyValue, ExportLogsServiceRequest, KeyValue, Validate},
};

use vector_core::{
    config::log_schema,
    event::{Event, EventMetadata, LogEvent, Value},
};

use crate::decoding::format::mezmo::open_telemetry::{DeserializerError, OpenTelemetryAnyValue};

pub fn parse_logs_request(bytes: &[u8]) -> vector_common::Result<smallvec::SmallVec<[Event; 1]>> {
    let parsed_logs = ExportLogsServiceRequest::try_from(bytes)
        .map_err(|e| DeserializerError::ProtobufParseError { source: e })?;

    parsed_logs
        .validate()
        .map_err(|e| DeserializerError::ProtobufValidationError { source: e })?;

    Ok(to_events(parsed_logs))
}

const MAX_METADATA_SIZE: usize = 32 * 1024;
const MAX_LOG_LEVEL_LEN: usize = 80;
const NANOS_IN_MILLIS: u64 = 1_000_000;

#[allow(clippy::too_many_lines)]
pub fn to_events(log_request: ExportLogsServiceRequest) -> SmallVec<[Event; 1]> {
    let log_count = log_request.resource_logs.iter().fold(0, |acc, rlgs| {
        rlgs.scope_logs
            .iter()
            .fold(acc, |acc, slgs| acc + slgs.log_records.len())
    });
    log_request.resource_logs.into_iter().fold(
        SmallVec::with_capacity(log_count),
        |mut acc, resource_logs| {
            // process resource
            let resource = resource_logs.resource;
            let resource_host_name = resource.and_then(|resource| {
                resource
                    .attributes
                    .into_iter()
                    .find(|KeyValue { key: k, .. }| k == "host.name")
                    .and_then(move |kv| match kv {
                        KeyValue {
                            value:
                                Some(AnyValue {
                                    value: AnyValueOneOfvalue::string_value(host_name),
                                }),
                            ..
                        } => Some(Value::from(host_name)),
                        _ => None,
                    })
            });

            acc.extend(resource_logs.scope_logs.into_iter().flat_map(|scope_logs| {
                scope_logs.log_records.into_iter().map(|log_record| {
                    // Assemble metadata
                    let mut attrs = BTreeMap::new();
                    if let Some(host_name) = &resource_host_name {
                        attrs.insert("hostname".to_string(), host_name.clone());
                    }

                    attrs.insert(
                        "trace.id".to_string(),
                        Value::from(faster_hex::hex_string(&log_record.trace_id)),
                    );
                    attrs.insert(
                        "span.id".to_string(),
                        Value::from(faster_hex::hex_string(&log_record.span_id)),
                    );

                    let mut internal_metadata = BTreeMap::new();

                    for kv in log_record.attributes.into_iter() {
                        if let KeyValue {
                            key: k,
                            value:
                                Some(AnyValue {
                                    value: AnyValueOneOfvalue::string_value(v),
                                }),
                        } = kv
                        {
                            attrs.insert(k.to_string(), {
                                let v = Value::from(Cow::from(
                                    &v[..std::cmp::min(v.len(), MAX_METADATA_SIZE)],
                                ));
                                if k == "appname" {
                                    internal_metadata.insert("app".to_string(), v.clone());
                                }
                                v
                            });
                        }
                    }

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

                    let sev = log_record.severity_text;
                    if !sev.is_empty() {
                        internal_metadata.insert(
                            "level".to_string(),
                            Value::from(Cow::from(
                                &sev[..std::cmp::min(sev.len(), MAX_LOG_LEVEL_LEN)],
                            )),
                        );
                    }

                    let line = match log_record.body {
                        Some(av) => OpenTelemetryAnyValue { value: av }.to_value(),
                        None => Value::Null,
                    };

                    let mut log_line = BTreeMap::from_iter([
                        // Add the user metadata
                        (
                            log_schema().user_metadata_key().to_string(),
                            Value::from(attrs),
                        ),
                        // Add the actual line
                        (log_schema().message_key().to_string(), line),
                    ]);
                    if !internal_metadata.is_empty() {
                        // Add our metadata
                        log_line.insert(
                            log_schema().metadata_key().to_string(),
                            Value::from(internal_metadata),
                        );
                    }

                    // Wrap line in mezmo format
                    let mut log_event = LogEvent::from_map(
                        std::collections::BTreeMap::from([(
                            log_schema().message_key().to_string(),
                            Value::Object(log_line),
                        )]),
                        EventMetadata::default(),
                    );

                    if let Some(timestamp_key) = log_schema().timestamp_key() {
                        log_event
                            .insert((lookup::PathPrefix::Event, timestamp_key), time_unix_millis);
                    }

                    Event::Log(log_event)
                })
            }));
            acc
        },
    )
}
