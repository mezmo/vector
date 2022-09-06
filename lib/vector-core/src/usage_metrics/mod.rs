use std::{collections::HashMap, env, mem, str::FromStr, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};
use value::Value;

use crate::{
    event::array::EventContainer,
    event::{EventArray, MetricValue},
    ByteSizeOf,
};
use flusher::{DbFlusher, MetricsFlusher};

use self::flusher::NoopFlusher;

const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 20;
const UNMATCHED_ROUTE: &str = "_unmatched";

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
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ComponentKind {
    Source,
    Sink,
}

impl FromStr for ComponentKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let to_parse = if s.starts_with("internal_") {
            &s[9..]
        } else {
            s
        };

        if to_parse == "source" {
            return Ok(ComponentKind::Source);
        }
        if to_parse == "sink" {
            return Ok(ComponentKind::Sink);
        }
        Err(Self::Err::InvalidFormat)
    }
}

#[derive(Debug, Default)]
pub(crate) struct UsageMetricsValue {
    /// Total number of events
    total_count: usize,
    /// Total size in bytes
    total_size: usize,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct UsageMetricsKey {
    account_id: String,
    pipeline_id: String,
    component_id: String,
    component_kind: ComponentKind,
}

impl UsageMetricsKey {
    fn new(
        account_id: String,
        pipeline_id: String,
        component_id: String,
        component_kind: ComponentKind,
    ) -> Self {
        Self {
            account_id,
            pipeline_id,
            component_id,
            component_kind,
        }
    }
}

impl FromStr for UsageMetricsKey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Possible valid values
        // - HTTP shared: 'v1:internal_source:http:shared'
        // - Source metrics shared: 'v1:internal_source:metrics:shared'
        // - Kafka: 'v1:kafka:internal_source:{component_id}:{pipeline_id}:{account_id}'
        // - Sink metrics shared: 'v1:internal_sink:metrics:shared'
        // - Normal sink (mezmo): 'v1:mezmo:sink:${sink_id}:${pipeline_id}:${account_id}'
        // - Normal sink (http): 'v1:http:sink:${sink_id}:${pipeline_id}:${account_id}'
        if !s.starts_with("v1:") {
            return Err(Self::Err::InvalidRoot);
        }
        let parts: Vec<&str> = s.split(":").collect();
        if parts.len() < 4 {
            return Err(Self::Err::InvalidFormat);
        }
        if parts.len() == 4 && parts[3] == "shared" {
            let kind = parts[1].parse()?;
            return Ok(UsageMetricsKey::new(
                "shared".into(),
                "shared".into(),
                parts[2].into(),
                kind,
            ));
        }
        if parts.len() == 6 {
            // Expected format is 'v1:{type}:{kind}:${component_id}:${pipeline_id}:${account_id}'
            return Ok(UsageMetricsKey::new(
                parts[5].into(),
                parts[4].into(),
                parts[3].into(),
                parts[2].parse()?,
            ));
        }

        Err(Self::Err::InvalidFormat)
    }
}

/// Counts the size of the payload and sends the pipeline-based usage metrics to the provided channel.
pub fn track_usage(tx: &UnboundedSender<UsageMetrics>, array: &EventArray, component_name: &str) {
    let key = component_name.parse();
    if let Err(_) = key {
        // This will likely spam the logs if there's an invalid component being tracked.
        // Maybe that's OK (i.e. "let's fix it") or maybe in the future we could use some kind of atomic op
        // to count the messages and silence it for a while
        trace!(message = "Failed to parse the component name for usage metrics.", %component_name);
        return;
    }

    match tx.send(UsageMetrics {
        key: key.unwrap(),
        events: array.len(),
        total_size: array_byte_size(array),
    }) {
        Err(_) => warn!("Usage metrics channel closed"),
        _ => {}
    }
}

/// Estimates the byte size of a group of events.
pub fn array_byte_size(array: &EventArray) -> usize {
    match array {
        // For logs: for each fields of a log event, add byte size of key and value.
        EventArray::Logs(a) => a
            .iter()
            .map(|l| {
                if let Some(fields) = l.all_fields() {
                    fields.map(|(k, v)| k.len() + value_size(v)).sum()
                } else {
                    0
                }
            })
            .sum(),
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
        // We currently don't support traces types, leave it as an oversized estimation
        EventArray::Traces(a) => a.allocated_bytes(),
    }
}

fn value_size(v: &Value) -> usize {
    match v {
        Value::Bytes(v) => v.len(),
        Value::Boolean(_) => 1,
        Value::Timestamp(_) => 8,
        Value::Integer(_) => 8,
        Value::Float(_) => 8,
        Value::Regex(_) => mem::size_of_val(v),
        Value::Object(v) => v.iter().map(|(k, v)| k.len() + value_size(v)).sum(),
        Value::Array(v) => v.iter().map(value_size).sum(),
        Value::Null => 4,
    }
}

/// Estimate the value of the metric, we currently don't have the metrics supported defined
/// but it's better to have something in-place for now.
fn metric_value_size(v: &MetricValue) -> usize {
    match v {
        MetricValue::Counter { .. } => 8,
        MetricValue::Gauge { .. } => 8,
        MetricValue::Set { values } => values.iter().map(|v| v.len()).sum::<usize>(),
        MetricValue::AggregatedHistogram { buckets, .. } => {
            12 + buckets.iter().map(|_| 12).sum::<usize>()
        }
        MetricValue::AggregatedSummary { quantiles, .. } => {
            12 + quantiles.iter().map(|_| 16).sum::<usize>()
        }
        MetricValue::Distribution { samples, .. } => samples.iter().map(|_| 12).sum::<usize>(),
        _ => v.allocated_bytes(),
    }
}

pub fn track_output_usage(
    tx: &UnboundedSender<UsageMetrics>,
    events_count: usize,
    total_size: usize,
    output_name: String,
) {
    let key = if output_name == UNMATCHED_ROUTE {
        UsageMetricsKey::new(
            "shared".into(),
            "shared".into(),
            "swimlane_unmatched".into(),
            ComponentKind::Source,
        )
    } else {
        // Expected format is '${component_id}:${pipeline_id}:${account_id}'
        let parts: Vec<&str> = output_name.split(":").collect();
        if parts.len() != 3 {
            // This is only called from our specific internal route transform
            trace!(message = "Failed to parse the route name for usage metrics.", %output_name);
            return;
        }
        UsageMetricsKey::new(
            parts[2].into(),
            parts[1].into(),
            parts[0].into(),
            ComponentKind::Source,
        )
    };

    match tx.send(UsageMetrics {
        key,
        events: events_count,
        total_size,
    }) {
        Err(_) => warn!("Usage metrics channel closed"),
        _ => {}
    }
}

pub async fn start_publishing_metrics(
    rx: UnboundedReceiver<UsageMetrics>,
) -> Result<(), MetricsPublishingError> {
    let agg_window = match std::env::var("USAGE_METRICS_FLUSH_INTERVAL_SECS") {
        Ok(interval_str) => {
            let secs = interval_str.parse().unwrap_or_else(|_| {
                warn!(
                    "USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable invalid, using default"
                );
                DEFAULT_FLUSH_INTERVAL_SECS
            });
            Duration::from_secs(secs)
        }
        Err(_) => {
            info!("USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable not set, using default");
            Duration::from_secs(DEFAULT_FLUSH_INTERVAL_SECS)
        }
    };

    let flusher = get_flusher().await?;
    start_publishing_metrics_with_flusher(rx, agg_window, flusher);
    Ok(())
}

async fn get_flusher() -> Result<Arc<dyn MetricsFlusher + Send>, MetricsPublishingError> {
    let endpoint_url = env::var("MEZMO_METRICS_DB_URL").ok();
    let pod_name = env::var("POD_NAME").unwrap_or("not-set".to_string());

    if let Some(endpoint_url) = endpoint_url {
        return Ok(Arc::new(
            DbFlusher::new(endpoint_url, &pod_name)
                .await
                .map_err(|_| MetricsPublishingError::FlusherError)?,
        ));
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
                    biased; // ensure the timeout is handled promptly
                    _ = &mut timeout => {
                        break;
                    },
                    Some(message) = rx.recv() => {
                        let mut value = aggregated.entry(message.key).or_default();
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
    // use mockall::mock;
    use crate::event::LogEvent;
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
        ($val: expr, $account_id: expr, $pipeline_id: expr, $comp_id: expr, $kind: expr) => {
            let key: Result<UsageMetricsKey, ParseError> = $val.parse();
            assert!(matches!(key, Ok(_)), "parse failed for {}", $val);
            assert_eq!(
                key.expect("to be ok"),
                UsageMetricsKey::new(
                    $account_id.into(),
                    $pipeline_id.into(),
                    $comp_id.into(),
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
            ComponentKind::Source
        );
        assert_parse_ok!(
            "v1:internal_source:metrics:shared",
            "shared",
            "shared",
            "metrics",
            ComponentKind::Source
        );
        assert_parse_ok!(
            "v1:kafka:internal_source:c1:p1:a1",
            "a1",
            "p1",
            "c1",
            ComponentKind::Source
        );
        assert_parse_ok!(
            "v1:internal_sink:metrics:shared",
            "shared",
            "shared",
            "metrics",
            ComponentKind::Sink
        );
        assert_parse_ok!(
            "v1:mezmo:sink:c1:p1:a1",
            "a1",
            "p1",
            "c1",
            ComponentKind::Sink
        );
        assert_parse_ok!(
            "v1:http:sink:c2:p1:a1",
            "a1",
            "p1",
            "c2",
            ComponentKind::Sink
        );
    }

    #[test]
    fn array_byte_size_log_simple_test() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("k1".into(), 1u8.into());
        event_map.insert("k2".into(), 1.into());
        event_map.insert("k3".into(), 1f64.into());
        event_map.insert("k4".into(), "abcd".into());
        let event: LogEvent = event_map.into();
        assert_eq!(array_byte_size(&event.into()), 36);
    }

    #[test]
    fn array_byte_size_log_nested_test() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        let mut nested_map: BTreeMap<String, Value> = BTreeMap::new();
        nested_map.insert("prop1".into(), 1u64.into());
        nested_map.insert("prop2".into(), "a".into());
        event_map.insert("k1".into(), Value::from(nested_map));
        let event: LogEvent = event_map.into();
        assert_eq!(array_byte_size(&event.into()), 25);
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
            component_kind: ComponentKind::Source,
        };

        let key2 = UsageMetricsKey {
            account_id: "d".into(),
            pipeline_id: "e".into(),
            component_id: "f".into(),
            component_kind: ComponentKind::Source,
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

    #[test]
    fn track_usage_should_enqueue_events_when_key_can_be_parsed() {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("k1".into(), 1u8.into());
        let event: LogEvent = event_map.into();
        let (tx, mut rx) = mpsc::unbounded_channel::<UsageMetrics>();

        track_usage(&tx, &event.clone().into(), "v1:mezmo:sink:c1:p1:a1".into());
        track_usage(
            &tx,
            &event.clone().into(),
            "v1:internal_source:metrics:shared".into(),
        );
        track_usage(
            &tx,
            &event.clone().into(),
            "v1:kafka:internal_source:c1:p1:a1".into(),
        );

        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
        assert!(
            rx.try_recv().is_err(),
            "no more items should be in the channel"
        );

        track_usage(&tx, &event.clone().into(), "invalid_name".into());
        assert!(
            rx.try_recv().is_err(),
            "event with invalid key should not be tracked"
        );
    }
}
