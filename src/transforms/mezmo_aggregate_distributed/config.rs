use redis::{
    aio::{ConnectionManager, ConnectionManagerConfig},
    RedisResult,
};
use snafu::prelude::*;
use std::time::Duration;
use vector_lib::configurable::configurable_component;
use vector_lib::enrichment::TableRegistry;
use vector_lib::{config::clone_input_definitions, configurable::component::GenerateConfig};

use crate::config::{
    DataType, Input, LogNamespace, OutputId, TransformConfig, TransformContext, TransformOutput,
};
use crate::mezmo_env_config;
use crate::schema::Definition;
use crate::transforms::{
    mezmo_common::state::{
        default_connection_response_timeout_ms, default_connection_retry_count,
        default_connection_retry_factor_ms, default_connection_retry_max_delay_ms,
        default_connection_string, default_connection_timeout_ms,
    },
    Transform,
};

use super::{MezmoAggregateDistributed, RedisCreateFailedSnafu, Strategy};

const DEFAULT_WINDOW_DURATION_MS: u32 = 10_000;
const DEFAULT_KEY_EXPIRY_GRACE_PERIOD_MS: u32 = 12 * 60 * 60 * 1000; // 12 hours
const DEFAULT_WINDOW_CARDINALITY_LIMIT: u32 = 20_000;

/// Configuration for the `mezmo_aggregate_distributed` transform.
#[configurable_component(transform("mezmo_aggregate_distributed", "Mezmo Aggregate V3"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MezmoAggregateDistributedConfig {
    /// The connection string for the datastore. The default value should be used
    /// in most cases to allow this to be dynamic.
    #[serde(default = "default_connection_string")]
    pub connection_string: String,

    /// The aggregation strategy to use.
    #[configurable(derived)]
    pub strategy: Strategy,

    /// The tumbling window duration in milliseconds to use when
    /// determining the aggregate values.
    #[serde(default = "default_window_duration_ms")]
    pub window_duration_ms: u32,

    /// Maximum number of windows to keep in state.
    #[serde(default = "default_window_cardinality_limit")]
    pub window_cardinality_limit: u32,

    /// Set how often the transform will check for events that are expired or have triggered
    /// a flush condition.
    #[serde(default = "default_flush_tick_ms")]
    pub flush_tick_ms: u64,

    /// The number of expired windows that will be flushed at once.
    #[serde(default = "default_flush_batch_size")]
    pub flush_batch_size: usize,

    /// Controls the max age of a key before it is expired automatically to prevent
    /// memory bloat. In practice keys are explicitly removed when the data is flushed,
    /// however this does not account for scenarios where the component is removed from
    /// the configuration, or when the process is not running for a period of time.
    /// Automatic expiration of keys will occur after `window_duration + grace_period`
    /// milliseconds from the end of each window.
    #[serde(default = "default_key_expiry_grace_period_ms")]
    pub key_expiry_grace_period_ms: u32,

    /// Connection-level properties and retry configuration.
    ///
    /// Note that the retry configuration options are used for both the connection
    /// itself via [`redis::ConnectionManager`], as well as within the transform via the
    /// vendored `tokio-retry` crate [`vector::sinks::util::retries`].
    /// TODO(LOG-21580): Consider using `backon` directly instead of `tokio-retry`
    /// for greater flexibility and consistency with `redis-rs`.

    /// A multiplicative factor that will be applied to the retry delay.
    #[serde(default = "default_connection_retry_factor_ms")]
    pub connection_retry_factor_ms: u64,

    /// The number of retry attempts, with an exponentially increasing delay
    #[serde(default = "default_connection_retry_count")]
    pub connection_retry_count: usize,

    /// The max duration of the retry delay.
    #[serde(default = "default_connection_retry_max_delay_ms")]
    pub connection_retry_max_delay_ms: u64,

    /// Each connection attempt to the server will time out after `connection_timeout`.
    #[serde(default = "default_connection_timeout_ms")]
    pub connection_timeout_ms: Duration,

    /// The new connection will time out operations after `response_timeout` has passed.
    #[serde(default = "default_connection_response_timeout_ms")]
    pub connection_response_timeout_ms: Duration,
}

const fn default_window_duration_ms() -> u32 {
    DEFAULT_WINDOW_DURATION_MS
}

const fn default_key_expiry_grace_period_ms() -> u32 {
    DEFAULT_KEY_EXPIRY_GRACE_PERIOD_MS
}

fn default_window_cardinality_limit() -> u32 {
    mezmo_env_config!(
        "MEZMO_AGGREGATION_CARDINALITY_LIMIT",
        DEFAULT_WINDOW_CARDINALITY_LIMIT
    )
}

fn default_flush_tick_ms() -> u64 {
    mezmo_env_config!("MEZMO_AGGREGATION_FLUSH_TICK_MS", 5000)
}

fn default_flush_batch_size() -> usize {
    mezmo_env_config!("MEZMO_AGGREGATION_FLUSH_BATCH_SIZE", 500)
}

impl GenerateConfig for MezmoAggregateDistributedConfig {
    fn generate_config() -> toml::value::Value {
        toml::value::Value::try_from(Self {
            strategy: Strategy::Sum,
            connection_string: default_connection_string(),
            window_duration_ms: default_window_duration_ms(),
            window_cardinality_limit: default_window_cardinality_limit(),
            flush_tick_ms: default_flush_tick_ms(),
            flush_batch_size: default_flush_batch_size(),
            key_expiry_grace_period_ms: default_key_expiry_grace_period_ms(),
            connection_retry_factor_ms: default_connection_retry_factor_ms(),
            connection_retry_count: default_connection_retry_count(),
            connection_retry_max_delay_ms: default_connection_retry_max_delay_ms(),
            connection_timeout_ms: default_connection_timeout_ms(),
            connection_response_timeout_ms: default_connection_response_timeout_ms(),
        })
        .expect("defaults values can be serialized to TOML")
    }
}

impl MezmoAggregateDistributedConfig {
    pub(super) async fn build_transform(
        &self,
        ctx: &TransformContext,
    ) -> crate::Result<MezmoAggregateDistributed> {
        let Some(mezmo_ctx) = ctx.mezmo_ctx.as_ref() else {
            return Err("Cannot create MezmoAggregateDistributed without a component key".into());
        };

        let conn = self.build_client().await.context(RedisCreateFailedSnafu)?;

        Ok(MezmoAggregateDistributed::new(
            conn,
            self.clone(),
            mezmo_ctx.clone(),
        ))
    }

    pub(super) async fn build_client(&self) -> RedisResult<ConnectionManager> {
        let client = redis::Client::open(self.connection_string.clone())?;

        let config = ConnectionManagerConfig::new()
            .set_factor(self.connection_retry_factor_ms)
            .set_number_of_retries(self.connection_retry_count)
            .set_max_delay(self.connection_retry_max_delay_ms)
            .set_connection_timeout(self.connection_timeout_ms)
            .set_response_timeout(self.connection_response_timeout_ms);

        ConnectionManager::new_with_config(client, config).await
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_aggregate_distributed")]
impl TransformConfig for MezmoAggregateDistributedConfig {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        self.build_transform(ctx).await.map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _enrichment_tables: TableRegistry,
        input_definitions: &[(OutputId, Definition)],
        _global_log_namespace: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(
            DataType::Log,
            clone_input_definitions(input_definitions),
        )]
    }
}
