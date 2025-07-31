use super::*;

use crate::mezmo_env_config;
use crate::{
    config::{DataType, Input, TransformConfig},
    schema,
    template::Template,
    transforms::{
        mezmo_common::state::{
            default_connection_response_timeout_ms, default_connection_retry_count,
            default_connection_retry_factor_ms, default_connection_retry_max_delay_ms,
            default_connection_string, default_connection_timeout_ms,
        },
        Transform,
    },
};
use redis::{
    aio::{ConnectionManager, ConnectionManagerConfig},
    RedisResult,
};
use serde_with::serde_as;
use snafu::ResultExt;
use std::time::Duration;
use vector_lib::config::{clone_input_definitions, LogNamespace, OutputId, TransformOutput};
use vector_lib::configurable::component::GenerateConfig;
use vector_lib::configurable::configurable_component;

const DEFAULT_WINDOW_DURATION_MS: u32 = 10_000;
const DEFAULT_WINDOW_CARDINALITY_LIMIT: u32 = 20_000;

/// Configuration for the `mezmo_throttle_distributed` transform.
#[serde_as]
#[configurable_component(transform(
    "mezmo_throttle_distributed",
    "Rate limit logs passing through a topology."
))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MezmoThrottleDistributedConfig {
    /// The connection string for the datastore. The default value should be used
    /// in most cases to allow this to be dynamic.
    #[serde(default = "default_connection_string")]
    pub connection_string: String,

    /// The number of events allowed for a given bucket per configured `window_duration_ms`.
    ///
    /// Each unique key as determined by `key_field` has its own `threshold`.
    pub(super) threshold: NonZeroU32,

    /// The time window in which the configured `threshold` is applied, in milliseconds.
    #[serde(default = "default_window_duration_ms")]
    pub(super) window_duration_ms: NonZeroU32,

    /// A prefix for all keys written by this component. This is useful for executing the same
    /// component against the same datastore, but with different purposes (e.g. "live" data
    /// vs "simulated" data), without affecting each other.
    pub key_prefix: Option<String>,

    /// The value to group events into separate buckets to be rate limited independently.
    ///
    /// If left unspecified, or if the event doesn't have `key_field`, then the event is not rate
    /// limited separately.
    pub(super) key_field: Option<Template>,

    /// A logical condition used to exclude events from sampling.
    pub(super) exclude: Option<AnyCondition>,

    /// Guard rail value that limits the number of unique key field values that the throttle
    /// component will hold. After this limit point, user errors will be generated and
    /// events will not be throttled.
    #[serde(default = "default_window_cardinality_limit")]
    pub(super) window_cardinality_limit: NonZeroU32,

    /// Connection-level properties and retry configuration.
    ///
    /// A multiplicative factor that will be applied to the retry delay.
    #[serde(default = "default_connection_retry_factor_ms")]
    pub connection_retry_factor_ms: u64,

    /// The number of retry attempts, with an exponentially increasing delay.
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

fn default_window_duration_ms() -> NonZeroU32 {
    NonZeroU32::new(DEFAULT_WINDOW_DURATION_MS).unwrap()
}

fn default_window_cardinality_limit() -> NonZeroU32 {
    mezmo_env_config!(
        "MEZMO_AGGREGATION_CARDINALITY_LIMIT",
        NonZeroU32::new(DEFAULT_WINDOW_CARDINALITY_LIMIT).unwrap()
    )
}

impl GenerateConfig for MezmoThrottleDistributedConfig {
    fn generate_config() -> toml::value::Value {
        toml::value::Value::try_from(Self {
            connection_string: default_connection_string(),
            threshold: NonZeroU32::new(1).unwrap(),
            key_prefix: None,
            key_field: None,
            exclude: None,
            window_duration_ms: default_window_duration_ms(),
            window_cardinality_limit: default_window_cardinality_limit(),
            connection_retry_factor_ms: default_connection_retry_factor_ms(),
            connection_retry_count: default_connection_retry_count(),
            connection_retry_max_delay_ms: default_connection_retry_max_delay_ms(),
            connection_timeout_ms: default_connection_timeout_ms(),
            connection_response_timeout_ms: default_connection_response_timeout_ms(),
        })
        .expect("defaults values can be serialized to TOML")
    }
}

impl MezmoThrottleDistributedConfig {
    pub(super) async fn build_transform(
        &self,
        ctx: &TransformContext,
    ) -> crate::Result<MezmoThrottleDistributed> {
        if ctx.mezmo_ctx.is_none() {
            return Err("Cannot create MezmoThrottleDistributed without a component key".into());
        };

        let conn = self.build_client().await.context(RedisCreateFailedSnafu)?;

        let exclude = self
            .exclude
            .as_ref()
            .map(|condition| condition.build(&ctx.enrichment_tables, ctx.mezmo_ctx.clone()))
            .transpose()?;

        let mezmo_ctx = ctx
            .mezmo_ctx
            .clone()
            .expect("MezmoContext is required by the config");

        MezmoThrottleDistributed::new(conn, self.clone(), exclude, mezmo_ctx)
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
#[typetag::serde(name = "mezmo_throttle_distributed")]
impl TransformConfig for MezmoThrottleDistributedConfig {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        self.build_transform(ctx).await.map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        input_definitions: &[(OutputId, schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        // The event is not modified, so the definition is passed through as-is
        vec![TransformOutput::new(
            DataType::Log,
            clone_input_definitions(input_definitions),
        )]
    }
}
