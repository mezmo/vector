use super::*;

/// Configuration for the `mezmo_throttle` transform.
#[serde_as]
#[configurable_component(transform(
    "mezmo_throttle",
    "Rate limit logs passing through a topology."
))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoThrottleConfig {
    /// The number of events allowed for a given bucket per configured `window_secs`.
    ///
    /// Each unique key has its own `threshold`.
    pub(super) threshold: u32,

    /// The time window in which the configured `threshold` is applied, in seconds.
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    #[configurable(metadata(docs::human_name = "Time Window"))]
    pub(super) window_secs: Duration,

    /// The value to group events into separate buckets to be rate limited independently.
    ///
    /// If left unspecified, or if the event doesn't have `key_field`, then the event is not rate
    /// limited separately.
    #[configurable(metadata(docs::examples = "{{ message }}", docs::examples = "{{ hostname }}",))]
    pub(super) key_field: Option<Template>,

    /// A logical condition used to exclude events from sampling.
    pub(super) exclude: Option<AnyCondition>,
}

impl_generate_config_from_default!(MezmoThrottleConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_throttle")]
impl TransformConfig for MezmoThrottleConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Throttle::new(self, context, ThrottleClock {}).map(Transform::event_task)
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
