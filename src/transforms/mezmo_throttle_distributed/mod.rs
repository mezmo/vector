use crate::{
    common::backoff::ExponentialBackoff,
    conditions::{AnyCondition, Condition},
    config::TransformContext,
    event::Event,
    internal_events::{
        MezmoThrottleDistributedCheckFailed, MezmoThrottleDistributedCheckRetried,
        MezmoThrottleDistributedEventChecked, MezmoThrottleDistributedEventThrottled,
        TemplateRenderingError,
    },
    transforms::TaskTransform,
};
use async_stream::stream;
use futures::{Stream, StreamExt};
use mezmo::{user_trace::handle_transform_error, MezmoContext};
use redis::{aio::ConnectionManager, ErrorKind, RedisError, RedisResult, Script};
use snafu::Snafu;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::LazyLock;
use std::{
    collections::hash_map::DefaultHasher,
    time::{SystemTime, UNIX_EPOCH},
};
use std::{
    hash::{Hash, Hasher},
    time::Duration,
};
use vector_lib::event::metric::mezmo::TransformError;

mod config;
use config::MezmoThrottleDistributedConfig;

#[cfg(feature = "mezmo-throttle-distributed-integration-tests")]
#[cfg(test)]
pub(crate) mod integration_tests;

static CHECK_SCRIPT: LazyLock<Script> =
    LazyLock::new(|| Script::new(include_str!("redis/check.lua")));

#[derive(Debug, Snafu)]
pub(super) enum ThrottleError {
    #[snafu(display("Creating Redis client failed: {source}"))]
    RedisCreateFailed { source: RedisError },
}

pub struct MezmoThrottleDistributed {
    conn: ConnectionManager,
    config: MezmoThrottleDistributedConfig,
    exclude: Option<Condition>,
    mezmo_ctx: MezmoContext,
}

impl MezmoThrottleDistributed {
    pub fn new(
        conn: ConnectionManager,
        config: MezmoThrottleDistributedConfig,
        exclude: Option<Condition>,
        mezmo_ctx: MezmoContext,
    ) -> crate::Result<Self> {
        Ok(Self {
            conn,
            config,
            exclude,
            mezmo_ctx,
        })
    }

    /// Key for the zset of active windows.
    fn get_active_windows_key(&self) -> String {
        let key = format!(
            "{{{}}}:{{{}}}:{{{}}}:throttle",
            self.mezmo_ctx.account_id,
            self.mezmo_ctx
                .pipeline_id
                .as_ref()
                .map_or("none".to_string(), |p| p.to_string()),
            self.mezmo_ctx.component_id,
        );

        match self.config.key_prefix {
            Some(ref prefix) => format!("{prefix}:{key}"),
            None => key,
        }
    }

    /// Key for this event window hash
    fn get_event_window_key(&self, hash: u64) -> String {
        let key = format!(
            "{{{}}}:{{{}}}:{{{}}}:throttle:{}",
            self.mezmo_ctx.account_id,
            self.mezmo_ctx
                .pipeline_id
                .as_ref()
                .map_or("none".to_string(), |p| p.to_string()),
            self.mezmo_ctx.component_id,
            hash,
        );

        match self.config.key_prefix {
            Some(ref prefix) => format!("{prefix}:{key}"),
            None => key,
        }
    }

    /// Renders a template provided by `key_field` and returns the hash of the resulting value.
    /// This is used to form a secondary bucket for the rate-limit based on the unique
    /// values of the field/template.
    fn get_key_field_hash(&self, event: &Event) -> u64 {
        let mut hasher = DefaultHasher::new();
        let value = self
            .config
            .key_field
            .as_ref()
            .and_then(|template| {
                template
                    .render_string(event)
                    .map_err(|error| {
                        emit!(TemplateRenderingError {
                            error,
                            field: Some("key_field"),
                            drop_event: false,
                        })
                    })
                    .ok()
            })
            .unwrap_or("".to_string());

        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Checks the rate limit for the event.
    async fn check(&mut self, event: &Event) -> Result<bool, RedisError> {
        let mut conn = self.conn.clone();
        let now: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis()
            .try_into()
            .expect("`now` timestamp is from the incredibly distant future");

        let hash = self.get_key_field_hash(event);
        let res: RedisResult<bool> = CHECK_SCRIPT
            .key(self.get_active_windows_key()) // active_windows_key
            .key(self.get_event_window_key(hash)) // event_window_key
            .arg(self.config.threshold)
            .arg(self.config.window_duration_ms)
            .arg(now)
            .arg(self.config.window_cardinality_limit)
            .invoke_async(&mut conn)
            .await;

        emit!(MezmoThrottleDistributedEventChecked);
        res
    }

    /// Checks the rate limit for the event, retrying if needed. This handles the case
    /// where a [[ConnectionManager]] client instance is being destroyed/recreated.
    /// In the event of an unrecoverable connection error or exceeding the cardinality
    /// limit, the rate limit is ignored and the event is allowed.
    async fn check_with_retry(&mut self, event: &Event) -> bool {
        let mut backoff = ExponentialBackoff::from_millis(2)
            .factor(self.config.connection_retry_factor_ms)
            .max_delay(Duration::from_millis(
                self.config.connection_retry_max_delay_ms,
            ));

        let mut attempt = 0;
        loop {
            match self.check(event).await {
                Ok(is_allowed) => return is_allowed,
                Err(err) if matches!(err.kind(), ErrorKind::ResponseError) => {
                    // Cardinality errors returned from the script are not retriable.
                    // Emit both internal logs and a user-facing log and allow the event.
                    emit!(MezmoThrottleDistributedCheckFailed {
                        err: err.to_string()
                    });
                    handle_transform_error(
                        &Some(self.mezmo_ctx.clone()),
                        TransformError::CardinalityLimitExceeded {
                            limit: self.config.window_cardinality_limit.into(),
                        },
                    );
                    return true;
                }
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.connection_retry_count {
                        emit!(MezmoThrottleDistributedCheckFailed {
                            err: err.to_string()
                        });
                        return true;
                    }

                    let delay = backoff.next().unwrap();
                    emit!(MezmoThrottleDistributedCheckRetried {
                        attempt,
                        delay_ms: delay.as_millis()
                    });

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

impl TaskTransform<Event> for MezmoThrottleDistributed {
    fn transform(
        mut self: Box<Self>,
        mut input_rx: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>>
    where
        Self: 'static,
    {
        Box::pin(stream! {
            loop {
                let done = tokio::select! {
                    maybe_event = input_rx.next() => {
                        match maybe_event {
                            None => true,
                            Some(event) => {
                                let (is_excluded, event) = match self.exclude.as_ref() {
                                    Some(condition) => {
                                        let (is_excluded, event) = condition.check(event);
                                        (is_excluded, event)
                                    },
                                    _ => (false, event)
                                };

                                let output = if is_excluded || self.check_with_retry(&event).await {
                                    Some(event)
                                } else {
                                    emit!(MezmoThrottleDistributedEventThrottled);
                                    None
                                };

                                if let Some(event) = output {
                                    yield event;
                                }
                                false
                            }
                        }
                    }
                };

                if done {
                    break;
                }
            }
        })
    }
}
