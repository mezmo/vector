use crate::sinks::opentelemetry::{
    models::{
        value_to_system_time, OpentelemetryAttributes, OpentelemetryModelMatch,
        OpentelemetryModelType, OpentelemetryResource, OpentelemetryScope, OpentelemetrySpanId,
        OpentelemetryTraceFlags, OpentelemetryTraceId, OpentelemetryTraceState,
    },
    sink::OpentelemetrySinkError,
};

use opentelemetry::trace::{Event as TraceEvent, Link, SpanContext, SpanKind, Status};

use opentelemetry_proto::tonic::trace::v1::status::StatusCode;
use opentelemetry_sdk::{
    export::trace::SpanData,
    trace::{SpanEvents, SpanLinks},
};
use std::borrow::Cow;
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

            if let (Some(metadata), Some(message)) = (metadata, message) {
                let resource = metadata.get("resource");
                let attributes = metadata.get("attributes");
                let scope = metadata.get("scope");

                let trace_id = message.get("trace_id");
                let span_id = message.get("span_id");
                let events = message.get("events");
                let links = message.get("links");

                if resource
                    .and(attributes)
                    .and(scope)
                    .and(trace_id)
                    .and(span_id)
                    .and(events)
                    .and(links)
                    .is_some()
                {
                    return Some(OpentelemetryModelType::Traces);
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
}

impl TryFrom<Event> for OpentelemetryTracesModel {
    type Error = OpentelemetrySinkError;

    fn try_from(buf_event: Event) -> Result<Self, Self::Error> {
        let trace = buf_event.as_log();

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

                        let description = match status.get("message") {
                            Some(Value::Bytes(message_bytes)) => {
                                Cow::from(String::from_utf8_lossy(message_bytes).into_owned())
                            }
                            _ => Cow::from(""),
                        };

                        match code {
                            StatusCode::Unset => Status::Unset,
                            StatusCode::Ok => Status::Ok,
                            StatusCode::Error => Status::Error { description },
                        }
                    }
                    _ => Status::default(),
                }
            } else {
                Status::default()
            };

            // The protocol/transport defines an `Unspecified` SpanKind, but this is no longer a valid
            // SDK/API variant. SpanKind::Internal is the default for the type and is used as such here.
            // https://github.com/open-telemetry/opentelemetry-rust/blob/6386f4599f7abc541dd46dc6d901044e45b59406/opentelemetry-stdout/src/trace/transform.rs#L122-L130
            let span_kind = match message.get("kind") {
                Some(Value::Integer(kind_int)) => match kind_int {
                    2 => SpanKind::Server,
                    3 => SpanKind::Client,
                    4 => SpanKind::Producer,
                    5 => SpanKind::Consumer,
                    _ => SpanKind::Internal,
                },
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
    use chrono::{NaiveDateTime, TimeZone, Utc};
    use opentelemetry::{
        trace::{
            Event as TraceEvent, Link, SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId,
            TraceState,
        },
        KeyValue,
    };
    use opentelemetry_sdk::{Resource, Scope};
    use vector_lib::event::{Event, EventMetadata, LogEvent};

    #[test]
    fn test_otlp_sink_trace_model_matcher_matches() {
        let event = Event::Log(LogEvent::from_map(
            btreemap! {
                "metadata" => Value::from(btreemap!{
                    "resource" => "resource",
                    "attributes" => "attributes",
                    "scope" => "scope",
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
                Some(OpentelemetryModelType::Traces)
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
            "start_timestamp" => Utc.from_utc_datetime(
                &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                    .expect("timestamp should be a valid timestamp"),
            ),
            "dropped_attributes_count" => 1,
            "dropped_events_count" => 2,
            "dropped_links_count" => 3,
            "end_timestamp" => Utc.from_utc_datetime(
                &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 12_u32)
                    .expect("timestamp should be a valid timestamp"),
            ),
            "events" => vec![
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_1_attr",
                    },
                    "dropped_attributes_count" => 4,
                    "name" => "test_name_1",
                    "timestamp" => Utc.from_utc_datetime(
                        &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 13_u32)
                            .expect("timestamp should be a valid timestamp"),
                    ),
                },
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_2_attr",
                    },
                    "dropped_attributes_count" => 5,
                    "name" => "test_name_2",
                    "timestamp" => Utc.from_utc_datetime(
                        &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 14_u32)
                            .expect("timestamp should be a valid timestamp"),
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
                Some(OpentelemetryModelType::Traces)
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
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                        .expect("timestamp should be a valid timestamp"),
                )
            )
        );

        assert_eq!(
            span_data.end_time,
            <chrono::DateTime<chrono::Utc> as Into<SystemTime>>::into(
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 12_u32)
                        .expect("timestamp should be a valid timestamp"),
                )
            )
        );

        let mut expected_events = SpanEvents::default();
        expected_events.events = vec![
            TraceEvent::new(
                "test_name_1",
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 13_u32)
                        .expect("timestamp should be a valid timestamp"),
                )
                .into(),
                vec![KeyValue::new(
                    "test".to_string(),
                    "test_event_1_attr".to_string(),
                )],
                4,
            ),
            TraceEvent::new(
                "test_name_2",
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 14_u32)
                        .expect("timestamp should be a valid timestamp"),
                )
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
}
