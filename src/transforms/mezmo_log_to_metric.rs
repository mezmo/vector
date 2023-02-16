use std::collections::BTreeMap;

use chrono::Utc;
use value::Value;
use vector_config::configurable_component;
use vector_core::config::{log_schema, LogNamespace};
use vector_core::event::metric::mezmo::TransformError;
use vector_core::event::metric::{Bucket, Quantile, Sample};
use vector_core::event::{Metric, MetricKind, MetricValue, StatisticKind, MetricTags};

use crate::mezmo::user_trace::handle_transform_error;
use crate::{
    config::{DataType, GenerateConfig, Input, Output, TransformConfig, TransformContext},
    event::Event,
    mezmo::MezmoContext,
    schema,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

/// Configuration for the `mezmo_log_to_metric` transform.
#[configurable_component(transform("mezmo_log_to_metric"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LogToMetricConfig {}

#[derive(Debug, Clone)]
pub struct LogToMetric {
    #[allow(dead_code)]
    config: LogToMetricConfig,

    /// The mezmo context used to surface errors
    mezmo_ctx: Option<MezmoContext>,
}

impl GenerateConfig for LogToMetricConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {}).unwrap()
    }
}

#[async_trait::async_trait]
impl TransformConfig for LogToMetricConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(LogToMetric::new(
            self.clone(),
            context.mezmo_ctx.clone(),
        )))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(&self, _: &schema::Definition, _: LogNamespace) -> Vec<Output> {
        vec![Output::default(DataType::Metric)]
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}

impl LogToMetric {
    pub const fn new(config: LogToMetricConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        LogToMetric { config, mezmo_ctx }
    }
}

pub fn to_metric(event: &Event) -> Result<Metric, TransformError> {
    let log = event.as_log();

    let timestamp = log
        .get(log_schema().timestamp_key())
        .and_then(Value::as_timestamp)
        .cloned()
        .or_else(|| Some(Utc::now()));
    let metadata = event.metadata().clone();

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

    Ok(
        Metric::new_with_metadata(name.to_string(), kind, value, metadata)
            .with_namespace(namespace)
            .with_tags(tags)
            .with_timestamp(timestamp),
    )
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

impl FunctionTransform for LogToMetric {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        // Metrics are "all or none" for a specific log. If a single fails, none are produced.
        let mut buffer: Option<Event> = None;

        match to_metric(&event) {
            Ok(metric) => {
                buffer = Some(Event::Metric(metric));
            }
            Err(err) => {
                handle_transform_error(&self.mezmo_ctx, err);
            }
        }

        // Metric generation was successful, publish it
        if let Some(event) = buffer {
            output.push(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{offset::TimeZone, DateTime, Utc};
    use futures_util::{Stream, StreamExt};
    use serial_test::serial;
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::sleep;
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;
    use crate::mezmo::user_trace::UserLogSubscription;
    use crate::test_util::components::assert_transform_compliance;
    use crate::transforms::test::create_topology_with_name;
    use crate::{
        config::log_schema,
        event::{
            metric::{Metric, MetricKind, MetricValue, StatisticKind},
            Event, LogEvent,
        },
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<LogToMetricConfig>();
    }

    fn ts() -> DateTime<Utc> {
        Utc.ymd(2018, 11, 14).and_hms_nano(8, 9, 10, 11)
    }

    fn create_event(key: &str, value: impl Into<Value> + std::fmt::Debug) -> Event {
        let mut log = Event::Log(LogEvent::from("i am a log"));
        log.as_mut_log().insert(key, value);
        log.as_mut_log().insert(log_schema().timestamp_key(), ts());
        log
    }

    /// Creates a log event that represents a metric
    fn create_metric_event(
        name: &str,
        type_name: &str,
        value: impl Into<Value> + std::fmt::Debug,
        namespace: Option<&str>,
    ) -> LogEvent {
        // Simple events have the following shape, inside .message
        // {
        //     "name": "my_metric_name",
        //     "tags": {},
        //     "kind": "absolute",
        //     "value": { "type": "counter", "value": 36 }
        //   }
        let mut message_map: BTreeMap<String, Value> = BTreeMap::new();
        let mut value_map: BTreeMap<String, Value> = BTreeMap::new();
        value_map.insert("type".into(), type_name.into());
        value_map.insert("value".into(), value.into());
        message_map.insert("name".into(), name.into());
        message_map.insert("kind".into(), "absolute".into());
        if let Some(namespace) = namespace {
            message_map.insert("namespace".into(), namespace.into());
        }
        message_map.insert("value".into(), Value::from(value_map));
        log_event_from_value(message_map)
    }

    /// Creates a log event with the shape {"message": value, "timestamp": ts()}
    fn log_event_from_value(value: impl Into<Value> + std::fmt::Debug) -> LogEvent {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("message".into(), value.into());
        event_map.insert("timestamp".into(), ts().into());
        event_map.into()
    }

    async fn do_transform(event: Event) -> Option<Event> {
        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let name = "v1:mezmo_log_to_metric:transform:ef757476-43a5-4e0d-b998-3db35dbde001:1515707f-f668-4ca1-8493-969e5b13e781:800e5a08-3e67-431c-bbf0-14aa94beafcc";
            let (topology, mut out) =
            create_topology_with_name(ReceiverStream::new(rx), LogToMetricConfig::default(), name).await;
            tx.send(event).await.unwrap();
            let result = tokio::time::timeout(Duration::from_secs(5), out.recv())
                .await
                .unwrap_or(None);
            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
            result
        })
        .await
    }

    async fn assert_error_message(
        mut log_stream: impl Stream<Item = LogEvent> + Unpin,
        expected: &str,
    ) {
        let user_log = tokio::select! {
            e = log_stream.next() => e,
            _ = sleep(Duration::from_secs(1)) => None,
        }
        .expect("The failure should be output to the user logs");

        let msg = user_log
            .get(".message")
            .expect("should contain a message")
            .as_str()
            .expect("message should be a string");
        assert_eq!(msg, expected);
    }

    #[tokio::test]
    async fn counter_test() {
        let event = create_metric_event("go_goroutines", "counter", 1.2, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "go_goroutines",
                MetricKind::Absolute,
                MetricValue::Counter { value: 1.2 },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    async fn gauge_test() {
        let event = create_metric_event(
            "go_memstats_alloc_bytes",
            "gauge",
            8.3,
            Some("my_namespace"),
        );
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "go_memstats_alloc_bytes",
                MetricKind::Absolute,
                MetricValue::Gauge { value: 8.3 },
                metadata,
            )
            .with_namespace(Some("my_namespace"))
            .with_timestamp(Some(ts()))
        );
    }
    #[tokio::test]
    async fn summary_test() {
        let map: BTreeMap<String, Value> = serde_json::from_str(
            r#"{
            "quantiles": [
                {
                  "quantile": 0,
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
                  "quantile": 1,
                  "value": 0.000144948
                }
              ],
            "count": 6,
            "sum": 0.000368255
        }"#,
        )
        .unwrap();
        let event = create_metric_event("go_gc_duration_seconds", "summary", map, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();
        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "go_gc_duration_seconds",
                MetricKind::Absolute,
                MetricValue::AggregatedSummary {
                    quantiles: vec![
                        Quantile {
                            quantile: 0.0,
                            value: 0.000017039
                        },
                        Quantile {
                            quantile: 0.25,
                            value: 0.000018094
                        },
                        Quantile {
                            quantile: 0.5,
                            value: 0.000066005
                        },
                        Quantile {
                            quantile: 0.75,
                            value: 0.000090725
                        },
                        Quantile {
                            quantile: 1.0,
                            value: 0.000144948
                        },
                    ],
                    count: 6,
                    sum: 0.000368255
                },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    async fn set_test() {
        let map: BTreeMap<String, Value> =
            serde_json::from_str(r#"{"values": ["a", "b"]}"#).unwrap();
        let event = create_metric_event("active_admin_users", "set", map, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "active_admin_users",
                MetricKind::Absolute,
                MetricValue::Set {
                    values: BTreeSet::from(["a".to_string(), "b".to_string()])
                },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    async fn distribution_test() {
        let map: BTreeMap<String, Value> = serde_json::from_str(
            r#"{
            "samples": [
                {"value": 1, "rate": 300},
                {"value": 2.2, "rate": 500}
            ],
            "statistic": "summary"
        }"#,
        )
        .unwrap();
        let event = create_metric_event("response_times", "distribution", map, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "response_times",
                MetricKind::Absolute,
                MetricValue::Distribution {
                    samples: vec![
                        Sample {
                            value: 1.0,
                            rate: 300
                        },
                        Sample {
                            value: 2.2,
                            rate: 500
                        }
                    ],
                    statistic: StatisticKind::Summary
                },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    async fn histogram_test() {
        let map: BTreeMap<String, Value> = serde_json::from_str(
            r#"{
            "buckets": [
                {
                    "upper_limit": 2,
                    "count": 1
                },
                {
                    "upper_limit": 4,
                    "count": 2
                },
                {
                    "upper_limit": 8,
                    "count": 3
                },
                {
                    "upper_limit": 16,
                    "count": 4
                },
                {
                    "upper_limit": 32,
                    "count": 5
                }
                ],
            "count": 20,
            "sum": 123
        }"#,
        )
        .unwrap();
        let event = create_metric_event("response_times", "histogram", map, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "response_times",
                MetricKind::Absolute,
                MetricValue::AggregatedHistogram {
                    buckets: vec![
                        Bucket {
                            upper_limit: 2.0,
                            count: 1
                        },
                        Bucket {
                            upper_limit: 4.0,
                            count: 2
                        },
                        Bucket {
                            upper_limit: 8.0,
                            count: 3
                        },
                        Bucket {
                            upper_limit: 16.0,
                            count: 4
                        },
                        Bucket {
                            upper_limit: 32.0,
                            count: 5
                        },
                    ],
                    count: 20,
                    sum: 123.0
                },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    #[serial]
    async fn parse_root_failure() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let event = create_event("k", "v");
        assert_eq!(do_transform(event).await, None);
        assert_error_message(log_stream, "Field 'message' type is not valid").await;
    }

    #[tokio::test]
    #[serial]
    async fn invalid_type_test() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let event = create_metric_event("response_times", "NON_EXISTING_TYPE", 123, None);
        assert_eq!(do_transform(event.into()).await, None);
        assert_error_message(
            log_stream,
            "Metric type 'NON_EXISTING_TYPE' is not supported",
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn invalid_subfield_test() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let map: BTreeMap<String, Value> =
            serde_json::from_str(r#"{"buckets": [], "count": null, "sum": 3}"#).unwrap();
        let event = create_metric_event("response_times", "histogram", map, None);
        assert_eq!(do_transform(event.into()).await, None);
        assert_error_message(log_stream, "Required field 'count' is null").await;
    }

    #[tokio::test]
    #[serial]
    async fn parse_value_failure_test() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let map: BTreeMap<String, Value> = serde_json::from_str(r#"{"hello": "world"}"#).unwrap();
        let event = log_event_from_value(map);
        assert_eq!(do_transform(event.into()).await, None);
        assert_error_message(
            log_stream,
            "Required field 'name' not found in the log event",
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn null_field_test() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let event = log_event_from_value(Value::Null);
        assert_eq!(do_transform(event.into()).await, None);
        assert_error_message(log_stream, "Field 'message' type is not valid").await;
    }
}
