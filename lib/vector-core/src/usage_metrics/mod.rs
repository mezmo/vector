use serde::Serialize;
use std::{collections::HashMap, env, str::FromStr, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};
use value::Value;
use vector_common::byte_size_of::ByteSizeOf;

use crate::{
    config::log_schema,
    event::EventArray,
    event::{array::EventContainer, MetricValue},
    usage_metrics::flusher::HttpFlusher,
};
use flusher::{DbFlusher, MetricsFlusher, StdErrFlusher};

use self::flusher::NoopFlusher;

const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 20;
const BASE_ARRAY_SIZE: usize = 8; // Add some overhead to the array and object size
const BASE_BTREE_SIZE: usize = 8;
static INTERNAL_TRANSFORM: ComponentKind = ComponentKind::Transform { internal: true };

mod flusher;

#[derive(Debug)]
pub struct UsageMetrics {
    key: UsageMetricsKey,
    events: usize,
    total_size: usize,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidRoot,
    InvalidFormat,
}

pub enum MetricsPublishingError {
    FlusherError,
    DbEndpointUrlNotSet,
    AuthNotSetError,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    Source { internal: bool },
    Sink,
    Transform { internal: bool },
}

impl FromStr for ComponentKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut internal = false;
        let to_parse = if let Some(s) = s.strip_prefix("internal_") {
            internal = true;
            s
        } else {
            s
        };

        match to_parse {
            "source" => Ok(ComponentKind::Source { internal }),
            "sink" => Ok(ComponentKind::Sink),
            "transform" => Ok(ComponentKind::Transform { internal }),
            _ => Err(Self::Err::InvalidFormat),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct UsageMetricsValue {
    /// Total number of events
    total_count: usize,
    /// Total size in bytes
    total_size: usize,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Serialize)]
pub struct UsageMetricsKey {
    account_id: String,
    pipeline_id: String,
    component_id: String,
    component_type: String,
    component_kind: ComponentKind,
}

impl UsageMetricsKey {
    fn new(
        account_id: String,
        pipeline_id: String,
        component_id: String,
        component_type: String,
        component_kind: ComponentKind,
    ) -> Self {
        Self {
            account_id,
            pipeline_id,
            component_id,
            component_type,
            component_kind,
        }
    }

    /// Determines whether the component with this key should be tracked
    fn is_tracked(&self) -> bool {
        match self.component_kind {
            ComponentKind::Source { internal } => {
                // Internal sources should not be tracked, remap transforms will be tracked instead
                !internal
            }
            ComponentKind::Sink => true,
            ComponentKind::Transform { internal } => {
                if internal {
                    // Only track source format transforms and the swimlanes
                    (self.component_type == "remap" && self.pipeline_id != "shared")
                        || (self.component_type == "route" && self.pipeline_id == "shared")
                } else {
                    false
                }
            }
        }
    }

    /// Gets the target key to use to track.
    /// For sinks and most sources, it's the same key
    /// For remap internal transforms, it's the source key
    fn target_key(&self) -> UsageMetricsKey {
        if self.component_kind == INTERNAL_TRANSFORM
            && self.component_type == "remap"
            && self.pipeline_id != "shared"
        {
            return UsageMetricsKey::new(
                self.account_id.clone(),
                self.pipeline_id.clone(),
                self.component_id.clone(),
                // The source type will NOT be used in the metric id, so it's safe to use whatever value
                "source_format".into(),
                ComponentKind::Source { internal: false },
            );
        }
        self.clone()
    }
}

impl FromStr for UsageMetricsKey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Possible valid values
        // - HTTP shared: 'v1:internal_source:http:shared'
        // - Swimlanes: 'v1:internal_transform:route:split_by_source'
        // - Source metrics shared: 'v1:internal_source:metrics:shared'
        // - Kafka: 'v1:kafka:internal_source:{component_id}:{pipeline_id}:{account_id}'
        // - Sink metrics shared: 'v1:internal_sink:metrics:shared'
        // - Normal sink (mezmo): 'v1:mezmo:sink:${sink_id}:${pipeline_id}:${account_id}'
        // - Normal sink (http): 'v1:http:sink:${sink_id}:${pipeline_id}:${account_id}'
        // - Internal source transform: 'v1:remap:internal_transform:${source_id}:${pipeline_id}:${account_id}'
        if !s.starts_with("v1:") {
            return Err(Self::Err::InvalidRoot);
        }
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 4 {
            return Err(Self::Err::InvalidFormat);
        }
        if parts.len() == 4 && (parts[3] == "shared" || parts[3] == "split_by_source") {
            let kind = parts[1].parse()?;
            let type_name = parts[2];
            return Ok(UsageMetricsKey::new(
                parts[3].into(),
                "shared".into(),
                type_name.into(),
                type_name.into(),
                kind,
            ));
        }
        if parts.len() == 6 {
            // Expected format is 'v1:{type}:{kind}:${component_id}:${pipeline_id}:${account_id}'
            return Ok(UsageMetricsKey::new(
                parts[5].into(),
                parts[4].into(),
                parts[3].into(),
                parts[1].into(),
                parts[2].parse()?,
            ));
        }

        Err(Self::Err::InvalidFormat)
    }
}

/// Counts the size of the payload and sends the pipeline-based usage metrics to the provided channel.
fn track_usage(
    tx: &UnboundedSender<UsageMetrics>,
    key: &UsageMetricsKey,
    events: usize,
    total_size: usize,
) {
    if tx
        .send(UsageMetrics {
            key: key.clone(),
            events,
            total_size,
        })
        .is_err()
    {
        warn!("Usage metrics channel closed");
    }
}

/// Gets a usage tracker for a component (source or sink)
pub fn get_component_usage_tracker(
    key: &Option<UsageMetricsKey>,
    tx: &UnboundedSender<UsageMetrics>,
) -> Box<dyn ComponentUsageTracker> {
    if let Some(key) = key {
        if key.is_tracked() {
            return Box::new(DefaultComponentTracker {
                metrics_tx: tx.clone(),
                key: key.target_key(),
            });
        }
    }

    Box::new(NoopTracker {})
}

/// Gets a usage tracker for a transform
pub fn get_transform_usage_tracker(
    key: &Option<UsageMetricsKey>,
    tx: &UnboundedSender<UsageMetrics>,
) -> Box<dyn OutputUsageTracker> {
    if let Some(key) = key {
        if key.is_tracked() {
            return Box::new(DefaultOutputTracker {
                metrics_tx: tx.clone(),
                target_key: key.target_key(),
            });
        }
    }

    Box::new(NoopTracker {})
}

/// Represents a tracker for a component
pub trait ComponentUsageTracker: Send + Sync {
    fn track(&self, array: &EventArray);
}

/// Represents a tracker for the output
pub trait OutputUsageTracker: Send + Sync {
    fn track_output(&self, events_count: usize, total_size: usize);
}

struct NoopTracker {}

impl ComponentUsageTracker for NoopTracker {
    fn track(&self, _array: &EventArray) {
        // Do nothing
    }
}

impl OutputUsageTracker for NoopTracker {
    fn track_output(&self, _events_count: usize, _total_size: usize) {
        // Do nothing
    }
}

struct DefaultComponentTracker {
    metrics_tx: UnboundedSender<UsageMetrics>,
    key: UsageMetricsKey,
}

impl ComponentUsageTracker for DefaultComponentTracker {
    fn track(&self, array: &EventArray) {
        track_usage(
            &self.metrics_tx,
            &self.key,
            array.len(),
            mezmo_byte_size(array),
        );
    }
}

struct DefaultOutputTracker {
    metrics_tx: UnboundedSender<UsageMetrics>,
    target_key: UsageMetricsKey,
}

impl OutputUsageTracker for DefaultOutputTracker {
    fn track_output(&self, events_count: usize, total_size: usize) {
        track_usage(&self.metrics_tx, &self.target_key, events_count, total_size);
    }
}

/// Estimates the byte size of a group of events according to Mezmo billing practices, documented in the ADR
/// <https://github.com/answerbook/pipeline-prototype/blob/main/doc/adr/0018-billing-ingress-egress.md>
pub fn mezmo_byte_size(array: &EventArray) -> usize {
    match array {
        // For logs: account for the value size of ".message" and ".meta"
        EventArray::Logs(a) => a
            .iter()
            .map(|l| {
                if let Some(fields) = l.as_map() {
                    // Account for the value of ".message" and ".meta"
                    fields.get(log_schema().message_key()).map_or(0, value_size)
                        + fields
                            .get(log_schema().user_metadata_key())
                            .map_or(0, value_size)
                } else {
                    0
                }
            })
            .sum(),
        // In the Mezmo pipeline, we only use the metrics type for internal metrics
        // User provided metric events are represented in vector as logs (NOT metrics)
        // For metrics: add byte size of the name, timestamp and value.
        EventArray::Metrics(a) => a
            .iter()
            .map(|m| {
                let series = m.series();
                let data = m.data();

                let mut result = series.name().allocated_bytes();
                if data.timestamp().is_some() {
                    result += 8;
                }
                result += metric_value_size(data.value());
                result
            })
            .sum(),
        // We currently don't support Traces type, leave it as an oversized estimation
        EventArray::Traces(a) => a.allocated_bytes(),
    }
}

pub fn value_size(v: &Value) -> usize {
    match v {
        Value::Bytes(v) => v.len(),
        Value::Boolean(_) => 1,
        Value::Timestamp(_) | Value::Integer(_) | Value::Float(_) => 8,
        Value::Regex(v) => v.as_str().len(),
        Value::Object(v) => {
            BASE_BTREE_SIZE
                + v.iter()
                    .map(|(k, v)| k.len() + value_size(v))
                    .sum::<usize>()
        }
        Value::Array(v) => BASE_ARRAY_SIZE + v.iter().map(value_size).sum::<usize>(),
        Value::Null => 0, // No value, just the type definition
    }
}

/// Estimate the value of the metric based on the type
fn metric_value_size(v: &MetricValue) -> usize {
    match v {
        MetricValue::Counter { .. } | MetricValue::Gauge { .. } => 8,
        MetricValue::Set { values } => {
            BASE_BTREE_SIZE + values.iter().map(std::string::String::len).sum::<usize>()
        }
        MetricValue::AggregatedHistogram { buckets, .. } => {
            16 + // count and sum
            BASE_ARRAY_SIZE + buckets.iter().map(|_| {
                16 // upper_limit and count
            }).sum::<usize>()
        }
        MetricValue::AggregatedSummary { quantiles, .. } => {
            16 + // count and sum
            BASE_ARRAY_SIZE + quantiles.iter().map(|_| {
                16 // quantile and value
            }).sum::<usize>()
        }
        MetricValue::Distribution { samples, .. } => {
            1 + BASE_ARRAY_SIZE
                + samples
                    .iter()
                    .map(|_| {
                        12 // value and rate
                    })
                    .sum::<usize>()
        }
        _ => v.allocated_bytes(),
    }
}

/// # Errors
///
/// Returns `Err` if it can't get a flusher
pub async fn start_publishing_metrics(
    rx: UnboundedReceiver<UsageMetrics>,
) -> Result<(), MetricsPublishingError> {
    let agg_window = if let Ok(interval_str) = std::env::var("USAGE_METRICS_FLUSH_INTERVAL_SECS") {
        let secs = interval_str.parse().unwrap_or_else(|_| {
            warn!("USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable invalid, using default");
            DEFAULT_FLUSH_INTERVAL_SECS
        });
        Duration::from_secs(secs)
    } else {
        info!("USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable not set, using default");
        Duration::from_secs(DEFAULT_FLUSH_INTERVAL_SECS)
    };

    let flusher = get_flusher(agg_window).await?;
    start_publishing_metrics_with_flusher(rx, agg_window, flusher);
    Ok(())
}

async fn get_flusher(
    agg_window: Duration,
) -> Result<Arc<dyn MetricsFlusher + Send>, MetricsPublishingError> {
    let db_url = env::var("MEZMO_METRICS_DB_URL").ok();
    let endpoint_url = env::var("MEZMO_METRICS_ENDPOINT_URL").ok();
    let pod_name = env::var("POD_NAME").unwrap_or_else(|_| "not-set".to_string());

    if let Some(db_url) = db_url {
        // Allow to be black-box tested
        if db_url == "log://stderr" {
            return Ok(Arc::new(StdErrFlusher {}));
        }

        return Ok(Arc::new(
            DbFlusher::new(db_url, &pod_name)
                .await
                .map_err(|_| MetricsPublishingError::FlusherError)?,
        ));
    }

    if let Some(endpoint_url) = endpoint_url {
        // Http endpoint used by Pulse
        let auth_token = env::var("MEZMO_LOCAL_DEPLOY_AUTH_TOKEN").ok();
        let headers = if let Some(token) = auth_token {
            HashMap::from([("Authorization".to_string(), format!("Token {token}"))])
        } else {
            return Err(MetricsPublishingError::AuthNotSetError);
        };

        return Ok(Arc::new(HttpFlusher::new(
            &pod_name,
            endpoint_url,
            headers,
            agg_window,
        )));
    }

    if cfg!(debug_assertions) {
        // Debug build, it's OK to have the metrics db setting undefined
        info!("MEZMO_METRICS_DB_URL environment variable not set, disabling usage metrics");
        return Ok(Arc::new(NoopFlusher {}));
    }

    // Release build, we must have the url to the metrics db set
    error!("MEZMO_METRICS_DB_URL environment variable not set");
    Err(MetricsPublishingError::DbEndpointUrlNotSet)
}

fn start_publishing_metrics_with_flusher(
    mut rx: UnboundedReceiver<UsageMetrics>,
    agg_window: Duration,
    flusher: Arc<dyn MetricsFlusher + Send>,
) {
    tokio::spawn(async move {
        info!("Start publishing usage metrics");

        let mut finished = false;

        while !finished {
            let mut event_count = 0;
            let mut aggregated: HashMap<_, UsageMetricsValue> = HashMap::new();
            let timeout = sleep(agg_window);
            tokio::pin!(timeout);

            // Aggregate all messages across this `agg_window`
            loop {
                tokio::select! {
                    // Use unbiased (pseudo random) branch selection, that way we support for immediate flushes
                    _ = &mut timeout => {
                        // Break the inner loop, start a new timer
                        break;
                    },
                    Some(message) = rx.recv() => {
                        let value = aggregated.entry(message.key).or_default();
                        value.total_count += message.events;
                        value.total_size += message.total_size;
                        event_count += message.events;
                    },
                    else => {
                        // Channel closed
                        finished = true;
                        break;
                    }
                }
            }

            if event_count > 0 {
                // Flush aggregated metrics
                debug!(
                    "Saving {} aggregated usage metrics from {} metrics events",
                    aggregated.len(),
                    event_count
                );

                flusher.save_metrics(aggregated).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use tokio::sync::mpsc;
    use value::Value;

    use crate::{config::log_schema, event::LogEvent};
    use async_trait::async_trait;

    use super::*;

    // Create a manual mock
    // We can't use mockall because we need to inspect the value from the struct after the field is moved.
    struct MockMetricsFlusher {
        tx: UnboundedSender<HashMap<UsageMetricsKey, UsageMetricsValue>>,
    }

    #[async_trait]
    impl flusher::MetricsFlusher for MockMetricsFlusher {
        async fn save_metrics(&self, metrics: HashMap<UsageMetricsKey, UsageMetricsValue>) {
            self.tx.send(metrics).unwrap();
        }
    }

    macro_rules! assert_parse_ok {
        ($val: expr, $account_id: expr, $pipeline_id: expr, $comp_id: expr, $type_name: expr, $kind: expr) => {
            let key: Result<UsageMetricsKey, ParseError> = $val.parse();
            assert!(matches!(key, Ok(_)), "parse failed for {}", $val);
            assert_eq!(
                key.expect("to be ok"),
                UsageMetricsKey::new(
                    $account_id.into(),
                    $pipeline_id.into(),
                    $comp_id.into(),
                    $type_name.into(),
                    $kind,
                )
            );
        };
    }

    #[test]
    fn usage_metrics_key_parse() {
        assert_parse_ok!(
            "v1:internal_source:http:shared",
            "shared",
            "shared",
            "http",
            "http",
            ComponentKind::Source { internal: true }
        );
        assert_parse_ok!(
            "v1:internal_source:metrics:shared",
            "shared",
            "shared",
            "metrics",
            "metrics",
            ComponentKind::Source { internal: true }
        );
        assert_parse_ok!(
            "v1:kafka:internal_source:c1:p1:a1",
            "a1",
            "p1",
            "c1",
            "kafka",
            ComponentKind::Source { internal: true }
        );
        assert_parse_ok!(
            "v1:internal_sink:metrics:shared",
            "shared",
            "shared",
            "metrics",
            "metrics",
            ComponentKind::Sink
        );
        assert_parse_ok!(
            "v1:mezmo:sink:c1:p1:a1",
            "a1",
            "p1",
            "c1",
            "mezmo",
            ComponentKind::Sink
        );
        assert_parse_ok!(
            "v1:http:sink:c2:p1:a1",
            "a1",
            "p1",
            "c2",
            "http",
            ComponentKind::Sink
        );
        assert_parse_ok!(
            "v1:remap:internal_transform:2abace7c-9262-11ed-9674-b24ec21211c7:24af544e-9262-11ed-9674-b24ec21211c7:83a8cb84-239e-11ed-8a72-4ef12c27e273",
            "83a8cb84-239e-11ed-8a72-4ef12c27e273",
            "24af544e-9262-11ed-9674-b24ec21211c7",
            "2abace7c-9262-11ed-9674-b24ec21211c7",
            "remap",
            ComponentKind::Transform { internal: true }
        );
    }

    #[test]
    fn is_tracked_test() {
        let value: UsageMetricsKey = "v1:http:sink:comp1:pipe1:account1".parse().unwrap();
        assert!(value.is_tracked(), "All sinks should be tracked");

        let value: UsageMetricsKey = "v1:s3:source:comp1:pipe1:account1".parse().unwrap();
        assert!(value.is_tracked(), "Most sources should be tracked");

        let value: UsageMetricsKey = "v1:remap:internal_transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        assert!(
            value.is_tracked(),
            "internal remap transform should be tracked"
        );

        let value: UsageMetricsKey = "v1:filter-by-field:transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        assert!(
            !value.is_tracked(),
            "User defined transforms should NOT be tracked"
        );

        let value: UsageMetricsKey = "v1:kafka:internal_source:comp1:pipe1:account1"
            .parse()
            .unwrap();
        assert!(!value.is_tracked(), "Kafka source should NOT be tracked");
    }

    #[test]
    fn target_key_test() {
        let value: UsageMetricsKey = "v1:s3:source:comp1:pipe1:account1".parse().unwrap();
        assert_eq!(
            value.target_key(),
            value,
            "Should return be a clone for most keys"
        );

        let value: UsageMetricsKey = "v1:http:sink:comp1:pipe1:account1".parse().unwrap();
        assert_eq!(
            value.target_key(),
            value,
            "Should return be a clone for most keys"
        );

        let value: UsageMetricsKey = "v1:remap:internal_transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        let target = value.target_key();
        assert_eq!(target.account_id, value.account_id);
        assert_eq!(target.pipeline_id, value.pipeline_id);
        assert_eq!(target.component_id, value.component_id);
        assert_eq!(
            target.component_kind,
            ComponentKind::Source { internal: false }
        );
        assert_eq!(target.component_type, "source_format");
    }

    #[test]
    fn mezmo_byte_size_log_message_number_test() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("this_is_ignored".into(), 1u8.into());
        event_map.insert("another_ignored".into(), 1.into());
        event_map.insert(log_schema().message_key().into(), 9.into());
        let event: LogEvent = event_map.into();
        assert_eq!(
            mezmo_byte_size(&event.into()),
            8,
            "only accounts for the value of message"
        );
    }

    #[test]
    fn mezmo_byte_size_log_message_and_meta_test() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("this_is_ignored".into(), 2.into());
        event_map.insert(log_schema().message_key().into(), "hello ".into());
        event_map.insert(log_schema().user_metadata_key().into(), "world".into());
        let event: LogEvent = event_map.into();
        assert_eq!(
            mezmo_byte_size(&event.into()),
            "hello world".len(),
            "value of message and meta"
        );
    }

    #[test]
    fn mezmo_byte_size_log_nested_test() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        let mut nested_map: BTreeMap<String, Value> = BTreeMap::new();
        nested_map.insert("prop1".into(), 1u64.into());
        nested_map.insert("prop2".into(), 1u8.into());
        nested_map.insert("prop3".into(), 1i32.into());
        nested_map.insert("prop4".into(), "abcd".into());
        event_map.insert(log_schema().message_key().into(), Value::from(nested_map));
        let event: LogEvent = event_map.into();
        assert_eq!(
            mezmo_byte_size(&event.into()),
            BASE_BTREE_SIZE + "propX".len() * 4 + 28
        );
    }

    #[tokio::test]
    async fn start_publishing_metrics_should_aggregate_and_publish_metrics_periodically() {
        let (metrics_tx, rx) = mpsc::unbounded_channel::<UsageMetrics>();

        let (result_tx, mut result_rx) =
            mpsc::unbounded_channel::<HashMap<UsageMetricsKey, UsageMetricsValue>>();
        let flusher = MockMetricsFlusher { tx: result_tx };

        let key1 = UsageMetricsKey {
            account_id: "a".into(),
            pipeline_id: "b".into(),
            component_id: "c".into(),
            component_type: "d".into(),
            component_kind: ComponentKind::Source { internal: false },
        };

        let key2 = UsageMetricsKey {
            account_id: "d".into(),
            pipeline_id: "e".into(),
            component_id: "f".into(),
            component_type: "g".into(),
            component_kind: ComponentKind::Source { internal: false },
        };

        metrics_tx
            .send(UsageMetrics {
                key: key1.clone(),
                events: 2,
                total_size: 10,
            })
            .unwrap();
        metrics_tx
            .send(UsageMetrics {
                key: key1.clone(),
                events: 4,
                total_size: 30,
            })
            .unwrap();
        metrics_tx
            .send(UsageMetrics {
                key: key2.clone(),
                events: 1,
                total_size: 123,
            })
            .unwrap();

        start_publishing_metrics_with_flusher(rx, Duration::from_millis(20), Arc::new(flusher));

        // It should publish 2 aggregated metrics
        let m = result_rx.recv().await.unwrap();
        assert_eq!(m.len(), 2);
        let v = m.get(&key1).unwrap();
        assert_eq!(v.total_count, 6);
        assert_eq!(v.total_size, 40);

        let v = m.get(&key2).unwrap();
        assert_eq!(v.total_count, 1);
        assert_eq!(v.total_size, 123);
    }
}
