use vector_config::configurable_component;
use vector_core::{config::LogNamespace, event::Value};

use codecs::decoding::MezmoDeserializer;

use crate::mezmo::user_trace::handle_deserializer_error;

use crate::{
    config::{DataType, GenerateConfig, Input, OutputId, TransformConfig, TransformContext},
    event::{Event, LogEvent},
    mezmo::MezmoContext,
    schema,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

use lookup::PathPrefix;
use vector_core::config::{log_schema, TransformOutput};

/// The Enum to choose a protobuf vendor.
#[configurable_component]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtobufVendors {
    /// This is a description
    #[default]
    OpenTelemetryLogs,
}

/// Configuration for the `protobuf_to_log` transform.
#[configurable_component(transform("protobuf_to_log"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ProtobufToLogConfig {
    /// This is a description
    #[serde(default)]
    pub vendor: ProtobufVendors,
}

#[derive(Debug, Clone)]
pub struct ProtobufToLog {
    #[allow(dead_code)]
    config: ProtobufToLogConfig,

    /// The mezmo context used to surface errors
    mezmo_ctx: Option<MezmoContext>,
}

impl GenerateConfig for ProtobufToLogConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            vendor: ProtobufVendors::default(),
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "protobuf_to_log")]
impl TransformConfig for ProtobufToLogConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(ProtobufToLog::new(
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
        vec![TransformOutput::new(
            DataType::Log,
            std::collections::HashMap::new(),
        )]
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}

impl ProtobufToLog {
    pub const fn new(config: ProtobufToLogConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        ProtobufToLog { config, mezmo_ctx }
    }
}

impl FunctionTransform for ProtobufToLog {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        let log = event.into_log();

        let message = log
            .get_message()
            .and_then(Value::as_bytes)
            .expect("Log event has no message");

        let deserializer = match self.config.vendor {
            ProtobufVendors::OpenTelemetryLogs => {
                MezmoDeserializer::build(&MezmoDeserializer::OpenTelemetryLogs)
            }
        };

        match deserializer.parse(message.clone(), LogNamespace::Legacy) {
            Ok(logs) => {
                // Log generation was successful, publish it
                for event in logs.into_iter().map(|event| event.into_log()) {
                    let mut log_event = LogEvent::new_with_metadata(log.metadata().clone());

                    if let Some(event_message) = event.get_message() {
                        log_event.insert(
                            (PathPrefix::Event, log_schema().message_key()),
                            event_message.to_owned(),
                        );
                    }

                    if let Some(user_metadata) =
                        log.get((PathPrefix::Event, log_schema().user_metadata_key()))
                    {
                        log_event.insert(
                            (PathPrefix::Event, log_schema().user_metadata_key()),
                            user_metadata.to_owned(),
                        );
                    }

                    if let Some(timestamp_key) = log_schema().timestamp_key() {
                        log_event.insert(
                            (PathPrefix::Event, timestamp_key),
                            event.get_timestamp().unwrap().clone(),
                        );
                    }

                    output.push(log_event.into());
                }
            }
            Err(err) => {
                handle_deserializer_error(&self.mezmo_ctx, err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use vector_core::event::Value;

    use crate::event::{Event, LogEvent};
    use crate::test_util::components::assert_transform_compliance;
    use crate::transforms::test::create_topology_with_name;

    #[test]
    fn generate_protobuf_config() {
        crate::test_util::test_generate_config::<ProtobufToLogConfig>();
    }

    fn ts() -> DateTime<Utc> {
        Utc::now()
    }

    fn log_event_from_bytes(msg: &[u8], metadata: &Value) -> LogEvent {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("message".into(), msg.into());
        event_map.insert("timestamp".into(), ts().into());
        event_map.insert("metadata".into(), metadata.clone());
        event_map.into()
    }

    async fn do_transform(event: Event, config: ProtobufToLogConfig) -> Option<Vec<Event>> {
        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let name = "v1:protobuf_to_log:transform:ef757476-43a5-4e0d-b998-3db35dbde001:1515707f-f668-4ca1-8493-969e5b13e781:800e5a08-3e67-431c-bbf0-14aa94beafcc";
            let (topology, mut out) =
            create_topology_with_name(ReceiverStream::new(rx), config, name).await;
            tx.send(event).await.unwrap();
            let mut result = Vec::new();

            while let Ok(item) = tokio::time::timeout(Duration::from_secs(2), out.recv()).await {
                if let Some(msg) = item {
                    result.push(msg)
                }
            }

            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
            Some(result)
        })
        .await
    }

    #[tokio::test]
    async fn protobuf_test() {
        let logs: &[u8] = b"\n\x85\x06\x12\x82\x06\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 0\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 1\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 2\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 3\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 4\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 5\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 6\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 7\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 8\x12$\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x12\n\x10test log line: 9\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 10\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 11\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 12\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 13\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 14\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 15\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 16\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 17\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 18\x12%\t@B\x0f\x00\x00\x00\x00\x00\x1a\x05ERROR*\x13\n\x11test log line: 19";

        let expect_metadata = Value::Object(BTreeMap::from([(
            "headers".to_owned(),
            Value::Object(BTreeMap::from([("key".into(), "value".into())])),
        )]));

        let event = log_event_from_bytes(logs, &expect_metadata);

        let result = do_transform(
            event.into(),
            ProtobufToLogConfig {
                vendor: ProtobufVendors::OpenTelemetryLogs,
            },
        )
        .await
        .unwrap();

        assert_eq!(20, result.len());

        for event in result {
            let log = &event.into_log();
            let event_metadata = log.get("metadata").expect("Metadata is empty");

            assert_eq!(*event_metadata, expect_metadata);
        }
    }
}
