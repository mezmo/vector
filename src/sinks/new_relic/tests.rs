use std::{convert::TryFrom, num::NonZeroU32};

use chrono::Utc;
use futures::{future::ready, stream};
use serde::Deserialize;
use serde_json::{json, to_value};
use vector_lib::config::{Tags, Telemetry, init_telemetry};
use vrl::value;

use super::*;
use crate::{
    config::{GenerateConfig, SinkConfig, SinkContext},
    event::{Event, LogEvent, Metric, MetricKind, MetricValue},
    test_util::{
        components::{
            DATA_VOLUME_SINK_TAGS, SINK_TAGS, run_and_assert_data_volume_sink_compliance,
            run_and_assert_sink_compliance,
        },
        http::{always_200_response, spawn_blackhole_http_server},
    },
};

#[test]
fn generate_config() {
    crate::test_util::test_generate_config::<NewRelicConfig>();
}

async fn sink() -> (VectorSink, Event) {
    let mock_endpoint = spawn_blackhole_http_server(always_200_response).await;

    let config = NewRelicConfig::generate_config().to_string();
    let mut config = NewRelicConfig::deserialize(
        toml::de::ValueDeserializer::parse(&config).expect("toml should deserialize"),
    )
    .expect("config should be valid");
    config.override_uri = Some(mock_endpoint);

    let context = SinkContext::default();
    let (sink, _healthcheck) = config.build(context).await.unwrap();

    let event = Event::Log(LogEvent::from("simple message"));

    (sink, event)
}

#[tokio::test]
async fn component_spec_compliance() {
    let (sink, event) = sink().await;
    run_and_assert_sink_compliance(sink, stream::once(ready(event)), &SINK_TAGS).await;
}

#[tokio::test]
async fn component_spec_compliance_data_volume() {
    // We need to configure Vector to emit the service and source tags.
    // The default is to not emit these.
    init_telemetry(
        Telemetry {
            tags: Tags {
                emit_service: true,
                emit_source: true,
            },
        },
        true,
    );

    let (sink, event) = sink().await;
    run_and_assert_data_volume_sink_compliance(
        sink,
        stream::once(ready(event)),
        &DATA_VOLUME_SINK_TAGS,
    )
    .await;
}

#[test]
fn generates_event_api_model_without_message_field() {
    let event = Event::Log(LogEvent::from(value!({
        "eventType": "TestEvent",
        "user": "Joe",
        "user_id": 123456,
    })));
    let model =
        EventsApiModel::try_from(vec![event]).expect("Failed mapping events into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "eventType": "TestEvent",
            "user": "Joe",
            "user_id": 123456,
        }])
    );
}

#[test]
fn generates_event_api_model_with_message_field() {
    let event = Event::Log(LogEvent::from(value!({
        "eventType": "TestEvent",
        "user": "Joe",
        "user_id": 123456,
        "message": "This is a message",
    })));
    let model =
        EventsApiModel::try_from(vec![event]).expect("Failed mapping events into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "eventType": "TestEvent",
            "user": "Joe",
            "user_id": 123456,
            "message": "This is a message",
        }])
    );
}

#[test]
fn generates_event_api_model_with_json_inside_message_field() {
    let event = Event::Log(LogEvent::from(value!({
        "eventType": "TestEvent",
        "user": "Joe",
        "user_id": 123456,
        "message": "{\"my_key\" : \"my_value\"}",
    })));
    let model =
        EventsApiModel::try_from(vec![event]).expect("Failed mapping events into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "eventType": "TestEvent",
            "user": "Joe",
            "user_id": 123456,
            "my_key": "my_value",
        }])
    );
}

#[test]
fn generates_event_api_model_with_dotted_fields() {
    let sub = value!({"two":"three"});
    let event = Event::Log(LogEvent::from(value!({
        "one.two": "Joe",
        "eventType": "TestEvent",
        "four": sub,
    })));
    let model =
        EventsApiModel::try_from(vec![event]).expect("Failed mapping events into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "eventType": "TestEvent",
            "one.two": "Joe",
            "four.two": "three",
        }])
    );
}

#[test]
fn generates_log_api_model_without_message_field() {
    let event = Event::Log(LogEvent::from(value!({"tag_key": "tag_value"})));
    let model = LogsApiModel::try_from(vec![event]).expect("Failed mapping logs into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "logs": [{
                "message": "log from vector",
                "attributes": {"tag_key": "tag_value"},
            }]
        }])
    );
}

#[test]
fn generates_log_api_model_with_message_field() {
    let event = Event::Log(LogEvent::from(value!({
        "tag_key": "tag_value",
        "message": "This is a message",
    })));
    let model = LogsApiModel::try_from(vec![event]).expect("Failed mapping logs into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "logs": [{
                "message": "This is a message",
                "attributes": {"tag_key": "tag_value"},
            }]
        }])
    );
}

#[test]
fn generates_log_api_model_with_dotted_fields() {
    let sub = value!({"four": 2});
    let event = Event::Log(LogEvent::from(value!({
        "one.two": 1,
        "three": sub,
    })));
    let model = LogsApiModel::try_from(vec![event]).expect("Failed mapping logs into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "logs": [{
                "message": "log from vector",
                "attributes": {
                    "one.two": 1,
                    "three": {"four": 2},
                },
            }]
        }])
    );
}

#[test]
fn generates_log_api_model_with_timestamp() {
    let stamp = Utc::now();
    let event = Event::Log(LogEvent::from(value!({
        "timestamp": stamp,
        "tag_key": "tag_value",
        "message": "This is a message",
    })));
    let model = LogsApiModel::try_from(vec![event]).expect("Failed mapping logs into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "logs": [{
                "message": "This is a message",
                "timestamp": stamp.timestamp_millis(),
                "attributes": {"tag_key": "tag_value"},
            }]
        }])
    );
}

#[test]
fn generates_metric_api_model_without_timestamp() {
    let event = Event::Metric(Metric::new(
        "my_metric",
        MetricKind::Absolute,
        MetricValue::Counter { value: 100.0 },
    ));
    let model =
        MetricsApiModel::try_from(vec![event]).expect("Failed mapping metrics into API model");
    let metrics = &model.0[0].metrics;

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "metrics": [{
                "name": "my_metric",
                "value": 100.0,
                "timestamp": metrics[0].timestamp,
                "type": "gauge",
            }]
        }])
    );
}

#[test]
fn generates_metric_api_model_with_timestamp() {
    let stamp = Utc::now();
    let m = Metric::new(
        "my_metric",
        MetricKind::Absolute,
        MetricValue::Counter { value: 100.0 },
    )
    .with_timestamp(Some(stamp));
    let event = Event::Metric(m);
    let model =
        MetricsApiModel::try_from(vec![event]).expect("Failed mapping metrics into API model");

    assert_eq!(
        to_value(model).unwrap(),
        json!([{
            "metrics": [{
                "name": "my_metric",
                "value": 100.0,
                "timestamp": stamp.timestamp_millis(),
                "type": "gauge",
            }]
        }])
    );
}

#[test]
fn generates_metric_api_model_incremental_counter() {
    let stamp = Utc::now();
    let m = Metric::new(
        "my_metric",
        MetricKind::Incremental,
        MetricValue::Counter { value: 100.0 },
    )
    .with_timestamp(Some(stamp))
    .with_interval_ms(NonZeroU32::new(1000));
    let event = Event::Metric(m);
    let model =
        MetricsApiModel::try_from(vec![event]).expect("Failed mapping metrics into API model");

    assert_eq!(
        to_value(model).unwrap(),
        json!([{
            "metrics": [{
                "name": "my_metric",
                "value": 100.0,
                "interval.ms": 1000,
                "timestamp": stamp.timestamp_millis(),
                "type": "count",
            }]
        }])
    );
}

/// A full span envelope matching the ingestion-service sample, used by the happy-path model test
/// and the trace component-spec compliance test.
fn sample_trace_event() -> Event {
    Event::Log(LogEvent::from(value!({
        "resource": {
            "attributes": {
                "service.name": "webapp",
                "host.name": "ldw-5dbcfb759c-jrwmp",
                "host.arch": "amd64",
                "process.pid": 1,
                "process.command_args": ["/usr/local/bin/node", "/opt/app/apps/api/src/webapp.js"],
                "service.version": "16.1.33",
            }
        },
        "scope": {
            "name": "@opentelemetry/instrumentation-express",
            "version": "0.57.0",
            "attributes": {},
        },
        "type": "trace",
        "record": {
            "traceId": "30d23cf53e9f28f2e22f03d8e62e9a75",
            "spanId": "1a5d223c717ed91a",
            "parentSpanId": "3d6e9066774f0aed",
            "name": "middleware - i18nextMiddleware",
            "kind": 1,
            "startTimeUnixNano": "1783696528282000000",
            "endTimeUnixNano": "1783696528282252457",
            "attributes": {
                "express.name": "i18nextMiddleware",
                "express.type": "middleware",
            },
            "status": { "code": 0 },
            "droppedAttributesCount": 0,
            "droppedEventsCount": 0,
            "droppedLinksCount": 0,
        },
    })))
}

fn trace_attributes(model: &TracesApiModel) -> serde_json::Value {
    to_value(&model.0[0].spans[0].attributes).unwrap()
}

#[test]
fn generates_trace_api_model_happy_path() {
    let model = TracesApiModel::try_from(vec![sample_trace_event()])
        .expect("Failed mapping trace into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "spans": [{
                "trace.id": "30d23cf53e9f28f2e22f03d8e62e9a75",
                "id": "1a5d223c717ed91a",
                "timestamp": 1_783_696_528_282i64,
                "attributes": {
                    "duration.ms": 0.252457,
                    "name": "middleware - i18nextMiddleware",
                    "parent.id": "3d6e9066774f0aed",
                    "service.name": "webapp",
                    "span.kind": "internal",
                    "otel.status_code": "UNSET",
                    "express.name": "i18nextMiddleware",
                    "express.type": "middleware",
                    "host.name": "ldw-5dbcfb759c-jrwmp",
                    "host.arch": "amd64",
                    "process.pid": 1,
                    "process.command_args": "[\"/usr/local/bin/node\",\"/opt/app/apps/api/src/webapp.js\"]",
                    "service.version": "16.1.33",
                    "otel.scope.name": "@opentelemetry/instrumentation-express",
                    "otel.scope.version": "0.57.0",
                }
            }]
        }])
    );
}

#[test]
fn generates_trace_api_model_from_message_wrapped_envelope() {
    // Real Mezmo pipeline shape: the span envelope is nested under `message`, alongside unrelated
    // root-level fields (timestamp/metadata/annotations) that must be ignored.
    let event = Event::Log(LogEvent::from(value!({
        "timestamp": "2026-07-15T14:43:37.709+00:00",
        "metadata": {"headers": {}, "query": {}},
        "annotations": {"app": "pipeline-service"},
        "message": {
            "resource": {"attributes": {
                "service.name": "pipeline-service",
                "host.name": "ip-10-30-30-239.ec2.internal",
            }},
            "scope": {
                "name": "@opentelemetry/instrumentation-pg",
                "version": "0.71.0",
                "attributes": {},
            },
            "type": "trace",
            "record": {
                "traceId": "9c5d5196ae3067ceefc8f125b9108cc3",
                "spanId": "0ae6f4b8542655cf",
                "parentSpanId": "822e8b7fd5656f48",
                "name": "pg-pool.connect",
                "kind": 3,
                "startTimeUnixNano": "1784126617709000000",
                "endTimeUnixNano": "1784126617709064969",
                "attributes": {"db.system": "postgresql", "net.peer.port": 5432},
                "status": {"code": 0},
            },
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    assert_eq!(
        to_value(&model).unwrap(),
        json!([{
            "spans": [{
                "trace.id": "9c5d5196ae3067ceefc8f125b9108cc3",
                "id": "0ae6f4b8542655cf",
                "timestamp": 1_784_126_617_709i64,
                "attributes": {
                    "duration.ms": 0.064969,
                    "name": "pg-pool.connect",
                    "parent.id": "822e8b7fd5656f48",
                    "service.name": "pipeline-service",
                    "span.kind": "client",
                    "otel.status_code": "UNSET",
                    "host.name": "ip-10-30-30-239.ec2.internal",
                    "db.system": "postgresql",
                    "net.peer.port": 5432,
                    "otel.scope.name": "@opentelemetry/instrumentation-pg",
                    "otel.scope.version": "0.71.0",
                }
            }]
        }])
    );
}

#[test]
fn generates_trace_api_model_timestamp_and_duration() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5002500000",
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    let span = &model.0[0].spans[0];
    assert_eq!(span.timestamp, Some(5000));
    assert_eq!(trace_attributes(&model)["duration.ms"], json!(2.5));
}

#[test]
fn trace_api_model_omits_parent_id_for_root_span() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    assert!(trace_attributes(&model).get("parent.id").is_none());
}

#[test]
fn trace_api_model_clamps_negative_duration() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "4000000000",
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    assert_eq!(trace_attributes(&model)["duration.ms"], json!(0.0));
}

#[test]
fn trace_api_model_drops_span_missing_ids() {
    let missing_trace_id = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
        },
    })));
    assert!(TracesApiModel::try_from(vec![missing_trace_id]).is_err());

    let missing_span_id = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "startTimeUnixNano": "5000000000",
        },
    })));
    assert!(TracesApiModel::try_from(vec![missing_span_id]).is_err());
}

#[test]
fn trace_api_model_drops_span_missing_or_zero_start() {
    let zero_start = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "0",
        },
    })));
    assert!(TracesApiModel::try_from(vec![zero_start]).is_err());

    let absent_start = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
        },
    })));
    assert!(TracesApiModel::try_from(vec![absent_start]).is_err());
}

#[test]
fn trace_api_model_maps_error_status() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
            "status": { "code": 2, "message": "boom" },
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    let attributes = trace_attributes(&model);
    assert_eq!(attributes["otel.status_code"], json!("ERROR"));
    assert_eq!(attributes["otel.status_description"], json!("boom"));
    assert_eq!(attributes["error"], json!(true));
}

#[test]
fn trace_api_model_maps_span_kind() {
    let server = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
            "kind": 2,
        },
    })));
    let model =
        TracesApiModel::try_from(vec![server]).expect("Failed mapping trace into API model");
    assert_eq!(trace_attributes(&model)["span.kind"], json!("server"));

    let unspecified = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
            "kind": 0,
        },
    })));
    let model =
        TracesApiModel::try_from(vec![unspecified]).expect("Failed mapping trace into API model");
    assert!(trace_attributes(&model).get("span.kind").is_none());
}

#[test]
fn trace_api_model_stringifies_events_and_links() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
            "events": [{ "name": "exception", "timeUnixNano": "5000000001" }],
            "links": [{ "traceId": "cccc", "spanId": "dddd" }],
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    let attributes = trace_attributes(&model);
    assert_eq!(
        attributes["otel.span.events"],
        json!("[{\"name\":\"exception\",\"timeUnixNano\":\"5000000001\"}]")
    );
    assert_eq!(
        attributes["otel.span.links"],
        json!("[{\"spanId\":\"dddd\",\"traceId\":\"cccc\"}]")
    );
}

#[test]
fn trace_api_model_strips_restricted_attributes() {
    let event = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "resource": {
            "attributes": {
                "entityGuid": "resource-guid",
                "kept": "resource-value",
            }
        },
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
            "attributes": {
                "guid": "x",
                "entity.guid": "y",
                "entity.name": "z",
                "entity.type": "w",
                "kept.span": "span-value",
            },
        },
    })));
    let model = TracesApiModel::try_from(vec![event]).expect("Failed mapping trace into API model");

    let attributes = trace_attributes(&model);
    for restricted in [
        "entityGuid",
        "guid",
        "entity.guid",
        "entity.name",
        "entity.type",
    ] {
        assert!(
            attributes.get(restricted).is_none(),
            "restricted attribute {restricted} should be stripped"
        );
    }
    assert_eq!(attributes["kept"], json!("resource-value"));
    assert_eq!(attributes["kept.span"], json!("span-value"));
}

#[test]
fn trace_api_model_combines_multiple_spans() {
    let first = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "1111",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
        },
    })));
    let second = Event::Log(LogEvent::from(value!({
        "type": "trace",
        "record": {
            "traceId": "aaaa",
            "spanId": "2222",
            "startTimeUnixNano": "5000000000",
            "endTimeUnixNano": "5000000000",
        },
    })));
    let model = TracesApiModel::try_from(vec![first, second])
        .expect("Failed mapping traces into API model");

    let value = to_value(&model).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["spans"].as_array().unwrap().len(), 2);
}

#[test]
fn trace_api_model_drops_non_trace_event() {
    let non_trace = Event::Log(LogEvent::from(value!({
        "type": "log",
        "record": {
            "traceId": "aaaa",
            "spanId": "bbbb",
            "startTimeUnixNano": "5000000000",
        },
    })));
    assert!(TracesApiModel::try_from(vec![non_trace]).is_err());
}

#[test]
fn trace_api_model_drops_non_log_event() {
    let metric = Event::Metric(Metric::new(
        "my_metric",
        MetricKind::Absolute,
        MetricValue::Counter { value: 100.0 },
    ));
    assert!(TracesApiModel::try_from(vec![metric]).is_err());
}

#[test]
fn trace_endpoints_resolve_by_region() {
    let base = NewRelicCredentials {
        license_key: "key".to_owned(),
        account_id: "123".to_owned(),
        api: NewRelicApi::Traces,
        region: NewRelicRegion::Us,
        override_uri: None,
    };
    assert_eq!(
        base.get_uri().to_string(),
        "https://trace-api.newrelic.com/trace/v1"
    );

    let eu = NewRelicCredentials {
        region: NewRelicRegion::Eu,
        ..base
    };
    assert_eq!(
        eu.get_uri().to_string(),
        "https://trace-api.eu.newrelic.com/trace/v1"
    );
}

async fn trace_sink() -> (VectorSink, Event) {
    let mock_endpoint = spawn_blackhole_http_server(always_200_response).await;

    let config = NewRelicConfig::generate_config().to_string();
    let mut config = NewRelicConfig::deserialize(
        toml::de::ValueDeserializer::parse(&config).expect("toml should deserialize"),
    )
    .expect("config should be valid");
    config.api = NewRelicApi::Traces;
    config.override_uri = Some(mock_endpoint);

    let context = SinkContext::default();
    let (sink, _healthcheck) = config.build(context).await.unwrap();

    (sink, sample_trace_event())
}

#[tokio::test]
async fn component_spec_compliance_traces() {
    let (sink, event) = trace_sink().await;
    run_and_assert_sink_compliance(sink, stream::once(ready(event)), &SINK_TAGS).await;
}
