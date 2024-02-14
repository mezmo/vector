use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::SystemTime;
use vector_core::event::metric::mezmo::IntoValue;

use smallvec::SmallVec;

use opentelemetry_rs::opentelemetry::{
    common::AnyValueOneOfvalue,
    trace::{AnyValue, ExportTraceServiceRequest, KeyValue, Validate},
};

use vector_core::{
    config::log_schema,
    event::{Event, EventMetadata, LogEvent, Value},
};

use crate::decoding::format::mezmo::open_telemetry::{DeserializerError, OpenTelemetryKeyValue};

const NANOS_IN_MILLIS: u64 = 1_000_000;

pub fn parse_traces_request(bytes: &[u8]) -> vector_common::Result<smallvec::SmallVec<[Event; 1]>> {
    let parsed_traces = ExportTraceServiceRequest::try_from(bytes)
        .map_err(|e| DeserializerError::ProtobufParseError { source: e })?;

    parsed_traces
        .validate()
        .map_err(|e| DeserializerError::ProtobufValidationError { source: e })?;

    Ok(to_events(parsed_traces))
}

fn extract<'a>(attributes: Vec<KeyValue<'a>>, key_name: &str) -> Option<Cow<'a, str>> {
    let mut out = None;

    for kv in attributes.into_iter() {
        if let KeyValue {
            key: k,
            value:
                Some(AnyValue {
                    value: AnyValueOneOfvalue::string_value(v),
                }),
        } = kv
        {
            if k == key_name {
                out = Some(v.clone());
                break;
            }
        }
    }

    out
}

fn string_to_value(value: String) -> Value {
    if !value.is_empty() {
        Value::from(value)
    } else {
        Value::Null
    }
}

fn nano_to_timestamp(time_unix_nano: u64) -> Value {
    Value::from(if time_unix_nano == 0 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|t| t.as_millis())
            .unwrap_or(0)
            .try_into()
            .unwrap_or(u64::MAX)
    } else {
        time_unix_nano / NANOS_IN_MILLIS
    })
}

#[allow(clippy::too_many_lines)]
pub fn to_events(trace_request: ExportTraceServiceRequest) -> SmallVec<[Event; 1]> {
    let trace_count = trace_request.resource_spans.iter().fold(0, |acc, rlgs| {
        rlgs.scope_spans
            .iter()
            .fold(acc, |acc, slgs| acc + slgs.spans.len())
    });

    trace_request.resource_spans.into_iter().fold(
        SmallVec::with_capacity(trace_count),
        |mut acc, resource_spans| {
            let mut resource_host_name = None;
            let resource = if let Some(resource) = resource_spans.resource.clone() {
                resource_host_name = extract(resource.attributes.clone(), "host.name");

                Value::Object(
                    [
                        (
                            "attributes".into(),
                            OpenTelemetryKeyValue {
                                attributes: resource.attributes,
                            }
                            .to_value(),
                        ),
                        (
                            "dropped_attributes_count".into(),
                            resource.dropped_attributes_count.into(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )
            } else {
                Value::Null
            };

            for scope_spans in resource_spans.scope_spans.into_iter() {
                let scope = if let Some(scope) = scope_spans.scope.clone() {
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            OpenTelemetryKeyValue {
                                attributes: scope.attributes,
                            }
                            .to_value(),
                        ),
                        (
                            "dropped_attributes_count".into(),
                            scope.dropped_attributes_count.into(),
                        ),
                        ("name".into(), string_to_value(scope.name.into())),
                        ("version".into(), string_to_value(scope.version.into())),
                    ]))
                } else {
                    Value::Null
                };

                acc.extend(scope_spans.spans.into_iter().map(|span| {
                    // Assemble trace message
                    let mut message = std::collections::BTreeMap::new();

                    message.insert("name".to_string(), string_to_value(span.name.into()));

                    if let Some(host_name) = &resource_host_name {
                        message.insert(
                            "hostname".to_string(),
                            string_to_value(host_name.to_string()),
                        );
                    }

                    message.insert(
                        "trace.id".to_string(),
                        Value::from(faster_hex::hex_string(&span.trace_id)),
                    );
                    message.insert("trace.state".to_string(), Value::from(span.trace_state));
                    message.insert(
                        "span.id".to_string(),
                        Value::from(faster_hex::hex_string(&span.span_id)),
                    );
                    message.insert(
                        "span.parent_id".to_string(),
                        Value::from(faster_hex::hex_string(&span.parent_span_id)),
                    );

                    let start_time_unix_nano = nano_to_timestamp(span.start_time_unix_nano);

                    message.insert("start_timestamp".to_string(), start_time_unix_nano.clone());

                    message.insert(
                        "end_timestamp".to_string(),
                        nano_to_timestamp(span.end_time_unix_nano),
                    );

                    message.insert("kind".to_string(), Value::from(span.kind as i32));

                    message.insert(
                        "dropped_attributes_count".into(),
                        span.dropped_attributes_count.into(),
                    );

                    message.insert(
                        "events".to_string(),
                        Value::Array(
                            span.events
                                .iter()
                                .map(|event| {
                                    Value::Object(BTreeMap::from([
                                        ("name".into(), string_to_value(event.name.clone().into())),
                                        (
                                            "timestamp".into(),
                                            nano_to_timestamp(event.time_unix_nano),
                                        ),
                                        (
                                            "attributes".into(),
                                            OpenTelemetryKeyValue {
                                                attributes: event.attributes.clone(),
                                            }
                                            .to_value(),
                                        ),
                                        (
                                            "dropped_attributes_count".into(),
                                            event.dropped_attributes_count.into(),
                                        ),
                                    ]))
                                })
                                .collect(),
                        ),
                    );

                    message.insert(
                        "dropped_events_count".into(),
                        span.dropped_events_count.into(),
                    );

                    message.insert(
                        "links".to_string(),
                        Value::Array(
                            span.links
                                .iter()
                                .map(|link| {
                                    Value::Object(BTreeMap::from([
                                        (
                                            "trace_id".into(),
                                            Value::from(faster_hex::hex_string(&link.trace_id)),
                                        ),
                                        (
                                            "span_id".into(),
                                            Value::from(faster_hex::hex_string(&link.span_id)),
                                        ),
                                        ("trace_state".into(), link.trace_state.clone().into()),
                                        (
                                            "attributes".into(),
                                            OpenTelemetryKeyValue {
                                                attributes: link.attributes.clone(),
                                            }
                                            .to_value(),
                                        ),
                                        (
                                            "dropped_attributes_count".into(),
                                            link.dropped_attributes_count.into(),
                                        ),
                                    ]))
                                })
                                .collect(),
                        ),
                    );

                    message.insert(
                        "dropped_links_count".into(),
                        span.dropped_links_count.into(),
                    );

                    if let Some(status) = span.status {
                        message.insert(
                            "status".to_string(),
                            Value::Object(BTreeMap::from([
                                (
                                    "message".into(),
                                    string_to_value(status.message.to_string()),
                                ),
                                ("code".into(), Value::from(status.code as i32)),
                            ])),
                        );
                    }

                    // Assemble metadata
                    let mut user_metadata = std::collections::BTreeMap::from_iter([(
                        "level".to_string(),
                        "trace".into(),
                    )]);

                    user_metadata.insert("resource".to_string(), resource.clone());
                    user_metadata.insert("scope".to_string(), scope.clone());

                    let filtered_attributes = OpenTelemetryKeyValue {
                        attributes: span.attributes,
                    }
                    .to_value();
                    user_metadata.insert("attributes".to_string(), filtered_attributes);

                    let log_line = BTreeMap::from_iter([
                        (
                            log_schema().message_key().unwrap().to_string(),
                            message.into(),
                        ),
                        (
                            log_schema().user_metadata_key().to_string(),
                            user_metadata.into(),
                        ),
                    ]);

                    let mut log_event = LogEvent::from_map(log_line, EventMetadata::default());

                    if let Some(timestamp_key) = log_schema().timestamp_key() {
                        log_event.insert(
                            (lookup::PathPrefix::Event, timestamp_key),
                            start_time_unix_nano,
                        );
                    }

                    Event::Log(log_event)
                }))
            }
            acc
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    use opentelemetry_rs::opentelemetry::metrics::{AnyValue, AnyValueOneOfvalue, KeyValue};
    use std::borrow::Cow;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_traces_deserialize_to_events() {
        use opentelemetry_rs::opentelemetry::trace::{
            ExportTraceServiceRequest, InstrumentationScope, Resource, ResourceSpans, ScopeSpans,
            Span, SpanEvent, SpanKind, SpanLink, Status, StatusCode,
        };

        let key_value = KeyValue {
            key: Cow::from("test"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("test")),
            }),
        };

        let traces_data = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![key_value.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value.clone()],
                        dropped_attributes_count: 10,
                    }),
                    spans: vec![Span {
                        trace_id: Cow::from("trace_id".as_bytes()),
                        span_id: Cow::from("span_id".as_bytes()),
                        parent_span_id: Cow::from("parent_span_id".as_bytes()),
                        trace_state: Cow::from("test_state"),
                        name: Cow::from("test_span_name"),
                        kind: SpanKind::SPAN_KIND_UNSPECIFIED,
                        start_time_unix_nano: 1681339577345243523,
                        end_time_unix_nano: 1681339577345243523,
                        attributes: vec![key_value.clone()],
                        dropped_attributes_count: 10,
                        events: vec![SpanEvent {
                            time_unix_nano: 1681339577345243523,
                            name: Cow::from("test_name"),
                            attributes: vec![key_value.clone()],
                            dropped_attributes_count: 10,
                        }],
                        dropped_events_count: 10,
                        dropped_links_count: 10,
                        links: vec![SpanLink {
                            trace_id: Cow::from("link_trace_id".as_bytes()),
                            span_id: Cow::from("link_span_id".as_bytes()),
                            trace_state: Cow::from("link_test_state"),
                            attributes: vec![key_value.clone()],
                            dropped_attributes_count: 10,
                        }],
                        status: Some(Status {
                            message: Cow::from("test_message"),
                            code: StatusCode::STATUS_CODE_OK,
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let traces = to_events(traces_data.clone());

        assert_eq!(
            *traces[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("name".into(), "test_span_name".into()),
                ("trace.id".into(), Value::from("74726163655f6964")),
                ("trace.state".into(), Value::from("test_state")),
                ("span.id".into(), Value::from("7370616e5f6964")),
                (
                    "span.parent_id".into(),
                    Value::from("706172656e745f7370616e5f6964")
                ),
                ("start_timestamp".into(), Value::Integer(1_681_339_577_345)),
                ("dropped_attributes_count".into(), Value::Integer(10)),
                ("dropped_events_count".into(), Value::Integer(10)),
                ("dropped_links_count".into(), Value::Integer(10)),
                ("end_timestamp".into(), Value::Integer(1_681_339_577_345)),
                (
                    "events".into(),
                    Value::Array(Vec::from([Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([("test".into(), "test".into()),]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("timestamp".into(), Value::Integer(1_681_339_577_345)),
                    ]))]))
                ),
                ("kind".into(), Value::Integer(0)),
                (
                    "links".into(),
                    Value::Array(Vec::from([Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([("test".into(), "test".into()),]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("span_id".into(), Value::from("6c696e6b5f7370616e5f6964")),
                        ("trace_id".into(), Value::from("6c696e6b5f74726163655f6964")),
                        ("trace_state".into(), Value::from("link_test_state")),
                    ]))]))
                ),
                (
                    "status".into(),
                    Value::Object(BTreeMap::from([
                        ("code".into(), Value::Integer(1)),
                        ("message".into(), Value::from("test_message")),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *traces[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([("test".into(), "test".into()),]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([("test".into(), "test".into()),]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), "1.2.3".into()),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([("test".into(), "test".into()),]))
                ),
                ("level".into(), "trace".into()),
            ]))
        );
    }

    #[test]
    fn otlp_traces_deserialize_parse_request() {
        let out: &[u8] = &[
            10, 179, 11, 10, 131, 3, 10, 32, 10, 21, 116, 101, 108, 101, 109, 101, 116, 114, 121,
            46, 115, 100, 107, 46, 118, 101, 114, 115, 105, 111, 110, 18, 7, 10, 5, 49, 46, 50, 46,
            49, 10, 37, 10, 18, 116, 101, 108, 101, 109, 101, 116, 114, 121, 46, 115, 100, 107, 46,
            110, 97, 109, 101, 18, 15, 10, 13, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101,
            116, 114, 121, 10, 34, 10, 22, 116, 101, 108, 101, 109, 101, 116, 114, 121, 46, 115,
            100, 107, 46, 108, 97, 110, 103, 117, 97, 103, 101, 18, 8, 10, 6, 101, 114, 108, 97,
            110, 103, 10, 36, 10, 12, 115, 101, 114, 118, 105, 99, 101, 46, 110, 97, 109, 101, 18,
            20, 10, 18, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97, 103, 115, 101, 114, 118,
            105, 99, 101, 10, 56, 10, 19, 115, 101, 114, 118, 105, 99, 101, 46, 105, 110, 115, 116,
            97, 110, 99, 101, 46, 105, 100, 18, 33, 10, 31, 102, 101, 97, 116, 117, 114, 101, 102,
            108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 64, 100, 54, 57, 100, 56, 53, 55, 49,
            51, 49, 97, 99, 10, 37, 10, 23, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117, 110,
            116, 105, 109, 101, 46, 118, 101, 114, 115, 105, 111, 110, 18, 10, 10, 8, 49, 49, 46,
            50, 46, 50, 46, 56, 10, 30, 10, 20, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117,
            110, 116, 105, 109, 101, 46, 110, 97, 109, 101, 18, 6, 10, 4, 66, 69, 65, 77, 10, 60,
            10, 27, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117, 110, 116, 105, 109, 101, 46,
            100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 18, 29, 10, 27, 69, 114, 108, 97,
            110, 103, 47, 79, 84, 80, 32, 50, 51, 32, 101, 114, 116, 115, 45, 49, 49, 46, 50, 46,
            50, 46, 56, 10, 47, 10, 23, 112, 114, 111, 99, 101, 115, 115, 46, 101, 120, 101, 99,
            117, 116, 97, 98, 108, 101, 46, 110, 97, 109, 101, 18, 20, 10, 18, 102, 101, 97, 116,
            117, 114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 18, 146, 4, 10, 30,
            10, 21, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101, 116, 114, 121, 95, 112, 104,
            111, 101, 110, 105, 120, 18, 5, 49, 46, 48, 46, 48, 18, 239, 3, 10, 16, 196, 206, 162,
            34, 18, 10, 86, 108, 234, 246, 51, 69, 0, 171, 1, 40, 18, 8, 62, 64, 179, 38, 163, 41,
            8, 151, 34, 0, 42, 1, 47, 48, 2, 57, 120, 196, 182, 220, 91, 196, 130, 23, 65, 57, 144,
            204, 220, 91, 196, 130, 23, 74, 61, 10, 12, 112, 104, 111, 101, 110, 105, 120, 46, 112,
            108, 117, 103, 18, 45, 10, 43, 69, 108, 105, 120, 105, 114, 46, 70, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 87, 101, 98, 46, 80, 97,
            103, 101, 67, 111, 110, 116, 114, 111, 108, 108, 101, 114, 74, 25, 10, 14, 112, 104,
            111, 101, 110, 105, 120, 46, 97, 99, 116, 105, 111, 110, 18, 7, 10, 5, 105, 110, 100,
            101, 120, 74, 25, 10, 13, 110, 101, 116, 46, 116, 114, 97, 110, 115, 112, 111, 114,
            116, 18, 8, 10, 6, 73, 80, 46, 84, 67, 80, 74, 21, 10, 13, 110, 101, 116, 46, 112, 101,
            101, 114, 46, 112, 111, 114, 116, 18, 4, 24, 178, 152, 2, 74, 26, 10, 11, 110, 101,
            116, 46, 112, 101, 101, 114, 46, 105, 112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48,
            46, 49, 74, 20, 10, 13, 110, 101, 116, 46, 104, 111, 115, 116, 46, 112, 111, 114, 116,
            18, 3, 24, 145, 63, 74, 26, 10, 11, 110, 101, 116, 46, 104, 111, 115, 116, 46, 105,
            112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48, 46, 49, 74, 32, 10, 15, 104, 116, 116,
            112, 46, 117, 115, 101, 114, 95, 97, 103, 101, 110, 116, 18, 13, 10, 11, 99, 117, 114,
            108, 47, 55, 46, 55, 52, 46, 48, 74, 18, 10, 11, 104, 116, 116, 112, 46, 116, 97, 114,
            103, 101, 116, 18, 3, 10, 1, 47, 74, 23, 10, 16, 104, 116, 116, 112, 46, 115, 116, 97,
            116, 117, 115, 95, 99, 111, 100, 101, 18, 3, 24, 200, 1, 74, 21, 10, 11, 104, 116, 116,
            112, 46, 115, 99, 104, 101, 109, 101, 18, 6, 10, 4, 104, 116, 116, 112, 74, 17, 10, 10,
            104, 116, 116, 112, 46, 114, 111, 117, 116, 101, 18, 3, 10, 1, 47, 74, 20, 10, 11, 104,
            116, 116, 112, 46, 109, 101, 116, 104, 111, 100, 18, 5, 10, 3, 71, 69, 84, 74, 24, 10,
            9, 104, 116, 116, 112, 46, 104, 111, 115, 116, 18, 11, 10, 9, 108, 111, 99, 97, 108,
            104, 111, 115, 116, 74, 20, 10, 11, 104, 116, 116, 112, 46, 102, 108, 97, 118, 111,
            114, 18, 5, 10, 3, 49, 46, 49, 74, 29, 10, 14, 104, 116, 116, 112, 46, 99, 108, 105,
            101, 110, 116, 95, 105, 112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48, 46, 49, 122, 0,
            18, 149, 4, 10, 27, 10, 18, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101, 116, 114,
            121, 95, 101, 99, 116, 111, 18, 5, 49, 46, 48, 46, 48, 18, 245, 3, 10, 16, 196, 206,
            162, 34, 18, 10, 86, 108, 234, 246, 51, 69, 0, 171, 1, 40, 18, 8, 117, 229, 127, 70, 9,
            173, 255, 14, 34, 8, 62, 64, 179, 38, 163, 41, 8, 151, 42, 42, 102, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 46, 114, 101, 112, 111,
            46, 113, 117, 101, 114, 121, 58, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97, 103,
            115, 48, 3, 57, 191, 36, 187, 220, 91, 196, 130, 23, 65, 78, 239, 198, 220, 91, 196,
            130, 23, 74, 30, 10, 23, 116, 111, 116, 97, 108, 95, 116, 105, 109, 101, 95, 109, 105,
            99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 3, 24, 162, 4, 74, 24, 10, 6, 115,
            111, 117, 114, 99, 101, 18, 14, 10, 12, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97,
            103, 115, 74, 29, 10, 23, 113, 117, 101, 117, 101, 95, 116, 105, 109, 101, 95, 109,
            105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 2, 24, 52, 74, 30, 10, 23,
            113, 117, 101, 114, 121, 95, 116, 105, 109, 101, 95, 109, 105, 99, 114, 111, 115, 101,
            99, 111, 110, 100, 115, 18, 3, 24, 233, 3, 74, 30, 10, 22, 105, 100, 108, 101, 95, 116,
            105, 109, 101, 95, 109, 105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 4, 24,
            243, 213, 35, 74, 30, 10, 24, 100, 101, 99, 111, 100, 101, 95, 116, 105, 109, 101, 95,
            109, 105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 2, 24, 5, 74, 31, 10, 6,
            100, 98, 46, 117, 114, 108, 18, 21, 10, 19, 101, 99, 116, 111, 58, 47, 47, 102, 102,
            115, 95, 112, 111, 115, 116, 103, 114, 101, 115, 74, 16, 10, 7, 100, 98, 46, 116, 121,
            112, 101, 18, 5, 10, 3, 115, 113, 108, 74, 136, 1, 10, 12, 100, 98, 46, 115, 116, 97,
            116, 101, 109, 101, 110, 116, 18, 120, 10, 118, 83, 69, 76, 69, 67, 84, 32, 102, 48,
            46, 34, 105, 100, 34, 44, 32, 102, 48, 46, 34, 100, 101, 115, 99, 114, 105, 112, 116,
            105, 111, 110, 34, 44, 32, 102, 48, 46, 34, 101, 110, 97, 98, 108, 101, 100, 34, 44,
            32, 102, 48, 46, 34, 110, 97, 109, 101, 34, 44, 32, 102, 48, 46, 34, 105, 110, 115,
            101, 114, 116, 101, 100, 95, 97, 116, 34, 44, 32, 102, 48, 46, 34, 117, 112, 100, 97,
            116, 101, 100, 95, 97, 116, 34, 32, 70, 82, 79, 77, 32, 34, 102, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 34, 32, 65, 83, 32, 102, 48, 74, 20, 10, 11, 100, 98,
            46, 105, 110, 115, 116, 97, 110, 99, 101, 18, 5, 10, 3, 102, 102, 115, 122, 0,
        ];

        let metrics = parse_traces_request(out).expect("Failed to parse");

        assert_eq!(metrics.len(), 2);

        let log = metrics[1].clone().into_log();

        let log_line = log.get("message.name").expect("Metric type is missed");

        assert_eq!(
            *log_line,
            Value::from("featureflagservice.repo.query:featureflags")
        );
    }
}
