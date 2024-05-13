use chrono::Utc;
use futures::StreamExt;
use vector_lib::codecs::BytesDeserializerConfig;
use vector_lib::config::SourceOutput;
use vector_lib::configurable::configurable_component;
use vector_lib::EstimatedJsonEncodedSizeOf;
use vector_lib::{config::LogNamespace, schema::Definition};

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
        let byte_size = log.estimated_json_encoded_size_of().get();
        emit!(InternalLogsBytesReceived { byte_size });
        emit!(InternalLogsEventsReceived {
            count: 1,
            byte_size: byte_size.into(),
        });

        log_namespace.insert_standard_vector_source_metadata(
            &mut log,
            MezmoUserLogsConfig::NAME,
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

#[cfg(test)]
mod tests {
    use futures::Stream;
    use tokio::time::{sleep, Duration};

    use super::*;
    use crate::{
        event::Event,
        mezmo::{user_trace::MezmoUserLog, MezmoContext},
        sinks::prelude::Value,
        test_util::{
            collect_ready,
            components::{assert_source_compliance, SOURCE_TAGS},
        },
    };

    // The exitence of the captured data wrapper is a hidden implementation detail.
    // This function will create the wrapper so the values can be tested more directly
    // against what the value of the `.meta.mezmo.captured_data` field will be.
    fn captured_data_wrapper(captured_data: Value) -> Value {
        Value::from(btreemap! {
            "captured_data" => captured_data
        })
    }

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

            // With `captured_data` using the `user_log` macro
            crate::user_log!(
                "debug",
                ctx,
                "captured debug",
                None,
                Some("captured string".into()),
                None
            );
            crate::user_log!("info", ctx, "captured info", None, Some(12345.into()), None);
            crate::user_log!("warn", ctx, "captured warn", None, Some(false.into()), None);
            crate::user_log!(
                "error",
                ctx,
                "captured error",
                None,
                Some(btreemap! { "my_key" => "my_val"}.into()),
                None
            );
            crate::user_log!(
                "error",
                ctx,
                "captured array error",
                None,
                Some(vec![1, 2, 3].into()),
                None
            );

            // `captured_data` using the overloaded macros
            crate::user_log_warn!(
                ctx,
                "overloaded warn msg",
                captured_data: "overloaded captured string".into()
            );
            crate::user_log_error!(ctx, "overloaded error msg", captured_data: 54321.into());

            let expected: Vec<(&str, &str, Option<Value>)> = vec![
                ("DEBUG", "debug msg", None),
                ("INFO", "info msg", None),
                ("WARN", "warn msg", None),
                ("ERROR", "error msg", None),
                (
                    "DEBUG",
                    "captured debug",
                    Some(captured_data_wrapper(Value::from("captured string"))),
                ),
                (
                    "INFO",
                    "captured info",
                    Some(captured_data_wrapper(Value::from(12345))),
                ),
                (
                    "WARN",
                    "captured warn",
                    Some(captured_data_wrapper(Value::from(false))),
                ),
                (
                    "ERROR",
                    "captured error",
                    Some(captured_data_wrapper(Value::from(
                        btreemap! { "my_key" => "my_val"},
                    ))),
                ),
                (
                    "ERROR",
                    "captured array error",
                    Some(captured_data_wrapper(Value::from(vec![1, 2, 3]))),
                ),
                (
                    "WARN",
                    "overloaded warn msg",
                    Some(captured_data_wrapper(Value::from(
                        "overloaded captured string",
                    ))),
                ),
                (
                    "ERROR",
                    "overloaded error msg",
                    Some(captured_data_wrapper(Value::from(54321))),
                ),
            ];

            sleep(Duration::from_millis(1)).await;
            let events = collect_ready(rx).await;

            let end = chrono::Utc::now();
            assert_eq!(events.len(), expected.len());

            for ((exp_level, exp_msg, captured_data), actual) in
                expected.into_iter().zip(events.iter())
            {
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

                if let Some(expected_captured_data) = captured_data {
                    let captured_data = log
                        .get(".meta.mezmo.captured_data")
                        .expect("should contain captured_data");

                    assert_eq!(captured_data, &expected_captured_data);
                }
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
