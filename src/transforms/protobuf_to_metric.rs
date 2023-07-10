use smallvec::SmallVec;
use vector_config::configurable_component;
use vector_core::{config::LogNamespace, event::Value};

use codecs::decoding::MezmoDeserializer;

use crate::mezmo::user_trace::handle_deserializer_error;

use crate::{
    config::{DataType, GenerateConfig, Input, Output, TransformConfig, TransformContext},
    event::{Event, LogEvent},
    mezmo::MezmoContext,
    schema,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

use lookup::PathPrefix;
use vector_core::config::log_schema;

/// The Enum to choose a protobuf vendor.
#[configurable_component]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtobufVendors {
    /// This is a description
    #[default]
    OpenTelemetryMetrics,
}

/// Configuration for the `protobuf_to_metric` transform.
#[configurable_component(transform("protobuf_to_metric"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ProtobufToMetricConfig {
    /// This is a description
    #[serde(default)]
    pub vendor: ProtobufVendors,
}

#[derive(Debug, Clone)]
pub struct ProtobufToMetric {
    #[allow(dead_code)]
    config: ProtobufToMetricConfig,

    /// The mezmo context used to surface errors
    mezmo_ctx: Option<MezmoContext>,
}

impl GenerateConfig for ProtobufToMetricConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            vendor: ProtobufVendors::default(),
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "protobuf_to_metric")]
impl TransformConfig for ProtobufToMetricConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(ProtobufToMetric::new(
            self.clone(),
            context.mezmo_ctx.clone(),
        )))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(&self, _: &schema::Definition, _: LogNamespace) -> Vec<Output> {
        vec![Output::default(DataType::Log)]
    }

    fn enable_concurrency(&self) -> bool {
        true
    }
}

impl ProtobufToMetric {
    pub const fn new(config: ProtobufToMetricConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        ProtobufToMetric { config, mezmo_ctx }
    }
}

impl FunctionTransform for ProtobufToMetric {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        let log = &event.into_log();
        let mut buffer: Option<SmallVec<[Event; 1]>> = None;

        let message = log
            .get_message()
            .and_then(Value::as_bytes)
            .expect("Log event has no message");

        let deserializer = match self.config.vendor {
            ProtobufVendors::OpenTelemetryMetrics => {
                MezmoDeserializer::build(&MezmoDeserializer::OpenTelemetryMetrics)
            }
        };

        match deserializer.parse(message.clone(), LogNamespace::Legacy) {
            Ok(metrics) => {
                buffer = Some(metrics);
            }
            Err(err) => {
                handle_deserializer_error(&self.mezmo_ctx, err);
            }
        }

        // Metric generation was successful, publish it
        if let Some(mut events) = buffer {
            while let Some(event) = events.pop() {
                let event = event.into_log();
                let mut log_event = LogEvent::new_with_metadata(log.metadata().clone());

                if let Some(event_message) = event.get_message() {
                    log_event.insert(
                        (PathPrefix::Event, log_schema().message_key()),
                        event_message.to_owned(),
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
        crate::test_util::test_generate_config::<ProtobufToMetricConfig>();
    }

    fn ts() -> DateTime<Utc> {
        Utc::now()
    }

    fn log_event_from_bytes(msg: &[u8]) -> LogEvent {
        let mut event_map: BTreeMap<String, Value> = BTreeMap::new();
        event_map.insert("message".into(), msg.into());
        event_map.insert("timestamp".into(), ts().into());
        event_map.into()
    }

    async fn do_transform(event: Event, config: ProtobufToMetricConfig) -> Option<Vec<Event>> {
        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let name = "v1:protobuf_to_metric:transform:ef757476-43a5-4e0d-b998-3db35dbde001:1515707f-f668-4ca1-8493-969e5b13e781:800e5a08-3e67-431c-bbf0-14aa94beafcc";
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
    async fn gauge_protobuf_test() {
        let metrics: &[u8] = b"\n\xd8\x18\n\x8f\x02\nR\n\x0ccontainer.id\x12B\n@e59af7f60d91b041aab241389682f45868d4198b456fc95060e9b48eb574f6fe\n\x1d\n\x0cservice.name\x12\r\n\x0bcartservice\n)\n\x11service.namespace\x12\x14\n\x12opentelemetry-demo\n%\n\x12telemetry.sdk.name\x12\x0f\n\ropentelemetry\n\"\n\x16telemetry.sdk.language\x12\x08\n\x06dotnet\n$\n\x15telemetry.sdk.version\x12\x0b\n\t1.4.0.788\x12\xc3\x16\n0\n%OpenTelemetry.Instrumentation.Runtime\x12\x071.1.0.1\x12\x94\x02\n+process.runtime.dotnet.gc.collections.count\x12ENumber of garbage collections that have occurred since process start.:\x9d\x01\n1\x114*\x8d\x15l\xe6]\x17\x19\xec\x1d-\xf1\xa3\xe6]\x171\0\0\0\0\0\0\0\0:\x14\n\ngeneration\x12\x06\n\x04gen2\n1\x114*\x8d\x15l\xe6]\x17\x19\xec\x1d-\xf1\xa3\xe6]\x171\0\0\0\0\0\0\0\0:\x14\n\ngeneration\x12\x06\n\x04gen1\n1\x114*\x8d\x15l\xe6]\x17\x19\xec\x1d-\xf1\xa3\xe6]\x171\0\0\0\0\0\0\0\0:\x14\n\ngeneration\x12\x06\n\x04gen0\x10\x02\x18\x01\x12\xed\x01\n&process.runtime.dotnet.gc.objects.size\x12\x9a\x01Count of bytes currently in use by objects in the GC heap that haven't been collected yet. Fragmentation and other GC committed memory pools are excluded.\x1a\x05bytes:\x1f\n\x1b\x11\xf0\xbb\xd5\x15l\xe6]\x17\x19\xb4\x1e-\xf1\xa3\xe6]\x171\xf0\xbbY\0\0\0\0\0\x10\x02\x12\x9c\x02\n*process.runtime.dotnet.gc.allocations.size\x12\xc3\x01Count of bytes allocated on the managed GC heap since the process start. .NET objects are allocated from this heap. Object allocations from unmanaged languages such as C/C++ do not use this heap.\x1a\x05bytes:!\n\x1b\x11X\x8e\xd7\x15l\xe6]\x17\x19\xe0\x1f-\xf1\xa3\xe6]\x171@\xb9U\0\0\0\0\0\x10\x02\x18\x01\x12\xb1\x01\n+process.runtime.dotnet.jit.il_compiled.size\x12XCount of bytes of intermediate language that have been compiled since the process start.\x1a\x05bytes:!\n\x1b\x110\xc9\xd8\x15l\xe6]\x17\x19\x90$-\xf1\xa3\xe6]\x171\xbef\n\0\0\0\0\0\x10\x02\x18\x01\x12\xe2\x02\n1process.runtime.dotnet.jit.methods_compiled.count\x12\x89\x02The number of times the JIT compiler compiled a method since the process start. The JIT compiler may be invoked multiple times for the same method to compile with different generic parameters, or because tiered compilation requested different optimization settings.:!\n\x1b\x11l\x87\xd9\x15l\xe6]\x17\x19\xbc%-\xf1\xa3\xe6]\x171\x16&\0\0\0\0\0\0\x10\x02\x18\x01\x12\xae\x01\n+process.runtime.dotnet.jit.compilation_time\x12XThe amount of time the JIT compiler has spent compiling methods since the process start.\x1a\x02ns:!\n\x1b\x11\xdc\xe9\xd9\x15l\xe6]\x17\x19 &-\xf1\xa3\xe6]\x171\xe4\xef\xda\x8c\0\0\0\0\x10\x02\x18\x01\x12\xbe\x02\n4process.runtime.dotnet.monitor.lock_contention.count\x12\xe2\x01The number of times there was contention when trying to acquire a monitor lock since the process start. Monitor locks are commonly acquired by using the lock keyword in C#, or by calling Monitor.Enter() and Monitor.TryEnter().:!\n\x1b\x11\x18D\xda\x15l\xe6]\x17\x19\x84&-\xf1\xa3\xe6]\x171\x0c\0\0\0\0\0\0\0\x10\x02\x18\x01\x12\x8c\x01\n0process.runtime.dotnet.thread_pool.threads.count\x127The number of thread pool threads that currently exist.:\x1f\n\x1b\x11\xd4\x91\xda\x15l\xe6]\x17\x19\xe8&-\xf1\xa3\xe6]\x171\x03\0\0\0\0\0\0\0\x10\x02\x12\xbc\x01\n8process.runtime.dotnet.thread_pool.completed_items.count\x12]The number of work items that have been processed by the thread pool since the process start.:!\n\x1b\x11@\x95\xfc\x15l\xe6]\x17\x19\xb0'-\xf1\xa3\xe6]\x171\xb9\x01\0\0\0\0\0\0\x10\x02\x18\x01\x12\xaa\x01\n/process.runtime.dotnet.thread_pool.queue.length\x12VThe number of work items that are currently queued to be processed by the thread pool.:\x1f\n\x1b\x11<\x02\xfd\x15l\xe6]\x17\x19\x14(-\xf1\xa3\xe6]\x171\0\0\0\0\0\0\0\0\x10\x02\x12\xdb\x02\n\"process.runtime.dotnet.timer.count\x12\x93\x02The number of timer instances that are currently active. Timers can be created by many sources such as System.Threading.Timer, Task.Delay, or the timeout in a CancellationSource. An active timer is registered to tick at some point in the future and has not yet been canceled.:\x1f\n\x1b\x11L[\xfd\x15l\xe6]\x17\x19\xdc(-\xf1\xa3\xe6]\x171\x05\0\0\0\0\0\0\0\x10\x02\x12\x84\x01\n'process.runtime.dotnet.assemblies.count\x128The number of .NET assemblies that are currently loaded.:\x1f\n\x1b\x11d\xa2\xfd\x15l\xe6]\x17\x19@)-\xf1\xa3\xe6]\x171m\0\0\0\0\0\0\0\x10\x02";

        let event = log_event_from_bytes(metrics);
        let result = do_transform(
            event.into(),
            ProtobufToMetricConfig {
                vendor: ProtobufVendors::OpenTelemetryMetrics,
            },
        )
        .await
        .unwrap();

        assert_eq!(14, result.len());
    }
}
