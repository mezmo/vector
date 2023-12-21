use std::borrow::Cow;
use std::collections::BTreeMap;

use chrono::Utc;

use crate::{
    config::log_schema,
    event::{
        metric::{
            mezmo::from_f64_or_zero, Bucket, Metric, MetricData, MetricKind, MetricName,
            MetricSeries, MetricTags, MetricTime, MetricValue, Quantile, Sample,
        },
        LogEvent, StatisticKind, Value,
    },
};

#[derive(Debug)]
pub enum TransformError {
    FieldNotFound { field: String },
    FieldInvalidType { field: String },
    InvalidMetricType { type_name: String },
    FieldNull { field: String },
    ParseIntOverflow { field: String },
    NumberTruncation { field: String },
}

fn parse_value(
    type_name: &str,
    value_object: &BTreeMap<String, Value>,
) -> Result<MetricValue, TransformError> {
    match type_name {
        "counter" => Ok(MetricValue::Counter {
            value: get_float(value_object, "value")?,
        }),
        "gauge" | "count" => Ok(MetricValue::Gauge {
            value: get_float(value_object, "value")?,
        }),
        "summary" => {
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

fn get_float(value_object: &BTreeMap<String, Value>, name: &str) -> Result<f64, TransformError> {
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

fn get_u64(value_object: &BTreeMap<String, Value>, name: &str) -> Result<u64, TransformError> {
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
    root: &'a BTreeMap<String, Value>,
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
    let timestamp = match log_schema().timestamp_key() {
        Some(path) => log
            .get((lookup::PathPrefix::Event, path))
            .and_then(Value::as_timestamp)
            .copied()
            .or_else(|| Some(Utc::now())),
        None => Some(Utc::now()),
    };

    let metadata = log.metadata().clone();

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
        .and_then(|v| v.as_str().map(|b| b.to_string()));

    let tags = if let Some(tags) = root.get("tags") {
        let tags = tags
            .as_object()
            .ok_or_else(|| TransformError::FieldInvalidType {
                field: "tags".into(),
            })?;
        let mut map = MetricTags::default();
        for (k, v) in tags.iter() {
            map.insert(
                k.clone(),
                v.as_str().map(|v| v.to_string()).ok_or_else(|| {
                    TransformError::FieldInvalidType {
                        field: "tags".into(),
                    }
                })?,
            );
        }

        Some(map)
    } else {
        None
    };

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
                interval_ms: None,
            },
            kind,
            value,
        },
        metadata,
    ))
}

fn from_buckets(buckets: &[Bucket]) -> Value {
    buckets
        .iter()
        .map(|b| {
            BTreeMap::from([
                ("upper_limit".to_owned(), from_f64_or_zero(b.upper_limit)),
                ("count".to_owned(), b.count.into()),
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
                ("value".to_owned(), from_f64_or_zero(s.value)),
                ("rate".to_owned(), s.rate.into()),
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
                ("value".to_owned(), from_f64_or_zero(q.value)),
                ("quantile".to_owned(), from_f64_or_zero(q.quantile)),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_tags(tags: &MetricTags) -> Value {
    tags.iter_all()
        .map(|(k, v)| (k.to_owned(), v.into()))
        .collect::<BTreeMap<_, _>>()
        .into()
}

/// # Panics
///
/// Will panic upon encountering unsupported metric type
pub fn from_metric(metric: &Metric) -> LogEvent {
    let mut values = BTreeMap::<String, Value>::new();

    values.insert("name".to_owned(), metric.name().into());
    if let Some(namespace) = metric.namespace() {
        values.insert("namespace".to_owned(), namespace.into());
    }
    values.insert(
        "kind".to_owned(),
        if metric.kind() == MetricKind::Absolute {
            "absolute"
        } else {
            "incremental"
        }
        .into(),
    );
    if let Some(tags) = metric.tags() {
        values.insert("tags".to_owned(), from_tags(tags));
    }

    values.insert(
        "value".to_owned(),
        match metric.value() {
            MetricValue::Counter { value } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "counter".into()),
                ("value".to_owned(), from_f64_or_zero(*value)),
            ])),
            MetricValue::Gauge { value } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "gauge".into()),
                ("value".to_owned(), from_f64_or_zero(*value)),
            ])),
            MetricValue::Set { values } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "set".into()),
                (
                    "value".to_owned(),
                    BTreeMap::from([(
                        "values".to_owned(),
                        Value::Array(values.iter().map(|i| i.clone().into()).collect()),
                    )])
                    .into(),
                ),
            ])),

            MetricValue::Distribution { samples, statistic } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "distribution".into()),
                (
                    "value".to_owned(),
                    BTreeMap::from([
                        ("samples".to_owned(), from_samples(samples)),
                        (
                            "statistic".to_owned(),
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
            ])),
            MetricValue::AggregatedSummary {
                quantiles,
                count,
                sum,
            } => BTreeMap::from([
                ("type".to_owned(), "summary".into()),
                (
                    "value".to_owned(),
                    BTreeMap::from([
                        ("quantiles".to_owned(), from_quantiles(quantiles)),
                        ("count".to_owned(), (*count).into()),
                        ("sum".to_owned(), from_f64_or_zero(*sum)),
                    ])
                    .into(),
                ),
            ])
            .into(),
            MetricValue::AggregatedHistogram {
                buckets,
                count,
                sum,
            } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "histogram".into()),
                (
                    "value".to_owned(),
                    Value::Object(BTreeMap::from([
                        ("buckets".to_owned(), from_buckets(buckets)),
                        ("count".to_owned(), (*count).into()),
                        ("sum".to_owned(), from_f64_or_zero(*sum)),
                    ])),
                ),
            ])),
            _ => panic!("unsupported metric value type"),
        },
    );

    LogEvent::from_map(
        BTreeMap::from([("message".to_owned(), Value::Object(values))]),
        Default::default(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use vrl::value::Value;

    use crate::event::LogEvent;

    use super::{from_metric, to_metric};

    #[test]
    fn counter() {
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                "name": "count",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "counter",
                    "value": 123.0
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
    fn count() {
        let mut expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                "name": "count",
                "kind": "absolute",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "count",
                    "value": 123.4
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
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                "name": "gauge",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "gauge",
                    "value": 456.0
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
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                "name": "set",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": {
                    "type": "set",
                    "value": { "values": ["a", "b", "c"] }
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
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
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
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
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
        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
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
}
