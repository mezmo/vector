use std::collections::HashMap;

use vector_lib::config::{LogNamespace, TransformOutput};
use vector_lib::configurable::configurable_component;
use vector_lib::event::metric::mezmo::to_metric;

use crate::{
    config::{DataType, GenerateConfig, Input, OutputId, TransformConfig, TransformContext},
    event::Event,
    schema,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};
use mezmo::{MezmoContext, user_trace::handle_transform_error};

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
#[typetag::serde(name = "mezmo_log_to_metric")]
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

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        _: &[(OutputId, schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        // Converting the log to a metric means we lose all incoming `Definition`s.
        vec![TransformOutput::new(DataType::Metric, HashMap::new())]
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

impl FunctionTransform for LogToMetric {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        // Metrics are "all or none" for a specific log. If a single fails, none are produced.
        let mut buffer: Option<Event> = None;

        match to_metric(&event.into_log()) {
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
    use chrono::{DateTime, NaiveDateTime, Utc, offset::TimeZone};
    use futures_util::{Stream, StreamExt};
    use serde_json;
    use serial_test::serial;
    use std::collections::{BTreeMap, BTreeSet};
    use std::num::NonZeroU32;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::sleep;
    use tokio_stream::wrappers::ReceiverStream;
    use vector_lib::event::metric::{Bucket, Quantile, Sample};
    use vector_lib::event::{KeyString, Value};

    use super::*;
    use crate::mezmo::user_trace::UserLogSubscription;
    use crate::test_util::components::assert_transform_compliance;
    use crate::transforms::test::create_topology_with_name;
    use crate::{
        config::log_schema,
        event::{
            Event, LogEvent,
            metric::{Metric, MetricKind, MetricValue, StatisticKind},
        },
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<LogToMetricConfig>();
    }

    fn ts() -> DateTime<Utc> {
        let dt = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2018, 11, 14).unwrap(),
            chrono::NaiveTime::from_hms_nano_opt(8, 9, 10, 11).unwrap(),
        );

        Utc.from_utc_datetime(&dt)
    }

    fn create_event(key: &str, value: impl Into<Value> + std::fmt::Debug) -> Event {
        let mut log = Event::Log(LogEvent::from("i am a log"));
        log.as_mut_log().insert(key, value);
        log.as_mut_log().insert(
            (
                vector_lib::lookup::PathPrefix::Event,
                log_schema().timestamp_key().unwrap(),
            ),
            ts(),
        );
        log
    }

    /// Creates a log event that represents a metric
    fn create_metric_event(
        name: &str,
        type_name: &str,
        value: impl Into<Value> + std::fmt::Debug,
        kind: &str,
        namespace: Option<&str>,
        interval_ms: Option<u32>,
    ) -> LogEvent {
        // Simple events have the following shape, inside .message
        // {
        //     "name": "my_metric_name",
        //     "tags": {},
        //     "kind": "absolute",
        //     "value": { "type": "counter", "value": 36 },
        //     "time": { "interval_ms": 2000 }
        //   }
        let mut message_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        let mut value_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        value_map.insert("type".into(), type_name.into());
        value_map.insert("value".into(), value.into());
        message_map.insert("name".into(), name.into());
        message_map.insert("kind".into(), kind.into());
        if let Some(interval_ms) = interval_ms {
            let mut time_map: BTreeMap<KeyString, Value> = BTreeMap::new();
            time_map.insert("interval_ms".into(), interval_ms.into());
            message_map.insert("time".into(), Value::from(time_map));
        }
        if let Some(namespace) = namespace {
            message_map.insert("namespace".into(), namespace.into());
        }
        message_map.insert("value".into(), Value::from(value_map));
        log_event_from_value(message_map)
    }

    /// Creates a log event with the shape {"message": value, "timestamp": ts()}
    fn log_event_from_value(value: impl Into<Value> + std::fmt::Debug) -> LogEvent {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
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

    // FIXME: Tests using this appear to be flakey when run locally all together. When run individually,
    // or within `make test environment=true`, they pass. This might have to do with the channels of
    // the UserSubscription being shared. Attempts to serialize it or share it did not work.
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
    #[serial]
    async fn counter_test() {
        let event = create_metric_event("go_goroutines", "counter", 1.2, "incremental", None, None);
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();

        assert_eq!(
            metric.into_metric(),
            Metric::new_with_metadata(
                "go_goroutines",
                MetricKind::Incremental,
                MetricValue::Counter { value: 1.2 },
                metadata,
            )
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    #[serial]
    async fn rate_counter_test() {
        let event = create_metric_event(
            "go_goroutines",
            "counter",
            1.2,
            "incremental",
            None,
            Some(1500),
        );
        let metadata = event.metadata().clone();
        let metric = do_transform(event.into()).await.unwrap();
        let new_metric = metric.into_metric();

        assert_eq!(
            new_metric,
            Metric::new_with_metadata(
                "go_goroutines",
                MetricKind::Incremental,
                MetricValue::Counter { value: 1.2 },
                metadata,
            )
            .with_timestamp(Some(ts()))
            .with_interval_ms(NonZeroU32::new(1500))
        );
    }

    #[tokio::test]
    #[serial]
    async fn gauge_test() {
        let event = create_metric_event(
            "go_memstats_alloc_bytes",
            "gauge",
            8.3,
            "absolute",
            Some("my_namespace"),
            None,
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
    #[serial]
    async fn gauge_with_interval_test() {
        let event = create_metric_event(
            "go_memstats_alloc_bytes",
            "gauge",
            8.3,
            "absolute",
            Some("my_namespace"),
            Some(2500),
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
            .with_interval_ms(NonZeroU32::new(2500))
            .with_namespace(Some("my_namespace"))
            .with_timestamp(Some(ts()))
        );
    }

    #[tokio::test]
    #[serial]
    async fn summary_test() {
        let map: BTreeMap<KeyString, Value> = serde_json::from_str(
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
        let event = create_metric_event(
            "go_gc_duration_seconds",
            "summary",
            map,
            "absolute",
            None,
            None,
        );
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
    #[serial]
    async fn set_test() {
        let map: BTreeMap<KeyString, Value> =
            serde_json::from_str(r#"{"values": ["a", "b"]}"#).unwrap();
        let event = create_metric_event("active_admin_users", "set", map, "absolute", None, None);
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
    #[serial]
    async fn distribution_test() {
        let map: BTreeMap<KeyString, Value> = serde_json::from_str(
            r#"{
            "samples": [
                {"value": 1, "rate": 300},
                {"value": 2.2, "rate": 500}
            ],
            "statistic": "summary"
        }"#,
        )
        .unwrap();
        let event = create_metric_event(
            "response_times",
            "distribution",
            map,
            "absolute",
            None,
            None,
        );
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
    #[serial]
    async fn histogram_test() {
        let map: BTreeMap<KeyString, Value> = serde_json::from_str(
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
        let event = create_metric_event("response_times", "histogram", map, "absolute", None, None);
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
        let event = create_metric_event(
            "response_times",
            "NON_EXISTING_TYPE",
            123,
            "absolute",
            None,
            None,
        );
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
        let map: BTreeMap<KeyString, Value> =
            serde_json::from_str(r#"{"buckets": [], "count": null, "sum": 3}"#).unwrap();
        let event = create_metric_event("response_times", "histogram", map, "absolute", None, None);
        assert_eq!(do_transform(event.into()).await, None);
        assert_error_message(log_stream, "Required field 'count' is null").await;
    }

    #[tokio::test]
    #[serial]
    async fn parse_value_failure_test() {
        let log_stream = UserLogSubscription::subscribe().into_stream();
        let map: BTreeMap<KeyString, Value> =
            serde_json::from_str(r#"{"hello": "world"}"#).unwrap();
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
