mod service;

use std::collections::{HashMap, HashSet};

use async_stream::stream;
use futures::Stream;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tokio::time::{self, Instant};
use vector_lib::configurable::configurable_component;
use vector_lib::schema::Definition;

use crate::{
    config::{self, provider::ProviderConfig, ConfigBuilder, TransformContext},
    internal_events::mezmo_config::{
        MezmoConfigBuildFailure, MezmoConfigBuilderCreate, MezmoConfigReloadSignalSend,
        MezmoConfigVrlValidation, MezmoConfigVrlValidationError, MezmoGenerateConfigError,
    },
    providers::BuildResult,
    signal,
};
use mezmo::{user_trace::MezmoUserLog, MezmoContext};

use self::service::{ConfigService, DefaultConfigService};

/// Request settings.
#[configurable_component]
#[derive(Clone, Debug)]
pub struct RequestConfig {
    /// HTTP headers to add to the request.
    #[serde(default)]
    pub headers: IndexMap<String, String>,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            headers: IndexMap::new(),
        }
    }
}

/// Configuration for the `mezmo_partition` provider.
#[configurable_component(provider("mezmo_partition"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct MezmoPartitionConfig {
    /// Url of the "POST latest revision" endpoint
    latest_revisions_url: String,

    /// Url of the "POST loaded revisions" endpoint
    loaded_revisions_url: String,

    /// Url of the "GET pipelines by partition" endpoint
    pipelines_by_partition_url: String,

    /// The partition identifier
    partition_id: String,

    #[configurable(derived)]
    request: RequestConfig,

    /// How often to poll the provider, in seconds.
    poll_interval_secs: u64,

    /// Validate and reject any invalid VRL snippets (currently supports only remap transforms)
    validate_vrl: bool,
}

// Serde requires Default trait
impl Default for MezmoPartitionConfig {
    fn default() -> Self {
        Self {
            latest_revisions_url:
                "http://pipeline-service/internal/pipelines/config/latest_revisions".into(),
            loaded_revisions_url:
                "http://pipeline-service/internal/pipelines/config/loaded_revisions".into(),
            pipelines_by_partition_url:
                "http://pipeline-service/internal/partitions/{partition_id}/pipelines".into(),
            partition_id: "sample_partition".into(),
            request: RequestConfig::default(),
            poll_interval_secs: 2,
            validate_vrl: false,
        }
    }
}

// Trait required by `ComponentDescription`
impl_generate_config_from_default!(MezmoPartitionConfig);

/// Polls the config endpoints, returning a stream of `ConfigBuilder`.
fn poll_config(
    poll_interval_secs: u64,
    mut mezmo_config_builder: MezmoConfigBuilder,
) -> impl Stream<Item = signal::SignalTo> {
    let poll_interval = time::Duration::from_secs(poll_interval_secs);
    let mut last_run = time::Instant::now();

    stream! {
        loop {
            if last_run.elapsed() < poll_interval {
                // Sleep at most poll_interval
                let delay = poll_interval - last_run.elapsed();
                debug!("Mezmo partition config sleeping for {delay:?}");
                time::sleep(delay).await;
            }

            last_run = time::Instant::now();

            debug!("Building incrementally from polling loop");
            match mezmo_config_builder.build_incrementally().await {
                Ok((Some(config_builder), loaded)) => {
                    emit!(MezmoConfigReloadSignalSend {});
                    yield signal::SignalTo::ReloadFromConfigBuilder(config_builder);
                    if !loaded.is_empty() {
                        mezmo_config_builder.service.set_loaded_revisions(loaded).await.unwrap_or_else(|e| {
                            error!("Error setting loaded revisions: {e}");
                        });
                    }
                },
                Ok((None, _)) => {
                    // No changes -> keep polling
                },
                Err(e) => {
                    emit!(MezmoConfigBuildFailure { error: e});
                },
            };
        }
    }
}

// Alias types for readability
type PipelineId = String;
type RevisionId = String;
type ProfilerTransformId = String;

struct MezmoConfigBuilder {
    service: Box<dyn ConfigService>,
    cache: HashMap<PipelineId, Revision>,

    pipelines: Option<Vec<PipelineId>>,
    common_config: Option<String>,

    validate_vrl: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Revision {
    id: RevisionId,
    profiler_transform_ids: Option<Vec<ProfilerTransformId>>,
    config: String,
}

impl MezmoConfigBuilder {
    fn new(config: &MezmoPartitionConfig) -> Self {
        MezmoConfigBuilder {
            service: Box::new(DefaultConfigService::new(config)),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: config.validate_vrl,
        }
    }

    /// Tries to build the configuration from scratch.
    /// It errors out (crashing the process) when getting the partition info fails or no config can be fetched.
    async fn build_all(&mut self) -> Result<ConfigBuilder, Vec<String>> {
        debug!("Initial configuration build started");
        trace!("Fetching pipelines by partition");
        let (pipelines, common_config) = self
            .service
            .get_pipelines_by_partition()
            .await
            .map_err(|e| vec![e])?;

        trace!("Fetching revisions for {} pipelines", pipelines.len());
        let revisions = self
            .service
            .get_new_revisions(
                pipelines
                    .iter()
                    .map(|id| (id.clone(), None, None))
                    .collect(),
            )
            .await
            .map_err(|e| vec![e])?;

        info!(
            "Building initial configuration for {} pipelines ({} revisions)",
            pipelines.len(),
            revisions.len()
        );
        self.pipelines = Some(pipelines);
        self.common_config = Some(common_config);
        let mut cache = HashMap::new();

        // Don't store in self.cache until the initial topology is built, then replace
        // self.cache with the new one.
        for (pipeline_id, revision) in revisions.into_iter() {
            cache.insert(pipeline_id, revision);
        }

        // Optimistically try to build the partition config from the initial cache.
        // If this fails, the cache is discarded and we fall back to an incremental build
        // via the execution of the provider's polling loop.
        let common_config = self.common_config.as_ref().unwrap();
        match generate_config(common_config, &cache, None, self.validate_vrl).await {
            Ok(r) => {
                info!(
                    "Initial configuration was successfully built with {} pipelines",
                    cache.len()
                );
                self.cache = cache;
                return Ok(r);
            }
            Err(errors) => {
                emit!(MezmoGenerateConfigError {
                    errors,
                    pipeline_id: None,
                    revision_id: None,
                    incremental: false,
                    cache_len: cache.len(),
                });
            }
        };

        warn!("Failed to build full configuration, falling back to incremental build");

        // Try to build using just the common config to allow the process to run
        // while the polling loop executes.
        cache = HashMap::new();
        match generate_config(common_config, &cache, None, self.validate_vrl).await {
            Ok(r) => {
                info!("Common configuration was successfully built without any pipeline");
                Ok(r)
            }
            Err(errors) => {
                emit!(MezmoGenerateConfigError {
                    errors: errors.clone(),
                    pipeline_id: None,
                    revision_id: None,
                    incremental: false,
                    cache_len: 0,
                });
                Err(errors)
            }
        }
    }

    /// Creates `ConfigBuilder` instances incrementally by each new revision found, returning
    /// once all valid config have been loaded.
    /// It errors out when getting the partition info or getting new revisions fail (should be retried periodically).
    async fn build_incrementally(
        &mut self,
    ) -> Result<(Option<ConfigBuilder>, Vec<(PipelineId, RevisionId)>), String> {
        debug!("Incremental config build starting");
        trace!("Fetching pipelines by partition");
        let (pipelines, common_config) = self.service.get_pipelines_by_partition().await?;

        // Compare pipelines ids, delete pipelines that no longer exist
        let pipelines_removed = self.remove_diff(&pipelines);

        let common_config_changed = self.common_config.as_deref() != Some(common_config.as_str());
        self.pipelines = Some(pipelines);
        self.common_config = Some(common_config);

        // Incrementally build the configuration
        let pipelines_with_changes = self.get_pipeline_with_config_changes().await?;

        info!(
            "Incrementally building configuration for {} pipelines pipelines_removed={} common_config_changed={}",
            pipelines_with_changes.len(),
            pipelines_removed,
            common_config_changed
        );

        let common_config = self.common_config.as_ref().unwrap();
        let mut result_builder = None;
        let mut loaded: Vec<(PipelineId, RevisionId)> = Vec::new();

        if pipelines_removed || common_config_changed {
            debug!(
                message = "Rebuilding existing topology configuration",
                pipelines_removed, common_config_changed
            );

            // VRL for existing pipelines has already been validated. If we are regenerating config
            // only due to `pipelines_removed`, we don't need to validate VRL again. Only re-validate if
            // the `common_config` has changed and we are configured to do so.
            let validate_vrl = common_config_changed && self.validate_vrl;
            match generate_config(common_config, &self.cache, None, validate_vrl).await {
                Ok(builder) => {
                    result_builder = Some(builder);
                }
                Err(errors) => {
                    emit!(MezmoGenerateConfigError {
                        errors,
                        pipeline_id: None,
                        revision_id: None,
                        incremental: true,
                        cache_len: self.cache.len(),
                    });
                }
            }
        }

        if pipelines_with_changes.is_empty() {
            info!("No config changes for individual pipelines")
        } else {
            info!(
                "Config changes for {} pipelines",
                pipelines_with_changes.len()
            )
        }

        for (pipeline_id, revision) in pipelines_with_changes.into_iter() {
            match self.cache.get_key_value(&pipeline_id) {
                Some(existing) if existing.1.id != revision.id => {
                    info!(
                        "Building config for new revision {} of pipeline {}",
                        &revision.id, &pipeline_id
                    );
                }
                Some(_) => {
                    info!(
                        "Refreshing config for revision {} of pipeline {}",
                        &revision.id, &pipeline_id
                    );
                }
                None => {
                    info!(
                        "Building config for new pipeline {} with revision {}",
                        &pipeline_id, &revision.id
                    );
                }
            }

            match generate_config(
                common_config,
                &self.cache,
                Some((&pipeline_id, &revision)),
                self.validate_vrl,
            )
            .await
            {
                Ok(builder) => {
                    loaded.push((pipeline_id.clone(), revision.id.clone()));
                    self.cache.insert(pipeline_id, revision);
                    result_builder = Some(builder);
                }
                Err(errors) => {
                    emit!(MezmoGenerateConfigError {
                        errors,
                        pipeline_id: Some(pipeline_id.clone()),
                        revision_id: Some(revision.id.clone()),
                        incremental: true,
                        cache_len: self.cache.len(),
                    });
                }
            }
        }

        if !loaded.is_empty() {
            info!(
                "Incremental config build generated {} configurations successfully",
                loaded.len()
            );
        }

        emit!(MezmoConfigBuilderCreate {
            revisions: self.cache.len()
        });
        Ok((result_builder, loaded))
    }

    /// Extracts the current revisions of each pipeline and fetches the new configuration (if any).
    async fn get_pipeline_with_config_changes(
        &self,
    ) -> Result<HashMap<PipelineId, Revision>, String> {
        let pipelines = self.pipelines.as_ref().expect("pipelines can not be None");
        let current_revisions: Vec<_> = pipelines
            .iter()
            .map(|pipeline_id| {
                let (revision_id, profiler_transform_ids) = self
                    .cache
                    .get(pipeline_id)
                    .map(|r| (r.id.clone(), r.profiler_transform_ids.clone()))
                    .unzip();
                (
                    pipeline_id.clone(),
                    revision_id,
                    profiler_transform_ids.flatten(),
                )
            })
            .collect();

        trace!(
            "Fetching new revisions ({} existing)",
            current_revisions.len()
        );
        self.service.get_new_revisions(current_revisions).await
    }

    fn remove_diff(&mut self, expected: &[PipelineId]) -> bool {
        let existing: HashSet<_> = self.cache.keys().collect();
        let expected: HashSet<_> = expected.iter().collect();
        let diff: Vec<_> = existing
            .difference(&expected)
            .map(|x| x.to_string())
            .collect();

        for pipeline_id in diff.iter() {
            trace!("Removing pipeline {} from cache", pipeline_id);
            self.cache.remove(pipeline_id);
        }

        !diff.is_empty()
    }
}

/// Attempts to create a `ConfigBuilder` with the provided revisions, with an optional updated revision.
async fn generate_config(
    common_config: &str,
    revisions: &HashMap<PipelineId, Revision>,
    updated: Option<(&PipelineId, &Revision)>,
    validate_vrl: bool,
) -> BuildResult {
    match updated {
        Some((pipeline_id, revision)) => {
            debug!(
                "Generating config for {} revisions, with updated revision {}:{}",
                revisions.len(),
                pipeline_id,
                revision.id
            );
        }
        None => {
            debug!("Generating config for {} revisions", revisions.len());
        }
    }

    let mut parts = Vec::with_capacity(revisions.len() + 2);
    parts.push(common_config);
    for (id, r) in revisions {
        match updated {
            Some((updated_id, _)) if updated_id == id => {
                // The updated revision will be added after the loop
                continue;
            }
            _ => {
                // Add the existing pipeline
                parts.push(&r.config);
            }
        }
    }

    // Add the updated revision
    if let Some((_, r)) = updated {
        parts.push(&r.config);
    }

    let config_str = parts.join("\n");

    trace!("Loading assembled config from {} parts", parts.len());

    let config_builder = config::load::<_, ConfigBuilder>(
        config_str.as_bytes(),
        crate::config::format::Format::Toml,
    )
    .inspect_err(|warnings| {
        if !warnings.is_empty() {
            warn!("{} warnings during config load", warnings.len());
            for warning in warnings.iter() {
                warn!("Config load warn: {}", warning);
            }
        }
    })?;

    trace!("Loaded assembled config");

    if !validate_vrl {
        debug!(message = "Skipping transform VRL validation", validate_vrl);
        return Ok(config_builder);
    }

    let start = Instant::now();
    let errors = if let Some((_, r)) = updated {
        // Update only validates the new pipeline's configuration
        debug!(
            "Validating transforms for updated pipeline revision {}",
            r.id
        );
        let config_builder = config::load::<_, ConfigBuilder>(
            r.config.as_bytes(),
            crate::config::format::Format::Toml,
        )?;
        // Warnings would have already been handled above in the full config load...
        validate_vrl_transforms(&config_builder).await
    } else {
        debug!(
            "Validating transforms for {} pipeline revisions",
            revisions.len()
        );
        validate_vrl_transforms(&config_builder).await
    };
    emit!(MezmoConfigVrlValidation {
        elapsed: Instant::now() - start
    });

    errors?; // Return errors after emitting the metric

    Ok(config_builder)
}

async fn validate_vrl_transforms(config_builder: &ConfigBuilder) -> Result<(), Vec<String>> {
    let mut failures = Vec::new();
    if let Ok(config) = config_builder.clone().build_no_validation() {
        let enrichment_tables = vector_lib::enrichment::tables::TableRegistry::default();

        for (key, transform) in config.transforms() {
            trace!("Validating transform {key}");

            // IMPORTANT: This is not properly setting up schema or enrichment
            // tables as part of the validation. These would need to be
            // added if we want to support those.
            //
            // Use the default schema for the legacy namespace for validation. Collecting definitions
            // from all ancestors is expensive for large graphs, and currently in our model everything is
            // using the default schema anyway.
            let schema_definitions = HashMap::new();
            let merged_definition = Definition::default_legacy_namespace();

            let transform = &transform.inner;
            // Handling only remaps currently, but could be extended in the future
            if transform.get_component_name() == "remap" {
                let mezmo_ctx = MezmoContext::try_from(key.clone().into_id()).ok();
                let context = TransformContext {
                    key: Some(key.clone()),
                    globals: config.global.clone(),
                    enrichment_tables: enrichment_tables.clone(),
                    schema_definitions,
                    merged_schema_definition: merged_definition.clone(),
                    mezmo_ctx: mezmo_ctx.clone(),
                    schema: config.schema,
                    extra_context: crate::extra_context::ExtraContext::default(),
                };
                // Compile the VRL snippet in the transform
                trace!("Compiling and validating VRL for transform {key}");
                if let Err(error) = transform.build(&context).await {
                    if let Some(ctx) = &mezmo_ctx {
                        match &ctx.pipeline_id {
                            Some(mezmo::ContextIdentifier::Value { id: _ }) => {
                                mezmo::user_log_error!(
                                    mezmo_ctx,
                                    "Error loading existing transform component. Please contact support"
                                );
                                failures.push(format!(
                                    "Error validating VRL in transform {key}: {error}"
                                ));
                            }
                            Some(mezmo::ContextIdentifier::Shared) => {
                                // This shouldn't happen...
                                failures
                                    .push(format!("Invalid VRL found in shared component {key}"));
                            }
                            None => {
                                // Ignore config validation for non-pipeline components (analysis)
                            }
                        }
                    }
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        emit!(MezmoConfigVrlValidationError {
            failure_count: failures.len() as u64
        });
        Err(failures)
    }
}

impl ProviderConfig for MezmoPartitionConfig {
    async fn build(&mut self, signal_handler: &mut signal::SignalHandler) -> BuildResult {
        let poll_interval_secs = self.poll_interval_secs;

        let mut mezmo_config_builder = MezmoConfigBuilder::new(self);
        let config_builder = mezmo_config_builder.build_all().await?;

        // Poll for changes to remote configuration.
        signal_handler.add(poll_config(poll_interval_secs, mezmo_config_builder));

        Ok(config_builder)
    }
}

#[cfg(test)]
mod tests {
    use crate::extra_context::ExtraContext;
    use crate::topology;

    use super::*;
    use mockall::mock;
    use serde_json::json;
    use wiremock::{
        matchers::{self, path},
        Mock, MockServer, ResponseTemplate,
    };

    macro_rules! S {
        ($x: expr) => {
            String::from($x)
        };
    }

    mock! {
        ConfigService {}
        #[async_trait::async_trait]
        impl service::ConfigService for ConfigService {
            async fn get_pipelines_by_partition(&self) -> Result<(Vec<PipelineId>, String), String>;
            async fn get_new_revisions(
                &self,
                current_revisions: Vec<(PipelineId, Option<RevisionId>, Option<Vec<ProfilerTransformId>>)>,
            ) -> Result<HashMap<PipelineId, Revision>, String>;
            async fn set_loaded_revisions(
                &self,
                revisions: Vec<(PipelineId, RevisionId)>,
            ) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn build_all_fails_when_getting_partition_info_fails_test() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Err(S!("get pipelines test error")));

        let mut b = new_test_builder(Box::new(service));

        let r = b.build_all().await;
        let errors = r.unwrap_err();
        assert_eq!(errors, vec!["get pipelines test error".to_string()]);
    }

    #[tokio::test]
    async fn build_all_fails_when_getting_revisions_fails_test() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((Vec::new(), "sample = true".to_string())));
        service
            .expect_get_new_revisions()
            .returning(|_| Err("get new revisions error".into()));

        let mut b = new_test_builder(Box::new(service));

        let r = b.build_all().await;
        let errors = r.unwrap_err();
        assert_eq!(errors, vec!["get new revisions error".to_string()]);
    }

    #[tokio::test]
    async fn build_all_should_load_minimal_conf_when_it_fails_to_load_individual_pipelines_test() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("a"),
                    profiler_transform_ids: None,
                    config: S!("invalid_test_config"),
                },
            )]))
        });

        let mut b = new_test_builder(Box::new(service));
        b.build_all().await.expect("to build successfully");
    }

    #[tokio::test]
    async fn build_all_should_allow_valid_vrl_when_validating() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("revision1"),
                    profiler_transform_ids: None,
                    config: S!(r#"
                    [sources.in]
                    type="stdin"

                    # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                    [transforms."v1:remap:transform:component1:pipeline1:account1"]
                    inputs=["in"]
                    type="remap"
                    source="""
                    i, err = parse_int("123")
                    """
                    "#),
                },
            )]))
        });

        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: true, // Validated, and should succeed
        };
        let config_builder = b.build_all().await.expect("to build successfully");
        assert!(
            b.cache.contains_key("pipeline1"),
            "pipeline should be in the cache"
        );
        let result = validate_config(config_builder).await;
        assert!(result.is_ok(), "expected the invalid VRL to be excluded");
    }

    #[tokio::test]
    async fn build_all_should_fail_on_invalid_vrl_when_disabled() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("revision1"),
                    profiler_transform_ids: None,
                    config: S!(r#"
                    [sources.in]
                    type="stdin"

                    # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                    [transforms."v1:remap:transform:component1:pipeline1:account1"]
                    inputs=["in"]
                    type="remap"
                    source="""
                    a = invalid("abc")
                    """
                    "#),
                },
            )]))
        });
        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: false, // Expect an error
        };
        let config_builder = b.build_all().await.expect("to build successfully");
        assert!(
            b.cache.contains_key("pipeline1"),
            "pipeline should be in the cache"
        );
        let result = validate_config(config_builder).await;
        assert!(
            result.is_err(),
            "expected an error when building invalid VRL without validation"
        );
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("undefined function")),
                "expected to fail with \"undefined function\""
            );
        }
    }

    #[tokio::test]
    async fn build_all_should_handle_invalid_vrl_when_validating() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("revision1"),
                    profiler_transform_ids: None,
                    config: S!(r#"
                    [sources.in]
                    type="stdin"

                    # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                    [transforms."v1:remap:transform:component1:pipeline1:account1"]
                    inputs=["in"]
                    type="remap"
                    source="""
                    a = invalid("abc")
                    """
                    "#),
                },
            )]))
        });
        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: true, // Expect no error to happen
        };
        let config_builder = b.build_all().await.expect("to build successfully");
        assert!(
            !b.cache.contains_key("pipeline1"),
            "pipeline should NOT be in the cache"
        );
        let result = validate_config(config_builder).await;
        assert!(result.is_ok(), "expected the invalid VRL to be excluded");
    }

    #[tokio::test]
    async fn build_incrementally_should_fail_when_getting_partition_info_fails_test() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Err(S!("get pipelines test error")));

        let mut b = new_test_builder(Box::new(service));
        let r = b.build_incrementally().await;
        assert!(matches!(r, Err(e) if e == "get pipelines test error"));
    }

    #[tokio::test]
    async fn build_incrementally_with_service_should_yield_only_changes() -> Result<(), String> {
        let partition_id = S!("part1");
        let pipelines_by_partition_url = "/internal/partitions/part1/pipelines";
        let latest_revisions_url = "/internal/pipelines/config/latest_revisions";
        let loaded_revisions_url = "/internal/pipelines/config/loaded_revisions";
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("GET"))
            .and(path(pipelines_by_partition_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                "pipeline_ids": ["pipeline1", "pipeline2"],
                "common_config_toml": "data_dir = \"/data/vector\""
            }"#,
                "application/json",
            ))
            .up_to_n_times(2)
            .with_priority(1)
            .mount(&mock_server)
            .await;

        // No pipelines after that
        Mock::given(matchers::method("GET"))
            .and(path(pipelines_by_partition_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                "pipeline_ids": [],
                "common_config_toml": "data_dir = \"/data/vector\""
            }"#,
                "application/json",
            ))
            .with_priority(2)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(path(latest_revisions_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{
                "pipeline1": {"id": "rev1", "config": "[sources.in1]\ntype = \"test_basic\"\n\n[sinks.out1]\ninputs = [\"in1\"]\ntype = \"test_basic\""},
                "pipeline2": {"id": "rev999", "profiler_transform_ids": ["prof1"], "config": "[sources.in2]\ntype = \"test_basic\"\n\n[sinks.out2]\ninputs = [\"in2\"]\ntype = \"test_basic\""}
            }"#, "application/json"))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(path(latest_revisions_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw("{}", "application/json"))
            .up_to_n_times(2)
            .with_priority(2)
            .mount(&mock_server)
            .await;

        let expected_loaded_revisions = json!({
            "revisions": [
                {"pipeline_id": "pipeline1", "revision_id": "rev1"},
                {"pipeline_id": "pipeline2", "revision_id": "rev999"}
            ]
        });
        Mock::given(matchers::method("POST"))
            .and(path(loaded_revisions_url))
            .and(matchers::body_json(&expected_loaded_revisions))
            .respond_with(ResponseTemplate::new(200).set_body_raw("{}", "application/json"))
            .with_priority(2)
            .mount(&mock_server)
            .await;

        let partition_config = MezmoPartitionConfig {
            latest_revisions_url: format!("{}{}", mock_server.uri(), latest_revisions_url),
            loaded_revisions_url: format!("{}{}", mock_server.uri(), loaded_revisions_url),
            pipelines_by_partition_url: format!(
                "{}{}",
                mock_server.uri(),
                pipelines_by_partition_url
            ),
            partition_id,
            request: RequestConfig::default(),
            poll_interval_secs: 0,
            validate_vrl: false,
        };

        let service = DefaultConfigService::new(&partition_config);
        let mut b = new_test_builder(Box::new(service));

        let (_, mut loaded) = b.build_incrementally().await?;
        loaded.sort_by_key(|x| x.0.clone());

        // First time
        assert_eq!(loaded.len(), 2, "Two pipelines");
        assert_eq!(
            loaded[0],
            ("pipeline1".into(), "rev1".into()),
            "First pipeline"
        );
        assert_eq!(
            loaded[1],
            ("pipeline2".into(), "rev999".into()),
            "Second pipeline"
        );
        assert_eq!(b.cache.len(), 2, "Pipelines cached");

        // First request for latest revisions should not include revision_id or
        // profiler_transform_ids
        assert_eq!(
            mock_server.received_requests().await.unwrap()[1].body,
            serde_json::to_vec(&json!({
                "revisions": [
                    {"pipeline_id":"pipeline1"},
                    {"pipeline_id":"pipeline2"}
                ]
            }))
            .unwrap()
        );

        // Second time
        let (config_builder, loaded) = b.build_incrementally().await?;
        assert_eq!(loaded.len(), 0, "No new events");
        assert!(config_builder.is_none(), "No new config");
        assert_eq!(b.cache.len(), 2, "Pipelines still cached");

        // Second request for latest revisions should include "profiler_transform_ids"
        assert_eq!(
            mock_server.received_requests().await.unwrap()[3].body,
            serde_json::to_vec(&json!({
                "revisions": [
                    {"pipeline_id":"pipeline1", "revision_id":"rev1"},
                    {"pipeline_id":"pipeline2", "revision_id":"rev999", "profiler_transform_ids": ["prof1"]}
                ]
            })).unwrap()
        );

        // Following times
        let (_, loaded) = b.build_incrementally().await?;
        assert_eq!(loaded.len(), 0, "No new events");
        assert_eq!(b.cache.len(), 0, "Pipelines removed from cache");
        Ok(())
    }

    #[tokio::test]
    async fn build_incrementally_should_reject_invalid_config() -> Result<(), String> {
        let partition_id = S!("part1");
        let pipelines_by_partition_url = "/internal/partitions/part1/pipelines";
        let latest_revisions_url = "/internal/pipelines/config/latest_revisions";
        let loaded_revisions_url = "/internal/pipelines/config/loaded_revisions";
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("GET"))
            .and(path(pipelines_by_partition_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                "pipeline_ids": ["pipeline1", "pipeline2", "pipeline3", "pipeline4"],
                "common_config_toml": "data_dir = \"/data/vector\""
            }"#,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(path(latest_revisions_url))
            .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{
                "pipeline1": {"id": "rev1", "config": "[sources.in]\ntype = \"test_basic\"\n\n[sinks.out]\ninputs = [\"in\"]\ntype = \"test_basic\""},
                "pipeline2": {"id": "rev2", "config": "\nTHIS_IS_INVALID"},
                "pipeline3": {"id": "rev3", "config": "[sources.in]\ntype = \"DOES_NOT_EXIST\"\n"},
                "pipeline4": {"id": "rev3", "config": "\n# THIS IS A GOOD ONE"}
            }"#, "application/json"))
            .mount(&mock_server)
            .await;

        let partition_config = MezmoPartitionConfig {
            latest_revisions_url: format!("{}{}", mock_server.uri(), latest_revisions_url),
            loaded_revisions_url: format!("{}{}", mock_server.uri(), loaded_revisions_url),
            pipelines_by_partition_url: format!(
                "{}{}",
                mock_server.uri(),
                pipelines_by_partition_url
            ),
            partition_id,
            request: RequestConfig::default(),
            poll_interval_secs: 0,
            validate_vrl: false,
        };

        let service = DefaultConfigService::new(&partition_config);
        let mut b = new_test_builder(Box::new(service));

        let (_, loaded) = b.build_incrementally().await?;
        assert_eq!(loaded.len(), 2, "First reload and then the good pipelines");
        assert!(b.cache.contains_key("pipeline1"), "Pipeline 1 cached");
        assert!(b.cache.contains_key("pipeline4"), "Pipeline 3 cached");
        assert_eq!(b.cache.len(), 2, "Pipelines cached");
        Ok(())
    }

    #[tokio::test]
    async fn build_incrementally_should_allow_valid_vrl_when_validating() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("revision1"),
                    profiler_transform_ids: None,
                    config: S!(r#"
                    [sources.in]
                    type="stdin"

                    # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                    [transforms."v1:remap:transform:component1:pipeline1:account1"]
                    inputs=["in"]
                    type="remap"
                    source="""
                    i, err = parse_int("123")
                    """
                    "#),
                },
            )]))
        });

        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: true, // Validated, and should succeed
        };
        let (config_builder, loaded) = b
            .build_incrementally()
            .await
            .expect("to build successfully");
        assert!(loaded
            .iter()
            .any(|(pipeline, _)| { pipeline == "pipeline1" })); // Loaded
        let result = validate_config(config_builder.unwrap()).await;
        assert!(result.is_ok(), "expected the invalid VRL to be excluded");
    }

    #[tokio::test]
    async fn build_incrementally_should_handle_invalid_vrl_when_validating() {
        let mut service = MockConfigService::new();
        service
            .expect_get_pipelines_by_partition()
            .returning(|| Ok((vec![S!("pipeline1")], S!("data_dir = \"/data/vector\""))));
        service.expect_get_new_revisions().returning(|_| {
            Ok(HashMap::from([(
                S!("pipeline1"),
                Revision {
                    id: S!("revision1"),
                    profiler_transform_ids: None,
                    config: S!(r#"
                    [sources.in]
                    type="stdin"

                    # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                    [transforms."v1:remap:transform:component1:pipeline1:account1"]
                    inputs=["in"]
                    type="remap"
                    source="""
                    a = invalid("abc")
                    """
                    "#),
                },
            )]))
        });
        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: true, // Expect no error to happen
        };
        let (config_builder, loaded) = b
            .build_incrementally()
            .await
            .expect("to build successfully");
        assert!(!loaded
            .iter()
            .any(|(pipeline, _)| { pipeline == "pipeline1" })); // Not loaded
        let result = validate_config(config_builder.unwrap()).await;
        assert!(result.is_ok(), "expected the invalid VRL to be excluded");
    }

    #[tokio::test]
    async fn build_incrementally_should_handle_invalid_vrl_when_common_config_changes() {
        let mut service = MockConfigService::new();
        service.expect_get_pipelines_by_partition().returning(|| {
            Ok((
                vec![S!("pipeline1")],
                S!("data_dir = \"/data/vector-changed\""),
            ))
        });

        service
            .expect_get_new_revisions()
            .returning(|_| Ok(HashMap::new()));

        let initial_pipelines = HashMap::from([(
            S!("pipeline1"),
            Revision {
                id: S!("revision1"),
                profiler_transform_ids: None,
                config: S!(r#"
                [sources.in]
                type="stdin"

                # Requires proper format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
                [transforms."v1:remap:transform:component1:pipeline1:account1"]
                inputs=["in"]
                type="remap"
                source="""
                a = invalid("abc")
                """
                "#),
            },
        )]);

        let mut b = MezmoConfigBuilder {
            service: Box::new(service),
            cache: initial_pipelines,
            pipelines: Some(vec![S!("pipeline1")]),
            common_config: Some(S!("data_dir = \"/data/vector\"")),
            validate_vrl: true,
        };
        let (config_builder, loaded) = b
            .build_incrementally()
            .await
            .expect("to build successfully");

        assert!(
            config_builder.is_none(),
            "no builder, existing pipeline still invalid"
        );
        assert!(!loaded
            .iter()
            .any(|(pipeline, _)| { pipeline == "pipeline1" })); // Still not loaded
    }

    fn new_test_builder(service: Box<dyn ConfigService>) -> MezmoConfigBuilder {
        MezmoConfigBuilder {
            service,
            cache: HashMap::new(),
            pipelines: None,
            common_config: None,
            validate_vrl: false,
        }
    }

    async fn validate_config(config_builder: ConfigBuilder) -> Result<(), Vec<String>> {
        let config = config_builder
            .build_no_validation()
            .expect("to build config successfully");

        let diff = config::ConfigDiff::initial(&config);
        topology::builder::TopologyPieces::build(
            &config,
            &diff,
            None,
            HashMap::new(),
            ExtraContext::default(),
        )
        .await
        .map(|_| ())
    }
}
