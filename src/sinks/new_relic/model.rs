use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    fmt::Debug,
};

use chrono::Utc;
use ordered_float::NotNan;
use serde::Serialize;
use vector_lib::{
    config::log_schema,
    event::{KeyString, ObjectMap},
    internal_event::{ComponentEventsDropped, INTENTIONAL, UNINTENTIONAL},
};
use vrl::event_path;

use super::NewRelicSinkError;
use crate::event::{Event, LogEvent, MetricKind, MetricValue, Value};

#[derive(Debug)]
pub(super) enum NewRelicApiModel {
    Metrics(MetricsApiModel),
    Events(EventsApiModel),
    Logs(LogsApiModel),
    Traces(TracesApiModel),
}

/// The metrics API data model.
///
/// Reference: https://docs.newrelic.com/docs/data-apis/ingest-apis/metric-api/report-metrics-metric-api/
#[derive(Debug, Serialize)]
pub(super) struct MetricsApiModel(pub [MetricDataStore; 1]);

#[derive(Debug, Serialize)]
pub(super) struct MetricDataStore {
    pub metrics: Vec<MetricData>,
}

#[derive(Debug, Serialize)]
pub(super) struct MetricData {
    #[serde(rename = "interval.ms", skip_serializing_if = "Option::is_none")]
    pub interval_ms: Option<i64>,
    pub name: String,
    pub r#type: &'static str,
    pub value: f64,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<BTreeMap<String, String>>,
}

impl MetricsApiModel {
    pub(super) const fn new(metrics: Vec<MetricData>) -> Self {
        Self([MetricDataStore { metrics }])
    }
}

impl TryFrom<Vec<Event>> for MetricsApiModel {
    type Error = NewRelicSinkError;

    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut num_non_metric_events = 0;
        let mut num_missing_interval = 0;
        let mut num_nan_value = 0;
        let mut num_unsupported_metric_type = 0;

        let metric_array: Vec<_> = buf_events
            .into_iter()
            .filter_map(|event| {
                let Some(metric) = event.try_into_metric() else {
                    num_non_metric_events += 1;
                    return None;
                };

                // Generate Value::Object() from BTreeMap<String, String>
                let (series, data, _) = metric.into_parts();

                // We only handle gauge and counter metrics
                // Extract value & type and set type-related attributes
                let (value, metric_type, interval_ms) = match (data.value, &data.kind) {
                    (MetricValue::Counter { value }, MetricKind::Incremental) => {
                        let Some(interval_ms) = data.time.interval_ms else {
                            // Incremental counter without an interval is worthless, skip this metric
                            num_missing_interval += 1;
                            return None;
                        };
                        (value, "count", Some(interval_ms.get() as i64))
                    }
                    (MetricValue::Counter { value }, MetricKind::Absolute)
                    | (MetricValue::Gauge { value }, _) => (value, "gauge", None),
                    _ => {
                        // Unsupported metric type
                        num_unsupported_metric_type += 1;
                        return None;
                    }
                };

                // Set name, type, value, timestamp, and attributes
                if value.is_nan() {
                    num_nan_value += 1;
                    return None;
                };

                let timestamp = data.time.timestamp.unwrap_or_else(Utc::now);
                Some(MetricData {
                    interval_ms,
                    name: series.name.name,
                    r#type: metric_type,
                    value,
                    timestamp: timestamp.timestamp_millis(),
                    attributes: series.tags.map(|tags| tags.into_iter_single().collect()),
                })
            })
            .collect();

        if num_non_metric_events > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_metric_events,
                reason: "non-metric event"
            });
        }
        if num_unsupported_metric_type > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_unsupported_metric_type,
                reason: "unsupported metric type"
            });
        }
        if num_nan_value > 0 {
            emit!(ComponentEventsDropped::<UNINTENTIONAL> {
                count: num_nan_value,
                reason: "NaN value not supported"
            });
        }
        if num_missing_interval > 0 {
            emit!(ComponentEventsDropped::<UNINTENTIONAL> {
                count: num_missing_interval,
                reason: "incremental counter missing interval"
            });
        }

        if !metric_array.is_empty() {
            Ok(Self::new(metric_array))
        } else {
            Err(NewRelicSinkError::new("No valid metrics to generate"))
        }
    }
}

/// The events API data mode.
///
/// Reference: https://docs.newrelic.com/docs/data-apis/ingest-apis/event-api/introduction-event-api/
#[derive(Debug, Serialize)]
pub(super) struct EventsApiModel(pub Vec<ObjectMap>);

impl EventsApiModel {
    pub(super) const fn new(events_array: Vec<ObjectMap>) -> Self {
        Self(events_array)
    }
}

impl TryFrom<Vec<Event>> for EventsApiModel {
    type Error = NewRelicSinkError;

    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut num_non_log_events = 0;
        let mut num_nan_value = 0;

        let events_array: Vec<ObjectMap> = buf_events
            .into_iter()
            .filter_map(|event| {
                let Some(log) = event.try_into_log() else {
                    num_non_log_events += 1;
                    return None;
                };

                let mut event_model = ObjectMap::new();
                for (k, v) in log.convert_to_fields_unquoted() {
                    event_model.insert(k, v.clone());
                }

                if let Some(message) = log.get(event_path!("message")) {
                    let message = message.to_string_lossy().replace("\\\"", "\"");
                    // If message contains a JSON string, parse it and insert all fields into self
                    if let serde_json::Result::Ok(json_map) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(&message)
                    {
                        for (k, v) in json_map {
                            match v {
                                serde_json::Value::String(s) => {
                                    event_model.insert(k.into(), Value::from(s));
                                }
                                serde_json::Value::Number(n) => {
                                    if let Some(f) = n.as_f64() {
                                        event_model.insert(
                                            k.into(),
                                            Value::from(NotNan::new(f).ok().or_else(|| {
                                                num_nan_value += 1;
                                                None
                                            })?),
                                        );
                                    } else {
                                        event_model.insert(k.into(), Value::from(n.as_i64()));
                                    }
                                }
                                serde_json::Value::Bool(b) => {
                                    event_model.insert(k.into(), Value::from(b));
                                }
                                _ => {
                                    // Note that arrays and nested objects are silently dropped.
                                }
                            }
                        }
                        event_model.remove("message");
                    }
                }

                if !event_model.contains_key("eventType") {
                    event_model.insert("eventType".into(), Value::from("VectorSink".to_owned()));
                }

                Some(event_model)
            })
            .collect();

        if num_non_log_events > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_log_events,
                reason: "non-log event"
            });
        }
        if num_nan_value > 0 {
            emit!(ComponentEventsDropped::<UNINTENTIONAL> {
                count: num_nan_value,
                reason: "NaN value not supported"
            });
        }

        if !events_array.is_empty() {
            Ok(Self::new(events_array))
        } else {
            Err(NewRelicSinkError::new("No valid events to generate"))
        }
    }
}

/// The logs API data model.
///
/// Reference: https://docs.newrelic.com/docs/logs/log-api/introduction-log-api/
#[derive(Serialize, Debug)]
pub(super) struct LogsApiModel(pub [LogDataStore; 1]);

#[derive(Serialize, Debug)]
pub(super) struct LogDataStore {
    pub logs: Vec<LogMessage>,
}

#[derive(Debug, PartialEq, Serialize)]
pub(super) struct LogMessage {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<Timestamp>,
    pub attributes: ObjectMap,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub(super) enum Timestamp {
    Numeric(i64),
    String(String),
}

impl LogsApiModel {
    pub(super) const fn new(logs: Vec<LogMessage>) -> Self {
        Self([LogDataStore { logs }])
    }
}

impl TryFrom<Vec<Event>> for LogsApiModel {
    type Error = NewRelicSinkError;

    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut num_non_log_events = 0;
        let mut num_non_object_events = 0;
        let message_key = log_schema().message_key_target_path().unwrap();
        let timestamp_key = log_schema().timestamp_key_target_path().unwrap();

        let logs_array: Vec<LogMessage> = buf_events
            .into_iter()
            .filter_map(|event| {
                let Some(mut log) = event.try_into_log() else {
                    num_non_log_events += 1;
                    return None;
                };

                let message = get_message_string(log.remove(message_key));
                let timestamp = log.remove(timestamp_key).and_then(map_timestamp_value);

                // We convert the log event into a logs API model simply by transmuting the type
                // wrapper and dropping all arrays, which are not supported by the API. We could
                // flatten out the keys, as this is what New Relic does internally, and we used to
                // do that, but the flattening iterator accessed through
                // `LogEvent::convert_to_fields` adds quotes to dotted fields names, which produces
                // broken attributes in New Relic, and nesting objects is actually a (slightly) more
                // efficient representation of the key names.
                let (value, _metadata) = log.into_parts();
                let Some(mut attributes) = value.into_object() else {
                    num_non_object_events += 1;
                    return None;
                };
                strip_arrays(&mut attributes);

                Some(LogMessage {
                    message,
                    timestamp,
                    attributes,
                })
            })
            .collect();

        if num_non_log_events > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_log_events,
                reason: "non-log event",
            });
        }
        if num_non_object_events > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_object_events,
                reason: "non-object event",
            });
        }

        if !logs_array.is_empty() {
            Ok(Self::new(logs_array))
        } else {
            Err(NewRelicSinkError::new("No valid logs to generate"))
        }
    }
}

/// The Trace API data model, using the New Relic-format trace payload.
///
/// Reference: https://docs.newrelic.com/docs/distributed-tracing/trace-api/report-new-relic-format-traces-trace-api/
#[derive(Debug, Serialize)]
pub(super) struct TracesApiModel(pub [SpanStore; 1]);

#[derive(Debug, Serialize)]
pub(super) struct SpanStore {
    pub spans: Vec<Span>,
}

#[derive(Debug, Serialize)]
pub(super) struct Span {
    #[serde(rename = "trace.id")]
    pub trace_id: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    pub attributes: ObjectMap,
}

impl TracesApiModel {
    pub(super) const fn new(spans: Vec<Span>) -> Self {
        Self([SpanStore { spans }])
    }
}

const NANOS_PER_MILLI_F64: f64 = 1_000_000.0;
const NANOS_PER_MILLI: i64 = 1_000_000;
const OTEL_STATUS_CODE_ERROR: i64 = 2;

/// Attribute keys that New Relic drops or that trigger undefined behavior. Stripped from any
/// passthrough attributes.
///
/// Reference: https://docs.newrelic.com/docs/distributed-tracing/trace-api/trace-api-general-requirements-limits/
const RESTRICTED_TRACE_ATTRIBUTES: [&str; 5] = [
    "entityGuid",
    "guid",
    "entity.guid",
    "entity.name",
    "entity.type",
];

/// Why a span envelope was dropped during translation. Determines which `ComponentEventsDropped`
/// internal event is emitted.
enum SpanDropReason {
    NonObject,
    NonTrace,
    MissingId,
    MissingStart,
}

impl TryFrom<Vec<Event>> for TracesApiModel {
    type Error = NewRelicSinkError;

    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut num_non_log = 0;
        let mut num_non_object = 0;
        let mut num_non_trace = 0;
        let mut num_missing_id = 0;
        let mut num_missing_start = 0;

        let spans: Vec<Span> = buf_events
            .into_iter()
            .filter_map(|event| {
                let Some(log) = event.try_into_log() else {
                    num_non_log += 1;
                    return None;
                };
                match span_from_log(log) {
                    Ok(span) => Some(span),
                    Err(SpanDropReason::NonObject) => {
                        num_non_object += 1;
                        None
                    }
                    Err(SpanDropReason::NonTrace) => {
                        num_non_trace += 1;
                        None
                    }
                    Err(SpanDropReason::MissingId) => {
                        num_missing_id += 1;
                        None
                    }
                    Err(SpanDropReason::MissingStart) => {
                        num_missing_start += 1;
                        None
                    }
                }
            })
            .collect();

        if num_non_log > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_log,
                reason: "non-log event",
            });
        }
        if num_non_object > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_object,
                reason: "non-object event",
            });
        }
        if num_non_trace > 0 {
            emit!(ComponentEventsDropped::<INTENTIONAL> {
                count: num_non_trace,
                reason: "non-trace event",
            });
        }
        if num_missing_id > 0 {
            emit!(ComponentEventsDropped::<UNINTENTIONAL> {
                count: num_missing_id,
                reason: "span missing trace/span id",
            });
        }
        if num_missing_start > 0 {
            emit!(ComponentEventsDropped::<UNINTENTIONAL> {
                count: num_missing_start,
                reason: "span missing start time",
            });
        }

        if !spans.is_empty() {
            Ok(Self::new(spans))
        } else {
            Err(NewRelicSinkError::new("No valid spans to generate"))
        }
    }
}

/// Translate a single span envelope (`{resource, scope, type, record}`) log into a New Relic
/// trace span. Reserved attributes are written after passthrough attributes so they win on any
/// key collision.
fn span_from_log(mut log: LogEvent) -> Result<Span, SpanDropReason> {
    // Mezmo pipeline events wrap the self-describing span envelope under the log schema message
    // key (`{message: {record, resource, scope, type}}`). Prefer that payload; fall back to the
    // event root for a bare root-level envelope.
    let mut envelope = match take_message_object(&mut log) {
        Some(message) => message,
        None => log
            .into_parts()
            .0
            .into_object()
            .ok_or(SpanDropReason::NonObject)?,
    };

    // The envelope carries `type: "trace"`. Reject other types when the field is present, but
    // tolerate its absence.
    if let Some(kind) = envelope.get("type").and_then(Value::as_str)
        && &*kind != "trace"
    {
        return Err(SpanDropReason::NonTrace);
    }

    let mut record = envelope
        .remove("record")
        .and_then(Value::into_object)
        .ok_or(SpanDropReason::MissingId)?;

    let trace_id = take_id(&mut record, "traceId").ok_or(SpanDropReason::MissingId)?;
    let id = take_id(&mut record, "spanId").ok_or(SpanDropReason::MissingId)?;

    let start_ns = record
        .remove("startTimeUnixNano")
        .as_ref()
        .and_then(parse_unix_nanos)
        .filter(|&ns| ns > 0)
        .ok_or(SpanDropReason::MissingStart)?;
    let end_ns = record
        .remove("endTimeUnixNano")
        .as_ref()
        .and_then(parse_unix_nanos);
    let duration_ms = end_ns
        .map(|end| ((end - start_ns) as f64 / NANOS_PER_MILLI_F64).max(0.0))
        .unwrap_or(0.0);

    let mut attributes = ObjectMap::new();

    // Passthrough: resource attributes (lowest priority). `service.name` is promoted to a
    // reserved attribute; restricted keys are stripped.
    let mut service_name = None;
    if let Some(mut resource) = envelope.remove("resource").and_then(Value::into_object)
        && let Some(resource_attributes) =
            resource.remove("attributes").and_then(Value::into_object)
    {
        for (key, value) in resource_attributes {
            if key.as_str() == "service.name" {
                service_name = coerce_attribute_value(value);
                continue;
            }
            insert_passthrough_attribute(&mut attributes, key, value);
        }
    }

    // Passthrough: span attributes (override resource attributes on collision).
    if let Some(span_attributes) = record.remove("attributes").and_then(Value::into_object) {
        for (key, value) in span_attributes {
            insert_passthrough_attribute(&mut attributes, key, value);
        }
    }

    // Reserved attributes.
    attributes.insert("duration.ms".into(), Value::from_f64_or_zero(duration_ms));
    if let Some(name) = record.remove("name").filter(|v| !v.is_null()) {
        attributes.insert("name".into(), name);
    }
    if let Some(parent_id) = record.remove("parentSpanId").filter(|v| !v.is_null()) {
        attributes.insert("parent.id".into(), parent_id);
    }
    if let Some(service_name) = service_name {
        attributes.insert("service.name".into(), service_name);
    }
    if let Some(kind) = record.remove("kind").as_ref().and_then(Value::as_integer)
        && let Some(kind) = map_span_kind(kind)
    {
        attributes.insert("span.kind".into(), Value::from(kind));
    }
    if let Some(mut status) = record.remove("status").and_then(Value::into_object) {
        if let Some(code) = status.remove("code").as_ref().and_then(Value::as_integer) {
            attributes.insert(
                "otel.status_code".into(),
                Value::from(map_status_code(code)),
            );
            if code == OTEL_STATUS_CODE_ERROR {
                attributes.insert("error".into(), Value::from(true));
            }
        }
        if let Some(message) = status.remove("message").filter(|v| !v.is_null()) {
            attributes.insert("otel.status_description".into(), message);
        }
    }
    if let Some(mut scope) = envelope.remove("scope").and_then(Value::into_object) {
        if let Some(name) = scope.remove("name").filter(|v| !v.is_null()) {
            attributes.insert("otel.scope.name".into(), name);
        }
        if let Some(version) = scope.remove("version").filter(|v| !v.is_null()) {
            attributes.insert("otel.scope.version".into(), version);
        }
    }
    if let Some(events) = record.remove("events").filter(|v| !v.is_null())
        && let Ok(json) = serde_json::to_string(&events)
    {
        attributes.insert("otel.span.events".into(), Value::from(json));
    }
    if let Some(links) = record.remove("links").filter(|v| !v.is_null())
        && let Ok(json) = serde_json::to_string(&links)
    {
        attributes.insert("otel.span.links".into(), Value::from(json));
    }

    Ok(Span {
        trace_id,
        id,
        timestamp: Some(start_ns / NANOS_PER_MILLI),
        attributes,
    })
}

fn insert_passthrough_attribute(attributes: &mut ObjectMap, key: KeyString, value: Value) {
    if is_restricted_trace_attribute(key.as_str()) {
        return;
    }
    if let Some(value) = coerce_attribute_value(value) {
        attributes.insert(key, value);
    }
}

/// Remove and return the span envelope when the event wraps it under the log schema message key
/// (`{message: {record, ...}}`), the shape Mezmo pipeline events use. Returns `None` when there is
/// no message object, so the caller falls back to treating the event root as the envelope.
fn take_message_object(log: &mut LogEvent) -> Option<ObjectMap> {
    let path = log_schema().message_key_target_path()?;
    log.remove(path).and_then(Value::into_object)
}

/// Remove a hex-string id (`traceId`/`spanId`) from the record, returning `None` when absent,
/// empty, or not a string.
fn take_id(record: &mut ObjectMap, key: &str) -> Option<String> {
    match record.remove(key) {
        Some(Value::Bytes(bytes)) => {
            let id = String::from_utf8_lossy(bytes.as_ref()).into_owned();
            (!id.is_empty()).then_some(id)
        }
        _ => None,
    }
}

/// Parse a `*UnixNano` value. The decoder keeps these as strings to preserve int64 precision, but
/// tolerate integers/floats as well.
fn parse_unix_nanos(value: &Value) -> Option<i64> {
    match value {
        Value::Bytes(bytes) => String::from_utf8_lossy(bytes.as_ref()).trim().parse().ok(),
        Value::Integer(n) => Some(*n),
        Value::Float(f) => Some(f.into_inner() as i64),
        _ => None,
    }
}

/// Map an OTel `SpanKind` integer to the New Relic `span.kind` string. `0` (unspecified) is
/// omitted.
const fn map_span_kind(kind: i64) -> Option<&'static str> {
    match kind {
        1 => Some("internal"),
        2 => Some("server"),
        3 => Some("client"),
        4 => Some("producer"),
        5 => Some("consumer"),
        _ => None,
    }
}

const fn map_status_code(code: i64) -> &'static str {
    match code {
        1 => "OK",
        2 => "ERROR",
        _ => "UNSET",
    }
}

fn is_restricted_trace_attribute(key: &str) -> bool {
    RESTRICTED_TRACE_ATTRIBUTES.contains(&key)
}

/// Coerce an attribute value to a New Relic trace attribute (string/number/boolean). Arrays and
/// objects are JSON-stringified; nulls drop the key.
fn coerce_attribute_value(value: Value) -> Option<Value> {
    match value {
        Value::Bytes(_) | Value::Integer(_) | Value::Float(_) | Value::Boolean(_) => Some(value),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(&value).ok().map(Value::from),
        Value::Null => None,
        other => Some(other),
    }
}

const MILLISECONDS: f64 = 1000.0;

/// Convert a value into a timestamp value. New Relic accepts either milliseconds or seconds since
/// epoch as an integer, or ISO8601-formatted timestamp as a string.
///
/// Reference: https://docs.newrelic.com/docs/logs/log-api/introduction-log-api/#json-logs
fn map_timestamp_value(value: Value) -> Option<Timestamp> {
    match value {
        Value::Timestamp(t) => Some(Timestamp::Numeric(t.timestamp_millis())),
        Value::Integer(n) => Some(Timestamp::Numeric(n)),
        Value::Float(f) => Some(Timestamp::Numeric((f.into_inner() * MILLISECONDS) as i64)),
        Value::Bytes(b) => Some(Timestamp::String(
            String::from_utf8_lossy(b.as_ref()).into(),
        )),
        _ => None,
    }
}

fn get_message_string(value: Option<Value>) -> String {
    match value {
        Some(Value::Bytes(bytes)) => String::from_utf8_lossy(bytes.as_ref()).into(),
        Some(value) => value.to_string(),
        None => "log from vector".to_string(),
    }
}

fn strip_arrays(obj: &mut ObjectMap) {
    obj.retain(|_key, value| !value.is_array());
    obj.iter_mut().for_each(|(_key, value)| {
        if let Some(obj) = value.as_object_mut() {
            strip_arrays(obj);
        }
    });
}
