use chrono::Utc;
use codecs::BytesDeserializerConfig;
use futures::StreamExt;
use vector_config::configurable_component;
use vector_core::config::SourceOutput;
use vector_core::EstimatedJsonEncodedSizeOf;
use vector_core::{config::LogNamespace, schema::Definition};

use crate::mezmo::user_trace::UserLogSubscription;
use crate::{
    config::{DataType, SourceConfig, SourceContext},
    event::Event,
    internal_events::{InternalLogsBytesReceived, InternalLogsEventsReceived, StreamClosedError},
    shutdown::ShutdownSignal,
    SourceSender,
};

/// Configuration for the `mezmo_user_logs` source.
#[configurable_component(source("mezmo_user_logs"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoUserLogsConfig {
    // Despite the name, this option is an opt-in boolean to a new namespaced LogEvent shape
    // https://github.com/vectordotdev/vector/issues/12187
    /// The namespace to use for logs. This overrides the global setting.
    #[configurable(metadata(docs::hidden))]
    #[serde(default)]
    log_namespace: Option<bool>,
}

impl_generate_config_from_default!(MezmoUserLogsConfig);

impl MezmoUserLogsConfig {
    fn schema_definition(&self, log_namespace: LogNamespace) -> Definition {
        BytesDeserializerConfig
            .schema_definition(log_namespace)
            .with_standard_vector_source_metadata()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_user_logs")]
impl SourceConfig for MezmoUserLogsConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let subscription = UserLogSubscription::subscribe();

        let log_namespace = cx.log_namespace(self.log_namespace);

        Ok(Box::pin(mezmo_user_logs(
            subscription,
            cx.out,
            cx.shutdown,
            log_namespace,
        )))
    }

    fn outputs(&self, global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        let schema_definition =
            self.schema_definition(global_log_namespace.merge(self.log_namespace));

        vec![SourceOutput::new_logs(DataType::Log, schema_definition)]
    }

    fn can_acknowledge(&self) -> bool {
        false
    }
}

async fn mezmo_user_logs(
    subscription: UserLogSubscription,
    mut out: SourceSender,
    shutdown: ShutdownSignal,
    log_namespace: LogNamespace,
) -> Result<(), ()> {
    let mut log_stream = subscription.into_stream().take_until(shutdown);

    while let Some(mut log) = log_stream.next().await {
        let byte_size = log.estimated_json_encoded_size_of();
        emit!(InternalLogsBytesReceived { byte_size });
        emit!(InternalLogsEventsReceived {
            count: 1,
            byte_size,
        });

        log_namespace.insert_standard_vector_source_metadata(
            &mut log,
            MezmoUserLogsConfig::NAME,
            Utc::now(),
        );

        if let Err(error) = out.send_event(Event::from(log)).await {
            // this wont trigger any infinite loop considering it stops the component
            emit!(StreamClosedError { error, count: 1 });
            return Err(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use futures::Stream;
    use tokio::time::{sleep, Duration};

    use super::*;
    use crate::{
        event::Event,
        mezmo::{user_trace::MezmoUserLog, MezmoContext},
        test_util::{
            collect_ready,
            components::{assert_source_compliance, SOURCE_TAGS},
        },
    };

    #[test]
    fn generates_config() {
        crate::test_util::test_generate_config::<MezmoUserLogsConfig>();
    }

    #[tokio::test]
    async fn receives_logs() {
        assert_source_compliance(&SOURCE_TAGS, async {
            let rx = start_source().await;
            let start = chrono::Utc::now();

            let id = "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
            let ctx = MezmoContext::try_from(id).ok();

            crate::user_log_debug!(ctx, "debug msg");
            crate::user_log_info!(ctx, "info msg");
            crate::user_log_warn!(ctx, "warn msg");
            crate::user_log_error!(ctx, "error msg");

            let expected = vec![
                ("DEBUG", "debug msg"),
                ("INFO", "info msg"),
                ("WARN", "warn msg"),
                ("ERROR", "error msg"),
            ];

            sleep(Duration::from_millis(1)).await;
            let events = collect_ready(rx).await;

            let end = chrono::Utc::now();
            assert_eq!(events.len(), expected.len());

            for ((exp_level, exp_msg), actual) in expected.into_iter().zip(events.iter()) {
                let log = actual.as_log();

                let timestamp = *log["timestamp"]
                    .as_timestamp()
                    .expect("timestamp isn't a timestamp");
                assert!(timestamp >= start);
                assert!(timestamp <= end);

                let level = log
                    .get(".meta.mezmo.level")
                    .expect("should contain a level value")
                    .as_str()
                    .expect("level should be a string");
                assert_eq!(level, exp_level);

                let account_id = log
                    .get(".meta.mezmo.account_id")
                    .expect("should contain account_id")
                    .as_str()
                    .expect("account_id should be a string");
                assert_eq!(account_id, "account_123");

                let pipeline_id = log
                    .get(".meta.mezmo.pipeline_id")
                    .expect("should contain pipeline_id")
                    .as_str()
                    .expect("pipeline_id should be a string");
                assert_eq!(pipeline_id, "pipeline_123");

                let component_id = log
                    .get(".meta.mezmo.component_id")
                    .expect("should contain component_id")
                    .as_str()
                    .expect("component_id should be a string");
                assert_eq!(component_id, "component_abc");

                let component_kind = log
                    .get(".meta.mezmo.component_kind")
                    .expect("should contain component_kind")
                    .as_str()
                    .expect("component_kind should be a string");
                assert_eq!(component_kind, "source");

                let internal = log
                    .get(".meta.mezmo.internal")
                    .expect("should contain internal")
                    .as_boolean()
                    .expect("internal should be a boolean");
                assert!(internal);

                let msg = log
                    .get(".message")
                    .expect("should contain a message")
                    .as_str()
                    .expect("message should be a string");
                assert_eq!(msg, exp_msg);
            }
        })
        .await;
    }

    async fn start_source() -> impl Stream<Item = Event> + Unpin {
        let (tx, rx) = SourceSender::new_test();

        let source = MezmoUserLogsConfig::default()
            .build(SourceContext::new_test(tx, None))
            .await
            .unwrap();
        tokio::spawn(source);
        sleep(Duration::from_millis(1)).await;
        rx
    }
}
