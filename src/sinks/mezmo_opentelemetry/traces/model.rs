use crate::sinks::mezmo_opentelemetry::{
    models::{
        OpentelemetryAttributes, OpentelemetryModelMatch, OpentelemetryModelType,
        OpentelemetryResource, OpentelemetryScope, OpentelemetrySpanId, OpentelemetryTraceFlags,
        OpentelemetryTraceId, OpentelemetryTraceState, native_enum_number, native_envelope,
        native_time, value_to_system_time,
    },
    sink::OpentelemetrySinkError,
};

use opentelemetry::trace::{
    Event as TraceEvent, Link, SpanContext, SpanId, SpanKind, Status, TraceId,
};

use opentelemetry_proto::tonic::trace::v1::{span::SpanKind as ProtoSpanKind, status::StatusCode};
use opentelemetry_sdk::{
    export::trace::SpanData,
    trace::{SpanEvents, SpanLinks},
};
use std::{borrow::Cow, time::SystemTime};
use vector_lib::{
    config::log_schema,
    event::{Event, Value},
    lookup::PathPrefix,
};

#[derive(Debug)]
pub struct OpentelemetryTracesModel(pub SpanData);

impl OpentelemetryModelMatch for OpentelemetryTracesModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        if let Some(trace) = event.maybe_as_log() {
            let message = trace.get_message();
            let metadata = trace.get((PathPrefix::Event, log_schema().user_metadata_key()));

            if native_envelope(trace, "trace").is_some() {
                let partitioner_key: OpentelemetrySpanId = metadata
                    .and_then(|metadata| metadata.get("span_uniq_id"))
                    .into();
                return Some(OpentelemetryModelType::Traces {
                    partitioner_key: partitioner_key.into(),
                });
            }

            if let (Some(metadata), Some(message)) = (metadata, message) {
                let resource = metadata.get("resource");
                let attributes = metadata.get("attributes");
                let scope = metadata.get("scope");

                let trace_id = message.get("trace_id");
                let span_id = message.get("span_id");
                let events = message.get("events");
                let links = message.get("links");
                let partitioner_key: OpentelemetrySpanId = metadata.get("span_uniq_id").into();

                if resource
                    .and(attributes)
                    .and(scope)
                    .and(trace_id)
                    .and(span_id)
                    .and(events)
                    .and(links)
                    .is_some()
                {
                    return Some(OpentelemetryModelType::Traces {
                        partitioner_key: partitioner_key.into(),
                    });
                }
            }
        }

        None
    }
}

impl OpentelemetryTracesModel {
    pub const fn new(trace: SpanData) -> Self {
        Self(trace)
    }

    fn from_native(message: &Value, record: &Value) -> Result<Self, OpentelemetrySinkError> {
        let trace_id = TraceId::from_bytes(native_id(record, "traceId")?);
        let span_id = SpanId::from_bytes(native_id(record, "spanId")?);
        let trace_flags: OpentelemetryTraceFlags = record.get("flags").into();
        let trace_state: OpentelemetryTraceState = record.get("traceState").into();
        let parent_span_id: OpentelemetrySpanId = record.get("parentSpanId").into();

        let span_context = SpanContext::new(
            trace_id,
            span_id,
            trace_flags.into(),
            false,
            trace_state.into(),
        );

        let kind = native_enum_number(record.get("kind"), |name| {
            ProtoSpanKind::from_str_name(name).map(i32::from)
        });
        let span_kind = span_kind_from_number(kind.unwrap_or_default());

        let status = match record.get("status") {
            Some(status) => {
                let code = native_enum_number(status.get("code"), |name| {
                    StatusCode::from_str_name(name).map(i32::from)
                });
                let code = code
                    .and_then(|code| i32::try_from(code).ok())
                    .and_then(|code| StatusCode::try_from(code).ok())
                    .unwrap_or(StatusCode::Unset);
                status_from_code(code, status.get("message"))
            }
            None => Status::default(),
        };

        let mut links = SpanLinks::default();
        links.links = match record.get("links") {
            Some(Value::Array(links)) => links.iter().filter_map(native_link).collect(),
            _ => vec![],
        };
        links.dropped_count = native_count(record.get("droppedLinksCount"));

        let mut events = SpanEvents::default();
        events.events = match record.get("events") {
            Some(Value::Array(events)) => events.iter().filter_map(native_event).collect(),
            _ => vec![],
        };
        events.dropped_count = native_count(record.get("droppedEventsCount"));

        let span_attributes: OpentelemetryAttributes = record.get("attributes").into();

        Ok(Self::new(SpanData {
            span_context,
            parent_span_id: parent_span_id.into(),
            span_kind,
            name: string_or_empty(record.get("name")),
            start_time: native_time_or_now(record.get("startTimeUnixNano")),
            end_time: native_time_or_now(record.get("endTimeUnixNano")),
            attributes: span_attributes.into(),
            dropped_attributes_count: native_count(record.get("droppedAttributesCount")),
            events,
            links,
            status,
            resource: Cow::Owned(OpentelemetryResource::from_native(message).into()),
            instrumentation_lib: OpentelemetryScope::from_native(message).into(),
        }))
    }
}

/// A span cannot be sent without its ids, so unlike the legacy path a native span with a missing,
/// malformed or all-zero (OTLP invalid) id fails instead of being sent with an invalid one.
fn native_id<const N: usize>(
    record: &Value,
    field: &str,
) -> Result<[u8; N], OpentelemetrySinkError> {
    let mut id = [0; N];
    if let Some(Value::Bytes(hex)) = record.get(field)
        && hex.len() == N * 2
        && faster_hex::hex_decode(hex, &mut id).is_ok()
        && id != [0; N]
    {
        return Ok(id);
    }

    Err(OpentelemetrySinkError::new(&format!(
        "Trace event is invalid: record.{field} must be a non-zero {N}-byte hex string"
    )))
}

/// The legacy path also falls back to the current time when a span time is missing.
fn native_time_or_now(value: Option<&Value>) -> SystemTime {
    native_time(value).unwrap_or_else(SystemTime::now)
}

fn native_count(value: Option<&Value>) -> u32 {
    match value {
        Some(Value::Integer(count)) => u32::try_from(*count).unwrap_or(0),
        _ => 0,
    }
}

fn string_or_empty(value: Option<&Value>) -> Cow<'static, str> {
    match value {
        Some(Value::Bytes(bytes)) => Cow::from(String::from_utf8_lossy(bytes).into_owned()),
        _ => Cow::from(""),
    }
}

fn native_link(value: &Value) -> Option<Link> {
    let Value::Object(link) = value else {
        return None;
    };

    let trace_id: OpentelemetryTraceId = link.get("traceId").into();
    let span_id: OpentelemetrySpanId = link.get("spanId").into();
    let trace_flags: OpentelemetryTraceFlags = link.get("flags").into();
    let trace_state: OpentelemetryTraceState = link.get("traceState").into();
    let attributes: OpentelemetryAttributes = link.get("attributes").into();

    let span_context = SpanContext::new(
        trace_id.into(),
        span_id.into(),
        trace_flags.into(),
        false,
        trace_state.into(),
    );

    let mut span_link = Link::new(span_context, attributes.into());
    span_link.dropped_attributes_count = native_count(link.get("droppedAttributesCount"));
    Some(span_link)
}

fn native_event(value: &Value) -> Option<TraceEvent> {
    let Value::Object(event) = value else {
        return None;
    };

    let attributes: OpentelemetryAttributes = event.get("attributes").into();

    Some(TraceEvent::new(
        string_or_empty(event.get("name")),
        native_time_or_now(event.get("timeUnixNano")),
        attributes.into(),
        native_count(event.get("droppedAttributesCount")),
    ))
}

// The protocol/transport defines an `Unspecified` SpanKind, but this is no longer a valid
// SDK/API variant. SpanKind::Internal is the default for the type and is used as such here.
// https://github.com/open-telemetry/opentelemetry-rust/blob/6386f4599f7abc541dd46dc6d901044e45b59406/opentelemetry-stdout/src/trace/transform.rs#L122-L130
const fn span_kind_from_number(kind: i64) -> SpanKind {
    match kind {
        2 => SpanKind::Server,
        3 => SpanKind::Client,
        4 => SpanKind::Producer,
        5 => SpanKind::Consumer,
        _ => SpanKind::Internal,
    }
}

fn status_from_code(code: StatusCode, message: Option<&Value>) -> Status {
    match code {
        StatusCode::Unset => Status::Unset,
        StatusCode::Ok => Status::Ok,
        StatusCode::Error => Status::Error {
            description: string_or_empty(message),
        },
    }
}

impl TryFrom<Event> for OpentelemetryTracesModel {
    type Error = OpentelemetrySinkError;

    fn try_from(buf_event: Event) -> Result<Self, Self::Error> {
        let trace = buf_event.as_log();

        if let Some((message, record)) = native_envelope(trace, "trace") {
            return Self::from_native(message, record);
        }

        // Extract SpanData.resource and SpanData.instrumentation_lib from event
        let resource = OpentelemetryResource::from(trace);
        let instrumentation_lib = OpentelemetryScope::from(trace);

        // Extract span attributes from metadata
        let span_attributes: OpentelemetryAttributes = trace
            .get((
                PathPrefix::Event,
                format!("{}.attributes", log_schema().user_metadata_key()).as_str(),
            ))
            .into();

        // Extract from message
        if let Some(message) = trace.get_message() {
            let name = match message.get("name") {
                Some(Value::Bytes(name_bytes)) => {
                    Cow::from(String::from_utf8_lossy(name_bytes).into_owned())
                }
                _ => Cow::from(""),
            };

            // LOG-19724: trace_flags are not currently captured/defined, this field was added
            // in a more recent version of the protocol, but our source does not include it in the
            // protocol impl it uses.
            // https://github.com/open-telemetry/opentelemetry-rust/commit/27b19b60261f342cec0559f26634ca8f02ed02ac#diff-cfa0a91439f7fb81c51a342043e87175f75c5394b0ff5c9aa7e55c3589a7bb11R91-R106
            let trace_flags: OpentelemetryTraceFlags = message.get("flags").into();
            let trace_id: OpentelemetryTraceId = message.get("trace_id").into();
            let trace_state: OpentelemetryTraceState = message.get("trace_state").into();
            let span_id: OpentelemetrySpanId = message.get("span_id").into();
            let parent_span_id: OpentelemetrySpanId = message.get("parent_span_id").into();

            // TODO: determine correct value for `is_remote`. This marker is not included
            // in the incoming request/event.
            let span_context = SpanContext::new(
                trace_id.into(),
                span_id.into(),
                trace_flags.into(),
                false,
                trace_state.into(),
            );

            let start_time =
                value_to_system_time(message.get("start_timestamp").unwrap_or(&Value::Null));
            let end_time =
                value_to_system_time(message.get("end_timestamp").unwrap_or(&Value::Null));

            let status = if let Some(status) = message.get("status") {
                match status {
                    Value::Object(_status) => {
                        let code = match status.get("code") {
                            Some(Value::Integer(code_int)) => {
                                StatusCode::try_from(*code_int as i32).unwrap_or(StatusCode::Unset)
                            }
                            _ => StatusCode::Unset,
                        };

                        status_from_code(code, status.get("message"))
                    }
                    _ => Status::default(),
                }
            } else {
                Status::default()
            };

            let span_kind = match message.get("kind") {
                Some(Value::Integer(kind_int)) => span_kind_from_number(*kind_int),
                _ => SpanKind::Internal,
            };

            let dropped_links_count = match message.get("dropped_links_count") {
                Some(Value::Integer(count)) => *count as u32,
                _ => 0,
            };
            let dropped_events_count = match message.get("dropped_events_count") {
                Some(Value::Integer(count)) => *count as u32,
                _ => 0,
            };
            let dropped_attributes_count = match message.get("dropped_attributes_count") {
                Some(Value::Integer(count)) => *count as u32,
                _ => 0,
            };

            // LOG-19721: determine correct behavior for the scenario where a subset of `link` or
            // `event` objects are not valid. Currently this discards any links that are not valid.
            let mut links = SpanLinks::default();
            links.links = match message.get("links") {
                Some(Value::Array(links)) => {
                    links.iter().filter_map(value_to_link).collect::<Vec<_>>()
                }
                _ => vec![],
            };
            links.dropped_count = dropped_links_count;

            let mut events = SpanEvents::default();
            events.events = match message.get("events") {
                Some(Value::Array(events)) => {
                    events.iter().filter_map(value_to_event).collect::<Vec<_>>()
                }
                _ => vec![],
            };
            events.dropped_count = dropped_events_count;

            Ok(Self::new(SpanData {
                span_context,
                parent_span_id: parent_span_id.into(),
                span_kind,
                name,
                start_time,
                end_time,
                attributes: span_attributes.into(),
                dropped_attributes_count,
                events,
                links,
                status,
                resource: Cow::Owned(resource.into()),
                instrumentation_lib: instrumentation_lib.into(),
            }))
        } else {
            Err(OpentelemetrySinkError::new(
                "Trace even is invalid. No message found",
            ))
        }
    }
}

fn value_to_link(value: &Value) -> Option<Link> {
    match value {
        Value::Object(link) => {
            let trace_flags: OpentelemetryTraceFlags = link.get("flags").into();
            let trace_state: OpentelemetryTraceState = link.get("trace_state").into();
            let trace_id: OpentelemetryTraceId = link.get("trace_id").into();
            let span_id: OpentelemetrySpanId = link.get("span_id").into();
            let span_attributes: OpentelemetryAttributes = link.get("attributes").into();
            let dropped_attributes_count = match link.get("dropped_attributes_count") {
                Some(Value::Integer(count)) => *count as u32,
                _ => 0,
            };

            // TODO: determine correct value for `is_remote`
            let span_context = SpanContext::new(
                trace_id.into(),
                span_id.into(),
                trace_flags.into(),
                false,
                trace_state.into(),
            );

            let mut link = Link::new(span_context, span_attributes.into());
            link.dropped_attributes_count = dropped_attributes_count;
            Some(link)
        }
        _ => None,
    }
}

fn value_to_event(value: &Value) -> Option<TraceEvent> {
    match value {
        Value::Object(event) => {
            let name = match event.get("name") {
                Some(Value::Bytes(name_bytes)) => {
                    Cow::from(String::from_utf8_lossy(name_bytes).into_owned())
                }
                _ => Cow::from(""),
            };

            let timestamp = value_to_system_time(event.get("timestamp").unwrap_or(&Value::Null));
            let attributes: OpentelemetryAttributes = event.get("attributes").into();
            let dropped_attributes_count = match event.get("dropped_attributes_count") {
                Some(Value::Integer(count)) => *count as u32,
                _ => 0,
            };

            Some(TraceEvent::new(
                name,
                timestamp,
                attributes.into(),
                dropped_attributes_count,
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, time::SystemTime};

    use super::*;

    use crate::event::Value;
    use crate::sinks::mezmo_opentelemetry::traces::encoding::group_spans;
    use chrono::DateTime;
    use opentelemetry::{
        Key, KeyValue,
        trace::{
            Event as TraceEvent, Link, SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId,
            TraceState,
        },
    };
    use opentelemetry_proto::tonic::common::v1::any_value;
    use opentelemetry_proto::tonic::resource::v1::Resource as ProtoResource;
    use opentelemetry_sdk::{Resource, Scope};
    use vector_lib::event::{Event, EventMetadata, LogEvent, ObjectMap};

    #[test]
    fn test_otlp_sink_trace_model_matcher_matches() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let event = Event::Log(LogEvent::from_map(
            btreemap! {
                "metadata" => Value::from(btreemap!{
                    "resource" => "resource",
                    "attributes" => "attributes",
                    "scope" => "scope",
                    "span_uniq_id" => uniq_id,
                }),
                "message" => Value::from(btreemap!{
                    "trace_id" => "trace_id",
                    "span_id" => "span_id",
                    "events" => "events",
                    "links" => "links",
                }),
            },
            EventMetadata::default(),
        ));

        assert!(
            matches!(
                OpentelemetryTracesModel::maybe_match(&event),
                Some(OpentelemetryModelType::Traces { partitioner_key: _ })
            ),
            "event matcher does not match expected event"
        );
    }

    #[test]
    fn test_otlp_sink_trace_model_matcher_not_a_trace() {
        let event = Event::Log(LogEvent::from_map(
            btreemap! {
                "metadata" => "metadata",
                "message" => "message",
            },
            EventMetadata::default(),
        ));

        assert!(matches!(
            OpentelemetryTracesModel::maybe_match(&event),
            None
        ),);
    }

    #[test]
    fn test_otlp_sink_event_to_trace_model() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let trace_id_hex = faster_hex::hex_string(&[
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ]);
        let span_id_hex = faster_hex::hex_string(&[76, 114, 27, 243, 62, 60, 175, 143]);
        let parent_span_id_hex = faster_hex::hex_string(&[79, 114, 27, 243, 61, 60, 175, 143]);
        let link_1_trace_id_hex = faster_hex::hex_string(&[
            96, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ]);
        let link_1_span_id_hex = faster_hex::hex_string(&[77, 114, 27, 243, 62, 60, 175, 143]);

        let message = btreemap! {
            "name" => "test_span_name",
            "trace_id" => Value::from(trace_id_hex.clone()),
            "trace_state" => "foo=,apple=banana",
            "span_id" => Value::from(span_id_hex.clone()),
            "parent_span_id" => Value::from(parent_span_id_hex.clone()),
            // LOG-19724: this field is not currently captured/defined in our source impl
            "flags" => 1,
            "start_timestamp" => (
                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                    .expect("timestamp should be a valid timestamp")
            ),
            "dropped_attributes_count" => 1,
            "dropped_events_count" => 2,
            "dropped_links_count" => 3,
            "end_timestamp" => (
                DateTime::from_timestamp(1_579_134_612_i64, 12_u32)
                    .expect("timestamp should be a valid timestamp")
            ),
            "events" => vec![
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_1_attr",
                    },
                    "dropped_attributes_count" => 4,
                    "name" => "test_name_1",
                    "timestamp" => (
                        DateTime::from_timestamp(1_579_134_612_i64, 13_u32)
                            .expect("timestamp should be a valid timestamp")
                    ),
                },
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_2_attr",
                    },
                    "dropped_attributes_count" => 5,
                    "name" => "test_name_2",
                    "timestamp" => (
                        DateTime::from_timestamp(1_579_134_612_i64, 14_u32)
                            .expect("timestamp should be a valid timestamp")
                    ),
                }
            ],
            "hostname" => Value::Null,
            "kind" => 2,
            "links" => vec![
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_link_1_attr",
                    },
                    "dropped_attributes_count" => 6,
                    "span_id" => Value::from(link_1_span_id_hex.clone()),
                    "trace_id" => Value::from(link_1_trace_id_hex.clone()),
                    "trace_state" => "bar=,carrot=broccoli",
                },
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_link_2_attr",
                    },
                    "dropped_attributes_count" => 7,
                    "span_id" => Value::from("invalid"),
                    "trace_id" => Value::from("invalid"),
                    "trace_state" => "invalid",
                }
            ],
            "status" => btreemap!{
                "code" => 2,
                "message" => "test_status_message",
            },
        };

        let metadata = btreemap! {
            "resource" => btreemap!{
                "attributes" => btreemap!{
                    "test" => "test_resource_attr",
                },
                "dropped_attributes_count" => 5,
                "schema_url" => "https://resource.example.com",
            },
            "scope" => btreemap!{
                "name" => "test_scope_name",
                "schema_url" => "https://scope.example.com",
                "version" => "1.2.3",
                "attributes" => btreemap!{
                    "test" => "test_scope_attr",
                },
            },
            "attributes" => btreemap!{
                "test" => "test_root_attr",
            },
            "level" => "trace",
            "span_uniq_id" => uniq_id,
        };

        let event = Event::Log(LogEvent::from_map(
            btreemap! {
                "metadata" => Value::from(metadata),
                "message" => Value::from(message),
            },
            EventMetadata::default(),
        ));

        assert!(
            matches!(
                OpentelemetryTracesModel::maybe_match(&event),
                Some(OpentelemetryModelType::Traces { partitioner_key: _ })
            ),
            "event matcher does not match expected event"
        );

        let model =
            OpentelemetryTracesModel::try_from(event).expect("event cannot be coerced into model");

        let span_data = model.0;

        let expected_span_context = SpanContext::new(
            TraceId::from_hex("5f467fe7bf42676c05e20ba4a90e448e").unwrap(),
            SpanId::from_hex("4c721bf33e3caf8f").unwrap(),
            TraceFlags::new(1),
            false,
            TraceState::from_str("foo=,apple=banana").unwrap(),
        );
        assert_eq!(span_data.span_context, expected_span_context);

        let expected_parent_span_id =
            SpanId::from_hex(parent_span_id_hex.clone().as_str()).unwrap();
        assert_eq!(span_data.parent_span_id, expected_parent_span_id);

        assert_eq!(span_data.span_kind, SpanKind::Server);

        assert_eq!(span_data.name, Cow::from("test_span_name"));

        assert_eq!(
            span_data.start_time,
            <chrono::DateTime<chrono::Utc> as Into<SystemTime>>::into(
                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                    .expect("timestamp should be a valid timestamp")
            )
        );

        assert_eq!(
            span_data.end_time,
            <chrono::DateTime<chrono::Utc> as Into<SystemTime>>::into(
                DateTime::from_timestamp(1_579_134_612_i64, 12_u32)
                    .expect("timestamp should be a valid timestamp")
            )
        );

        let mut expected_events = SpanEvents::default();
        expected_events.events = vec![
            TraceEvent::new(
                "test_name_1",
                (DateTime::from_timestamp(1_579_134_612_i64, 13_u32)
                    .expect("timestamp should be a valid timestamp"))
                .into(),
                vec![KeyValue::new(
                    "test".to_string(),
                    "test_event_1_attr".to_string(),
                )],
                4,
            ),
            TraceEvent::new(
                "test_name_2",
                (DateTime::from_timestamp(1_579_134_612_i64, 14_u32)
                    .expect("timestamp should be a valid timestamp"))
                .into(),
                vec![KeyValue::new(
                    "test".to_string(),
                    "test_event_2_attr".to_string(),
                )],
                5,
            ),
        ];
        expected_events.dropped_count = 2;
        assert_eq!(span_data.events, expected_events);

        let mut expected_links = SpanLinks::default();
        let mut expected_link_1 = Link::new(
            SpanContext::new(
                TraceId::from_hex(link_1_trace_id_hex.clone().as_str()).unwrap(),
                SpanId::from_hex(link_1_span_id_hex.clone().as_str()).unwrap(),
                TraceFlags::new(0),
                false,
                TraceState::from_str("bar=,carrot=broccoli").unwrap(),
            ),
            vec![KeyValue::new(
                "test".to_string(),
                "test_link_1_attr".to_string(),
            )],
        );
        let mut expected_link_2 = Link::new(
            SpanContext::new(
                TraceId::INVALID,
                SpanId::INVALID,
                TraceFlags::new(0),
                false,
                TraceState::NONE,
            ),
            vec![KeyValue::new(
                "test".to_string(),
                "test_link_2_attr".to_string(),
            )],
        );
        expected_link_1.dropped_attributes_count = 6;
        expected_link_2.dropped_attributes_count = 7;
        expected_links.links = vec![expected_link_1, expected_link_2];
        expected_links.dropped_count = 3;
        assert_eq!(span_data.links, expected_links);

        let expected_resource = Cow::Owned(Resource::from_schema_url(
            vec![KeyValue::new(
                "test".to_string(),
                "test_resource_attr".to_string(),
            )],
            "https://resource.example.com",
        ));
        assert_eq!(span_data.resource, expected_resource);

        let expected_instrumentation_lib = Scope::new(
            "test_scope_name".to_string(),
            Some("1.2.3".to_string()),
            Some("https://scope.example.com".to_string()),
            Some(vec![KeyValue::new(
                "test".to_string(),
                "test_scope_attr".to_string(),
            )]),
        );

        assert_eq!(
            span_data.status,
            Status::Error {
                description: "test_status_message".into()
            }
        );
        assert_eq!(span_data.instrumentation_lib, expected_instrumentation_lib);
        assert_eq!(span_data.dropped_attributes_count, 1);
    }

    const NATIVE_TRACE_ID: &str = "5f467fe7bf42676c05e20ba4a90e448e";
    const NATIVE_SPAN_ID: &str = "4c721bf33e3caf8f";

    fn native_span_event(service: &str, scope: &str, record: ObjectMap) -> Event {
        Event::Log(LogEvent::from_map(
            btreemap! {
                "message" => btreemap! {
                    "type" => "trace",
                    "resource" => btreemap! {
                        "attributes" => btreemap! { "service.name" => service },
                    },
                    "scope" => btreemap! {
                        "name" => scope,
                        "version" => "1.0.0",
                        "attributes" => btreemap! { "scope.attr" => "scoped" },
                    },
                    "record" => record,
                },
            },
            EventMetadata::default(),
        ))
    }

    fn native_span_record(name: &str) -> ObjectMap {
        btreemap! {
            "traceId" => NATIVE_TRACE_ID,
            "spanId" => NATIVE_SPAN_ID,
            "name" => name,
        }
    }

    fn native_span_data(record: ObjectMap) -> SpanData {
        let event = native_span_event("checkout", "checkout.scope", record);
        OpentelemetryTracesModel::try_from(event)
            .expect("native span cannot be coerced into model")
            .0
    }

    fn service_name(resource: Option<ProtoResource>) -> String {
        let attributes = resource.expect("resource should be set").attributes;
        let service = attributes
            .into_iter()
            .find(|attribute| attribute.key == "service.name")
            .and_then(|attribute| attribute.value?.value);

        match service {
            Some(any_value::Value::StringValue(service)) => service,
            other => panic!("unexpected service.name: {other:?}"),
        }
    }

    fn nanos(nanos: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)
    }

    #[test]
    fn test_otlp_sink_trace_model_matcher_matches_native_span() {
        let event = native_span_event("checkout", "checkout.scope", native_span_record("span"));

        assert_eq!(
            OpentelemetryTracesModel::maybe_match(&event),
            Some(OpentelemetryModelType::Traces {
                partitioner_key: [0; 8]
            })
        );
    }

    #[test]
    fn test_otlp_sink_native_span_to_trace_model() {
        let link_trace_id = "605f7fe7bf42676c05e20ba4a90e448e";
        let link_span_id = "4d721bf33e3caf8f";
        let parent_span_id = "4f721bf33d3caf8f";

        let span_data = native_span_data(btreemap! {
            "traceId" => NATIVE_TRACE_ID,
            "spanId" => NATIVE_SPAN_ID,
            "parentSpanId" => parent_span_id,
            "name" => "charge",
            "kind" => "SPAN_KIND_SERVER",
            "traceState" => "foo=,apple=banana",
            "flags" => 1,
            "startTimeUnixNano" => "1792065600000000001",
            "endTimeUnixNano" => 1_792_065_600_000_000_002_i64,
            "attributes" => btreemap! {
                "http.method" => "GET",
                "http.status_code" => 200,
            },
            "status" => btreemap! {
                "code" => "STATUS_CODE_ERROR",
                "message" => "payment declined",
            },
            "events" => vec![btreemap! {
                "timeUnixNano" => "1792065600000000003",
                "name" => "retry",
                "attributes" => btreemap! { "attempt" => 2 },
                "droppedAttributesCount" => 4,
            }],
            "links" => vec![
                btreemap! {
                    "traceId" => link_trace_id,
                    "spanId" => link_span_id,
                    "traceState" => "bar=,carrot=broccoli",
                    "flags" => 1,
                    "attributes" => btreemap! { "link.attr" => "linked" },
                    "droppedAttributesCount" => 6,
                },
                btreemap! {
                    "traceId" => "invalid",
                    "spanId" => "invalid",
                },
            ],
            "droppedAttributesCount" => 1,
            "droppedEventsCount" => 2,
            "droppedLinksCount" => 3,
        });

        assert_eq!(
            span_data.span_context,
            SpanContext::new(
                TraceId::from_hex(NATIVE_TRACE_ID).unwrap(),
                SpanId::from_hex(NATIVE_SPAN_ID).unwrap(),
                TraceFlags::new(1),
                false,
                TraceState::from_str("foo=,apple=banana").unwrap(),
            )
        );
        assert_eq!(
            span_data.parent_span_id,
            SpanId::from_hex(parent_span_id).unwrap()
        );
        assert_eq!(span_data.name, Cow::from("charge"));
        assert_eq!(span_data.span_kind, SpanKind::Server);
        assert_eq!(span_data.start_time, nanos(1_792_065_600_000_000_001));
        assert_eq!(span_data.end_time, nanos(1_792_065_600_000_000_002));
        assert_eq!(
            span_data.attributes,
            vec![
                KeyValue::new("http.method", "GET"),
                KeyValue::new("http.status_code", 200_i64),
            ]
        );
        assert_eq!(
            span_data.status,
            Status::Error {
                description: "payment declined".into()
            }
        );
        assert_eq!(span_data.dropped_attributes_count, 1);

        let expected_event = TraceEvent::new(
            "retry",
            nanos(1_792_065_600_000_000_003),
            vec![KeyValue::new("attempt", 2_i64)],
            4,
        );
        let mut expected_events = SpanEvents::default();
        expected_events.events = vec![expected_event];
        expected_events.dropped_count = 2;
        assert_eq!(span_data.events, expected_events);

        let mut expected_link = Link::new(
            SpanContext::new(
                TraceId::from_hex(link_trace_id).unwrap(),
                SpanId::from_hex(link_span_id).unwrap(),
                TraceFlags::new(1),
                false,
                TraceState::from_str("bar=,carrot=broccoli").unwrap(),
            ),
            vec![KeyValue::new("link.attr", "linked")],
        );
        expected_link.dropped_attributes_count = 6;
        let invalid_link = Link::new(
            SpanContext::new(
                TraceId::INVALID,
                SpanId::INVALID,
                TraceFlags::new(0),
                false,
                TraceState::NONE,
            ),
            vec![],
        );
        let mut expected_links = SpanLinks::default();
        expected_links.links = vec![expected_link, invalid_link];
        expected_links.dropped_count = 3;
        assert_eq!(span_data.links, expected_links);

        assert_eq!(
            span_data.resource.get(Key::new("service.name")),
            Some("checkout".into())
        );
        assert_eq!(span_data.instrumentation_lib.name, "checkout.scope");
        assert_eq!(
            span_data.instrumentation_lib.version,
            Some(Cow::from("1.0.0"))
        );
        assert_eq!(
            span_data.instrumentation_lib.attributes,
            vec![KeyValue::new("scope.attr", "scoped")]
        );
    }

    #[test]
    fn test_otlp_sink_native_span_defaults() {
        let span_data = native_span_data(native_span_record("bare"));

        assert_eq!(span_data.parent_span_id, SpanId::INVALID);
        assert_eq!(span_data.span_kind, SpanKind::Internal);
        assert_eq!(span_data.status, Status::Unset);
        assert!(span_data.attributes.is_empty());
        assert_eq!(span_data.events, SpanEvents::default());
        assert_eq!(span_data.links, SpanLinks::default());
        assert_eq!(span_data.dropped_attributes_count, 0);
    }

    #[test]
    fn test_otlp_sink_native_span_kind_and_status_code() {
        for (kind, expected) in [
            (Value::from("SPAN_KIND_SERVER"), SpanKind::Server),
            (Value::from("SPAN_KIND_CLIENT"), SpanKind::Client),
            (Value::from("SPAN_KIND_PRODUCER"), SpanKind::Producer),
            (Value::from("SPAN_KIND_CONSUMER"), SpanKind::Consumer),
            (Value::from("SPAN_KIND_INTERNAL"), SpanKind::Internal),
            (Value::from("SPAN_KIND_UNSPECIFIED"), SpanKind::Internal),
            (Value::from(3), SpanKind::Client),
            (Value::from(0), SpanKind::Internal),
        ] {
            let mut record = native_span_record("kind");
            record.insert("kind".into(), kind.clone());
            let span_kind = native_span_data(record).span_kind;

            assert_eq!(span_kind, expected, "kind {kind}");
        }

        let error = Status::Error {
            description: "failed".into(),
        };
        for (code, expected) in [
            (Value::from("STATUS_CODE_UNSET"), Status::Unset),
            (Value::from("STATUS_CODE_OK"), Status::Ok),
            (Value::from("STATUS_CODE_ERROR"), error.clone()),
            (Value::from(1), Status::Ok),
            (Value::from(2), error),
        ] {
            let status: ObjectMap = btreemap! {
                "code" => code.clone(),
                "message" => "failed",
            };
            let mut record = native_span_record("status");
            record.insert("status".into(), status.into());
            let status = native_span_data(record).status;

            assert_eq!(status, expected, "code {code}");
        }
    }

    #[test]
    fn test_otlp_sink_native_span_requires_ids() {
        let mut record = native_span_record("no trace id");
        record.remove("traceId");
        let error = OpentelemetryTracesModel::try_from(native_span_event("a", "b", record))
            .expect_err("a span without traceId must not convert");
        assert!(
            error.to_string().contains("record.traceId"),
            "unexpected error: {error}"
        );

        let mut record = native_span_record("bad span id");
        record.insert("spanId".into(), Value::from("not-hex!"));
        let error = OpentelemetryTracesModel::try_from(native_span_event("a", "b", record))
            .expect_err("a span with a malformed spanId must not convert");
        assert!(
            error.to_string().contains("record.spanId"),
            "unexpected error: {error}"
        );

        let mut record = native_span_record("zero trace id");
        record.insert("traceId".into(), Value::from("0".repeat(32)));
        let error = OpentelemetryTracesModel::try_from(native_span_event("a", "b", record))
            .expect_err("a span with an all-zero traceId must not convert");
        assert!(
            error.to_string().contains("record.traceId"),
            "unexpected error: {error}"
        );

        let mut record = native_span_record("zero span id");
        record.insert("spanId".into(), Value::from("0".repeat(16)));
        let error = OpentelemetryTracesModel::try_from(native_span_event("a", "b", record))
            .expect_err("a span with an all-zero spanId must not convert");
        assert!(
            error.to_string().contains("record.spanId"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_otlp_sink_group_spans_by_resource_and_scope() {
        let models = [
            ("cart", "cart.http", "cart-http-1"),
            ("billing", "billing.db", "billing-db-1"),
            ("cart", "cart.db", "cart-db-1"),
            ("cart", "cart.http", "cart-http-2"),
        ]
        .into_iter()
        .map(|(service, scope, name)| {
            let event = native_span_event(service, scope, native_span_record(name));
            OpentelemetryTracesModel::try_from(event)
                .expect("native span cannot be coerced into model")
        })
        .collect::<Vec<_>>();

        let resource_spans = group_spans(models);
        assert_eq!(resource_spans.len(), 2);

        let mut scope_counts = vec![];
        let mut placements = vec![];
        for resource_spans in resource_spans {
            let service = service_name(resource_spans.resource);
            scope_counts.push((service.clone(), resource_spans.scope_spans.len()));

            for scope_spans in resource_spans.scope_spans {
                let scope = scope_spans.scope.expect("scope should be set").name;
                for span in scope_spans.spans {
                    placements.push(format!("{service}/{scope}/{}", span.name));
                }
            }
        }
        scope_counts.sort();
        placements.sort();

        assert_eq!(
            scope_counts,
            [("billing".to_string(), 1), ("cart".to_string(), 2)]
        );
        assert_eq!(
            placements,
            [
                "billing/billing.db/billing-db-1",
                "cart/cart.db/cart-db-1",
                "cart/cart.http/cart-http-1",
                "cart/cart.http/cart-http-2",
            ]
        );
    }
}
