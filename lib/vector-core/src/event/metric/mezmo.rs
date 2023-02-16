use std::collections::BTreeMap;

use chrono::Utc;

use crate::{
    config::log_schema,
    event::{
        metric::{Bucket, Metric, MetricKind, MetricTags, MetricValue, Quantile, Sample},
        LogEvent, StatisticKind, Value,
    },
};

use super::{MetricData, MetricName, MetricSeries, MetricTime};

#[derive(Debug)]
pub enum TransformError {
    FieldNotFound { field: String },
    FieldInvalidType { field: String },
    InvalidMetricType { type_name: String },
    FieldNull { field: String },
    ParseIntOverflow { field: String },
}

fn parse_value(
    type_name: &str,
    value_object: &BTreeMap<String, Value>,
) -> Result<MetricValue, TransformError> {
    match type_name {
        "counter" => Ok(MetricValue::Counter {
            value: get_float(value_object, "value")?,
        }),
        "gauge" => Ok(MetricValue::Gauge {
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
        rate: get_u64(value, "rate")? as u32,
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
        Value::Integer(v) => Ok(*v as f64),
        Value::Float(v) => Ok(v.into_inner().clone()),
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

pub fn to_metric(log: LogEvent) -> Result<Metric, TransformError> {
    let timestamp = log
        .get(log_schema().timestamp_key())
        .and_then(Value::as_timestamp)
        .cloned()
        .or_else(|| Some(Utc::now()));
    let metadata = log.metadata().clone();

    let root = log
        .get(log_schema().message_key())
        .ok_or_else(|| TransformError::FieldNotFound {
            field: log_schema().message_key().into(),
        })?
        .as_object()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: log_schema().message_key().into(),
        })?;

    let name =
        get_property(root, "name")?
            .as_str()
            .ok_or_else(|| TransformError::FieldInvalidType {
                field: "name".into(),
            })?;
    let namespace = root
        .get("namespace")
        .map(|v| v.as_str().map(|b| b.to_string()))
        .flatten();

    let tags = if let Some(tags) = root.get("tags") {
        let tags = tags
            .as_object()
            .ok_or_else(|| TransformError::FieldInvalidType {
                field: "tags".into(),
            })?;
        let mut map = MetricTags::default();
        for (k, v) in tags.into_iter() {
            map.insert(
                k.to_owned(),
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
    let type_name = get_property(value_object, "type")?
        .as_str()
        .ok_or_else(|| TransformError::FieldInvalidType {
            field: "value.type".into(),
        })?;

    let value = parse_value(type_name.as_ref(), value_object)?;

    Ok(Metric::from_parts(
        MetricSeries {
            name: MetricName {
                name: name.into(),
                namespace: namespace,
            },
            tags: tags,
        },
        MetricData {
            time: MetricTime {
                timestamp: timestamp,
                interval_ms: None,
            },
            kind,
            value,
        },
        metadata,
    ))
}

fn from_buckets(buckets: &Vec<Bucket>) -> Value {
    buckets
        .iter()
        .map(|b| {
            BTreeMap::from([
                ("upper_limit".to_owned(), b.upper_limit.into()),
                ("count".to_owned(), b.count.into()),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_samples(samples: &Vec<Sample>) -> Value {
    samples
        .iter()
        .map(|s| {
            BTreeMap::from([
                ("value".to_owned(), s.value.into()),
                ("rate".to_owned(), s.rate.into()),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_quantiles(quantiles: &Vec<Quantile>) -> Value {
    quantiles
        .iter()
        .map(|q| {
            BTreeMap::from([
                ("value".to_owned(), q.value.into()),
                ("quantile".to_owned(), q.quantile.into()),
            ])
            .into()
        })
        .collect::<Vec<Value>>()
        .into()
}

fn from_tags(tags: &MetricTags) -> Value {
    BTreeMap::from_iter(tags.iter_all().map(|(k, v)| ((k.to_owned(), v.into())))).into()
}

pub fn from_metric(metric: Metric) -> LogEvent {
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
                ("value".to_owned(), (*value).into()),
            ])),
            MetricValue::Gauge { value } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "gauge".into()),
                ("value".to_owned(), (*value).into()),
            ])),
            MetricValue::Set { values } => Value::Object(BTreeMap::from([
                ("type".to_owned(), "set".into()),
                (
                    "value".to_owned(),
                    BTreeMap::from([(
                        "values".to_owned(),
                        Value::from(Value::Array(
                            values.iter().map(|i| i.clone().into()).collect(),
                        )),
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
                        ("sum".to_owned(), (*sum).into()),
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
                        ("sum".to_owned(), (*sum).into()),
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

    use value::Value;

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

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

        let actual = from_metric(to_metric(expected.clone()).unwrap());

        assert_eq!(expected, actual);
    }
}
