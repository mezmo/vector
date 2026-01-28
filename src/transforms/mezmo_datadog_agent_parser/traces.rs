use ordered_float::NotNan;

use crate::config::log_schema;
use crate::event::{Event, MaybeAsLogMut, ObjectMap, Value};
use crate::internal_events::MezmoDatadogAgentParserInvalidItem;
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

        let result = match version.as_str() {
            "v1" => transform_trace_v1(message_obj),
            "v2" => transform_tracer_payload(message_obj),
            _ => {
                return Err(TransformDatadogEventError::from(
                    event,
                    &format!("Unsupported payload version: {version}"),
                ))
            }
        };
        let mut message = match result {
            Ok(message) => message,
            Err(msg) => return Err(TransformDatadogEventError::from(event, &msg)),
        };
        message.insert("payload_version".into(), Value::from(version));

        let message_path = match log_schema().message_key_target_path() {
            Some(path) => path,
            None => {
                return Err(TransformDatadogEventError::from(
                    event,
                    "Missing message key",
                ))
            }
        };
        log.insert(message_path, Value::Object(message));
        parser.strip_fields(&mut event);

        Ok(vec![event])
    }
}

fn transform_trace_v1(message: &ObjectMap) -> Result<ObjectMap, String> {
    if message.get("transactions").is_some() {
        return Ok(transform_transaction(message));
    }
    Ok(transform_api_trace(message))
}

/// APITrace: https://github.com/mezmo/vector/blob/f5010f6b3e3cda9f201429c3802cee94efb16586/proto/vector/dd_trace.proto#L20
/// Transforms an APITrace item
/// See Kong implementation: https://github.com/answerbook/pipeline-gateway-kong/blob/af22369e9c53d319aa808391f06eae5ac0e8d036/kong/plugins/pipeline-routing/lib/datadog.lua#L83
fn transform_api_trace(input: &ObjectMap) -> ObjectMap {
    let mut output = ObjectMap::new();

    copy_string(input, "hostName", &mut output, "host");
    copy_string(input, "env", &mut output, "env");
    copy_u64(input, "traceID", &mut output, "trace_id");
    copy_timestamp_nanos(input, "startTime", &mut output, "start_time");
    copy_timestamp_nanos(input, "endTime", &mut output, "end_time");

    if let Some(spans) = input.get("spans").and_then(Value::as_array) {
        let transformed: Vec<Value> = spans
            .iter()
            .filter_map(|span_val| {
                if let Some(obj) = span_val.as_object() {
                    Some(obj)
                } else {
                    emit!(MezmoDatadogAgentParserInvalidItem {
                        error: "Span is not an object",
                        item_type: "span",
                        event_type: Some("trace"),
                    });
                    None
                }
            })
            .map(|span| transform_span(span, None, None, None))
            .collect();
        if !transformed.is_empty() {
            output.insert("spans".into(), Value::Array(transformed));
        }
    }

    output
}

/// Transactions are basically spans: https://github.com/mezmo/vector/blob/f5010f6b3e3cda9f201429c3802cee94efb16586/proto/vector/dd_trace.proto#L11
/// Transforms an APITrace item
/// See Kong implementation: https://github.com/answerbook/pipeline-gateway-kong/blob/af22369e9c53d319aa808391f06eae5ac0e8d036/kong/plugins/pipeline-routing/lib/datadog.lua#L83
fn transform_transaction(input: &ObjectMap) -> ObjectMap {
    let mut output = ObjectMap::new();

    copy_string(input, "hostName", &mut output, "host");
    copy_string(input, "env", &mut output, "env");

    let spans: Vec<Value> = input
        .get("transactions")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|txn_val| {
                    if let Some(obj) = txn_val.as_object() {
                        Some(obj)
                    } else {
                        emit!(MezmoDatadogAgentParserInvalidItem {
                            error: "Transaction is not an object",
                            item_type: "transaction",
                            event_type: Some("trace"),
                        });
                        None
                    }
                })
                .map(|span| transform_span(span, Some(true), None, None))
                .collect()
        })
        .unwrap_or_default();

    output.insert("spans".into(), Value::Array(spans));
    output
}

/// TracerPayload: https://github.com/DataDog/datadog-agent/blob/main/pkg/proto/datadog/trace/tracer_payload.proto
fn transform_tracer_payload(input: &ObjectMap) -> Result<ObjectMap, String> {
    let mut output = ObjectMap::new();

    copy_string(input, "hostName", &mut output, "host");
    copy_string(input, "env", &mut output, "env");
    copy_string(input, "agentVersion", &mut output, "agent_version");
    copy_string(input, "containerID", &mut output, "container_id");
    copy_string(input, "languageName", &mut output, "language_name");
    copy_string(input, "languageVersion", &mut output, "language_version");
    copy_string(input, "tracerVersion", &mut output, "tracer_version");
    copy_string(input, "runtimeID", &mut output, "runtime_id");
    copy_string(input, "appVersion", &mut output, "app_version");
    copy_float(input, "targetTPS", &mut output, "target_tps");
    copy_float(input, "errorTPS", &mut output, "error_tps");
    copy_bool(
        input,
        "rareSamplerEnabled",
        &mut output,
        "rare_sampler_enabled",
    );

    let payload_tags = input.get("tags").and_then(Value::as_object);

    let spans: Vec<Value> = input
        .get("chunks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|chunk| {
            if let Some(obj) = chunk.as_object() {
                Some(obj)
            } else {
                emit!(MezmoDatadogAgentParserInvalidItem {
                    error: "Chunk is not an object",
                    item_type: "chunk",
                    event_type: Some("trace"),
                });
                None
            }
        })
        .flat_map(|chunk| process_chunk(chunk, payload_tags))
        .collect();

    output.insert("spans".into(), Value::Array(spans));
    Ok(output)
}

fn process_chunk(chunk: &ObjectMap, payload_tags: Option<&ObjectMap>) -> Vec<Value> {
    let priority = chunk.get("priority").and_then(Value::as_integer);
    let origin = chunk.get("origin").and_then(Value::as_str);
    let dropped = chunk.get("droppedTrace").and_then(Value::as_boolean);
    let chunk_tags = chunk.get("tags").and_then(Value::as_object);

    chunk
        .get("spans")
        .and_then(Value::as_array)
        .map(|spans| {
            spans
                .iter()
                .filter_map(|span_val| {
                    if let Some(obj) = span_val.as_object() {
                        Some(obj)
                    } else {
                        emit!(MezmoDatadogAgentParserInvalidItem {
                            error: "Span is not an object",
                            item_type: "span",
                            event_type: Some("trace"),
                        });
                        None
                    }
                })
                .map(|span| {
                    let mut transformed =
                        transform_span(span, dropped, priority, Some((chunk_tags, payload_tags)));

                    if let (Some(origin_str), Value::Object(ref mut obj)) =
                        (&origin, &mut transformed)
                    {
                        obj.insert(
                            "origin".into(),
                            Value::from(origin_str.clone().into_owned()),
                        );
                    }

                    transformed
                })
                .collect()
        })
        .unwrap_or_default()
}

fn transform_span(
    input: &ObjectMap,
    dropped: Option<bool>,
    priority: Option<i64>,
    extra_tags: Option<(Option<&ObjectMap>, Option<&ObjectMap>)>, // (chunk_tags, payload_tags)
) -> Value {
    let mut output = ObjectMap::new();

    copy_string(input, "service", &mut output, "service");
    copy_string(input, "name", &mut output, "name");
    copy_string(input, "resource", &mut output, "resource");
    copy_string(input, "type", &mut output, "type");

    copy_u64(input, "traceID", &mut output, "trace_id");
    copy_u64(input, "spanID", &mut output, "span_id");
    copy_u64(input, "parentID", &mut output, "parent_id");

    copy_timestamp_nanos(input, "start", &mut output, "start");
    copy_i64(input, "duration", &mut output, "duration");

    copy_i64(input, "error", &mut output, "error");

    copy_string_object(input, "meta", &mut output, "meta");
    copy_metrics(input, "metrics", &mut output, "metrics");
    copy_meta_struct(input, "metaStruct", &mut output, "meta_struct");

    // Simplified Tag Merging
    let mut tags = ObjectMap::new();

    if let Some((chunk_tags, payload_tags)) = extra_tags {
        if let Some(chunk_tags) = chunk_tags {
            merge_string_map(chunk_tags, &mut tags);
        }
        if let Some(payload_tags) = payload_tags {
            merge_string_map(payload_tags, &mut tags);
        }
    }
    if let Some(span_tags) = input.get("tags").and_then(Value::as_object) {
        merge_string_map(span_tags, &mut tags);
    }

    if !tags.is_empty() {
        output.insert("tags".into(), Value::Object(tags));
    }

    if let Some(d) = dropped {
        output.insert("dropped".into(), Value::Boolean(d));
    }
    if let Some(p) = priority {
        output.insert("priority".into(), Value::from(p));
    }

    Value::Object(output)
}

fn copy_string(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(val) = src.get(src_key) {
        if val.as_str().is_some() {
            dst.insert(dst_key.into(), val.clone());
        }
    }
}

fn copy_u64(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(v) = src.get(src_key).and_then(Value::as_integer) {
        if u64::try_from(v).is_ok() {
            dst.insert(dst_key.into(), Value::from(v));
        }
    }
}

fn copy_i64(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(v) = src.get(src_key).and_then(Value::as_integer) {
        dst.insert(dst_key.into(), Value::from(v));
    }
}

fn copy_timestamp_nanos(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    let src_timestamp = src
        .get(src_key)
        .and_then(|v| parse_timestamp(v, TimestampUnit::Nanoseconds));

    if let Some(timestamp) = src_timestamp {
        dst.insert(dst_key.into(), Value::from(timestamp));
    }
}

fn copy_float(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(f) = src.get(src_key).and_then(value_to_f64) {
        if let Ok(nn) = NotNan::new(f) {
            dst.insert(dst_key.into(), Value::Float(nn));
        }
    }
}

fn copy_bool(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(b) = src.get(src_key).and_then(Value::as_boolean) {
        dst.insert(dst_key.into(), Value::Boolean(b));
    }
}

fn copy_string_object(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(Value::Object(obj)) = src.get(src_key) {
        copy_object_map(obj, dst, dst_key, |value| {
            if value.as_str().is_some() {
                Some(value.clone())
            } else {
                None
            }
        });
    }
}

fn copy_metrics(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(Value::Object(obj)) = src.get(src_key) {
        copy_object_map(obj, dst, dst_key, |value| {
            Some(
                value
                    .as_integer()
                    .map(|v| v as f64)
                    .or_else(|| value.as_float().map(|v| v.into_inner()))
                    .and_then(|f| NotNan::new(f).ok())
                    .map(Value::Float)
                    .unwrap_or(Value::Null),
            )
        });
    }
}

fn copy_meta_struct(src: &ObjectMap, src_key: &str, dst: &mut ObjectMap, dst_key: &str) {
    if let Some(Value::Object(obj)) = src.get(src_key) {
        copy_object_map(obj, dst, dst_key, |value| match value {
            Value::Bytes(bytes) => Some(Value::Bytes(bytes.clone())),
            _ => Some(Value::Null),
        });
    }
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

fn merge_string_map(src: &ObjectMap, dst: &mut ObjectMap) {
    for (key, value) in src {
        if value.as_str().is_some() {
            dst.insert(key.clone(), value.clone());
        }
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
                "traceID": 12345,
                "startTime": 1_000_000_000i64,
                "endTime": 2_000_000_000i64,
                "spans": [
                    {
                        "service": "web",
                        "name": "http.request",
                        "resource": "/api/users",
                        "traceID": 12345,
                        "spanID": 1,
                        "parentID": 0,
                        "start": 1_000_000_000i64,
                        "duration": 500_000_000i64,
                        "error": 0,
                        "meta": {"http.method": "GET"},
                        "metrics": {"_sample_rate": 1.0}
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
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(
            message
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("tracer-host".to_string())
        );
        assert_eq!(
            message
                .get("env")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("production".to_string())
        );
        assert_eq!(
            message.get("trace_id").and_then(Value::as_integer),
            Some(12345)
        );
        assert_eq!(
            message.get("start_time"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(1_000_000_000)))
        );
        assert_eq!(
            message.get("end_time"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(2_000_000_000)))
        );

        let spans = message.get("spans").and_then(|v| v.as_array()).unwrap();
        let span = spans[0].as_object().unwrap();
        assert_eq!(
            span.get("start"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(1_000_000_000)))
        );
        assert_eq!(
            span.get("trace_id").and_then(Value::as_integer),
            Some(12345)
        );
        assert_eq!(span.get("span_id").and_then(Value::as_integer), Some(1));
        assert_eq!(span.get("parent_id").and_then(Value::as_integer), Some(0));
    }

    #[test]
    fn test_transform_v1_transactions() {
        let event = build_event(
            serde_json::json!({
                "hostName": "myhost",
                "env": "production",
                "mezmo_payload_version": "v1",
                "transactions": [
                    {
                        "service": "web",
                        "name": "txn",
                        "traceID": 12345,
                        "spanID": 1,
                        "start": 1_000_000_000i64,
                        "duration": 500_000_000i64,
                        "error": 0
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
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        let spans = message.get("spans").and_then(|v| v.as_array()).unwrap();
        let span = spans[0].as_object().unwrap();
        assert_eq!(span.get("dropped"), Some(&Value::Boolean(true)));
        assert_eq!(
            message
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("myhost".to_string())
        );
        assert_eq!(
            message
                .get("env")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("production".to_string())
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
                                "traceID": 67890,
                                "spanID": 2,
                                "parentID": 1,
                                "start": 2_000_000_000i64,
                                "duration": 100_000_000i64,
                                "error": 0,
                                "tags": {"span_tag": "value"},
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
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(
            message
                .get("agent_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("7.0.0".to_string())
        );
        assert_eq!(
            message
                .get("target_tps")
                .and_then(Value::as_float)
                .map(|value| value.into_inner()),
            Some(10.0)
        );
        assert_eq!(
            message
                .get("error_tps")
                .and_then(Value::as_float)
                .map(|value| value.into_inner()),
            Some(1.0)
        );
        assert_eq!(
            message.get("rare_sampler_enabled"),
            Some(&Value::Boolean(true))
        );
        assert_eq!(
            message
                .get("container_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("abc123".to_string())
        );
        assert_eq!(
            message
                .get("language_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("python".to_string())
        );
        assert_eq!(
            message
                .get("language_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("3.9".to_string())
        );
        assert_eq!(
            message
                .get("tracer_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("1.0.0".to_string())
        );
        assert_eq!(
            message
                .get("runtime_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("runtime-123".to_string())
        );
        assert_eq!(
            message
                .get("app_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("2.0.0".to_string())
        );
        assert_eq!(
            message
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("myhost".to_string())
        );
        assert_eq!(
            message
                .get("env")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("staging".to_string())
        );

        let spans = message.get("spans").and_then(|v| v.as_array()).unwrap();
        let span = spans[0].as_object().unwrap();
        let tags = span.get("tags").and_then(|v| v.as_object()).unwrap();

        assert_eq!(
            tags.get("span_tag").and_then(Value::as_str).as_deref(),
            Some("value")
        );
        assert_eq!(
            tags.get("chunk_tag").and_then(Value::as_str).as_deref(),
            Some("value")
        );
        assert_eq!(
            tags.get("payload_tag").and_then(Value::as_str).as_deref(),
            Some("value")
        );
        assert_eq!(span.get("priority").and_then(Value::as_integer), Some(1));
        assert_eq!(
            span.get("origin").and_then(Value::as_str).as_deref(),
            Some("lambda")
        );
        assert_eq!(span.get("dropped"), Some(&Value::Boolean(true)));
        assert_eq!(
            span.get("start"),
            Some(&Value::Timestamp(Utc.timestamp_nanos(2_000_000_000)))
        );

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
}
