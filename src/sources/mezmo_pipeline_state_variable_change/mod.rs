use chrono::Utc;
use futures::StreamExt;
use vector_lib::EstimatedJsonEncodedSizeOf;
use vector_lib::codecs::BytesDeserializerConfig;
use vector_lib::config::SourceOutput;
use vector_lib::configurable::configurable_component;
use vector_lib::{config::LogNamespace, schema::Definition};

use crate::{
    SourceSender,
    config::{DataType, SourceConfig, SourceContext},
    event::Event,
    internal_events::{InternalLogsBytesReceived, InternalLogsEventsReceived, StreamClosedError},
    shutdown::ShutdownSignal,
};
use mezmo::pipeline_state_variable_change_action::PipelineStateVariableChangeActionSubscription;

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
            emit!(StreamClosedError { count: 1 });
            return Err(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use futures::Stream;
    use serial_test::serial;
    use tokio::time::{Duration, sleep};

    use super::*;
    use crate::{
        event::Event,
        test_util::{
            collect_ready,
            components::{SOURCE_TAGS, assert_source_compliance},
        },
    };
    use mezmo::MezmoContext;

    #[test]
    #[serial]
    fn generates_config() {
        crate::test_util::test_generate_config::<MezmoPipelineStateVariableChangeConfig>();
    }

    #[tokio::test]
    #[serial]
    async fn receives_logs_string() {
        assert_source_compliance(&SOURCE_TAGS, async {
            let rx = start_source().await;

            let id = "v1:js-script:internal_source:component_abc:pipeline_abc:account_a".to_owned();
            let ctx = MezmoContext::try_from(id).ok();
            mezmo::set_pipeline_state_variable!(
                &ctx,
                None,
                "test_variable1".to_string(),
                vrl::value::Value::from("test_value1".to_string())
            );

            sleep(Duration::from_millis(1)).await;
            let events = collect_ready(rx).await;

            let event = events.into_iter().next().unwrap();
            let actual = event.as_log();

            let event_type = actual
                .get(".meta.mezmo.pipeline_event_action")
                .expect("should contain a pipeline_event_action type")
                .as_str()
                .expect("pipeline_event_action should be a string");
            assert_eq!(event_type, "set_pipeline_state_variable".to_string());

            let account_id = actual
                .get(".meta.mezmo.account_id")
                .expect("should contain account_id")
                .as_str()
                .expect("account_id should be a string");
            assert_eq!(account_id, "account_a");

            let pipeline_id = actual
                .get(".meta.mezmo.pipeline_id")
                .expect("should contain pipeline_id")
                .as_str()
                .expect("pipeline_id should be a string");
            assert_eq!(pipeline_id, "pipeline_abc");

            let component_id = actual
                .get(".meta.mezmo.component_id")
                .expect("should contain component_id")
                .as_str()
                .expect("component_id should be a string");
            assert_eq!(component_id, "component_abc");

            let component_kind = actual
                .get(".meta.mezmo.component_kind")
                .expect("should contain component_kind")
                .as_str()
                .expect("component_kind should be a string");
            assert_eq!(component_kind, "source");

            let internal = actual
                .get(".meta.mezmo.internal")
                .expect("should contain internal")
                .as_boolean()
                .expect("internal should be a boolean");
            assert!(internal);

            let name = actual
                .get(".message.name")
                .expect("should contain message name")
                .as_str()
                .expect("message.name should be a string");
            assert_eq!(name, "test_variable1");

            let value = actual
                .get(".message.value")
                .expect("should contain a message value")
                .as_str()
                .expect("message.value should be a string");
            assert_eq!(value, "test_value1");
        })
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn receives_logs_number() {
        assert_source_compliance(&SOURCE_TAGS, async {
            let rx = start_source().await;

            let id = "v1:js-script:internal_source:component_abc:pipeline_abc:account_b".to_owned();
            let ctx = MezmoContext::try_from(id).ok();
            mezmo::set_pipeline_state_variable!(
                &ctx,
                None,
                "test_variable1".to_string(),
                vrl::value::Value::from(123)
            );

            sleep(Duration::from_millis(1)).await;
            let events = collect_ready(rx).await;

            let event = events.into_iter().next().unwrap();
            let actual = event.as_log();

            let event_type = actual
                .get(".meta.mezmo.pipeline_event_action")
                .expect("should contain a pipeline_event_action type")
                .as_str()
                .expect("pipeline_event_action should be a string");
            assert_eq!(event_type, "set_pipeline_state_variable".to_string());

            let account_id = actual
                .get(".meta.mezmo.account_id")
                .expect("should contain account_id")
                .as_str()
                .expect("account_id should be a string");
            assert_eq!(account_id, "account_b");

            let pipeline_id = actual
                .get(".meta.mezmo.pipeline_id")
                .expect("should contain pipeline_id")
                .as_str()
                .expect("pipeline_id should be a string");
            assert_eq!(pipeline_id, "pipeline_abc");

            let component_id = actual
                .get(".meta.mezmo.component_id")
                .expect("should contain component_id")
                .as_str()
                .expect("component_id should be a string");
            assert_eq!(component_id, "component_abc");

            let component_kind = actual
                .get(".meta.mezmo.component_kind")
                .expect("should contain component_kind")
                .as_str()
                .expect("component_kind should be a string");
            assert_eq!(component_kind, "source");

            let internal = actual
                .get(".meta.mezmo.internal")
                .expect("should contain internal")
                .as_boolean()
                .expect("internal should be a boolean");
            assert!(internal);

            let name = actual
                .get(".message.name")
                .expect("should contain message name")
                .as_str()
                .expect("message.name should be a string");
            assert_eq!(name, "test_variable1");

            let value = actual
                .get(".message.value")
                .expect("should contain a message value")
                .as_integer()
                .expect("message.value should be a number");
            assert_eq!(value, 123);
        })
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn receives_logs_array() {
        assert_source_compliance(&SOURCE_TAGS, async {
            let rx = start_source().await;

            let id = "v1:js-script:internal_source:component_abc:pipeline_abc:account_c".to_owned();
            let ctx = MezmoContext::try_from(id).ok();
            mezmo::set_pipeline_state_variable!(
                &ctx,
                None,
                "test_variable1".to_string(),
                vrl::value::Value::from(vec![1, 2, 3])
            );

            sleep(Duration::from_millis(1)).await;
            let events = collect_ready(rx).await;

            let event = events.into_iter().next().unwrap();
            let actual = event.as_log();

            let event_type = actual
                .get(".meta.mezmo.pipeline_event_action")
                .expect("should contain a pipeline_event_action type")
                .as_str()
                .expect("pipeline_event_action should be a string");
            assert_eq!(event_type, "set_pipeline_state_variable".to_string());

            let account_id = actual
                .get(".meta.mezmo.account_id")
                .expect("should contain account_id")
                .as_str()
                .expect("account_id should be a string");
            assert_eq!(account_id, "account_c");

            let pipeline_id = actual
                .get(".meta.mezmo.pipeline_id")
                .expect("should contain pipeline_id")
                .as_str()
                .expect("pipeline_id should be a string");
            assert_eq!(pipeline_id, "pipeline_abc");

            let component_id = actual
                .get(".meta.mezmo.component_id")
                .expect("should contain component_id")
                .as_str()
                .expect("component_id should be a string");
            assert_eq!(component_id, "component_abc");

            let component_kind = actual
                .get(".meta.mezmo.component_kind")
                .expect("should contain component_kind")
                .as_str()
                .expect("component_kind should be a string");
            assert_eq!(component_kind, "source");

            let internal = actual
                .get(".meta.mezmo.internal")
                .expect("should contain internal")
                .as_boolean()
                .expect("internal should be a boolean");
            assert!(internal);

            let name = actual
                .get(".message.name")
                .expect("should contain message name")
                .as_str()
                .expect("message.name should be a string");
            assert_eq!(name, "test_variable1");

            let value = actual
                .get(".message.value")
                .expect("should contain a message value")
                .as_array()
                .expect("message.value should be a an array");

            assert_eq!(value.len(), 3);
            assert_eq!(value[0].as_integer(), Some(1));
            assert_eq!(value[1].as_integer(), Some(2));
            assert_eq!(value[2].as_integer(), Some(3));
        })
        .await;
    }

    async fn start_source() -> impl Stream<Item = Event> + Unpin {
        let (tx, rx) = SourceSender::new_test();

        let source = MezmoPipelineStateVariableChangeConfig::default()
            .build(SourceContext::new_test(tx, None))
            .await
            .unwrap();
        tokio::spawn(source);
        sleep(Duration::from_millis(1)).await;
        rx
    }
}
