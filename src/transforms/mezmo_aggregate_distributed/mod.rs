use crate::common::backoff::ExponentialBackoff;
use crate::internal_events::{
    MezmoAggregateDistributedEventRecorded, MezmoAggregateDistributedFlushFailed,
    MezmoAggregateDistributedFlushed, MezmoAggregateDistributedRecordFailed,
    MezmoAggregateDistributedRecordRetried,
};
use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use mezmo::{MezmoContext, user_trace::handle_transform_error};
use redis::{
    AsyncCommands, ErrorKind, RedisError, RedisResult, Script, ToRedisArgs, aio::ConnectionManager,
};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::{select, time::sleep};
use vector_lib::config::log_schema;
use vector_lib::configurable::configurable_component;
use vector_lib::event::metric::mezmo::TransformError;
use vector_lib::event::{LogEvent, Metric};
use vector_lib::{
    event::{Event, MetricValue, metric::mezmo::to_metric},
    transform::TaskTransform,
};
use vrl::value::{KeyString, Value};

mod config;
use config::MezmoAggregateDistributedConfig;

#[cfg(feature = "mezmo-aggregate-distributed-integration-tests")]
#[cfg(test)]
pub(crate) mod integration_tests;

static RECORD_SCRIPT: LazyLock<Script> =
    LazyLock::new(|| Script::new(include_str!("redis/record.lua")));
static FLUSH_SCRIPT: LazyLock<Script> =
    LazyLock::new(|| Script::new(include_str!("redis/flush.lua")));

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

    /// Minimum observed value over the window
    Min,

    /// Maximum observed value over the window
    Max,
}

impl Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::Sum => write!(f, "sum"),
            Strategy::Avg => write!(f, "avg"),
            Strategy::Min => write!(f, "min"),
            Strategy::Max => write!(f, "max"),
        }
    }
}

#[derive(Debug, Snafu)]
pub(super) enum AggregateError {
    #[snafu(display("Creating Redis client failed: {}", source))]
    RedisCreateFailed { source: RedisError },
}

#[derive(Debug, Deserialize, Serialize)]
struct FlushedWindow {
    count: u32,
    #[serde(deserialize_with = "deserialize_json_string")]
    fields: serde_json::Value,
    value: f64,
    strategy: Strategy,
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

enum FlushUntil {
    Now,
    End,
}

impl ToRedisArgs for FlushUntil {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            FlushUntil::Now => Utc::now().timestamp_millis().write_redis_args(out),
            FlushUntil::End => "+inf".write_redis_args(out),
        }
    }
}

impl FlushedWindow {
    /// Converts a FlushedWindow into a Log Event
    fn into_event(self) -> Event {
        let message_path = log_schema()
            .message_key_target_path()
            .expect("message key to always be defined");
        let timestamp_path = log_schema()
            .timestamp_key_target_path()
            .expect("timestamp key to always be defined");
        let mut log: LogEvent = self
            .fields
            .clone()
            .try_into()
            .expect("deserialized `fields` is a valid serde_json::Value::Object");

        let value_type = log.remove("value_type");
        log.insert("value", self.value_for_type(value_type));
        log.insert("strategy", self.strategy.to_string());
        log.insert("window_start", self.window_start_ts);
        log.insert("window_end", self.window_end_ts);
        log.insert("count", self.count);

        log.rename_key(".", message_path);
        log.insert(timestamp_path, self.window_end_ts);

        Event::Log(log)
    }

    /// Creates the appropriate value representation within a LogEvent, for this
    /// window, based on the provided `value_type`.
    fn value_for_type(&self, value_type: Option<Value>) -> Value {
        let value = match self.strategy {
            Strategy::Sum | Strategy::Min | Strategy::Max => self.value,
            Strategy::Avg => self.value / (self.count as f64),
        };

        Value::Object(btreemap! {
            KeyString::from("type") => value_type.unwrap_or(Value::Null),
            KeyString::from("value") => Value::from(value),
        })
    }
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
        let key = format!(
            "{{{}}}:{{{}}}:{{{}}}:aggregate:{}",
            self.mezmo_ctx.account_id,
            self.mezmo_ctx
                .pipeline_id
                .as_ref()
                .map_or("none".to_string(), |p| p.to_string()),
            self.mezmo_ctx.component_id,
            self.config.strategy,
        );

        match self.config.key_prefix {
            Some(ref prefix) => format!("{prefix}:{key}"),
            None => key,
        }
    }

    /// Key for this event window hash
    fn get_event_window_key(&self, hash: u64, timestamp: i64) -> String {
        let key = format!(
            "{{{}}}:{{{}}}:{{{}}}:aggregate:{}:{}:{}",
            self.mezmo_ctx.account_id,
            self.mezmo_ctx
                .pipeline_id
                .as_ref()
                .map_or("none".to_string(), |p| p.to_string()),
            self.mezmo_ctx.component_id,
            self.config.strategy,
            hash,
            timestamp,
        );

        match self.config.key_prefix {
            Some(ref prefix) => format!("{prefix}:{key}"),
            None => key,
        }
    }

    /// Generates a hashed code based on the root metric event fields. The fields
    /// are returned alongside their hash and are used to form the output event.
    fn get_event_fields(&self, event: &Metric) -> (u64, Value) {
        let mut hasher = DefaultHasher::new();

        let mut fields: BTreeMap<KeyString, Value> = BTreeMap::new();
        let kind = event.kind();
        let name = event.name();
        let namespace = event.namespace();
        let value_type = event.value().as_name();
        let tags = event.tags();

        kind.hash(&mut hasher);
        name.hash(&mut hasher);
        namespace.hash(&mut hasher);
        value_type.hash(&mut hasher);
        tags.hash(&mut hasher);

        fields.insert("kind".to_string().into(), kind.into());
        fields.insert("name".to_string().into(), name.into());
        fields.insert("namespace".to_string().into(), namespace.into());
        fields.insert("value_type".to_string().into(), value_type.into());

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

    /// Derives a timestamp to control the flush of output events based on configuration.
    /// This ensures the window kept "open" for at least the `window_duration_ms`, regardless
    /// of the timing of when the event was recevied by this component within the window.
    /// An additional `flush_grace_period_ms` is added to account for processing delays on
    /// either the client or processing side.
    fn get_flush_timestamp(&self, window_start_ts: i64) -> i64 {
        let window_duration_ms = i64::from(self.config.window_duration_ms);
        let from_start_ts = window_start_ts + window_duration_ms;
        let from_now_ts = Utc::now().timestamp_millis() + window_duration_ms;

        std::cmp::max(from_start_ts, from_now_ts) + i64::from(self.config.flush_grace_period_ms)
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
    async fn record(&mut self, event: &Metric) -> Result<(), RedisError> {
        let (hash, fields) = self.get_event_fields(event);
        let event_ts = self.get_event_timestamp(event);
        let window_start_ts = self.align_window_timestamp(event_ts);
        let window_flush_ts = self.get_flush_timestamp(window_start_ts);
        let active_windows_key = self.get_active_windows_key();
        let event_window_key = self.get_event_window_key(hash, window_start_ts);

        let mut conn = self.conn.clone();

        let value: f64 = match event.value() {
            MetricValue::Counter { value } => *value,
            MetricValue::Gauge { value } => *value,
            // TODO: consider other metric types for sum/avg?
            _ => {
                let err = TransformError::InvalidMetricType {
                    type_name: event.value().as_name().to_string(),
                };

                emit!(MezmoAggregateDistributedRecordFailed {
                    drop_reason: "Unsupported metric event",
                    err: err.to_string()
                });

                handle_transform_error(&Some(self.mezmo_ctx.clone()), err);
                return Ok(());
            }
        };

        let result: RedisResult<()> = RECORD_SCRIPT
            .key(active_windows_key)
            .key(event_window_key)
            .arg(window_start_ts)
            .arg(window_flush_ts)
            .arg(self.config.window_duration_ms)
            .arg(self.config.window_cardinality_limit)
            .arg(self.config.key_expiry_grace_period_ms)
            .arg(self.config.strategy.to_string())
            .arg(encode_json(&fields))
            .arg(value)
            .invoke_async(&mut conn)
            .await;

        result?;

        Ok(())
    }

    /// Records the value from the event against the datastore with retry logic.
    /// This handlees the case where a [[ConnectionManager]] client instance is being
    /// destroyed/recreated.
    async fn record_with_retry(&mut self, event: &Metric) {
        let mut backoff = ExponentialBackoff::from_millis(2)
            .factor(self.config.connection_retry_factor_ms)
            .max_delay(Duration::from_millis(
                self.config.connection_retry_max_delay_ms,
            ));

        let mut attempt = 0;
        loop {
            match self.record(event).await {
                Ok(_) => {
                    emit!(MezmoAggregateDistributedEventRecorded);
                    return;
                }
                Err(err) if matches!(err.kind(), ErrorKind::ResponseError) => {
                    // Cardinality errors returned from the script are not retriable.
                    // Emit both internal logs and a user-facing log.
                    // Events that exceed the cardinality limit are dropped.
                    emit!(MezmoAggregateDistributedRecordFailed {
                        drop_reason: "Cardinality limit exceeded",
                        err: err.to_string()
                    });
                    handle_transform_error(
                        &Some(self.mezmo_ctx.clone()),
                        TransformError::CardinalityLimitExceeded {
                            limit: self.config.window_cardinality_limit.into(),
                        },
                    );
                    return;
                }
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.connection_retry_count {
                        emit!(MezmoAggregateDistributedRecordFailed {
                            drop_reason: "Unable to send value to datastore",
                            err: err.to_string()
                        });

                        return;
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
    async fn flush_finalized(&self, output: &mut Vec<Event>, until: FlushUntil) {
        let active_windows_key = self.get_active_windows_key();
        let mut conn = self.conn.clone();

        let result: RedisResult<Vec<String>> =
            conn.zrangebyscore(&active_windows_key, 0, until).await;

        let expired_window_keys = match result {
            Ok(keys) => {
                if keys.is_empty() {
                    return; // nothing to flush
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

        for flush_batch in expired_window_keys.chunks(self.config.flush_batch_size) {
            let mut invocation = FLUSH_SCRIPT.prepare_invoke();
            invocation.key(&active_windows_key);
            for key in flush_batch {
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
                        output.push(flushed_window.into_event());
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
                        debug!(flush_tick_ms = &self.config.flush_tick_ms, "Flushing from interval");
                        self.flush_finalized(&mut output, FlushUntil::Now).await;
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
                            self.record_with_retry(&metric).await;
                        } else {
                            // shutting down...
                            done = true;
                            if self.config.flush_all_on_shutdown {
                                debug!("Flushing on shutdown");
                                self.flush_finalized(&mut output, FlushUntil::End).await;
                            }
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
