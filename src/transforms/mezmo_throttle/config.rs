use super::*;

use crate::{
    config::{DataType, Input, TransformConfig},
    schema,
    template::Template,
    transforms::Transform,
};
use serde_with::serde_as;
use vector_lib::config::{clone_input_definitions, LogNamespace, OutputId, TransformOutput};
use vector_lib::configurable::configurable_component;

/// Configuration for the `mezmo_throttle` transform.
#[serde_as]
#[configurable_component(transform(
    "mezmo_throttle",
    "Rate limit logs passing through a topology."
))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoThrottleConfig {
    /// The number of events allowed for a given bucket per configured `window_ms`.
    ///
    /// Each unique key has its own `threshold`.
    pub(super) threshold: u32,

    /// The time window in which the configured `threshold` is applied, in milliseconds.
    #[configurable(metadata(docs::human_name = "Time Window"))]
    pub(super) window_ms: u64,

    /// The value to group events into separate buckets to be rate limited independently.
    ///
    /// If left unspecified, or if the event doesn't have `key_field`, then the event is not rate
    /// limited separately.
    #[configurable(metadata(docs::examples = "{{ message }}", docs::examples = "{{ hostname }}",))]
    pub(super) key_field: Option<Template>,

    /// A logical condition used to exclude events from sampling.
    pub(super) exclude: Option<AnyCondition>,

    /// Sets the base path for the persistence connection.
    /// NOTE: Leaving this value empty will disable state persistence.
    #[serde(default = "default_state_persistence_base_path")]
    pub(super) state_persistence_base_path: Option<String>,

    /// Set how often the state of this transform will be persisted to the [PersistenceConnection]
    /// storage backend.
    #[serde(default = "default_state_persistence_tick_ms")]
    pub(super) state_persistence_tick_ms: u64,

    /// The maximum amount of jitter (ms) to add to the `state_persistence_tick_ms`
    /// flush interval.
    #[serde(default = "default_state_persistence_max_jitter_ms")]
    pub(super) state_persistence_max_jitter_ms: u64,
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

impl_generate_config_from_default!(MezmoThrottleConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_throttle")]
impl TransformConfig for MezmoThrottleConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Throttle::new(self, context, ThrottleClock::new()).map(Transform::event_task)
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
