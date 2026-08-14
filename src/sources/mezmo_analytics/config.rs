use vector_lib::{
    config::LogNamespace,
    configurable::configurable_component,
    mezmo::analytics::{AnalyticsOutput, AnalyticsSubscription},
    schema::Definition,
};

use crate::config::{DataType, SourceConfig, SourceContext, SourceOutput};

/// Configuration for the `mezmo_analytics` source.
#[configurable_component(source(
    "mezmo_analytics",
    "Expose Mezmo generated analytic records such as usage metrics."
))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoAnalyticsConfig {}

impl_generate_config_from_default!(MezmoAnalyticsConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_analytics")]
impl SourceConfig for MezmoAnalyticsConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<crate::sources::Source> {
        Ok(Box::pin(super::run(
            AnalyticsSubscription::subscribe_all(),
            cx.out,
            cx.shutdown,
        )))
    }

    fn outputs(&self, _global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        AnalyticsOutput::all()
            .into_iter()
            .map(|output| {
                SourceOutput::new_maybe_logs(DataType::Log, Definition::any())
                    .with_port(output.as_str())
            })
            .collect()
    }

    fn can_acknowledge(&self) -> bool {
        false
    }
}
