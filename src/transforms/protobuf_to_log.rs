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

    /// This is a description
    OpenTelemetryTraces,
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
            ProtobufVendors::OpenTelemetryTraces => {
                MezmoDeserializer::build(&MezmoDeserializer::OpenTelemetryTraces)
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
    async fn metric_protobuf_test() {
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

    #[tokio::test]
    async fn trace_protobuf_test() {
        let traces: &[u8] = &[
            10, 179, 11, 10, 131, 3, 10, 32, 10, 21, 116, 101, 108, 101, 109, 101, 116, 114, 121,
            46, 115, 100, 107, 46, 118, 101, 114, 115, 105, 111, 110, 18, 7, 10, 5, 49, 46, 50, 46,
            49, 10, 37, 10, 18, 116, 101, 108, 101, 109, 101, 116, 114, 121, 46, 115, 100, 107, 46,
            110, 97, 109, 101, 18, 15, 10, 13, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101,
            116, 114, 121, 10, 34, 10, 22, 116, 101, 108, 101, 109, 101, 116, 114, 121, 46, 115,
            100, 107, 46, 108, 97, 110, 103, 117, 97, 103, 101, 18, 8, 10, 6, 101, 114, 108, 97,
            110, 103, 10, 36, 10, 12, 115, 101, 114, 118, 105, 99, 101, 46, 110, 97, 109, 101, 18,
            20, 10, 18, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97, 103, 115, 101, 114, 118,
            105, 99, 101, 10, 56, 10, 19, 115, 101, 114, 118, 105, 99, 101, 46, 105, 110, 115, 116,
            97, 110, 99, 101, 46, 105, 100, 18, 33, 10, 31, 102, 101, 97, 116, 117, 114, 101, 102,
            108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 64, 100, 54, 57, 100, 56, 53, 55, 49,
            51, 49, 97, 99, 10, 37, 10, 23, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117, 110,
            116, 105, 109, 101, 46, 118, 101, 114, 115, 105, 111, 110, 18, 10, 10, 8, 49, 49, 46,
            50, 46, 50, 46, 56, 10, 30, 10, 20, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117,
            110, 116, 105, 109, 101, 46, 110, 97, 109, 101, 18, 6, 10, 4, 66, 69, 65, 77, 10, 60,
            10, 27, 112, 114, 111, 99, 101, 115, 115, 46, 114, 117, 110, 116, 105, 109, 101, 46,
            100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 18, 29, 10, 27, 69, 114, 108, 97,
            110, 103, 47, 79, 84, 80, 32, 50, 51, 32, 101, 114, 116, 115, 45, 49, 49, 46, 50, 46,
            50, 46, 56, 10, 47, 10, 23, 112, 114, 111, 99, 101, 115, 115, 46, 101, 120, 101, 99,
            117, 116, 97, 98, 108, 101, 46, 110, 97, 109, 101, 18, 20, 10, 18, 102, 101, 97, 116,
            117, 114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 18, 146, 4, 10, 30,
            10, 21, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101, 116, 114, 121, 95, 112, 104,
            111, 101, 110, 105, 120, 18, 5, 49, 46, 48, 46, 48, 18, 239, 3, 10, 16, 196, 206, 162,
            34, 18, 10, 86, 108, 234, 246, 51, 69, 0, 171, 1, 40, 18, 8, 62, 64, 179, 38, 163, 41,
            8, 151, 34, 0, 42, 1, 47, 48, 2, 57, 120, 196, 182, 220, 91, 196, 130, 23, 65, 57, 144,
            204, 220, 91, 196, 130, 23, 74, 61, 10, 12, 112, 104, 111, 101, 110, 105, 120, 46, 112,
            108, 117, 103, 18, 45, 10, 43, 69, 108, 105, 120, 105, 114, 46, 70, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 87, 101, 98, 46, 80, 97,
            103, 101, 67, 111, 110, 116, 114, 111, 108, 108, 101, 114, 74, 25, 10, 14, 112, 104,
            111, 101, 110, 105, 120, 46, 97, 99, 116, 105, 111, 110, 18, 7, 10, 5, 105, 110, 100,
            101, 120, 74, 25, 10, 13, 110, 101, 116, 46, 116, 114, 97, 110, 115, 112, 111, 114,
            116, 18, 8, 10, 6, 73, 80, 46, 84, 67, 80, 74, 21, 10, 13, 110, 101, 116, 46, 112, 101,
            101, 114, 46, 112, 111, 114, 116, 18, 4, 24, 178, 152, 2, 74, 26, 10, 11, 110, 101,
            116, 46, 112, 101, 101, 114, 46, 105, 112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48,
            46, 49, 74, 20, 10, 13, 110, 101, 116, 46, 104, 111, 115, 116, 46, 112, 111, 114, 116,
            18, 3, 24, 145, 63, 74, 26, 10, 11, 110, 101, 116, 46, 104, 111, 115, 116, 46, 105,
            112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48, 46, 49, 74, 32, 10, 15, 104, 116, 116,
            112, 46, 117, 115, 101, 114, 95, 97, 103, 101, 110, 116, 18, 13, 10, 11, 99, 117, 114,
            108, 47, 55, 46, 55, 52, 46, 48, 74, 18, 10, 11, 104, 116, 116, 112, 46, 116, 97, 114,
            103, 101, 116, 18, 3, 10, 1, 47, 74, 23, 10, 16, 104, 116, 116, 112, 46, 115, 116, 97,
            116, 117, 115, 95, 99, 111, 100, 101, 18, 3, 24, 200, 1, 74, 21, 10, 11, 104, 116, 116,
            112, 46, 115, 99, 104, 101, 109, 101, 18, 6, 10, 4, 104, 116, 116, 112, 74, 17, 10, 10,
            104, 116, 116, 112, 46, 114, 111, 117, 116, 101, 18, 3, 10, 1, 47, 74, 20, 10, 11, 104,
            116, 116, 112, 46, 109, 101, 116, 104, 111, 100, 18, 5, 10, 3, 71, 69, 84, 74, 24, 10,
            9, 104, 116, 116, 112, 46, 104, 111, 115, 116, 18, 11, 10, 9, 108, 111, 99, 97, 108,
            104, 111, 115, 116, 74, 20, 10, 11, 104, 116, 116, 112, 46, 102, 108, 97, 118, 111,
            114, 18, 5, 10, 3, 49, 46, 49, 74, 29, 10, 14, 104, 116, 116, 112, 46, 99, 108, 105,
            101, 110, 116, 95, 105, 112, 18, 11, 10, 9, 49, 50, 55, 46, 48, 46, 48, 46, 49, 122, 0,
            18, 149, 4, 10, 27, 10, 18, 111, 112, 101, 110, 116, 101, 108, 101, 109, 101, 116, 114,
            121, 95, 101, 99, 116, 111, 18, 5, 49, 46, 48, 46, 48, 18, 245, 3, 10, 16, 196, 206,
            162, 34, 18, 10, 86, 108, 234, 246, 51, 69, 0, 171, 1, 40, 18, 8, 117, 229, 127, 70, 9,
            173, 255, 14, 34, 8, 62, 64, 179, 38, 163, 41, 8, 151, 42, 42, 102, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 101, 114, 118, 105, 99, 101, 46, 114, 101, 112, 111,
            46, 113, 117, 101, 114, 121, 58, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97, 103,
            115, 48, 3, 57, 191, 36, 187, 220, 91, 196, 130, 23, 65, 78, 239, 198, 220, 91, 196,
            130, 23, 74, 30, 10, 23, 116, 111, 116, 97, 108, 95, 116, 105, 109, 101, 95, 109, 105,
            99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 3, 24, 162, 4, 74, 24, 10, 6, 115,
            111, 117, 114, 99, 101, 18, 14, 10, 12, 102, 101, 97, 116, 117, 114, 101, 102, 108, 97,
            103, 115, 74, 29, 10, 23, 113, 117, 101, 117, 101, 95, 116, 105, 109, 101, 95, 109,
            105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 2, 24, 52, 74, 30, 10, 23,
            113, 117, 101, 114, 121, 95, 116, 105, 109, 101, 95, 109, 105, 99, 114, 111, 115, 101,
            99, 111, 110, 100, 115, 18, 3, 24, 233, 3, 74, 30, 10, 22, 105, 100, 108, 101, 95, 116,
            105, 109, 101, 95, 109, 105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 4, 24,
            243, 213, 35, 74, 30, 10, 24, 100, 101, 99, 111, 100, 101, 95, 116, 105, 109, 101, 95,
            109, 105, 99, 114, 111, 115, 101, 99, 111, 110, 100, 115, 18, 2, 24, 5, 74, 31, 10, 6,
            100, 98, 46, 117, 114, 108, 18, 21, 10, 19, 101, 99, 116, 111, 58, 47, 47, 102, 102,
            115, 95, 112, 111, 115, 116, 103, 114, 101, 115, 74, 16, 10, 7, 100, 98, 46, 116, 121,
            112, 101, 18, 5, 10, 3, 115, 113, 108, 74, 136, 1, 10, 12, 100, 98, 46, 115, 116, 97,
            116, 101, 109, 101, 110, 116, 18, 120, 10, 118, 83, 69, 76, 69, 67, 84, 32, 102, 48,
            46, 34, 105, 100, 34, 44, 32, 102, 48, 46, 34, 100, 101, 115, 99, 114, 105, 112, 116,
            105, 111, 110, 34, 44, 32, 102, 48, 46, 34, 101, 110, 97, 98, 108, 101, 100, 34, 44,
            32, 102, 48, 46, 34, 110, 97, 109, 101, 34, 44, 32, 102, 48, 46, 34, 105, 110, 115,
            101, 114, 116, 101, 100, 95, 97, 116, 34, 44, 32, 102, 48, 46, 34, 117, 112, 100, 97,
            116, 101, 100, 95, 97, 116, 34, 32, 70, 82, 79, 77, 32, 34, 102, 101, 97, 116, 117,
            114, 101, 102, 108, 97, 103, 115, 34, 32, 65, 83, 32, 102, 48, 74, 20, 10, 11, 100, 98,
            46, 105, 110, 115, 116, 97, 110, 99, 101, 18, 5, 10, 3, 102, 102, 115, 122, 0,
        ];

        let expect_metadata = Value::Object(BTreeMap::from([(
            "headers".to_owned(),
            Value::Object(BTreeMap::from([("key".into(), "value".into())])),
        )]));

        let event = log_event_from_bytes(traces, &expect_metadata);

        let result = do_transform(
            event.into(),
            ProtobufToLogConfig {
                vendor: ProtobufVendors::OpenTelemetryTraces,
            },
        )
        .await
        .unwrap();

        assert_eq!(2, result.len());

        for event in result {
            let log = &event.into_log();
            let event_metadata = log.get("metadata").expect("Metadata is empty");

            assert_eq!(*event_metadata, expect_metadata);
        }
    }
}
