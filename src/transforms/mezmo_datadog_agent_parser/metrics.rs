use crate::common::datadog::DatadogMetricType;
use crate::config::log_schema;
use crate::event::{Event, MaybeAsLogMut, MetricKind, ObjectMap, Value};

use super::common::{get_message_object, parse_timestamp, TimestampUnit};
use super::MezmoDatadogAgentParser;

/// Transform a metric event into the normalized MezmoMetric log format.
///
/// The incoming event is a log event with the metric data in `.message`.
/// The payload version determines whether this is a v1 or v2 series metric.
/// Metrics data is intercepted and decoded in Kong with each item in the series
/// field emitted as a separate event
/// See: https://github.com/answerbook/pipeline-gateway-kong/blob/6fefc73374b32996b4bbb5ab1052eb2e6e6d3293/kong/plugins/pipeline-routing/lib/datadog.lua#L157
pub fn transform_metric(
    mut event: Event,
    parser: &MezmoDatadogAgentParser,
) -> Result<Vec<Event>, String> {
    let version = parser
        .get_payload_version(&event)
        .unwrap_or_else(|| "v2".to_string());

    let message_obj = {
        let log = event
            .maybe_as_log_mut()
            .ok_or_else(|| "Event is not a log".to_string())?;
        get_message_object(log)?.clone()
    };

    let output_messages = match version.as_str() {
        "v1" => transform_series_v1(&message_obj)?,
        "v2" => transform_series_v2(&message_obj)?,
        _ => transform_series_v2(&message_obj)?,
    };

    parser.build_events_from_messages(event, output_messages)
}

/// Transform a sketch event into the normalized MezmoMetric log format.
pub fn transform_sketch(
    mut event: Event,
    parser: &MezmoDatadogAgentParser,
) -> Result<Vec<Event>, String> {
    let message_obj = {
        let log = event
            .maybe_as_log_mut()
            .ok_or_else(|| "Event is not a log".to_string())?;
        get_message_object(log)?.clone()
    };

    parser.build_events_from_messages(event, transform_sketch_payload(&message_obj)?)
}

/// Transforms Datadog series v1 metrics (/api/v1/series)
/// v1 metrics store points as a list of lists ([[timestamp, value]])
fn transform_series_v1(message: &ObjectMap) -> Result<Vec<(ObjectMap, Option<Value>)>, String> {
    let metric_name = message
        .get("metric")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing metric name".to_string())?;

    let metric_type = message
        .get("type")
        .ok_or_else(|| "Missing metric type".to_string())
        .and_then(parse_metric_type)?;
    let metric_kind = MetricKind::from(&metric_type);

    let points = message
        .get("points")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing points".to_string())?;

    let mut tags = message
        .get("tags")
        .and_then(|v| v.as_array())
        .map(parse_tag_array)
        .unwrap_or_default();

    insert_source_type_name(message, &mut tags);

    let (namespace, name) = namespace_name_from_dd_metric(&metric_name);

    let interval = message
        .get("interval")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32)
        .unwrap_or(0);

    let mut outputs = Vec::new();

    for point in points {
        let point = point
            .as_array()
            .ok_or_else(|| "Missing point value".to_string())?;

        if point.len() < 2 {
            return Err("Missing point value".to_string());
        }

        let point_timestamp = parse_point_timestamp(&point[0])?;
        let point_value = parse_metric_value(&point[1])?;

        let mut output = ObjectMap::new();
        output.insert("name".into(), Value::from(name.to_string()));
        if let Some(ns) = namespace {
            output.insert("namespace".into(), Value::from(ns.to_string()));
        }

        output.insert("kind".into(), Value::from(metric_kind_as_str(metric_kind)));
        output.insert("type".into(), Value::from(metric_type.as_metric_kind_str()));

        if !tags.is_empty() {
            output.insert("tags".into(), Value::Object(tags.clone()));
        }

        let interval = match metric_type {
            DatadogMetricType::Rate => {
                // Rates are converted to type: counter
                // See: https://github.com/mezmo/vector/blob/f5010f6b3e3cda9f201429c3802cee94efb16586/src/sources/datadog_agent/metrics.rs#L280
                let interval = if interval != 0 { interval } else { 1 };
                output.insert("value".into(), Value::from(point_value * (interval as f64)));
                Some(interval)
            }
            DatadogMetricType::Gauge => {
                output.insert("value".into(), Value::from(point_value));
                if interval > 0 {
                    Some(interval)
                } else {
                    None
                }
            }
            DatadogMetricType::Count => {
                output.insert("value".into(), Value::from(point_value));
                None
            }
        };

        let mut time_obj = ObjectMap::new();
        if let Some(interval) = interval {
            time_obj.insert("interval_ms".into(), Value::from(interval * 1000));
        }
        output.insert("time".into(), Value::Object(time_obj));

        if let Some(host) = message.get("host") {
            output.insert("host".into(), host.clone());
        }

        outputs.push((output, Some(point_timestamp)));
    }

    Ok(outputs)
}

/// Transforms Datadog series v1 metrics (/api/v1/series)
/// v2 metrics store points as a list of objects ([{timestamp, value}])
fn transform_series_v2(message: &ObjectMap) -> Result<Vec<(ObjectMap, Option<Value>)>, String> {
    let metric_name = message
        .get("metric")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing metric name".to_string())?;

    let metric_type = message
        .get("type")
        .ok_or_else(|| "Missing metric type".to_string())
        .and_then(parse_metric_type)?;
    let metric_kind = MetricKind::from(&metric_type);

    let points = message
        .get("points")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing points".to_string())?;

    let mut tags = message
        .get("tags")
        .and_then(|v| v.as_array())
        .map(parse_tag_array)
        .unwrap_or_default();

    let (host_from_resource, resource_tags) = extract_resources(message);
    tags.extend(resource_tags);
    insert_source_type_name(message, &mut tags);

    let (namespace, name) = namespace_name_from_dd_metric(&metric_name);

    let interval = message
        .get("interval")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32)
        .unwrap_or(0);

    let mut outputs = Vec::new();

    for point in points {
        let point = point
            .as_object()
            .ok_or_else(|| "Missing point value".to_string())?;

        let point_value = point
            .get("value")
            .ok_or_else(|| "Missing point value".to_string())
            .and_then(parse_metric_value)?;
        let point_timestamp = point
            .get("timestamp")
            .ok_or_else(|| "Missing point timestamp".to_string())
            .and_then(parse_point_timestamp)?;

        let mut output = ObjectMap::new();
        output.insert("name".into(), Value::from(name.to_string()));
        if let Some(ns) = namespace {
            output.insert("namespace".into(), Value::from(ns.to_string()));
        }
        output.insert("kind".into(), Value::from(metric_kind_as_str(metric_kind)));
        output.insert("type".into(), Value::from(metric_type.as_metric_kind_str()));

        if !tags.is_empty() {
            output.insert("tags".into(), Value::Object(tags.clone()));
        }

        let interval = match metric_type {
            DatadogMetricType::Rate => {
                let interval = if interval != 0 { interval } else { 1 };
                output.insert("value".into(), Value::from(point_value * (interval as f64)));
                Some(interval)
            }
            DatadogMetricType::Gauge => {
                output.insert("value".into(), Value::from(point_value));
                if interval > 0 {
                    Some(interval)
                } else {
                    None
                }
            }
            DatadogMetricType::Count => {
                output.insert("value".into(), Value::from(point_value));
                None
            }
        };

        let mut time_obj = ObjectMap::new();
        if let Some(interval) = interval {
            time_obj.insert("interval_ms".into(), Value::from(interval * 1000));
        }
        output.insert("time".into(), Value::Object(time_obj));

        if let Some(host) = host_from_resource.as_ref() {
            output.insert("host".into(), host.clone());
        }

        outputs.push((output, Some(point_timestamp)));
    }

    Ok(outputs)
}

/// Transform a Datadog sketch (custom metric)
/// Schema: https://github.com/DataDog/agent-payload/blob/b14d89ab49cdb6a4618c2ac76c0b21e4e5423c5b/proto/metrics/agent_payload.proto#L91
/// In Kong, the sketches field of the agent payload is unrolled and forwarded to vector
/// See: https://github.com/answerbook/pipeline-gateway-kong/blob/6fefc73374b32996b4bbb5ab1052eb2e6e6d3293/kong/plugins/pipeline-routing/lib/datadog.lua#L201
fn transform_sketch_payload(
    message: &ObjectMap,
) -> Result<Vec<(ObjectMap, Option<Value>)>, String> {
    let metric_name = message
        .get("metric")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing metric name".to_string())?;

    let mut tags = message
        .get("tags")
        .and_then(|v| v.as_array())
        .map(parse_tag_array)
        .unwrap_or_default();

    insert_sketch_host_tag(message, &mut tags);

    let (namespace, metric_name) = namespace_name_from_dd_metric(&metric_name);

    let dog_sketches = message
        .get("dogsketches")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing sketches".to_string())?;

    let mut outputs = Vec::new();

    for sketch in dog_sketches {
        let mut sketch_value = sketch.clone();
        let mut output = ObjectMap::new();
        output.insert("name".into(), Value::from(metric_name.to_string()));
        if let Some(namespace) = namespace {
            output.insert("namespace".into(), Value::from(namespace.to_string()));
        }

        output.insert(
            "kind".into(),
            Value::from(metric_kind_as_str(MetricKind::Incremental)),
        );

        if !tags.is_empty() {
            output.insert("tags".into(), Value::Object(tags.clone()));
        }

        output.insert("type".into(), Value::from("sketch"));

        let timestamp = sketch
            .as_object()
            .and_then(|obj| obj.get("ts"))
            .and_then(|value| parse_point_timestamp(value).ok());

        if let Value::Object(ref mut sketch) = sketch_value {
            sketch.remove("ts");
        }

        output.insert("value".into(), sketch_value);
        outputs.push((output, timestamp));
    }

    Ok(outputs)
}

fn parse_tag_array(tags: &[Value]) -> ObjectMap {
    tags.iter()
        .filter_map(|tag| tag.as_str())
        .map(|tag| {
            let (key, val) = match tag.split_once(':') {
                Some((prefix, suffix)) => (prefix, Value::from(suffix.to_string())),
                None => (tag.as_ref(), Value::Null),
            };
            (key.to_string().into(), val)
        })
        .collect()
}

fn parse_metric_type(metric_type: &Value) -> Result<DatadogMetricType, String> {
    // Some legacy agents send the metric type as an integer
    if let Some(metric_type) = metric_type.as_integer() {
        return match metric_type {
            1 => Ok(DatadogMetricType::Count),
            2 => Ok(DatadogMetricType::Rate),
            3 => Ok(DatadogMetricType::Gauge),
            _ => Err("Unknown metric type".to_string()),
        };
    }

    if let Some(metric_type) = metric_type.as_str() {
        let normalized = metric_type.trim().to_ascii_lowercase();
        return match normalized.as_str() {
            "count" => Ok(DatadogMetricType::Count),
            "rate" => Ok(DatadogMetricType::Rate),
            "gauge" => Ok(DatadogMetricType::Gauge),
            _ => Err("Unknown metric type".to_string()),
        };
    }

    Err("Unknown metric type".to_string())
}

fn parse_metric_value(value: &Value) -> Result<f64, String> {
    match value {
        Value::Float(val) => Ok(val.into_inner()),
        Value::Integer(val) => Ok(*val as f64),
        _ => Err("Missing point value".to_string()),
    }
}

fn insert_source_type_name(message: &ObjectMap, tags: &mut ObjectMap) {
    if let Some(source_type_name) = message.get("source_type_name").and_then(|v| v.as_str()) {
        if !source_type_name.is_empty() {
            tags.insert(
                "source_type_name".into(),
                Value::from(source_type_name.to_string()),
            );
        }
    }
}

fn insert_sketch_host_tag(message: &ObjectMap, tags: &mut ObjectMap) {
    let host = match message.get("host") {
        Some(host) => host,
        None => return,
    };

    let host_key = match log_schema().host_key() {
        Some(host_key) => host_key.to_string(),
        None => return,
    };

    tags.insert(host_key.into(), host.clone());
}

fn extract_resources(message: &ObjectMap) -> (Option<Value>, ObjectMap) {
    let mut host = None;
    let mut resource_tags = ObjectMap::new();
    let resources = match message.get("resources").and_then(|v| v.as_array()) {
        Some(resources) => resources,
        None => return (host, resource_tags),
    };

    for resource in resources {
        let resource_item = match resource.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let resource_type = match resource_item.get("type").and_then(|v| v.as_str()) {
            Some(resource_type) => resource_type,
            None => continue,
        };

        let resource_name = match resource_item.get("name") {
            Some(resource_name) => resource_name,
            None => continue,
        };

        if resource_type == "host" {
            host = Some(resource_name.clone());
            continue;
        }

        resource_tags.insert(
            format!("resource.{resource_type}").into(),
            resource_name.clone(),
        );
    }

    (host, resource_tags)
}

fn parse_point_timestamp(value: &Value) -> Result<Value, String> {
    if let Some(timestamp) = parse_timestamp(value, TimestampUnit::Seconds) {
        return Ok(Value::Timestamp(timestamp));
    }

    let error = match value {
        Value::Integer(_) | Value::Float(_) => "Invalid point timestamp",
        _ => "Missing point timestamp",
    };

    Err(error.to_string())
}

impl<'a> From<&'a DatadogMetricType> for MetricKind {
    fn from(value: &'a DatadogMetricType) -> Self {
        match value {
            DatadogMetricType::Gauge => MetricKind::Absolute,
            DatadogMetricType::Count | DatadogMetricType::Rate => MetricKind::Incremental,
        }
    }
}

trait AsMetricKindStr {
    fn as_metric_kind_str(&self) -> &'static str;
}

impl AsMetricKindStr for DatadogMetricType {
    fn as_metric_kind_str(&self) -> &'static str {
        match self {
            DatadogMetricType::Gauge => "gauge",
            DatadogMetricType::Count | DatadogMetricType::Rate => "counter",
        }
    }
}

const fn metric_kind_as_str(kind: MetricKind) -> &'static str {
    match kind {
        MetricKind::Incremental => "incremental",
        MetricKind::Absolute => "absolute",
    }
}

fn namespace_name_from_dd_metric(dd_metric_name: &str) -> (Option<&str>, &str) {
    match dd_metric_name.split_once('.') {
        Some((namespace, name)) => (Some(namespace), name),
        None => (None, dd_metric_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::LogEvent;
    use crate::transforms::mezmo_datadog_agent_parser::{
        MezmoDatadogAgentParser, MezmoDatadogAgentParserConfig,
    };
    use chrono::{TimeZone, Utc};

    fn create_v2_metric_event() -> Event {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.usage",
                "type": 3,
                "points": [{"timestamp": 1234567890, "value": 42.5}, {"timestamp": 1234567891, "value": 43.0}],
                "tags": ["env:prod", "host:server1"],
                "resources": [
                    {"type": "host", "name": "myhost"},
                    {"type": "cluster", "name": "cluster-a"}
                ],
                "source_type_name": "my-source"
            }),
        );
        Event::Log(log)
    }

    fn create_v1_metric_event() -> Event {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v1",
                "metric": "system.memory.used",
                "type": "gauge",
                "points": [[1234567890, 1024.0], [1234567891, 1025.0]],
                "tags": ["env:staging"],
                "host": "testhost",
                "source_type_name": "my-source"
            }),
        );
        Event::Log(log)
    }

    #[test]
    fn test_transform_v2_metric() {
        let event = create_v2_metric_event();
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        assert_eq!(results.len(), 2);

        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(
            message
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("cpu.usage".to_string())
        );
        assert_eq!(
            message
                .get("namespace")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("system".to_string())
        );

        let tags = message.get("tags").unwrap().as_object().unwrap();
        assert!(matches!(
            tags.get("resource.cluster").and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "cluster-a"
        ));
        assert!(matches!(
            tags.get("source_type_name").and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "my-source"
        ));

        let val = message.get("value").unwrap().as_float().unwrap();
        assert_eq!(val, ordered_float::NotNan::new(42.5).unwrap());

        let timestamp = log
            .get(log_schema().timestamp_key_target_path().unwrap())
            .and_then(Value::as_timestamp)
            .expect("timestamp should be set");
        assert_eq!(
            *timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );

        let second_log = results[1].as_log();
        let second_log_message = second_log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();
        let second_log_value = second_log_message.get("value").unwrap().as_float().unwrap();
        assert_eq!(second_log_value, ordered_float::NotNan::new(43.0).unwrap());
    }

    #[test]
    fn test_transform_v1_metric() {
        let event = create_v1_metric_event();
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        assert_eq!(results.len(), 2);

        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(
            message
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("memory.used".to_string())
        );
        assert_eq!(
            message
                .get("namespace")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("system".to_string())
        );

        let tags = message.get("tags").unwrap().as_object().unwrap();
        assert!(matches!(
            tags.get("source_type_name").and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "my-source"
        ));

        let timestamp = log
            .get(log_schema().timestamp_key_target_path().unwrap())
            .and_then(Value::as_timestamp)
            .expect("timestamp should be set");
        assert_eq!(
            *timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );
    }

    #[test]
    fn test_parse_tag_array() {
        let tags = vec![
            Value::from("env:prod"),
            Value::from("host:server1"),
            Value::from("bare_tag"),
        ];

        let result = parse_tag_array(&tags);

        assert_eq!(
            result
                .get("env")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("prod".to_string())
        );
        assert_eq!(
            result
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("server1".to_string())
        );
        assert_eq!(
            result.get("bare_tag").map(|v| matches!(v, Value::Null)),
            Some(true)
        );
    }

    #[test]
    fn test_rate_metric() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.rate",
                "type": 2, // Rate
                "interval": 10, // 10s
                "points": [{"timestamp": 1234567890, "value": 5.0}],
                "tags": [],
                "resources": []
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        let val = message.get("value").unwrap().as_float().unwrap();
        // value = 5.0 * 10 = 50.0
        assert_eq!(val, ordered_float::NotNan::new(50.0).unwrap());

        let time_obj = message.get("time").unwrap().as_object().unwrap();
        let interval_ms = time_obj.get("interval_ms").unwrap().as_integer().unwrap();
        // interval_ms = 10 * 1000 = 10000
        assert_eq!(interval_ms, 10000);

        assert_eq!(
            message.get("kind").unwrap().as_str().unwrap(),
            "incremental"
        );
        assert_eq!(message.get("type").unwrap().as_str().unwrap(), "counter")
    }

    #[test]
    fn test_string_metric_type() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.usage",
                "type": "GAUGE",
                "points": [{"timestamp": 1234567890, "value": 42.5}],
                "tags": [],
                "resources": []
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        let message = results[0]
            .as_log()
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert!(matches!(
            message.get("type").and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "gauge"
        ));
    }

    #[test]
    fn test_gauge_interval_metric() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.gauge",
                "type": 3, // Gauge
                "interval": 10, // 10s
                "points": [{"timestamp": 1234567890, "value": 5.0}],
                "tags": [],
                "resources": []
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        let time_obj = message.get("time").unwrap().as_object().unwrap();
        let interval_ms = time_obj.get("interval_ms").unwrap().as_integer().unwrap();
        assert_eq!(interval_ms, 10000);
    }

    #[test]
    fn test_count_interval_metric() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.count",
                "type": 1, // Count
                "interval": 10, // 10s
                "points": [{"timestamp": 1234567890, "value": 5.0}],
                "tags": [],
                "resources": []
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_metric(event, &parser).unwrap();
        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        let time_obj = message.get("time").unwrap().as_object().unwrap();
        assert!(time_obj.get("interval_ms").is_none());
    }

    #[test]
    fn test_sketch_metric() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "metric": "system.cpu.sketch",
                "tags": ["env:prod", "bare_tag"],
                "host": "testhost",
                "dogsketches": [
                    {
                        "cnt": 12,
                        "min": 1.0,
                        "max": 9.0,
                        "sum": 15.0,
                        "avg": 4.5,
                        "k": [1, 2],
                        "n": [3, 4],
                        "ts": 1234567890
                    }
                ]
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_sketch(event, &parser).unwrap();
        assert_eq!(results.len(), 1);

        let message = results[0]
            .as_log()
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        assert!(matches!(
            message.get("kind").and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "incremental"
        ));
        assert!(matches!(
            message
                .get("type")
                .and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "sketch"
        ));
        let tags = message.get("tags").unwrap().as_object().unwrap();
        let host_key = log_schema().host_key().unwrap().to_string();
        assert!(matches!(
            tags.get(host_key.as_str()).and_then(|v| v.as_str()),
            Some(value) if value.as_ref() == "testhost"
        ));

        let timestamp = results[0]
            .as_log()
            .get(log_schema().timestamp_key_target_path().unwrap())
            .and_then(Value::as_timestamp)
            .expect("timestamp should be set");
        assert_eq!(
            *timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );
    }

    #[test]
    fn test_sketch_message_ts() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "metric": "system.cpu.sketch",
                "tags": ["env:prod", "bare_tag"],
                "host": "testhost",
                "dogsketches": [
                    {
                        "cnt": 12,
                        "min": 1.0,
                        "max": 9.0,
                        "sum": 15.0,
                        "avg": 4.5,
                        "k": [1, 2],
                        "n": [3, 4],
                        "ts": 1234567890
                    }
                ]
            }),
        );
        let event = Event::Log(log);
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config);

        let results = transform_sketch(event, &parser).unwrap();
        assert_eq!(results.len(), 1);

        let log = results[0].as_log();
        let message = log
            .get(log_schema().message_key_target_path().unwrap())
            .unwrap()
            .as_object()
            .unwrap();

        let value_obj = message.get("value").unwrap().as_object().unwrap();
        assert!(value_obj.get("ts").is_none());
        assert_eq!(value_obj.get("cnt").and_then(|v| v.as_integer()), Some(12));

        let timestamp = log
            .get(log_schema().timestamp_key_target_path().unwrap())
            .and_then(Value::as_timestamp)
            .expect("timestamp should be set");
        assert_eq!(
            *timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );
    }

    #[test]
    fn test_parse_point_timestamp_value_variants() {
        let timestamp = Utc.timestamp_opt(1234567890, 0).single().unwrap();
        let value = Value::Timestamp(timestamp);
        let parsed = parse_point_timestamp(&value).unwrap();
        let parsed_timestamp = parsed.as_timestamp().expect("timestamp should be set");
        assert_eq!(*parsed_timestamp, timestamp);

        let value = Value::from(1234567890);
        let parsed = parse_point_timestamp(&value).unwrap();
        let parsed_timestamp = parsed.as_timestamp().expect("timestamp should be set");
        assert_eq!(
            *parsed_timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );

        let value = Value::from(1234567890.0);
        let parsed = parse_point_timestamp(&value).unwrap();
        let parsed_timestamp = parsed.as_timestamp().expect("timestamp should be set");
        assert_eq!(
            *parsed_timestamp,
            Utc.timestamp_opt(1234567890, 0).single().unwrap()
        );

        let value = Value::from(f64::INFINITY);
        let err = parse_point_timestamp(&value).unwrap_err();
        assert_eq!(err, "Invalid point timestamp");

        let value = Value::from("not-a-timestamp");
        let err = parse_point_timestamp(&value).unwrap_err();
        assert_eq!(err, "Missing point timestamp");
    }
}
