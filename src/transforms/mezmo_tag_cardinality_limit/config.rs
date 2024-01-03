use std::collections::{HashMap, HashSet};

use crate::config::{DataType, GenerateConfig, Input, OutputId, TransformConfig, TransformContext};
use crate::schema;
use crate::transforms::Transform;
use vector_config::configurable_component;
use vector_core::config::{LogNamespace, TransformOutput};

use super::TagCardinalityLimit;

/// Configuration for the `tag_cardinality_limit` transform.
#[configurable_component(transform("mezmo_tag_cardinality_limit"))]
#[derive(Clone, Debug)]
pub struct TagCardinalityLimitConfig {
    /// How many distinct values to accept for any given key.
    #[serde(default = "default_value_limit")]
    pub value_limit: usize,

    #[configurable(derived)]
    #[serde(default = "default_limit_exceeded_action")]
    pub limit_exceeded_action: LimitExceededAction,

    #[serde(flatten)]
    pub mode: Mode,

    /// Maximum size for a tag value.
    ///
    /// Tag values bigger than this size will be truncated.
    #[serde(default = "default_max_tag_size")]
    pub max_tag_size: usize,

    /// Tags names to limit cardinality. If not provided then all tag names are considered.
    #[serde(default)]
    pub tags: Option<HashSet<String>>,

    /// Tags names to exclude from cardinality limiting. If not provided then no tags are
    /// explicitly excluded.
    #[serde(default)]
    pub exclude_tags: Option<HashSet<String>>,
}

/// Controls the approach taken for tracking tag cardinality.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
#[configurable(metadata(
    docs::enum_tag_description = "Controls the approach taken for tracking tag cardinality."
))]
pub enum Mode {
    /// Tracks cardinality exactly.
    ///
    /// This mode has higher memory requirements than `probabilistic`, but never falsely outputs
    /// metrics with new tags after the limit has been hit.
    Exact,

    /// Tracks cardinality probabilistically.
    ///
    /// This mode has lower memory requirements than `exact`, but may occasionally allow metric
    /// events to pass through the transform even when they contain new tags that exceed the
    /// configured limit. The rate at which this happens can be controlled by changing the value of
    /// `cache_size_per_tag`.
    Probabilistic(BloomFilterConfig),
}

/// Bloom filter configuration in probabilistic mode.
#[configurable_component]
#[derive(Clone, Debug)]
pub struct BloomFilterConfig {
    /// The size of the cache for detecting duplicate tags, in bytes.
    ///
    /// The larger the cache size, the less likely it is to have a false positive, or a case where
    /// we allow a new value for tag even after we have reached the configured limits.
    #[serde(default = "default_cache_size")]
    pub cache_size_per_key: usize,
}

/// Possible actions to take when an event arrives that would exceed the cardinality limit for one
/// or more of its tags.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LimitExceededAction {
    /// Drop the tag(s) that would exceed the configured limit.
    DropTag,

    /// Drop the entire event itself.
    DropEvent,
}

const fn default_limit_exceeded_action() -> LimitExceededAction {
    LimitExceededAction::DropTag
}

const fn default_value_limit() -> usize {
    500
}

pub(crate) const fn default_cache_size() -> usize {
    600 // 600 bytes gives ~1% false positive rate for 500 items
}

pub(crate) const fn default_max_tag_size() -> usize {
    256
}

impl GenerateConfig for TagCardinalityLimitConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            mode: Mode::Exact,
            value_limit: default_value_limit(),
            limit_exceeded_action: default_limit_exceeded_action(),
            max_tag_size: default_max_tag_size(),
            tags: None,
            exclude_tags: None,
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_tag_cardinality_limit")]
impl TransformConfig for TagCardinalityLimitConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::event_task(TagCardinalityLimit::new(
            self.clone(),
            context.mezmo_ctx.clone(),
        )))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: enrichment::TableRegistry,
        _: &[(OutputId, schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Log, HashMap::new())]
    }
}
