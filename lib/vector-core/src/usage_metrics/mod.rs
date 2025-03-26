use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::iter::Sum;
use std::sync::OnceLock;
use std::time::Instant;
use std::{collections::HashMap, env, str::FromStr, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};
use vector_common::{
    byte_size_of::ByteSizeOf,
    internal_event::{emit, usage_metrics::AggregatedProfileChanged},
};

use vrl::value::{KeyString, Value};

use crate::{
    config::log_schema,
    event::EventArray,
    event::{array::EventContainer, MetricValue},
    usage_metrics::flusher::HttpFlusher,
};
use flusher::{DbFlusher, MetricsFlusher, StdErrFlusher};

use self::flusher::NoopFlusher;

const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 20;
const DEFAULT_PROFILE_FLUSH_INTERVAL: Duration = Duration::from_secs(60);
const BASE_ARRAY_SIZE: usize = 8; // Add some overhead to the array and object size
const BASE_BTREE_SIZE: usize = 8;
static INTERNAL_TRANSFORM: ComponentKind = ComponentKind::Transform { internal: true };

mod flusher;
mod integration_tests;

/// Represents an aggregated view of events per component for billing and profiling.
#[derive(Debug)]
pub struct UsageMetrics {
    key: UsageMetricsKey,
    billing: Option<UsageMetricsValue>,
    usage_by_annotation: Option<AnnotationMap>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidRoot,
    InvalidFormat,
}

#[derive(Debug)]
pub enum MetricsPublishingError {
    FlusherError,
    DbEndpointUrlNotSet,
    DbEndpointUrlInvalid,
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

#[derive(Debug, Default, Serialize, PartialEq)]
pub(crate) struct UsageMetricsValue {
    /// Total number of events
    total_count: usize,
    /// Total size in bytes
    total_size: usize,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Serialize)]
pub struct UsageMetricsKey {
    account_id: String,
    /// Determines the pipeline id of the component. When None, it represents a component that
    /// is not part of a pipeline config, e.g., analysis phase.
    pipeline_id: Option<String>,
    /// Component id as a string (not uuid) as it might contain the output name
    /// and other separators.
    component_id: String,
    component_type: String,
    component_kind: ComponentKind,
}

impl UsageMetricsKey {
    fn new(
        account_id: String,
        pipeline_id: Option<String>,
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

    /// Determines whether the component with this key should be tracked for profiling.
    /// Currently only during the analysis phase.
    fn is_tracked_for_profiling(&self) -> bool {
        // The absence of a pipeline id means that the component is part of the analysis phase
        if self.pipeline_id.is_some() {
            return false;
        }

        let ComponentKind::Transform { internal } = self.component_kind else {
            return false;
        };

        // log classification should be tracked for both
        // shared source usage and transform profiling
        !internal
            && (self.component_type == "mezmo_log_classification"
                || self.component_type == "data-profiler")
    }

    /// Determines whether the component with this key should be tracked for billing
    fn is_tracked_for_billing(&self) -> bool {
        if self.pipeline_id.is_none() {
            return false;
        }

        match self.component_kind {
            ComponentKind::Source { internal } => {
                // Internal sources should not be tracked, remap transforms will be tracked instead
                !internal
            }
            ComponentKind::Sink => true,
            ComponentKind::Transform { internal } => {
                // Only track source format transforms and the swimlanes
                internal
                    && self.component_type == "remap"
                    && self.pipeline_id != Some("shared".into())
            }
        }
    }

    /// Determines whether the component with this key should be tracked for profiling or billing
    fn is_tracked(&self) -> bool {
        self.is_tracked_for_billing() || self.is_tracked_for_profiling()
    }

    /// Gets the target key to use to track.
    /// For sinks and most sources, it's the same key
    /// For remap internal transforms, it's the source key
    fn target_key(&self) -> UsageMetricsKey {
        if self.component_kind == INTERNAL_TRANSFORM
            && self.component_type == "remap"
            && self.pipeline_id != Some("shared".into())
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
                Some("shared".into()),
                type_name.into(),
                type_name.into(),
                kind,
            ));
        }
        let (account_id, pipeline_id) = if parts.len() == 6 {
            // Expected format is 'v1:{type}:{kind}:${component_id}:${pipeline_id}:${account_id}'
            (parts[5].into(), Some(parts[4].into()))
        } else if parts.len() == 5 {
            (parts[4].into(), None)
        } else {
            return Err(Self::Err::InvalidFormat);
        };

        Ok(UsageMetricsKey::new(
            account_id,
            pipeline_id,
            parts[3].into(),
            parts[1].into(),
            parts[2].parse()?,
        ))
    }
}

/// A set of annotation keys and values (allow listing).
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AnnotationSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    log_type: Option<String>,
}

impl AnnotationSet {
    pub fn is_empty(&self) -> bool {
        self.app.is_none() && self.host.is_none() && self.level.is_none() && self.log_type.is_none()
    }
}

type AnnotationMap = HashMap<AnnotationSet, UsageMetricsValue>;

/// Represents aggregated size and count information for events
#[derive(Debug, Default)]
pub struct UsageProfileValue {
    /// Total number of events
    total_count: usize,
    /// Total size in bytes
    total_size: usize,
    /// A map of annotation keys with count and size as values
    usage_by_annotation: AnnotationMap,
}

impl Sum<UsageProfileValue> for UsageProfileValue {
    fn sum<I: Iterator<Item = UsageProfileValue>>(iter: I) -> Self {
        iter.fold(Default::default(), |a, b| {
            let mut usage_by_annotation = a.usage_by_annotation;
            for (k, v) in b.usage_by_annotation {
                add_annotation_value(&mut usage_by_annotation, k, &v);
            }

            UsageProfileValue {
                total_count: a.total_count + b.total_count,
                total_size: a.total_size + b.total_size,
                usage_by_annotation,
            }
        })
    }
}

/// Sends the pipeline-based usage metrics to the provided channel.
fn track_usage(
    tx: &UnboundedSender<UsageMetrics>,
    key: &UsageMetricsKey,
    usage: UsageProfileValue,
) {
    let mut billing = None;
    let mut usage_by_annotation = None;

    if key.is_tracked_for_billing() {
        billing = Some(UsageMetricsValue {
            total_count: usage.total_count,
            total_size: usage.total_size,
        });
    }

    if is_profile_enabled()
        && key.is_tracked_for_profiling()
        && !usage.usage_by_annotation.is_empty()
    {
        usage_by_annotation = Some(usage.usage_by_annotation);
    }

    if billing.is_none() && usage_by_annotation.is_none() {
        // Ignore
        return;
    }

    if tx
        .send(UsageMetrics {
            key: key.clone(),
            billing,
            usage_by_annotation,
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
                // Use the target key (e.g. source), not the original key (e.g. source format)
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
    fn track_output(&self, usage: UsageProfileValue);
    fn get_size_and_profile(&self, array: &EventArray) -> UsageProfileValue;
}

struct NoopTracker {}

impl ComponentUsageTracker for NoopTracker {
    fn track(&self, _array: &EventArray) {
        // Do nothing
    }
}

impl OutputUsageTracker for NoopTracker {
    fn track_output(&self, _usage: UsageProfileValue) {
        // Do nothing
    }

    fn get_size_and_profile(&self, _array: &EventArray) -> UsageProfileValue {
        Default::default()
    }
}

struct DefaultComponentTracker {
    metrics_tx: UnboundedSender<UsageMetrics>,
    key: UsageMetricsKey,
}

impl ComponentUsageTracker for DefaultComponentTracker {
    fn track(&self, array: &EventArray) {
        let usage = get_size_and_profile(array);
        track_usage(&self.metrics_tx, &self.key, usage);
    }
}

struct DefaultOutputTracker {
    metrics_tx: UnboundedSender<UsageMetrics>,
    target_key: UsageMetricsKey,
}

impl OutputUsageTracker for DefaultOutputTracker {
    fn track_output(&self, usage: UsageProfileValue) {
        track_usage(&self.metrics_tx, &self.target_key, usage);
    }

    fn get_size_and_profile(&self, array: &EventArray) -> UsageProfileValue {
        get_size_and_profile(array)
    }
}

/// Sums the count and size by set, returning true when a new key is added
fn add_annotation_value(
    map: &mut AnnotationMap,
    key: AnnotationSet,
    value: &UsageMetricsValue,
) -> bool {
    let mut new_entry = false;
    let entry_value = match map.entry(key) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(k) => {
            new_entry = true;
            k.insert(Default::default())
        }
    };
    entry_value.total_count += value.total_count;
    entry_value.total_size += value.total_size;
    new_entry
}

/// Estimates the byte size of a group of events according to Mezmo billing practices, documented in the ADR
/// <https://github.com/answerbook/pipeline-prototype/blob/main/doc/adr/0018-billing-ingress-egress.md>
fn get_size_and_profile(array: &EventArray) -> UsageProfileValue {
    match array {
        EventArray::Logs(a) => {
            let total_count = array.len();
            let mut total_size = 0;
            let mut usage_by_annotation = AnnotationMap::new();
            for log_event in a {
                if let Some(fields) = log_event.as_map() {
                    let size = log_event_size(fields, include_metadata_in_size());
                    total_size += size;

                    if let Some(annotation_set) = get_annotations(fields) {
                        add_annotation_value(
                            &mut usage_by_annotation,
                            annotation_set,
                            &UsageMetricsValue {
                                total_count: 1,
                                total_size: size,
                            },
                        );
                    }
                }
            }

            UsageProfileValue {
                total_count,
                total_size,
                usage_by_annotation,
            }
        }
        // In the Mezmo pipeline, we only use the metrics type for internal metrics
        // User provided metric events are represented in vector as logs (NOT metrics)
        // For metrics: add byte size of the name, timestamp and value.
        EventArray::Metrics(a) => {
            let mut total_count = 0;
            let mut total_size = 0;
            for m in a {
                let series = m.series();
                let data = m.data();

                total_size += series.name().allocated_bytes();
                if data.timestamp().is_some() {
                    total_size += 8;
                }
                total_size += metric_value_size(data.value());
                total_count += 1;
            }

            UsageProfileValue {
                total_count,
                total_size,
                usage_by_annotation: Default::default(),
            }
        }
        // We currently don't support Traces type, leave it as an oversized estimation
        EventArray::Traces(a) => UsageProfileValue {
            total_count: a.len(),
            total_size: a.allocated_bytes(),
            usage_by_annotation: Default::default(),
        },
    }
}

/// Determines whether the feature for tracking usage annotations is enabled.
fn is_profile_enabled() -> bool {
    // Inaccessible outside of the function but it isn't dropped at the end of the function
    static IS_ENABLED: OnceLock<bool> = OnceLock::new();

    *IS_ENABLED.get_or_init(|| {
        env::var("USAGE_METRICS_PROFILE_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false)
    })
}

/// Determines whether we track size from `.metadata` or not.
/// TRUE by default.
pub fn include_metadata_in_size() -> bool {
    // Inaccessible outside of the function but it isn't dropped at the end of the function
    static SHOULD_TRACK_METADATA: OnceLock<bool> = OnceLock::new();

    *SHOULD_TRACK_METADATA.get_or_init(|| {
        env::var("USAGE_METRICS_TRACK_METADATA_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true)
    })
}

pub fn get_annotations(fields: &BTreeMap<KeyString, Value>) -> Option<AnnotationSet> {
    let annotations_field = fields.get(log_schema().annotations_key())?.as_object()?;

    let set = AnnotationSet {
        app: get_string_field(annotations_field, "app"),
        host: get_string_field(annotations_field, "host"),
        level: get_string_field(annotations_field, "level"),
        log_type: get_log_type(annotations_field),
    };

    // Annotations can be defined without the relevant properties
    if set.is_empty() {
        None
    } else {
        Some(set)
    }
}

fn get_string_field(fields: &BTreeMap<KeyString, Value>, key: &str) -> Option<String> {
    let bytes = fields.get(key)?.as_bytes();
    std::str::from_utf8(bytes?)
        .map(std::string::ToString::to_string)
        .ok()
}

fn get_log_type(fields: &BTreeMap<KeyString, Value>) -> Option<String> {
    // Log type is stored as `classification.event_types = {"MY_LOG_TYPE": 1}`
    let classification = fields.get("classification")?.as_object()?;
    let event_types = classification.get("event_types")?.as_object()?;
    let (log_type, _) = event_types.first_key_value()?;

    Some(log_type.to_string())
}

/// Estimate the byte size of a single [Value]
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

/// Estimate the byte size of all fields within an event, accounting for
/// both the value of ".message" and ".metadata" top-level fields.
pub fn log_event_size(event_map: &BTreeMap<KeyString, Value>, include_metadata: bool) -> usize {
    let mut size = event_map
        .get::<KeyString>(&log_schema().message_key().unwrap().to_string().into())
        .map_or(0, value_size);

    if include_metadata {
        size += event_map
            .get::<KeyString>(&log_schema().user_metadata_key().into())
            .map_or(0, value_size);
    }

    size
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
    let agg_window = if let Ok(interval_str) = env::var("USAGE_METRICS_FLUSH_INTERVAL_SECS") {
        let secs = interval_str.parse().unwrap_or_else(|_| {
            warn!("USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable invalid, using default");
            DEFAULT_FLUSH_INTERVAL_SECS
        });
        Duration::from_secs(secs)
    } else {
        info!("USAGE_METRICS_FLUSH_INTERVAL_SECS environment variable not set, using default");
        Duration::from_secs(DEFAULT_FLUSH_INTERVAL_SECS)
    };

    let profile_flush_interval = env::var("USAGE_METRICS_PROFILE_FLUSH_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse().map(Duration::from_secs).ok())
        .unwrap_or(DEFAULT_PROFILE_FLUSH_INTERVAL);

    let flusher = get_flusher(agg_window).await?;
    start_publishing_metrics_with_flusher(rx, agg_window, profile_flush_interval, flusher);
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
            DbFlusher::new(&pod_name)
                .await
                .map_err(|_| MetricsPublishingError::FlusherError)?,
        ));
    }

    if let Some(endpoint_url) = endpoint_url {
        // Http endpoint used by Pulse
        let auth_token = env::var("MEZMO_LOCAL_DEPLOY_AUTH_TOKEN").ok();
        let headers = if let Some(token) = auth_token {
            HashMap::from([("Authorization".into(), format!("Token {token}"))])
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
    profile_agg_window: Duration,
    flusher: Arc<dyn MetricsFlusher + Send>,
) {
    tokio::spawn(async move {
        info!("Start publishing usage metrics");

        let mut finished = false;
        let mut aggregated_profiles: HashMap<UsageMetricsKey, AnnotationMap> = HashMap::new();
        let mut profile_entries_count = 0;
        let mut start_profile = Instant::now();

        while !finished {
            let mut billing_events_count = 0;
            let mut aggregated_billing: HashMap<UsageMetricsKey, UsageMetricsValue> =
                HashMap::new();
            let timeout = sleep(agg_window);
            tokio::pin!(timeout);

            // Aggregate all messages across this `agg_window`
            loop {
                tokio::select! {
                    // Use unbiased (pseudo random) branch selection, that way we support for immediate flushes
                    () = &mut timeout => {
                        // Break the inner loop, start a new timer
                        break;
                    },
                    Some(message) = rx.recv() => {
                        if let Some(billing) = &message.billing {
                            // Billing tracking enabled for the component
                            if let Some(value) = aggregated_billing.get_mut(&message.key) {
                                // Prevent cloning with HashMap::entry() for the most common case
                                value.total_count += billing.total_count;
                                value.total_size += billing.total_size;
                            } else {
                                aggregated_billing.insert(
                                    message.key.clone(),
                                    UsageMetricsValue {
                                        total_count: billing.total_count,
                                        total_size: billing.total_size,
                                    }
                                );
                            }
                            billing_events_count += billing.total_count;
                        }

                        if let Some(usage_by_annotation) = message.usage_by_annotation {
                            // Profile/annotation tracking is enabled
                            let profile = aggregated_profiles.entry(message.key).or_default();
                            let profile_entries_initial_count = profile_entries_count;
                            for (k, v) in usage_by_annotation {
                                if add_annotation_value(profile, k, &v) {
                                    profile_entries_count += 1;
                                }
                            }

                            if profile_entries_count > profile_entries_initial_count {
                                emit(AggregatedProfileChanged {
                                    count: profile_entries_count
                                });
                            }
                        }
                    },
                    else => {
                        // Channel closed
                        finished = true;
                        break;
                    }
                }
            }

            if billing_events_count > 0 {
                // Flush aggregated metrics
                debug!(
                    "Saving {} aggregated usage metrics from {} metrics events",
                    aggregated_billing.len(),
                    billing_events_count
                );

                // Flush billing metrics in the foreground
                flusher.save_billing_metrics(aggregated_billing).await;
            }

            if start_profile.elapsed() > profile_agg_window && !aggregated_profiles.is_empty() {
                // Flush aggregated profiles
                let flusher = Arc::clone(&flusher);

                // Flush profiles in the background
                tokio::spawn(async move {
                    flusher.save_profile_metrics(aggregated_profiles).await;
                });

                // Reset the profiles
                aggregated_profiles = HashMap::new();
                profile_entries_count = 0;
                start_profile = Instant::now();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use assay::assay;
    use std::collections::{BTreeMap, HashMap};
    use tokio::sync::mpsc;
    use vrl::value::Value;

    use crate::{config::log_schema, event::LogEvent};
    use async_trait::async_trait;

    use super::*;

    // Create a manual mock
    // We can't use mockall because we need to inspect the value from the struct after the field is moved.
    struct MockMetricsFlusher {
        billing_tx: UnboundedSender<HashMap<UsageMetricsKey, UsageMetricsValue>>,
        profile_tx: UnboundedSender<HashMap<UsageMetricsKey, AnnotationMap>>,
    }

    #[async_trait]
    impl MetricsFlusher for MockMetricsFlusher {
        async fn save_billing_metrics(&self, metrics: HashMap<UsageMetricsKey, UsageMetricsValue>) {
            self.billing_tx.send(metrics).unwrap();
        }

        async fn save_profile_metrics(&self, metrics: HashMap<UsageMetricsKey, AnnotationMap>) {
            self.profile_tx.send(metrics).unwrap();
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
                    Some($pipeline_id.into()),
                    $comp_id.into(),
                    $type_name.into(),
                    $kind,
                )
            );
        };
    }

    fn track_usage_test(key: &UsageMetricsKey, event: LogEvent) -> UnboundedReceiver<UsageMetrics> {
        let (tx, rx) = mpsc::unbounded_channel::<UsageMetrics>();
        let usage_profile = get_size_and_profile(&event.into());

        track_usage(&tx, key, usage_profile);
        rx
    }

    fn annotation_path(parts: &[&str]) -> String {
        log_schema().annotations_key().to_string() + "." + parts.join(".").as_str()
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
        assert!(
            value.is_tracked_for_billing(),
            "All sinks should be tracked"
        );

        let value: UsageMetricsKey = "v1:s3:source:comp1:pipe1:account1".parse().unwrap();
        assert!(
            value.is_tracked_for_billing(),
            "Most sources should be tracked"
        );
        assert!(value.is_tracked(), "as it's tracked for billing -> true");

        let value: UsageMetricsKey = "v1:remap:internal_transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        assert!(
            value.is_tracked_for_billing(),
            "internal remap transform is tracked for billing"
        );
        assert!(
            !value.is_tracked_for_profiling(),
            "not tracked for profiling"
        );

        let value: UsageMetricsKey = "v1:mezmo_log_classification:transform:comp1:account1"
            .parse()
            .unwrap();
        assert!(value.is_tracked());
        assert!(value.is_tracked_for_profiling());
        assert!(!value.is_tracked_for_billing());

        let value: UsageMetricsKey = "v1:data-profiler:transform:comp1:account1".parse().unwrap();
        assert!(value.is_tracked());
        assert!(value.is_tracked_for_profiling());
        assert!(!value.is_tracked_for_billing());

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
        assert!(
            !value.is_tracked_for_billing(),
            "Kafka source should NOT be tracked"
        );
    }

    #[test]
    fn log_event_size_test() {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert(
            log_schema().message_key().unwrap().to_string().into(),
            "foo".into(),
        );
        event_map.insert(
            log_schema().user_metadata_key().to_string().into(),
            "foo".into(),
        );

        assert_eq!(log_event_size(&event_map, true), 6);
        assert_eq!(log_event_size(&event_map, false), 3);
    }

    #[assay(env = [("USAGE_METRICS_PROFILE_ENABLED", "true")])]
    fn track_usage_key_not_tracked_either() {
        let key: UsageMetricsKey = "v1:filter-by-field:transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert(
            log_schema().message_key().unwrap().to_string().into(),
            "foo".into(),
        );
        let event: LogEvent = event_map.into();

        let mut rx = track_usage_test(&key, event);
        assert!(rx.try_recv().is_err());
    }

    #[assay(env = [("USAGE_METRICS_PROFILE_ENABLED", "true")])]
    fn track_usage_key_billing_only() {
        let key: UsageMetricsKey = "v1:remap:internal_transform:comp1:pipe1:account1"
            .parse()
            .unwrap();
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert(
            log_schema().message_key().unwrap().to_string().into(),
            "the message".into(),
        );

        let mut event: LogEvent = event_map.into();
        event.insert(annotation_path(vec!["app"].as_ref()).as_str(), "app-1");

        let mut rx = track_usage_test(&key, event);
        let tracked = rx.try_recv();
        assert!(tracked.is_ok());

        let tracked = tracked.unwrap();
        assert_eq!(tracked.key, key, "should be the same key");
        assert!(tracked.billing.is_some(), "should contain billing metrics");
        assert!(
            tracked.usage_by_annotation.is_none(),
            "should NOT contain profiling metrics"
        );
    }

    #[assay(env = [("USAGE_METRICS_PROFILE_ENABLED", "true")])]
    fn track_usage_key_profiling_only() {
        let component_ids = vec![
            "v1:mezmo_log_classification:transform:comp1:account1",
            "v1:data-profiler:transform:comp2:account1",
        ];

        for component_id in component_ids {
            let key: UsageMetricsKey = component_id.parse().unwrap();
            let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
            event_map.insert(
                log_schema().message_key().unwrap().to_string().into(),
                "the message".into(),
            );

            let mut event: LogEvent = event_map.into();
            event.insert(annotation_path(vec!["app"].as_ref()).as_str(), "app-1");

            let mut rx = track_usage_test(&key, event);
            let tracked = rx.try_recv();
            assert!(tracked.is_ok());

            let tracked = tracked.unwrap();
            assert_eq!(tracked.key, key, "should be the same key");
            assert!(
                tracked.billing.is_none(),
                "should NOT contain billing metrics"
            );
            assert!(
                tracked.usage_by_annotation.is_some(),
                "should contain profiling metrics"
            );
        }
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
    fn get_annotations_empty_annotations_return_none() {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert(
            log_schema().annotations_key().into(),
            Value::Object(BTreeMap::new()),
        );
        assert!(get_annotations(&event_map).is_none());
    }

    #[test]
    fn annotation_set_none_properties_are_not_serialized() {
        let set = AnnotationSet::default();
        assert!(set.is_empty());
        let str_json = serde_json::to_string(&set).unwrap();
        assert_eq!(
            str_json, "{}",
            "Should not include any empty property by default"
        );
    }

    #[test]
    fn get_size_and_profile_log_message_number_test() {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert("this_is_ignored".into(), 1u8.into());
        event_map.insert("another_ignored".into(), 1.into());
        event_map.insert(
            KeyString::from(log_schema().message_key().unwrap().to_string()),
            9.into(),
        );
        let event: LogEvent = event_map.into();
        let usage_profile = get_size_and_profile(&event.into());
        assert_eq!(
            usage_profile.total_size, 8,
            "only accounts for the value of message"
        );
    }

    #[test]
    fn get_size_and_profile_log_message_and_meta_test() {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        event_map.insert("this_is_ignored".into(), 2.into());
        event_map.insert(
            KeyString::from(log_schema().message_key().unwrap().to_string()),
            "hello ".into(),
        );
        event_map.insert(log_schema().user_metadata_key().into(), "world".into());
        let event: LogEvent = event_map.into();
        assert_eq!(
            get_size_and_profile(&event.into()).total_size,
            "hello world".len(),
            "value of message and meta"
        );
    }

    #[test]
    fn get_size_and_profile_log_nested_test() {
        let mut event_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        let mut nested_map: BTreeMap<KeyString, Value> = BTreeMap::new();
        nested_map.insert("prop1".into(), 1u64.into());
        nested_map.insert("prop2".into(), 1u8.into());
        nested_map.insert("prop3".into(), 1i32.into());
        nested_map.insert("prop4".into(), "abcd".into());
        event_map.insert(
            KeyString::from(log_schema().message_key().unwrap().to_string()),
            Value::from(nested_map),
        );
        let event: LogEvent = event_map.into();
        assert_eq!(
            get_size_and_profile(&event.into()).total_size,
            BASE_BTREE_SIZE + "propX".len() * 4 + 28
        );
    }

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn start_publishing_metrics_should_aggregate_and_publish_metrics_periodically() {
        let (metrics_tx, rx) = mpsc::unbounded_channel::<UsageMetrics>();

        let (billing_tx, mut billing_rx) =
            mpsc::unbounded_channel::<HashMap<UsageMetricsKey, UsageMetricsValue>>();
        let (profile_tx, mut profile_rx) =
            mpsc::unbounded_channel::<HashMap<UsageMetricsKey, AnnotationMap>>();
        let flusher = MockMetricsFlusher {
            billing_tx,
            profile_tx,
        };

        let key1 = UsageMetricsKey {
            account_id: "a".into(),
            pipeline_id: Some("b".into()),
            component_id: "c".into(),
            component_type: "d".into(),
            component_kind: ComponentKind::Source { internal: false },
        };

        let key2 = UsageMetricsKey {
            account_id: "d".into(),
            pipeline_id: Some("e".into()),
            component_id: "f".into(),
            component_type: "g".into(),
            component_kind: ComponentKind::Source { internal: false },
        };

        metrics_tx
            .send(UsageMetrics {
                key: key1.clone(),
                billing: Some(UsageMetricsValue {
                    total_count: 2,
                    total_size: 10,
                }),
                usage_by_annotation: Some(HashMap::from([(
                    AnnotationSet {
                        app: Some("app1".into()),
                        host: Some("host1".into()),
                        level: None,
                        log_type: Some("HTTPD_ERRORLOG".into()),
                    },
                    UsageMetricsValue {
                        total_count: 1,
                        total_size: 5,
                    },
                )])),
            })
            .unwrap();
        metrics_tx
            .send(UsageMetrics {
                key: key1.clone(),
                billing: Some(UsageMetricsValue {
                    total_count: 4,
                    total_size: 30,
                }),
                usage_by_annotation: Some(HashMap::from([
                    (
                        AnnotationSet {
                            app: Some("app1".into()),
                            host: Some("host1".into()),
                            level: None,
                            log_type: Some("HTTPD_ERRORLOG".into()),
                        },
                        UsageMetricsValue {
                            total_count: 3,
                            total_size: 20,
                        },
                    ),
                    (
                        AnnotationSet {
                            app: Some("app2".into()),
                            host: None,
                            level: None,
                            log_type: None,
                        },
                        UsageMetricsValue {
                            total_count: 1,
                            total_size: 10,
                        },
                    ),
                ])),
            })
            .unwrap();
        metrics_tx
            .send(UsageMetrics {
                key: key2.clone(),
                billing: Some(UsageMetricsValue {
                    total_count: 1,
                    total_size: 123,
                }),
                usage_by_annotation: None,
            })
            .unwrap();

        start_publishing_metrics_with_flusher(
            rx,
            Duration::from_millis(20),
            Duration::from_millis(100),
            Arc::new(flusher),
        );

        // It should publish 2 aggregated metrics
        let m = billing_rx.recv().await.unwrap();
        assert_eq!(m.len(), 2);
        let v = m.get(&key1).unwrap();
        assert_eq!(v.total_count, 6);
        assert_eq!(v.total_size, 40);

        let v = m.get(&key2).unwrap();
        assert_eq!(v.total_count, 1);
        assert_eq!(v.total_size, 123);

        let profiles = profile_rx.recv().await.expect("to receive a profile map");
        let v = profiles.get(&key1).unwrap();
        assert_eq!(
            v,
            &HashMap::from([
                (
                    AnnotationSet {
                        app: Some("app1".into()),
                        host: Some("host1".into()),
                        level: None,
                        log_type: Some("HTTPD_ERRORLOG".into()),
                    },
                    UsageMetricsValue {
                        total_count: 4,
                        total_size: 25,
                    },
                ),
                (
                    AnnotationSet {
                        app: Some("app2".into()),
                        host: None,
                        level: None,
                        log_type: None,
                    },
                    UsageMetricsValue {
                        total_count: 1,
                        total_size: 10,
                    },
                ),
            ])
        );
    }
}
