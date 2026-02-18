use crate::{
    config::log_schema,
    event::{
        KeyString, LogEvent, StatisticKind, Value,
        metric::{
            Bucket, Metric, MetricArbitrary, MetricData, MetricKind, MetricName, MetricSeries,
            MetricTags, MetricTime, MetricValue, Quantile, Sample, mezmo::from_f64_or_zero,
        },
    },
    metrics::AgentDDSketch,
};
use chrono::Utc;
use lookup::PathPrefix;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::num::NonZeroU32;

#[derive(Debug)]
pub enum TransformError {
    FieldNotFound { field: String },
    FieldInvalidType { field: String },
    InvalidMetricType { type_name: String },
    FieldNull { field: String },
    ParseIntOverflow { field: String },
    NumberTruncation { field: String },
    CardinalityLimitExceeded { limit: u32 },
}

/// Note that the Display implementation must be appropriate as a user-facing error.
impl Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::FieldNull { field } => {
                write!(f, "Required field '{field}' is null")
            }
            TransformError::FieldNotFound { field } => {
                write!(f, "Required field '{field}' not found in the log event")
            }
            TransformError::FieldInvalidType { field } => {
                write!(f, "Field '{field}' type is not valid")
            }
            TransformError::InvalidMetricType { type_name } => {
                write!(f, "Metric type '{type_name}' is not supported")
            }
            TransformError::ParseIntOverflow { field } => {
                write!(
                    f,
                    "Field '{field}' could not be parsed as an unsigned integer"
                )
            }
            TransformError::NumberTruncation { field } => {
                write!(f, "Field '{field}' was truncated during parsing")
            }
            TransformError::CardinalityLimitExceeded { limit } => {
                write!(f, "Cardinality limit of {limit} exceeded")
            }
        }
    }
}

const OTLP_METADATA_FIELDS: [&str; 5] = [
    "resource",
    "scope",
    "attributes",
    "data_provider",
    "original_type",
];

fn parse_value(
    type_name: &str,
    value_object: &BTreeMap<KeyString, Value>,
) -> Result<MetricValue, TransformError> {
    match type_name {
        "counter" => Ok(MetricValue::Counter {
            value: get_float(value_object, "value")?,
        }),
        "gauge" | "count" => Ok(MetricValue::Gauge {
            value: get_float(value_object, "value")?,
        }),
        "sketch" => build_sketch(value_object),
        "summary" => build_summary(value_object),
        "histogram" => {
            let value_object = get_property(value_object, "value")?
                .as_object()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value".into(),
                })?;

            let buckets: Result<Vec<_>, _> = get_property(value_object, "buckets")?
                .as_array()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.buckets".into(),
                })?
                .iter()
                .map(parse_bucket)
                .collect();

            Ok(MetricValue::AggregatedHistogram {
                buckets: buckets?,
                count: get_u64(value_object, "count")?,
                sum: get_float(value_object, "sum")?,
            })
        }
        "distribution" => {
            let value_object = get_property(value_object, "value")?
                .as_object()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value".into(),
                })?;

            let samples: Result<Vec<_>, _> = get_property(value_object, "samples")?
                .as_array()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.samples".into(),
                })?
                .iter()
                .map(parse_sample)
                .collect();
            let statistic = get_property(value_object, "statistic")?
                .as_str()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.statistic".into(),
                })?;

            Ok(MetricValue::Distribution {
                samples: samples?,
                statistic: match statistic.as_ref() {
                    "histogram" => Ok(StatisticKind::Histogram),
                    "summary" => Ok(StatisticKind::Summary),
                    _ => Err(TransformError::FieldInvalidType {
                        field: "value.statistic".into(),
                    }),
                }?,
            })
        }
        "set" => {
            let value_object = get_property(value_object, "value")?
                .as_object()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value".into(),
                })?;

            let values: Result<Vec<_>, _> = get_property(value_object, "values")?
                .as_array()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.values".into(),
                })?
                .iter()
                .map(parse_string)
                .collect();

            Ok(MetricValue::Set {
                values: values?.into_iter().collect(),
            })
        }
        other => Err(TransformError::InvalidMetricType {
            type_name: other.to_string(),
        }),
    }
}

fn build_summary(value_object: &BTreeMap<KeyString, Value>) -> Result<MetricValue, TransformError> {
    let value_object = get_property(value_object, "value")?
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "value".into(),
        })?;

    let quantiles: Result<Vec<_>, _> = get_property(value_object, "quantiles")?
        .as_array()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "value.quantiles".into(),
        })?
        .iter()
        .map(parse_quantile)
        .collect();

    Ok(MetricValue::AggregatedSummary {
        quantiles: quantiles?,
        count: get_u64(value_object, "count")?,
        sum: get_float(value_object, "sum")?,
    })
}

fn build_sketch(value: &BTreeMap<KeyString, Value>) -> Result<MetricValue, TransformError> {
    let sketch = get_property(value, "value")?.as_object().ok_or_else(|| {
        TransformError::FieldInvalidType {
            field: "value".into(),
        }
    })?;
    let keys = sketch
        .get("k")
        .and_then(Value::as_array)
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "value.k".into(),
        })?
        .iter()
        .map(|val| {
            val.as_integer()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.k".into(),
                })
                .and_then(|v| {
                    i16::try_from(v).map_err(|_| TransformError::NumberTruncation {
                        field: "value.k".into(),
                    })
                })
        })
        .collect::<Result<Vec<i16>, _>>()?;

    let counts = sketch
        .get("n")
        .and_then(Value::as_array)
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "value.n".into(),
        })?
        .iter()
        .map(|val| {
            val.as_integer()
                .ok_or_else(|| TransformError::FieldInvalidType {
                    field: "value.n".into(),
                })
                .and_then(|v| {
                    u16::try_from(v).map_err(|_| TransformError::NumberTruncation {
                        field: "value.n".into(),
                    })
                })
        })
        .collect::<Result<Vec<u16>, _>>()?;

    let count = get_u64(sketch, "cnt")?;
    let count = u32::try_from(count).map_err(|_| TransformError::NumberTruncation {
        field: "value.cnt".into(),
    })?;
    let min = get_float(sketch, "min")?;
    let max = get_float(sketch, "max")?;
    let sum = get_float(sketch, "sum")?;
    let avg = get_float(sketch, "avg")?;

    let sketch = AgentDDSketch::from_raw(count, min, max, sum, avg, &keys, &counts)
        .unwrap_or_else(AgentDDSketch::with_agent_defaults);
    Ok(MetricValue::from(sketch))
}

fn parse_quantile(value: &Value) -> Result<Quantile, TransformError> {
    let value = value
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "quantile".into(),
        })?;
    Ok(Quantile {
        quantile: get_float(value, "quantile")?,
        value: get_float(value, "value")?,
    })
}

fn parse_bucket(value: &Value) -> Result<Bucket, TransformError> {
    let value = value
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "bucket".into(),
        })?;
    Ok(Bucket {
        upper_limit: get_float(value, "upper_limit")?,
        count: get_u64(value, "count")?,
    })
}

fn parse_sample(value: &Value) -> Result<Sample, TransformError> {
    let value = value
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "sample".into(),
        })?;
    Ok(Sample {
        value: get_float(value, "value")?,
        rate: std::convert::TryInto::<u32>::try_into(get_u64(value, "rate")?).map_err(|_| {
            TransformError::NumberTruncation {
                field: "rate".into(),
            }
        })?,
    })
}

fn parse_string(value: &Value) -> Result<String, TransformError> {
    let value = value
        .as_str()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "sample".into(),
        })?;
    Ok(value.to_string())
}

fn parse_arbitrary(
    value_object: &BTreeMap<KeyString, Value>,
    user_metadata: &BTreeMap<KeyString, Value>,
) -> MetricArbitrary {
    let mut filtered_value = value_object
        .iter()
        .filter_map(|(key, value)| {
            if key.as_str() == "type" || key.as_str() == "value" {
                return None;
            }

            Some((key.clone(), value.clone()))
        })
        .collect::<BTreeMap<KeyString, Value>>();

    // Fetch OTLP specific metadata fields.
    let filtered_metadata: BTreeMap<KeyString, Value> = user_metadata
        .iter()
        .filter_map(|(key, value)| {
            let i = OTLP_METADATA_FIELDS.iter().position(|&v| v == key.as_str());
            if i.is_some() {
                return Some((key.clone(), value.clone()));
            }

            None
        })
        .collect();

    if !filtered_metadata.is_empty() {
        filtered_value.insert(
            log_schema().user_metadata_key().into(),
            Value::Object(filtered_metadata),
        );
    }

    MetricArbitrary {
        value: filtered_value,
    }
}

fn get_float(value_object: &BTreeMap<KeyString, Value>, name: &str) -> Result<f64, TransformError> {
    let value = get_property(value_object, name)?;

    // Depending on the serialization format and input value (which we don't control)
    // a float value might appear as a Value::Float or Value::Integer
    match value {
        Value::Integer(v) => {
            if v < &(2i64.pow(52)) {
                #[allow(clippy::cast_precision_loss)]
                Ok(*v as f64)
            } else {
                Err(TransformError::NumberTruncation { field: name.into() })
            }
        }
        Value::Float(v) => Ok(v.into_inner()),
        _ => Err(TransformError::FieldInvalidType { field: name.into() }),
    }
}

fn get_u64(value_object: &BTreeMap<KeyString, Value>, name: &str) -> Result<u64, TransformError> {
    let value = get_property(value_object, name)?
        .as_integer()
        .ok_or_else(|| TransformError::FieldInvalidType { field: name.into() })?;

    if value < 0 {
        // Internally represented as a i64, any negative value overflows
        return Err(TransformError::ParseIntOverflow { field: name.into() });
    }

    Ok(value as u64)
}

fn get_property<'a>(
    root: &'a BTreeMap<KeyString, Value>,
    property_name: &'a str,
) -> Result<&'a Value, TransformError> {
    match root.get(property_name) {
        None => Err(TransformError::FieldNotFound {
            field: property_name.to_string(),
        }),
        Some(Value::Null) => Err(TransformError::FieldNull {
            field: property_name.to_string(),
        }),
        Some(value) => Ok(value),
    }
}

/// # Errors
///
/// Will return `Err` if any field transformations fail
pub fn to_metric(log: &LogEvent) -> Result<Metric, TransformError> {
    let timestamp = log_schema()
        .timestamp_key()
        .and_then(|path| {
            log.get((lookup::PathPrefix::Event, path))
                .and_then(Value::as_timestamp)
                .copied()
        })
        .or_else(|| Some(Utc::now()));

    let metadata = log.metadata().clone();
    let user_metadata = match log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
        Some(Value::Object(metadata)) => metadata,
        _ => &BTreeMap::new(),
    };

    let root = log
        .get(log_schema().message_key_target_path().unwrap())
        .ok_or_else(|| TransformError::FieldNotFound {
            field: log_schema().message_key().unwrap().to_string(),
        })?
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: log_schema().message_key().unwrap().to_string(),
        })?;

    let name =
        get_property(root, "name")?
            .as_str()
            .ok_or_else(|| TransformError::FieldInvalidType {
                field: "name".into(),
            })?;
    let namespace = root
        .get("namespace")
        .and_then(Value::as_str)
        .map(String::from);

    let tags = root
        .get("tags")
        .and_then(Value::as_object)
        .map(|tags| {
            tags.iter()
                .map(|(k, v)| {
                    v.as_str()
                        .map(|v| (k.to_string(), v.to_string()))
                        .ok_or_else(|| TransformError::FieldInvalidType {
                            field: "tags".into(),
                        })
                })
                .collect::<Result<MetricTags, _>>()
        })
        .transpose()?;

    let kind: MetricKind = get_property(root, "kind")?
        .clone()
        .try_into()
        .map_err(|_| TransformError::FieldInvalidType {
            field: "kind".into(),
        })?;
    let value_object = get_property(root, "value")?.as_object().ok_or_else(|| {
        TransformError::FieldInvalidType {
            field: "value".into(),
        }
    })?;

    let interval_ms = root
        .get("time")
        .and_then(|time| time.as_object())
        .and_then(|time_object| time_object.get("interval_ms"))
        .and_then(Value::as_integer)
        .and_then(|interval_ms| u32::try_from(interval_ms).ok())
        .and_then(NonZeroU32::new);

    // this is trying to be tolerant of some sloppy metrics exporters, some of
    // which will emit a numeric value without a type. We're setting a type
    // that will succeed on a number, and the accumulation will be informed
    // by the "kind"
    let type_name = match value_object.get("type") {
        Some(Value::Null) => Err(TransformError::FieldNull {
            field: "value.type".into(),
        }),
        Some(t) => Ok(t.as_str().ok_or_else(|| TransformError::FieldInvalidType {
            field: "value.type".into(),
        })?),
        None => Ok(Cow::Borrowed("gauge")),
    }?;

    let value = parse_value(type_name.as_ref(), value_object)?;
    let arbitrary = parse_arbitrary(value_object, user_metadata);

    Ok(Metric::from_parts(
        MetricSeries {
            name: MetricName {
                name: name.into(),
                namespace,
            },
            tags,
        },
        MetricData {
            time: MetricTime {
                timestamp,
                interval_ms,
            },
            kind,
            value,
            arbitrary,
        },
        metadata,
    ))
}

fn from_buckets(buckets: &[Bucket]) -> Value {
    buckets
        .iter()
        .map(|b| {
            BTreeMap::from([
                ("upper_limit".into(), from_f64_or_zero(b.upper_limit)),
                ("count".into(), b.count.into()),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_samples(samples: &[Sample]) -> Value {
    samples
        .iter()
        .map(|s| {
            BTreeMap::from([
                ("value".into(), from_f64_or_zero(s.value)),
                ("rate".into(), s.rate.into()),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_quantiles(quantiles: &[Quantile]) -> Value {
    quantiles
        .iter()
        .map(|q| {
            BTreeMap::from([
                ("value".into(), from_f64_or_zero(q.value)),
                ("quantile".into(), from_f64_or_zero(q.quantile)),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_tags(tags: &MetricTags) -> Value {
    tags.iter_all()
        .map(|(k, v)| (k.into(), v.into()))
        .collect::<BTreeMap<_, _>>()
        .into()
}

/// # Panics
///
/// Will panic upon encountering unsupported metric type
pub fn from_metric(metric: &Metric) -> LogEvent {
    let mut values = BTreeMap::<KeyString, Value>::new();

    values.insert("name".into(), metric.name().into());
    if let Some(namespace) = metric.namespace() {
        values.insert("namespace".into(), namespace.into());
    }
    values.insert(
        "kind".into(),
        if metric.kind() == MetricKind::Absolute {
            "absolute"
        } else {
            "incremental"
        }
        .into(),
    );
    if let Some(tags) = metric.tags() {
        values.insert("tags".into(), from_tags(tags));
    }

    let mut value = build_metric_value(metric);

    value.extend(metric.arbitrary_value().value().clone());

    value.remove(log_schema().user_metadata_key());

    values.insert("value".into(), value.into());

    if let Some(interval_ms) = metric.interval_ms() {
        values.insert(
            "time".into(),
            BTreeMap::from([("interval_ms".into(), interval_ms.get().into())]).into(),
        );
    }

    LogEvent::from_map(
        BTreeMap::from([("message".into(), Value::Object(values))]),
        Default::default(),
    )
}

fn build_metric_value(metric: &Metric) -> BTreeMap<KeyString, Value> {
    match metric.value() {
        MetricValue::Counter { value } => BTreeMap::from([
            ("type".into(), "counter".into()),
            ("value".into(), from_f64_or_zero(*value)),
        ]),
        MetricValue::Gauge { value } => BTreeMap::from([
            ("type".into(), "gauge".into()),
            ("value".into(), from_f64_or_zero(*value)),
        ]),
        MetricValue::Set { values } => BTreeMap::from([
            ("type".into(), "set".into()),
            (
                "value".into(),
                BTreeMap::from([(
                    "values".into(),
                    Value::Array(values.iter().map(|i| i.clone().into()).collect()),
                )])
                .into(),
            ),
        ]),
        MetricValue::Distribution { samples, statistic } => BTreeMap::from([
            ("type".into(), "distribution".into()),
            (
                "value".into(),
                BTreeMap::from([
                    ("samples".into(), from_samples(samples)),
                    (
                        "statistic".into(),
                        if statistic == &StatisticKind::Histogram {
                            "histogram"
                        } else {
                            "summary"
                        }
                        .into(),
                    ),
                ])
                .into(),
            ),
        ]),
        MetricValue::AggregatedSummary {
            quantiles,
            count,
            sum,
        } => BTreeMap::from([
            ("type".into(), "summary".into()),
            (
                "value".into(),
                BTreeMap::from([
                    ("quantiles".into(), from_quantiles(quantiles)),
                    ("count".into(), (*count).into()),
                    ("sum".into(), from_f64_or_zero(*sum)),
                ])
                .into(),
            ),
        ]),
        MetricValue::AggregatedHistogram {
            buckets,
            count,
            sum,
        } => BTreeMap::from([
            ("type".into(), "histogram".into()),
            (
                "value".into(),
                Value::Object(BTreeMap::from([
                    ("buckets".into(), from_buckets(buckets)),
                    ("count".into(), (*count).into()),
                    ("sum".into(), from_f64_or_zero(*sum)),
                ])),
            ),
        ]),
        _ => panic!("unsupported metric value type"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use vrl::value::{KeyString, Value};

    use crate::event::{LogEvent, metric::MetricSketch};

    use super::{from_metric, to_metric};

    #[test]
    fn counter() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "count",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "counter",
                    "value": 123.0,
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn rate_log_event() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "count",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "counter",
                    "value": 123.0,
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                },
                "time": {
                    "interval_ms": 1000
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let metric = to_metric(&expected).unwrap();
        let actual = from_metric(&metric);

        assert_eq!(expected, actual);
        assert_eq!(1000, metric.interval_ms().unwrap().get());
    }

    #[test]
    fn count() {
        let mut expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "count",
                "kind": "absolute",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "count",
                    "value": 123.4,
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        // note: presently we materialize a MetricValue::Gauge. This causes
        // the resulting metric to have 'type' 'guage'. This "makes it work"
        // for now, but may not be desirable long term. This just characterizes
        // what's there.
        expected.insert("message.value.type", "gauge");
        assert_eq!(expected, actual);
    }

    #[test]
    fn gauge() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "gauge",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "gauge",
                    "value": 456.0,
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn set() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "set",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "set",
                    "value": { "values": ["a", "b", "c"] },
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn summary() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "summary",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "summary",
                    "value": {
                        "quantiles": [
                            {
                              "quantile": 0.0,
                              "value": 0.000017039
                            },
                            {
                              "quantile": 0.25,
                              "value": 0.000018094
                            },
                            {
                              "quantile": 0.5,
                              "value": 0.000066005
                            },
                            {
                              "quantile": 0.75,
                              "value": 0.000090725
                            },
                            {
                              "quantile": 1.0,
                              "value": 0.000144948
                            }
                          ],
                        "count": 6,
                        "sum": 0.000368255
                    },
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn histogram() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "histogram",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "histogram",
                    "value": {
                        "buckets": [
                            {
                                "upper_limit": 2.0,
                                "count": 1
                            },
                            {
                                "upper_limit": 4.0,
                                "count": 2
                            },
                            {
                                "upper_limit": 8.0,
                                "count": 3
                            },
                            {
                                "upper_limit": 16.0,
                                "count": 4
                            },
                            {
                                "upper_limit": 32.0,
                                "count": 5
                            }
                            ],
                        "count": 20,
                        "sum": 123.0
                    },
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn distribution() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                "name": "distribution",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "distribution",
                    "value": {
                        "samples": [
                            {"value": 1.0, "rate": 300},
                            {"value": 2.2, "rate": 500}
                        ],
                        "statistic": "summary"
                    },
                    "name": "test_name",
                    "description": "description",
                    "attributes": {
                        "attribute": "value"
                    }
                }
            }
        }"#,
        )
        .unwrap()
        .into();

        let actual = from_metric(&to_metric(&expected).unwrap());

        assert_eq!(expected, actual);
    }

    #[test]
    fn sketch() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<KeyString, Value>>(
            r#"{
                "message": {
                    "name": "sketch-name",
                    "kind": "incremental",
                    "namespace": "ns",
                    "tags": {"k1": "v1"},
                    "value": {
                        "type": "sketch",
                        "value": {
                            "cnt": 12,
                            "min": 1.0,
                            "max": 9.0,
                            "sum": 15.0,
                            "avg": 4.5,
                            "k": [1, 2],
                            "n": [3, 4]
                        },
                        "name": "test_name",
                        "description": "description",
                        "attributes": {
                            "attribute": "value"
                        }
                    }
                }
            }"#,
        )
        .unwrap()
        .into();

        let metric = to_metric(&expected).unwrap();

        match metric.value() {
            crate::event::metric::MetricValue::Sketch { sketch } => match sketch {
                MetricSketch::AgentDDSketch(ddsketch) => {
                    assert_eq!(ddsketch.count(), 12);
                    assert_eq!(ddsketch.min(), Some(1.0));
                    assert_eq!(ddsketch.max(), Some(9.0));
                    assert_eq!(ddsketch.sum(), Some(15.0));
                    assert_eq!(ddsketch.avg(), Some(4.5));
                    let bin_map = ddsketch.bin_map();
                    let (keys, counts) = bin_map.into_parts();
                    assert_eq!(keys, vec![1, 2]);
                    assert_eq!(counts, vec![3, 4]);
                }
            },
            _ => panic!("expected sketch metric value"),
        }
    }
}
