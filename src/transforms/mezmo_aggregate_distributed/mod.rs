use crate::internal_events::{
    MezmoAggregateDistributedEventRecorded, MezmoAggregateDistributedFlushFailed,
    MezmoAggregateDistributedFlushed, MezmoAggregateDistributedRecordFailed,
    MezmoAggregateDistributedRecordRetried,
};
use crate::sinks::util::retries::ExponentialBackoff;
use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use mezmo::{user_trace::handle_transform_error, MezmoContext};
use once_cell::sync::Lazy;
use redis::AsyncCommands;
use redis::{aio::ConnectionManager, RedisError, RedisResult, Script};
use serde::{Deserialize, Serialize};
use snafu::futures::TryFutureExt;
use snafu::Snafu;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::time::Duration;
use tokio::{select, time::sleep};
use vector_lib::config::log_schema;
use vector_lib::configurable::configurable_component;
use vector_lib::event::metric::mezmo::TransformError;
use vector_lib::event::{LogEvent, Metric};
use vector_lib::{
    event::{metric::mezmo::to_metric, Event, MetricValue},
    transform::TaskTransform,
};
use vrl::value::{KeyString, Value};

mod config;
use config::MezmoAggregateDistributedConfig;

#[cfg(feature = "mezmo-aggregate-distributed-integration-tests")]
#[cfg(test)]
pub(crate) mod integration_tests;

static SUM_SCRIPT: Lazy<Script> = Lazy::new(|| Script::new(include_str!("redis/sum.lua")));
static FLUSH_SCRIPT: Lazy<Script> = Lazy::new(|| Script::new(include_str!("redis/flush.lua")));

/// Configuration for a strategy
#[configurable_component]
#[configurable(metadata(docs::enum_tag_description = "The aggregation strategy."))]
#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub(super) enum Strategy {
    /// Sum numeric values
    #[derivative(Default)]
    Sum,

    /// Average numeric values
    Avg,
}

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::Sum => write!(f, "sum"),
            Strategy::Avg => write!(f, "avg"),
        }
    }
}

impl Strategy {
    pub fn script(&self) -> &'static Script {
        match self {
            Strategy::Sum => &SUM_SCRIPT,
            Strategy::Avg => &SUM_SCRIPT,
        }
    }
}

#[derive(Debug, Snafu)]
pub(super) enum AggregateError {
    #[snafu(display("Creating Redis client failed: {}", source))]
    RedisCreateFailed { source: RedisError },

    #[snafu(display("Error recording event value: {}", source))]
    RecordFailed { source: RedisError },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FlushedWindow {
    count: u32,
    #[serde(deserialize_with = "deserialize_json_string")]
    fields: serde_json::Value,
    value: f64,
    window_end_ts: u64,
    window_start_ts: u64,
}

fn deserialize_json_string<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

pub struct MezmoAggregateDistributed {
    conn: ConnectionManager,
    config: MezmoAggregateDistributedConfig,
    mezmo_ctx: MezmoContext,
}

impl MezmoAggregateDistributed {
    const fn new(
        conn: ConnectionManager,
        config: MezmoAggregateDistributedConfig,
        mezmo_ctx: MezmoContext,
    ) -> Self {
        Self {
            conn,
            config,
            mezmo_ctx,
        }
    }

    /// Key for the zset of active windows.
    fn get_active_windows_key(&self) -> String {
        format!(
            "aggregate:{}:{{{}}}",
            self.config.strategy, self.mezmo_ctx.component_id
        )
    }

    /// Key for this event window hash
    fn get_event_window_key(&self, hash: u64, timestamp: i64) -> String {
        format!(
            "aggregate:{}:{{{}}}:{}:{}",
            self.config.strategy, self.mezmo_ctx.component_id, hash, timestamp
        )
    }

    /// Generates a hashed code based on the root metric event fields. The fields
    /// are returned alongside their hash and are used to form the output event.
    fn get_event_fields(&self, event: &Metric) -> (u64, Value) {
        let mut hasher = DefaultHasher::new();

        let mut fields: BTreeMap<KeyString, Value> = BTreeMap::new();
        let kind = event.kind();
        let name = event.name();
        let namespace = event.namespace();
        let tags = event.tags();

        kind.hash(&mut hasher);
        name.hash(&mut hasher);
        namespace.hash(&mut hasher);
        tags.hash(&mut hasher);

        fields.insert("kind".to_string().into(), kind.into());
        fields.insert("name".to_string().into(), name.into());
        fields.insert("namespace".to_string().into(), namespace.into());

        if let Some(tags) = tags {
            let tags: BTreeMap<KeyString, Value> =
                tags.iter_all().map(|(k, v)| (k.into(), v.into())).collect();
            fields.insert("tags".to_string().into(), Value::from(tags));
        } else {
            fields.insert("tags".to_string().into(), Value::Null);
        }

        (hasher.finish(), fields.into())
    }

    /// Aligns a timestamp to the closest tumbling window boundary based on the config.
    fn align_window_timestamp(&self, timestamp: i64) -> i64 {
        timestamp - (timestamp % i64::from(self.config.window_duration_ms))
    }

    /// Extract the timestamp from the event based on the user configuration.
    ///
    /// Defaults to the current timestamp if the field/value is not present.
    fn get_event_timestamp(&self, event: &Metric) -> i64 {
        let event_timestamp = event.timestamp();

        match event_timestamp {
            Some(dt) => dt.timestamp_millis(),
            _ => Utc::now().timestamp_millis(),
        }
    }

    /// Evaluates and records the value from the event against the datastore.
    async fn record(&mut self, event: &Metric) -> Result<(), AggregateError> {
        let (hash, fields) = self.get_event_fields(event);
        let event_ts = self.get_event_timestamp(event);
        let window_start_ts = self.align_window_timestamp(event_ts);
        let active_windows_key = self.get_active_windows_key();
        let event_window_key = self.get_event_window_key(hash, window_start_ts);

        let mut conn = self.conn.clone();

        match self.config.strategy {
            Strategy::Sum | Strategy::Avg => {
                let value: f64 = match event.value() {
                    MetricValue::Counter { value } => *value,
                    MetricValue::Gauge { value } => *value,
                    // TODO: consider other metric types for sum/avg?
                    _ => {
                        let err = TransformError::InvalidMetricType {
                            type_name: event.value().to_string(),
                        };

                        emit!(MezmoAggregateDistributedRecordFailed {
                            drop_reason: "Unsupported metric event",
                            err: err.to_string()
                        });

                        handle_transform_error(
                            &Some(self.mezmo_ctx.clone()),
                            TransformError::InvalidMetricType {
                                type_name: event.value().to_string(),
                            },
                        );
                        return Ok(());
                    }
                };

                self.config
                    .strategy
                    .script()
                    .key(active_windows_key)
                    .key(event_window_key)
                    .arg(window_start_ts)
                    .arg(self.config.window_duration_ms)
                    .arg(self.config.key_expiry_grace_period_ms)
                    .arg(encode_json(&fields))
                    .arg(value)
                    .invoke_async(&mut conn)
                    .context(RecordFailedSnafu)
                    .await?;
            }
        }

        Ok(())
    }

    /// Records the value from the event against the datastore with retry logic.
    /// This handlees the case where a [[ConnectionManager]] client instance is being
    /// destroyed/recreated.
    async fn record_with_retry(&mut self, event: &Metric) -> Result<(), AggregateError> {
        let mut backoff = ExponentialBackoff::from_millis(2)
            .factor(self.config.connection_retry_factor_ms)
            .max_delay(Duration::from_millis(
                self.config.connection_retry_max_delay_ms,
            ));

        let mut attempt = 0;
        loop {
            match self.record(event).await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.connection_retry_count {
                        return Err(err);
                    }

                    let delay = backoff.next().unwrap();
                    emit!(MezmoAggregateDistributedRecordRetried {
                        attempt,
                        delay_ms: delay.as_millis()
                    });

                    sleep(delay).await;
                }
            }
        }
    }

    /// Flush the aggregated data from the datastore and clears the
    /// window state.
    async fn flush_finalized(&self, output: &mut Vec<Event>) {
        let active_windows_key = self.get_active_windows_key();
        let mut conn = self.conn.clone();

        let result: RedisResult<Vec<String>> = conn
            .zrangebyscore(&active_windows_key, "0", Utc::now().timestamp_millis())
            .await;

        let expired_window_keys = match result {
            Ok(keys) => {
                if keys.is_empty() {
                    return;
                }
                keys
            }
            Err(err) => {
                emit!(MezmoAggregateDistributedFlushFailed {
                    err: err.to_string()
                });
                return;
            }
        };

        let message_path = log_schema()
            .message_key_target_path()
            .expect("message key to always be defined");

        let mut invocation = FLUSH_SCRIPT.prepare_invoke();
        invocation.key(active_windows_key);
        for key in expired_window_keys {
            invocation.key(key);
        }

        let result: RedisResult<String> = invocation.invoke_async(&mut conn).await;
        match result {
            Ok(resp) => {
                let flushed: Vec<FlushedWindow> =
                    serde_json::from_str(&resp).expect("script response is valid JSON");

                let event_count = flushed
                    .len()
                    .try_into()
                    .expect("usize didn't fit in u64, are we on 32-bit?");

                for flushed_window in flushed {
                    let mut log: LogEvent = flushed_window
                        .fields
                        .try_into()
                        .expect("deserialized `fields` is a valid serde_json::Value::Object");

                    log.insert("value_type", self.config.strategy.to_string());
                    log.insert("window_start", flushed_window.window_start_ts);
                    log.insert("window_end", flushed_window.window_end_ts);

                    match self.config.strategy {
                        Strategy::Sum => {
                            log.insert("count", flushed_window.count);
                            log.insert("value", flushed_window.value);
                        }
                        Strategy::Avg => {
                            log.insert("count", flushed_window.count);
                            log.insert(
                                "value",
                                flushed_window.value / (flushed_window.count as f64),
                            );
                        }
                    }

                    log.rename_key(".", message_path);
                    output.push(Event::Log(log));
                }

                emit!(MezmoAggregateDistributedFlushed { event_count });
            }
            Err(error) => {
                emit!(MezmoAggregateDistributedFlushFailed {
                    err: error.to_string()
                });
            }
        }
    }
}

impl TaskTransform<Event> for MezmoAggregateDistributed {
    fn transform(
        mut self: Box<Self>,
        mut input_events: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
        Box::pin(stream! {
            let mut flush_interval = tokio::time::interval(Duration::from_millis(self.config.flush_tick_ms));
            let mut output:Vec<Event> = Vec::new();
            let mut done = false;

            while !done {
                select! {
                    _ = flush_interval.tick() => {
                        self.flush_finalized(&mut output).await;
                    },
                    maybe_event = input_events.next() => {
                        if let Some(event) = maybe_event {
                            // Only support Mezmo metric-shaped LogEvents. Warn the user if a non-metric
                            // shape is provided, drop the event and move on.
                            let metric = match to_metric(event.as_log()) {
                                Ok(metric) => metric,
                                Err(err) => {
                                    emit!(MezmoAggregateDistributedRecordFailed {
                                        drop_reason: "Invalid metric event",
                                        err: err.to_string()
                                    });
                                    handle_transform_error(&Some(self.mezmo_ctx.clone()), err);
                                    continue;
                                }
                            };

                            // The ConnectionManager interface handles reconnecting, retries, exp backoff, etc
                            // in the event of a connection-level failure.
                            match self.record_with_retry(&metric).await {
                                Ok(_) => {
                                    emit!(MezmoAggregateDistributedEventRecorded);
                                }
                                Err(err) => {
                                    emit!(MezmoAggregateDistributedRecordFailed {
                                        drop_reason: "Unable to send value to datastore",
                                        err: err.to_string()
                                    });
                                }
                            }
                        } else {
                            // shutting down...
                            self.flush_finalized(&mut output).await;
                            done = true;
                        }
                    }
                }

                for event in output.drain(..) {
                    yield event;
                }
            }
        })
    }
}

fn encode_json(value: &Value) -> String {
    serde_json::to_string(&value).expect("vrl::value::Value can always be serialized as JSON")
}
