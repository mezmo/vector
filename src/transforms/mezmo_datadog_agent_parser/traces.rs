use ordered_float::NotNan;
use vector_lib::event::TraceEvent;
use vrl::event_path;

use crate::config::log_schema;
use crate::event::{Event, MaybeAsLogMut, ObjectMap, Value};
use crate::internal_events::{
    MezmoDatadogAgentParserDroppedSpan, MezmoDatadogAgentParserInvalidItem,
};
use crate::transforms::mezmo_datadog_agent_parser::common::{parse_timestamp, TimestampUnit};

use super::common::get_message_object;
use super::{MezmoDatadogAgentParser, TransformDatadogEvent, TransformDatadogEventError};

pub(super) struct DatadogTraceEvent;

impl TransformDatadogEvent for DatadogTraceEvent {
    fn transform(
        mut event: Event,
        parser: &MezmoDatadogAgentParser,
    ) -> Result<Vec<Event>, TransformDatadogEventError> {
        let version = parser
            .get_payload_version(&event)
            .unwrap_or_else(|| "v2".to_string());

        let log_result = event
            .maybe_as_log_mut()
            .ok_or_else(|| "Event is not a log".to_string());

        let log = match log_result {
            Ok(log) => log,
            Err(msg) => return Err(TransformDatadogEventError::from(event, &msg)),
        };
        let message_obj = match get_message_object(log) {
            Ok(message_obj) => message_obj,
            Err(msg) => return Err(TransformDatadogEventError::from(event, &msg)),
        };

        let trace_events = match version.as_str() {
            "v1" => build_v0_traces(message_obj),
            "v2" => build_v1_traces(message_obj),
            _ => {
                return Err(TransformDatadogEventError::from(
                    event,
                    &format!("Unsupported payload version: {version}"),
                ))
            }
        };

        let trace_events = match trace_events {
            Ok(events) => events,
            Err(msg) => return Err(TransformDatadogEventError::from(event, &msg)),
        };

        parser
            .build_events_from_payloads(
                event,
                trace_events
                    .into_iter()
                    .map(|trace| {
                        let (fields, _metadata) = trace.into_parts();
                        (fields, None)
                    })
                    .collect(),
            )
            .map_err(|(msg, event)| TransformDatadogEventError::from(*event, &msg))
    }
}

/// Trace v0 shape
/// APITrace item + Transactions (spans) + Agent payload common fields
fn build_v0_traces(api_trace: &ObjectMap) -> Result<Vec<TraceEvent>, String> {
    let host_key_path = log_schema()
        .host_key_target_path()
        .expect("global host key path must be a valid");
    let source_type_path = log_schema()
        .source_type_key_target_path()
        .expect("global source type key path must be valid");

    let transactions = api_trace.get("transactions").and_then(Value::as_array);
    let api_trace_present = api_trace.get("traceID").is_some()
        || api_trace.get("startTime").is_some()
        || api_trace.get("endTime").is_some()
        || api_trace.get("spans").is_some();

    let capacity = transactions.map_or(0, |t| t.len()) + usize::from(api_trace_present);
    if capacity == 0 {
        return Ok(Vec::new());
    }
    let env = api_trace
        .get("env")
        .filter(|v| v.as_str().is_some())
        .cloned();
    let host_name = api_trace
        .get("hostName")
        .filter(|v| v.as_str().is_some())
        .cloned();
    let source_type = Value::from("datadog_agent");
    let payload_version = Value::from("v1");

    let transaction_spans = transactions
        .into_iter()
        .flat_map(|transactions| filter_objects(transactions, "transaction"))
        .filter_map(|span| {
            transform_span(span).map(|span_value| {
                let mut event = TraceEvent::default();
                event.insert(source_type_path, source_type.clone());
                event.insert(event_path!("payload_version"), payload_version.clone());
                if let Some(env) = &env {
                    event.insert(event_path!("env"), env.clone());
                }
                if let Some(host_name) = &host_name {
                    event.insert(host_key_path, host_name.clone());
                }
                event.insert(event_path!("dropped"), Value::Boolean(true));
                event.insert(event_path!("spans"), vec![span_value]);
                event
            })
        });

    let trace_event = api_trace_present.then(|| {
        let mut event = TraceEvent::default();
        event.insert(source_type_path, source_type.clone());
        event.insert(event_path!("payload_version"), payload_version.clone());
        if let Some(env) = &env {
            event.insert(event_path!("env"), env.clone());
        }
        if let Some(host_name) = &host_name {
            event.insert(host_key_path, host_name.clone());
        }

        if let Some(trace_id) = api_trace.get("traceID").and_then(value_to_i64) {
            event.insert(event_path!("trace_id"), trace_id);
        }
        if let Some(start_time) = api_trace
            .get("startTime")
            .and_then(|t| parse_timestamp(t, TimestampUnit::Nanoseconds))
        {
            event.insert(event_path!("start_time"), start_time);
        }
        if let Some(end_time) = api_trace
            .get("endTime")
            .and_then(|t| parse_timestamp(t, TimestampUnit::Nanoseconds))
        {
            event.insert(event_path!("end_time"), end_time);
        }

        if let Some(spans) = api_trace.get("spans").and_then(Value::as_array) {
            let trace_spans: Vec<Value> = filter_objects(spans, "span")
                .filter_map(transform_span)
                .collect();
            if !trace_spans.is_empty() {
                event.insert(event_path!("spans"), trace_spans);
            }
        }

        event
    });

    let mut traces = Vec::with_capacity(capacity);
    traces.extend(transaction_spans.chain(trace_event));
    Ok(traces)
}

/// TracerPayload: https://github.com/DataDog/datadog-agent/blob/main/pkg/proto/datadog/trace/tracer_payload.proto
/// In Kong we unroll the tracerPayloads into a list of items. The agent payload version fields are
/// added to each TracerPayload item.
/// See: https://github.com/answerbook/pipeline-gateway-kong/blob/d7b0a8de56782c625971e250de8617ee8f50c893/kong/plugins/pipeline-routing/lib/datadog.lua#L83
fn build_v1_traces(tracer_payload: &ObjectMap) -> Result<Vec<TraceEvent>, String> {
    let host_key_path = log_schema()
        .host_key_target_path()
        .expect("global host key path must be a valid");
    let source_type_path = log_schema()
        .source_type_key_target_path()
        .expect("global source type key path must be valid");
    let payload_tags = tracer_payload.get("tags").and_then(Value::as_object);

    let chunks = match tracer_payload.get("chunks").and_then(Value::as_array) {
        Some(chunks) => chunks,
        None => return Ok(Vec::new()),
    };
    let env = tracer_payload
        .get("env")
        .filter(|v| v.as_str().is_some())
        .cloned();
    let host_name = tracer_payload
        .get("hostName")
        .filter(|v| v.as_str().is_some())
        .cloned();

    let source_type = Value::from("datadog_agent");
    let payload_version = Value::from("v2");

    let mut traces = Vec::with_capacity(chunks.len());
    traces.extend(filter_objects(chunks, "chunk").map(|chunk_object| {
        let mut event = transform_chunk(chunk_object, payload_tags);

        event.insert(source_type_path, source_type.clone());
        event.insert(event_path!("payload_version"), payload_version.clone());

        if let Some(val) = &env {
            event.insert(event_path!("env"), val.clone());
        }
        if let Some(val) = &host_name {
            event.insert(host_key_path, val.clone());
        }
        // TracerPayload fields: https://github.com/DataDog/datadog-agent/blob/9e536fdac2790c9475d2209e6d044b7538f8998c/pkg/proto/datadog/trace/tracer_payload.proto#L29
        if let Some(val) = tracer_payload
            .get("containerID")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("container_id"), val.clone());
        }
        if let Some(val) = tracer_payload
            .get("languageName")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("language_name"), val.clone());
        }
        if let Some(val) = tracer_payload
            .get("languageVersion")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("language_version"), val.clone());
        }
        if let Some(val) = tracer_payload
            .get("tracerVersion")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("tracer_version"), val.clone());
        }
        if let Some(val) = tracer_payload
            .get("runtimeID")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("runtime_id"), val.clone());
        }
        if let Some(val) = tracer_payload
            .get("appVersion")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("app_version"), val.clone());
        }
        // AgentPayload fields: https://github.com/DataDog/datadog-agent/blob/9e536fdac2790c9475d2209e6d044b7538f8998c/pkg/proto/datadog/trace/agent_payload.proto#L12
        if let Some(val) = tracer_payload
            .get("agentVersion")
            .filter(|v| v.as_str().is_some())
        {
            event.insert(event_path!("agent_version"), val.clone());
        }
        if let Some(val) = tracer_payload.get("targetTPS").and_then(value_to_f64) {
            if let Ok(target_tps) = NotNan::new(val) {
                event.insert(event_path!("target_tps"), Value::Float(target_tps));
            }
        }
        if let Some(val) = tracer_payload.get("errorTPS").and_then(value_to_f64) {
            if let Ok(error_tps) = NotNan::new(val) {
                event.insert(event_path!("error_tps"), Value::Float(error_tps));
            }
        }
        if let Some(sampler_enabled) = tracer_payload
            .get("rareSamplerEnabled")
            .and_then(Value::as_boolean)
        {
            event.insert(
                event_path!("rare_sampler_enabled"),
                Value::Boolean(sampler_enabled),
            );
        }

        event
    }));

    Ok(traces)
}

fn transform_chunk(chunk: &ObjectMap, payload_tags: Option<&ObjectMap>) -> TraceEvent {
    let mut event = TraceEvent::default();

    if let Some(priority) = chunk.get("priority").and_then(value_to_i64) {
        event.insert(event_path!("priority"), priority);
    }
    if let Some(origin) = chunk.get("origin").and_then(Value::as_str) {
        event.insert(event_path!("origin"), origin);
    }
    if let Some(dropped) = chunk.get("droppedTrace").and_then(Value::as_boolean) {
        event.insert(event_path!("dropped"), dropped);
    }
    // Payload tags consists of the TracerPayload and AgentPayload tags.
    // The data is combined in Kong gateway
    let mut tags = ObjectMap::new();
    if let Some(payload_tags) = payload_tags {
        tags.extend(payload_tags.clone());
    }
    if let Some(chunk_tags) = chunk.get("tags").and_then(Value::as_object) {
        tags.extend(chunk_tags.clone());
    }
    event.insert(event_path!("tags"), Value::Object(tags));

    let spans: Vec<Value> = chunk
        .get("spans")
        .and_then(Value::as_array)
        .map(|span_values| {
            filter_objects(span_values, "span")
                .filter_map(transform_span)
                .collect()
        })
        .unwrap_or_default();

    event.insert(event_path!("spans"), spans);
    event
}

fn transform_span(input_span: &ObjectMap) -> Option<Value> {
    let mut span = ObjectMap::new();

    for key in ["service", "name", "resource", "type"] {
        if let Some(val) = input_span.get(key).filter(|v| v.as_str().is_some()) {
            span.insert(key.into(), val.clone());
        }
    }

    let trace_id = input_span.get("traceID").and_then(value_to_i64);
    let span_id = input_span.get("spanID").and_then(value_to_i64);

    if trace_id.is_none() || span_id.is_none() {
        let missing_fields: Vec<&str> = [
            ("traceID", trace_id.is_none()),
            ("spanID", span_id.is_none()),
        ]
        .into_iter()
        .filter_map(|(field, missing)| missing.then_some(field))
        .collect();
        let missing_fields = missing_fields.join(",");
        emit!(MezmoDatadogAgentParserDroppedSpan {
            missing_fields: &missing_fields,
        });
        return None;
    }

    span.insert(
        "trace_id".into(),
        trace_id.expect("must have traceID for span").into(),
    );
    span.insert(
        "span_id".into(),
        span_id.expect("must have spanID for span").into(),
    );
    if let Some(val) = input_span.get("parentID").and_then(value_to_i64) {
        span.insert("parent_id".into(), val.into());
    }

    if let Some(ts) = input_span
        .get("start")
        .and_then(|v| parse_timestamp(v, TimestampUnit::Nanoseconds))
    {
        span.insert("start".into(), ts.into());
    }

    for (src_key, dst_key) in [("duration", "duration"), ("error", "error")] {
        if let Some(v) = input_span.get(src_key).and_then(value_to_i64) {
            span.insert(dst_key.into(), v.into());
        }
    }

    if let Some(meta) = input_span.get("meta").and_then(Value::as_object) {
        copy_object_map(meta, &mut span, "meta", |meta_value| {
            meta_value.as_str().map(|_| meta_value.clone())
        });
    }
    if let Some(metrics) = input_span.get("metrics").and_then(Value::as_object) {
        copy_object_map(metrics, &mut span, "metrics", |metric_value| {
            value_to_f64(metric_value)
                .and_then(|val| NotNan::new(val).ok())
                .map(Value::Float)
        });
    }
    if let Some(meta_struct) = input_span.get("metaStruct").and_then(Value::as_object) {
        copy_object_map(meta_struct, &mut span, "meta_struct", |val| {
            val.as_bytes().map(|_| val.clone())
        });
    }

    Some(Value::Object(span))
}

/// Datadog agent sends numeric integer values as u64
/// The Value enum has no type for u64. If an incoming numeric field is larger than i64,
/// it is converted into Value::Float.
/// Float values are rejected by Datadog where u64 is expected
/// The downcast from float to i64 here is deliberate; Datadog accepts the negative
/// values but won't accept floats. The traces sink silently drops events if a numeric
/// field is a float instead of i64
/// See open vector issue: https://github.com/vectordotdev/vector/issues/14687
fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(value) => Some(*value),
        Value::Float(value) => Some(value.into_inner() as i64),
        _ => value
            .as_str()
            .and_then(|value| value.parse::<i64>().ok())
            .or_else(|| {
                value
                    .as_str()
                    .and_then(|value| value.parse::<f64>().ok())
                    .map(|value| value as i64)
            }),
    }
}

fn filter_objects<'a>(
    items: &'a [Value],
    item_type: &'static str,
) -> impl Iterator<Item = &'a ObjectMap> {
    items.iter().filter_map(move |item| {
        item.as_object().or_else(|| {
            emit!(MezmoDatadogAgentParserInvalidItem {
                error: &format!("{item_type} is not an object"),
                item_type,
                event_type: Some("trace"),
            });
            None
        })
    })
}

fn copy_object_map<F>(src: &ObjectMap, dst: &mut ObjectMap, dst_key: &str, mut convert: F)
where
    F: FnMut(&Value) -> Option<Value>,
{
    if src.is_empty() {
        return;
    }

    let mapped: ObjectMap = src
        .iter()
        .filter_map(|(key, value)| convert(value).map(|mapped| (key.clone(), mapped)))
        .collect();

    if !mapped.is_empty() {
        dst.insert(dst_key.into(), Value::Object(mapped));
    }
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Float(value) => Some(value.into_inner()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::TransformDatadogEvent;
    use super::*;
    use crate::event::LogEvent;
    use crate::transforms::mezmo_datadog_agent_parser::{
        MezmoDatadogAgentParser, MezmoDatadogAgentParserConfig,
    };
    use bytes::Bytes;
    use chrono::{TimeZone, Utc};

    fn build_event(message: Value) -> Event {
        let mut log = LogEvent::default();
        log.insert(log_schema().message_key_target_path().unwrap(), message);
        Event::Log(log)
    }

    #[test]
    fn test_transform_v1_trace() {
        let event = build_event(
            serde_json::json!({
                "hostName": "tracer-host",
                "env": "production",
                "mezmo_payload_version": "v1",
                "traceID": "8759615994146109196",
                "startTime": 1_000_000_000i64,
                "endTime": 2_000_000_000i64,
                "spans": [
                    {
                        "service": "web",
                        "name": "http.request",
                        "resource": "/api/users",
                        "traceID": "8759615994146109196",
                        "spanID": "1309967388576301557",
                        "parentID": "0",
                        "start": 1_000_000_000i64,
                        "duration": 500_000_000i64,
                        "error": 0,
                        "meta": {"http.method": "GET"},
                        "metrics": {"_sample_rate": 1.0}
                    }
                ],
                "transactions": [
                    {
                        "service": "txn-service",
                        "name": "txn-name",
                        "traceID": 99999,
                        "spanID": 100,
                        "start": 3_000_000_000i64,
                        "duration": 200_000_000i64,
                        "error": 1
                    }
                ],
            })
            .into(),
        );

        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config, None);

        let results = DatadogTraceEvent::transform(event, &parser).unwrap();
        assert_eq!(
            results.len(),
            2,
            "should have transaction trace and api trace"
        );

        let host_key_path = log_schema().host_key_target_path().unwrap();
        let message_path = log_schema().message_key_target_path().unwrap();

        // First trace is from transactions (dropped=true)
        let transaction_log = results[0].as_log();
        let transaction_message = transaction_log
            .get(message_path)
            .and_then(Value::as_object)
            .expect("message object");
        let transaction_trace = TraceEvent::from(transaction_message.clone());

        assert_eq!(
            transaction_trace.get(event_path!("dropped")),
            Some(&Value::Boolean(true))
        );
        assert_eq!(
            transaction_trace
                .get(event_path!("payload_version"))
                .and_then(|v| v.as_str())
                .as_deref(),
            Some("v1")
        );
        assert_eq!(
            transaction_trace
                .get(host_key_path)
                .and_then(|v| v.as_str())
                .as_deref(),
            Some("tracer-host")
        );
        assert_eq!(
            transaction_trace
                .get(event_path!("env"))
                .and_then(|v| v.as_str())
                .as_deref(),
            Some("production")
        );

        let transaction_spans = transaction_trace
            .get(event_path!("spans"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(transaction_spans.len(), 1);
        let transaction_span = transaction_spans[0].as_object().unwrap();
        assert_eq!(
            transaction_span
                .get("service")
                .and_then(Value::as_str)
                .as_deref(),
            Some("txn-service")
        );
        assert_eq!(
            transaction_span
                .get("name")
                .and_then(Value::as_str)
                .as_deref(),
            Some("txn-name")
        );
        assert_eq!(
            transaction_span.get("trace_id").and_then(Value::as_integer),
            Some(99999)
        );
        assert_eq!(
            transaction_span.get("span_id").and_then(Value::as_integer),
            Some(100)
        );
        assert_eq!(
            transaction_span.get("error").and_then(Value::as_integer),
            Some(1)
        );

        // Second trace is from api trace fields
        let trace_log = results[1].as_log();
        let trace_message = trace_log
            .get(message_path)
            .and_then(Value::as_object)
            .expect("message object");
        let trace = TraceEvent::from(trace_message.clone());

        assert_eq!(
            trace
                .get(event_path!("payload_version"))
                .and_then(|v| v.as_str())
                .as_deref(),
            Some("v1")
        );
        assert_eq!(
            trace
                .get(host_key_path)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("tracer-host".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("env"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("production".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("trace_id"))
                .and_then(Value::as_integer),
            Some(8_759_615_994_146_109_196)
        );
        assert_eq!(
            trace.get(event_path!("start_time")),
            Some(&Value::Timestamp(Utc.timestamp_nanos(1_000_000_000)))
        );
        assert_eq!(
            trace.get(event_path!("end_time")),
            Some(&Value::Timestamp(Utc.timestamp_nanos(2_000_000_000)))
        );

        let spans = trace
            .get(event_path!("spans"))
            .and_then(|v| v.as_array())
            .unwrap();
        let span = spans[0].as_object().unwrap();
        assert_eq!(
            span.get("service").and_then(Value::as_str).as_deref(),
            Some("web")
        );
        assert_eq!(
            span.get("name").and_then(Value::as_str).as_deref(),
            Some("http.request")
        );
        assert_eq!(
            span.get("resource").and_then(Value::as_str).as_deref(),
            Some("/api/users")
        );
        assert_eq!(
            span.get("trace_id").and_then(Value::as_integer),
            Some(8_759_615_994_146_109_196)
        );
        assert_eq!(
            span.get("span_id").and_then(Value::as_integer),
            Some(1_309_967_388_576_301_557)
        );
        assert_eq!(span.get("parent_id").and_then(Value::as_integer), Some(0));
        assert_eq!(
            span.get("start"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(1_000_000_000)))
        );
        assert_eq!(
            span.get("duration").and_then(Value::as_integer),
            Some(500_000_000)
        );
        assert_eq!(span.get("error").and_then(Value::as_integer), Some(0));

        let meta = span.get("meta").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            meta.get("http.method").and_then(Value::as_str).as_deref(),
            Some("GET")
        );

        let metrics = span.get("metrics").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            metrics.get("_sample_rate"),
            Some(&Value::Float(NotNan::new(1.0).unwrap()))
        );
    }

    #[test]
    fn test_transform_v2_trace() {
        let event = build_event(
            serde_json::json!({
                "hostName": "myhost",
                "env": "staging",
                "mezmo_payload_version": "v2",
                "agentVersion": "7.0.0",
                "targetTPS": 10.0,
                "errorTPS": 1.0,
                "rareSamplerEnabled": true,
                "containerID": "abc123",
                "languageName": "python",
                "languageVersion": "3.9",
                "tracerVersion": "1.0.0",
                "runtimeID": "runtime-123",
                "appVersion": "2.0.0",
                "tags": {"payload_tag": "value"},
                "chunks": [
                    {
                        "priority": 1,
                        "origin": "lambda",
                        "droppedTrace": true,
                        "tags": {"chunk_tag": "value"},
                        "spans": [
                            {
                                "service": "api",
                                "name": "handler",
                                "resource": "process",
                                "traceID": 875_961_599_414i64,
                                "spanID": 1_309_967_388i64,
                                "parentID": "0",
                                "start": 2_000_000_000i64,
                                "duration": 100_000_000i64,
                                "error": 0,
                                "metrics": {"_sample_rate": 1},
                                "metaStruct": {"blob": "data"}
                            }
                        ]
                    }
                ]
            })
            .into(),
        );

        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config, None);

        let mut results = DatadogTraceEvent::transform(event, &parser).unwrap();
        let result = results.pop().expect("transformed event");
        let log = result.as_log();
        let message_path = log_schema().message_key_target_path().unwrap();
        let message = log
            .get(message_path)
            .and_then(Value::as_object)
            .expect("message object");
        let trace = TraceEvent::from(message.clone());
        let host_key_path = log_schema().host_key_target_path().unwrap();

        assert_eq!(
            trace
                .get(event_path!("payload_version"))
                .and_then(|v| v.as_str())
                .as_deref(),
            Some("v2")
        );
        assert_eq!(
            trace
                .get(event_path!("agent_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("7.0.0".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("target_tps"))
                .and_then(Value::as_float)
                .map(|value| value.into_inner()),
            Some(10.0)
        );
        assert_eq!(
            trace
                .get(event_path!("error_tps"))
                .and_then(Value::as_float)
                .map(|value| value.into_inner()),
            Some(1.0)
        );
        assert_eq!(
            trace.get(event_path!("rare_sampler_enabled")),
            Some(&Value::Boolean(true))
        );
        assert_eq!(
            trace
                .get(event_path!("container_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("abc123".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("language_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("python".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("language_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("3.9".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("tracer_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("1.0.0".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("runtime_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("runtime-123".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("app_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("2.0.0".to_string())
        );
        assert_eq!(
            trace
                .get(host_key_path)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("myhost".to_string())
        );
        assert_eq!(
            trace
                .get(event_path!("env"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("staging".to_string())
        );

        let spans = trace
            .get(event_path!("spans"))
            .and_then(|v| v.as_array())
            .unwrap();
        let span = spans[0].as_object().unwrap();
        let tags = trace
            .get(event_path!("tags"))
            .and_then(|v| v.as_object())
            .unwrap();

        assert_eq!(tags.len(), 2);
        assert_eq!(
            tags.get("chunk_tag").and_then(Value::as_str).as_deref(),
            Some("value")
        );
        assert_eq!(
            tags.get("payload_tag").and_then(Value::as_str).as_deref(),
            Some("value")
        );
        assert_eq!(
            trace
                .get(event_path!("priority"))
                .and_then(Value::as_integer),
            Some(1)
        );
        assert_eq!(
            trace
                .get(event_path!("origin"))
                .and_then(Value::as_str)
                .as_deref(),
            Some("lambda")
        );
        assert_eq!(
            trace.get(event_path!("dropped")),
            Some(&Value::Boolean(true))
        );
        assert_eq!(
            span.get("service").and_then(Value::as_str).as_deref(),
            Some("api")
        );
        assert_eq!(
            span.get("name").and_then(Value::as_str).as_deref(),
            Some("handler")
        );
        assert_eq!(
            span.get("resource").and_then(Value::as_str).as_deref(),
            Some("process")
        );
        assert_eq!(
            span.get("trace_id").and_then(Value::as_integer),
            Some(875_961_599_414)
        );
        assert_eq!(
            span.get("span_id").and_then(Value::as_integer),
            Some(1_309_967_388)
        );
        assert_eq!(span.get("parent_id").and_then(Value::as_integer), Some(0));
        assert_eq!(
            span.get("start"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(2_000_000_000)))
        );
        assert_eq!(
            span.get("duration").and_then(Value::as_integer),
            Some(100_000_000)
        );
        assert_eq!(span.get("error").and_then(Value::as_integer), Some(0));

        let metrics = span.get("metrics").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            metrics.get("_sample_rate"),
            Some(&Value::Float(NotNan::new(1.0).unwrap()))
        );

        let meta_struct = span.get("meta_struct").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            meta_struct.get("blob"),
            Some(&Value::Bytes(Bytes::from_static(b"data")))
        );
    }

    #[test]
    fn test_transform_span_with_missing_non_required_fields() {
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "service": "test-service",
            "traceID": 123,
            "spanID": 456
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();

        assert_eq!(
            span.get("service").and_then(Value::as_str).as_deref(),
            Some("test-service")
        );
        assert_eq!(span.get("trace_id").and_then(Value::as_integer), Some(123));
        assert_eq!(span.get("span_id").and_then(Value::as_integer), Some(456));
        assert!(span.get("name").is_none());
        assert!(span.get("resource").is_none());
        assert!(span.get("type").is_none());
        assert!(span.get("parent_id").is_none());
        assert!(span.get("start").is_none());
        assert!(span.get("duration").is_none());
        assert!(span.get("error").is_none());
        assert!(span.get("meta").is_none());
        assert!(span.get("metrics").is_none());
        assert!(span.get("meta_struct").is_none());
    }

    #[test]
    fn test_transform_span_meta_omits_non_string_values() {
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": 123,
            "spanID": 456,
            "meta": {
                "valid_string": "hello",
                "number": 123,
                "bool": true,
                "null": null,
                "array": [1, 2, 3],
                "object": {"nested": "value"}
            }
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();
        let meta = span.get("meta").and_then(Value::as_object).unwrap();

        assert_eq!(meta.len(), 1);
        assert_eq!(
            meta.get("valid_string").and_then(Value::as_str).as_deref(),
            Some("hello")
        );
        assert!(meta.get("number").is_none());
        assert!(meta.get("bool").is_none());
        assert!(meta.get("null").is_none());
        assert!(meta.get("array").is_none());
        assert!(meta.get("object").is_none());
    }

    #[test]
    fn test_transform_span_metrics_omits_non_numeric_values() {
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": 123,
            "spanID": 456,
            "metrics": {
                "int_value": 42,
                "float_value": 3.14,
                "string_value": "not a number",
                "bool_value": true,
                "null_value": null
            }
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();
        let metrics = span.get("metrics").and_then(Value::as_object).unwrap();

        assert_eq!(metrics.len(), 2);
        assert_eq!(
            metrics
                .get("int_value")
                .and_then(Value::as_float)
                .map(|f| f.into_inner()),
            Some(42.0)
        );
        assert_eq!(
            metrics
                .get("float_value")
                .and_then(Value::as_float)
                .map(|f| f.into_inner()),
            Some(3.14)
        );
        assert!(metrics.get("string_value").is_none());
        assert!(metrics.get("bool_value").is_none());
        assert!(metrics.get("null_value").is_none());
    }

    #[test]
    fn test_transform_span_meta_struct_omits_non_bytes_values() {
        // JSON strings become Value::Bytes, so they ARE included
        // Only non-bytes types (numbers, bools, arrays, objects) are omitted
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": 123,
            "spanID": 456,
            "metaStruct": {
                "bytes_value": "this becomes bytes",
                "number_value": 123,
                "bool_value": true,
                "array_value": [1, 2],
                "object_value": {"nested": "val"}
            }
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();
        let meta_struct = span.get("meta_struct").and_then(Value::as_object).unwrap();

        // Only bytes_value should be included (JSON strings become Bytes)
        assert_eq!(meta_struct.len(), 1);
        assert_eq!(
            meta_struct.get("bytes_value"),
            Some(&Value::Bytes(Bytes::from_static(b"this becomes bytes")))
        );
        assert!(meta_struct.get("number_value").is_none());
        assert!(meta_struct.get("bool_value").is_none());
        assert!(meta_struct.get("array_value").is_none());
        assert!(meta_struct.get("object_value").is_none());
    }

    #[test]
    fn test_transform_span_empty_objects_not_set() {
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": 123,
            "spanID": 456,
            "meta": {},
            "metrics": {},
            "metaStruct": {}
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();

        assert!(span.get("meta").is_none());
        assert!(span.get("metrics").is_none());
        assert!(span.get("meta_struct").is_none());
    }

    #[test]
    fn transform_span_drops_when_missing_required_fields() {
        let missing_trace_id: ObjectMap = serde_json::from_value(serde_json::json!({
            "spanID": 456
        }))
        .unwrap();
        let missing_span_id: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": 123
        }))
        .unwrap();

        assert!(transform_span(&missing_trace_id).is_none());
        assert!(transform_span(&missing_span_id).is_none());
    }

    #[test]
    fn transform_span_truncates_float_values() {
        let input: ObjectMap = serde_json::from_value(serde_json::json!({
            "traceID": "1.23e5",
            "spanID": 4.56e6,
            "parentID": "7.89e2",
            "start": 1.5e9,
            "duration": "1.11e3",
            "error": 0.0,
        }))
        .unwrap();

        let result = transform_span(&input).expect("span");
        let span = result.as_object().unwrap();

        assert_eq!(
            span.get("trace_id").and_then(Value::as_integer),
            Some(123_000)
        );
        assert_eq!(
            span.get("span_id").and_then(Value::as_integer),
            Some(4_560_000)
        );
        assert_eq!(span.get("parent_id").and_then(Value::as_integer), Some(789));
        assert_eq!(
            span.get("start"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(1_500_000_000)))
        );
        assert_eq!(
            span.get("duration").and_then(Value::as_integer),
            Some(1_110)
        );
        assert_eq!(span.get("error").and_then(Value::as_integer), Some(0));
    }
}
