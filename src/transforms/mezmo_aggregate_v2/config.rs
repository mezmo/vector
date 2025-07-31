use super::*;
use crate::conditions::AnyCondition;
use crate::config::{
    DataType, GenerateConfig, Input, LogNamespace, OutputId, TransformConfig, TransformContext,
    TransformOutput,
};
use crate::mezmo::persistence::RocksDBPersistenceConnection;
use crate::mezmo_env_config;
use crate::schema::Definition;
use crate::transforms::remap::RemapConfig;
use crate::transforms::Transform;
use vector_lib::config::clone_input_definitions;
use vector_lib::configurable::configurable_component;
use vector_lib::enrichment::TableRegistry;
use vrl::path::parse_target_path;

/// Configuration for the `sliding_aggregate` transform.
/// The `sliding_aggregate` transform has been renamed. This exists for backwards compatibility.
#[configurable_component(transform("sliding_aggregate", "Mezmo sliding aggregate"))]
#[configurable(metadata(
    deprecated = "The `sliding_aggregate` transform has been renamed. Please use `mezmo_aggregate_v2` instead."
))]
#[derive(Clone, Debug)]
pub struct SlidingAggregateConfig(MezmoAggregateV2Config);

impl GenerateConfig for SlidingAggregateConfig {
    fn generate_config() -> toml::Value {
        <MezmoAggregateV2Config as GenerateConfig>::generate_config()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "sliding_aggregate")]
impl TransformConfig for SlidingAggregateConfig {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        warn!("DEPRECATED: The `sliding_aggregate` transform has been renamed. Please use `mezmo_aggregate_v2` instead.");
        self.0.build(ctx).await
    }

    fn input(&self) -> Input {
        self.0.input()
    }

    fn outputs(
        &self,
        enrichment_tables: TableRegistry,
        input_definitions: &[(OutputId, Definition)],
        global_log_namespace: LogNamespace,
    ) -> Vec<TransformOutput> {
        self.0
            .outputs(enrichment_tables, input_definitions, global_log_namespace)
    }
}

/// Configuration for the `mezmo_aggregate_v2` transform.
#[configurable_component(transform("mezmo_aggregate_v2", "Mezmo Aggregate V2"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoAggregateV2Config {
    /// The sliding window duration in milliseconds to use when
    /// determining the aggregate values.
    #[serde(default = "default_window_duration_ms")]
    window_duration_ms: u32,

    /// Set how often the transform will check for events that are expired or have triggered
    /// a flush condition.
    #[serde(default = "default_flush_tick_ms")]
    flush_tick_ms: u64,

    /// Maximum number of keys to maintain in the transform's map
    #[serde(default = "default_mem_cardinality_limit")]
    mem_cardinality_limit: u32,

    /// The maximum number of sliding window structs to allow per metric series
    #[serde(default = "default_mem_window_limit")]
    mem_window_limit: u32,

    /// The minimum window over which to aggregate data
    #[serde(default = "default_min_window_size_ms")]
    min_window_size_ms: u32,

    // LOG-18567: Two additional properties that were considered as part of the spike but
    // pushed out of the spike effort. They include:
    //   * mem_cardinality_limit - the maximum number of keys to maintain in the transform's BTreeMap.
    //   * mem_window_limit - the maximum number of sliding window structs to allow per metric series.
    /// Define a VRL condition that when it evaluates to true on the aggregated event will
    /// also cause the aggregated event to be flushed to the output stream. This may produce
    /// values outside of their sliding window because of a true condition.
    flush_condition: Option<AnyCondition>,

    /// The different field paths that form the event identity. Events with matching values
    /// will be aggregated together.
    event_id_fields: Vec<String>,

    /// An optional path to the timestamp field on the event. If a field path isn't supplied
    /// or if the event is missing the timestamp, the current system time will be used.
    event_timestamp_field: Option<String>,

    /// The VRL program that produces a new accumulated aggregate event from the prior accumulated
    /// event and the new event.
    source: String,

    /// Sets the base path for the persistence connection.
    /// NOTE: Leaving this value empty will disable state persistence.
    #[serde(default = "default_state_persistence_base_path")]
    state_persistence_base_path: Option<String>,

    /// Set how often the state of this transform will be persisted to the [PersistenceConnection]
    /// storage backend.
    #[serde(default = "default_state_persistence_tick_ms")]
    state_persistence_tick_ms: u64,

    /// The maximum amount of jitter (ms) to add to the `state_persistence_tick_ms`
    /// flush interval.
    #[serde(default = "default_state_persistence_max_jitter_ms")]
    state_persistence_max_jitter_ms: u64,
}

const fn default_window_duration_ms() -> u32 {
    2 * 1000
}

const fn default_flush_tick_ms() -> u64 {
    1000
}

fn default_mem_cardinality_limit() -> u32 {
    mezmo_env_config!("MEZMO_AGGREGATION_CARDINALITY_LIMIT", 20_000)
}

const fn default_mem_window_limit() -> u32 {
    200
}

const fn default_min_window_size_ms() -> u32 {
    5000
}

const fn default_state_persistence_base_path() -> Option<String> {
    None
}

const fn default_state_persistence_tick_ms() -> u64 {
    30000
}

const fn default_state_persistence_max_jitter_ms() -> u64 {
    750
}

impl_generate_config_from_default!(MezmoAggregateV2Config);
impl MezmoAggregateV2Config {
    /// This method does all of the work of turning a MezmoAggregateV2Config instance into a
    /// MezmoAggregateV2 instance. It's separate from the build() method so tests get the
    /// actual MezmoAggregateV2 type and can then reach into the type to target test cases.
    pub(super) async fn internal_build(
        &self,
        ctx: &TransformContext,
    ) -> crate::Result<MezmoAggregateV2> {
        // Leverage the remap transform to build the VRL program from the string source code. This
        // could be moved into a shared function between the two but this works.
        let remap_config = RemapConfig {
            source: Some(self.source.clone()),
            ..Default::default()
        };
        let (program, _, _) = remap_config.compile_vrl_program(
            ctx.enrichment_tables.clone(),
            ctx.merged_schema_definition.clone(),
            ctx.mezmo_ctx.clone(),
        )?;

        let mut event_key_fields = Vec::new();
        for id_path in &self.event_id_fields {
            event_key_fields.push(parse_target_path(id_path)?);
        }

        let flush_condition = self
            .flush_condition
            .as_ref()
            .map(|cond| cond.build(&ctx.enrichment_tables, ctx.mezmo_ctx.clone()))
            .transpose()?;

        let window_size_ms = self.window_duration_ms as i64; // ok cast since u32::MAX < i64::MAX

        let event_timestamp_field = self
            .event_timestamp_field
            .as_ref()
            .map(|s| parse_target_path(s.as_str()))
            .transpose()?;

        let state_persistence_tick_ms = self.state_persistence_tick_ms;
        let state_persistence_max_jitter_ms = self.state_persistence_max_jitter_ms;
        let state_persistence: Option<Arc<dyn PersistenceConnection>> =
            match (&self.state_persistence_base_path, ctx.mezmo_ctx.clone()) {
                (Some(base_path), Some(mezmo_ctx)) => Some(Arc::new(
                    RocksDBPersistenceConnection::new(base_path, &mezmo_ctx)?,
                )),
                (_, Some(mezmo_ctx)) => {
                    debug!(
                        "MezmoAggregateV2: state persistence not enabled for component {}",
                        mezmo_ctx.id()
                    );
                    None
                }
                (_, _) => None,
            };

        Ok(MezmoAggregateV2::new(
            self.flush_tick_ms,
            event_key_fields,
            program,
            event_timestamp_field,
            flush_condition,
            ctx.mezmo_ctx.clone(),
            AggregatorLimits::new(
                self.mem_window_limit,
                self.mem_cardinality_limit,
                self.min_window_size_ms,
                window_size_ms,
            ),
            state_persistence,
            state_persistence_tick_ms,
            state_persistence_max_jitter_ms,
        ))
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_aggregate_v2")]
impl TransformConfig for MezmoAggregateV2Config {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        self.internal_build(ctx).await.map(Transform::event_task)
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

#[derive(Debug, Clone)]
pub struct AggregatorLimits {
    /// The maximum number of sliding window structs to allow per metric series
    pub mem_window_limit: u32,
    /// Maximum number of keys to maintain in the transform's map
    pub mem_cardinality_limit: u32,
    /// A new window is allocated only if an event's time surpasses the last
    /// saved window's time by a minimum window size
    pub min_window_size_ms: u32,
    /// The size of each sliding window in milliseconds. Arriving events
    /// aggregate into a window if their timestamp falls within the window
    pub window_duration_ms: i64,
}

impl AggregatorLimits {
    pub(super) const fn new(
        mem_window_limit: u32,
        mem_cardinality_limit: u32,
        min_window_size_ms: u32,
        window_duration_ms: i64,
    ) -> Self {
        Self {
            mem_window_limit,
            mem_cardinality_limit,
            min_window_size_ms,
            window_duration_ms,
        }
    }
}
