use chrono::Utc;
// use futures::StreamExt;
use vector_lib::codecs::BytesDeserializerConfig;
use vector_lib::config::SourceOutput;
use vector_lib::configurable::configurable_component;
// use vector_lib::EstimatedJsonEncodedSizeOf;
use vector_lib::{config::LogNamespace, schema::Definition};

use crate::mezmo::pipeline_state_variable_change_action::PipelineStateVariableChangeActionSubscription;
use crate::{
    config::{DataType, SourceConfig, SourceContext},
    event::Event,
    internal_events::{InternalLogsBytesReceived, InternalLogsEventsReceived, StreamClosedError},
    shutdown::ShutdownSignal,
    SourceSender,
};

/// Configuration for the Mezmo Pipeline State Variable Change source component.
#[configurable_component(source("mezmo_pipeline_state_variable_change"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoPipelineStateVariableChangeConfig {
    // Despite the name, this option is an opt-in boolean to a new namespaced LogEvent shape
    // https://github.com/vectordotdev/vector/issues/12187
    /// The namespace to use for logs. This overrides the global setting.
    #[configurable(metadata(docs::hidden))]
    #[serde(default)]
    log_namespace: Option<bool>,
}

impl_generate_config_from_default!(MezmoPipelineStateVariableChangeConfig);

impl MezmoPipelineStateVariableChangeConfig {
    fn schema_definition(&self, log_namespace: LogNamespace) -> Definition {
        BytesDeserializerConfig
            .schema_definition(log_namespace)
            .with_standard_vector_source_metadata()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_pipeline_state_variable_change")]
impl SourceConfig for MezmoPipelineStateVariableChangeConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let subscription = PipelineStateVariableChangeActionSubscription::subscribe();

        let log_namespace = cx.log_namespace(self.log_namespace);

        Ok(Box::pin(mezmo_pipeline_state_variable_change(
            subscription,
            cx.out,
            cx.shutdown,
            log_namespace,
        )))
    }

    fn outputs(&self, global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        let schema_definition =
            self.schema_definition(global_log_namespace.merge(self.log_namespace));

        vec![SourceOutput::new_maybe_logs(
            DataType::Log,
            schema_definition,
        )]
    }

    fn can_acknowledge(&self) -> bool {
        false
    }
}

async fn mezmo_pipeline_state_variable_change(
    subscription: PipelineStateVariableChangeActionSubscription,
    mut out: SourceSender,
    shutdown: ShutdownSignal,
    log_namespace: LogNamespace,
) -> Result<(), ()> {
    let mut log_stream = subscription.into_stream().take_until(shutdown);

    while let Some(mut log) = log_stream.next().await {
        let byte_size = log.estimated_json_encoded_size_of().get();
        emit!(InternalLogsBytesReceived { byte_size });
        emit!(InternalLogsEventsReceived {
            count: 1,
            byte_size: byte_size.into(),
        });

        log_namespace.insert_standard_vector_source_metadata(
            &mut log,
            MezmoPipelineStateVariableChangeConfig::NAME,
            Utc::now(),
        );

        if let Err(_) = out.send_event(Event::from(log)).await {
            // this wont trigger any infinite loop considering it stops the component
            emit!(StreamClosedError { count: 1 });
            return Err(());
        }
    }

    Ok(())
}
