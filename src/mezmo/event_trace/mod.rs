use crate::config::ComponentKey;
use crate::event::{Event, EventMetadata};
use crate::transforms::{SyncTransform, TaskTransform, TransformOutputsBuf};
use bytes::Buf;
use futures::Stream;
use futures_util::StreamExt;
use mezmo::MezmoContext;
use std::env;
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::SystemTime;
use vector_lib::event::EventArray;
use vrl::core::Value;

const MEZMO_EVENT_TRACE_ENABLED: &str = "MEZMO_EVENT_TRACE_ENABLED";
const MEZMO_TRACE_KEY: &str = "mezmo_trace";

static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
fn is_enabled() -> bool {
    *TRACE_ENABLED.get_or_init(|| {
        if let Ok(value) = env::var(MEZMO_EVENT_TRACE_ENABLED) {
            let value = value.to_ascii_lowercase();
            if value == "1" || value == "true" || value == "t" {
                info!("event tracing is enabled");
                return true;
            }
        }

        info!("event tracing is globally disabled");
        false
    })
}

fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn add_trace_data(id: &str, internal: bool, elapsed: i64, metadata: &mut EventMetadata) {
    let metadata_val = metadata.value_mut();
    let trace_rec = Value::from(btreemap! {
        "id" => id.to_owned(),
        "internal" => internal,
        "elapsed" => Value::from(elapsed),
    });
    match metadata_val.get_mut(MEZMO_TRACE_KEY) {
        None => {
            metadata_val.insert(MEZMO_TRACE_KEY, Value::from(vec![trace_rec]));
        }
        Some(Value::Array(x)) => {
            x.push(trace_rec);
        }
        val => {
            warn!("{MEZMO_TRACE_KEY} value for id={id} is not an array: {val:?} ");
        }
    }
}

#[derive(Clone)]
pub struct MezmoSyncTransformTrace {
    inner: Box<dyn SyncTransform>,
    key: String,
    internal: bool,
}

impl MezmoSyncTransformTrace {
    pub fn maybe_wrap(key: ComponentKey, inner: Box<dyn SyncTransform>) -> Box<dyn SyncTransform> {
        match MezmoContext::try_from(key.into_id()) {
            Ok(ctx) if is_enabled() => Box::new(Self {
                key: ctx.component_id,
                internal: ctx.internal,
                inner,
            }),
            _ => inner,
        }
    }
}

impl SyncTransform for MezmoSyncTransformTrace {
    fn transform(&mut self, event: Event, output: &mut TransformOutputsBuf) {
        let start = current_time();
        self.inner.as_mut().transform(event, output);

        // Ignoring the overflow here is probably fine for tracing because i64::MAX nanoseconds is
        // approximately 2,562,047 hours. We're more likely to have a vector release or pod roll
        // before we would need to worry about silent precision loss.
        let duration = (current_time() - start) as i64;

        if let Some(primary_buffer) = &mut output.primary_buffer {
            for mut event in primary_buffer.events_mut() {
                add_trace_data(&self.key, self.internal, duration, event.metadata_mut());
            }
        }
        for buffer in &mut output.named_buffers.values_mut() {
            for mut event in buffer.events_mut() {
                add_trace_data(&self.key, self.internal, duration, event.metadata_mut());
            }
        }
    }
}

pub struct MezmoTaskTransformTrace {
    inner: Box<dyn TaskTransform<EventArray>>,
    key: String,
    internal: bool,
}

impl MezmoTaskTransformTrace {
    pub fn maybe_wrap(
        key: ComponentKey,
        inner: Box<dyn TaskTransform<EventArray>>,
    ) -> Box<dyn TaskTransform<EventArray>> {
        match MezmoContext::try_from(key.into_id()) {
            Ok(ctx) if is_enabled() => Box::new(Self {
                key: ctx.component_id,
                internal: ctx.internal,
                inner,
            }),
            _ => inner,
        }
    }
}

impl TaskTransform<EventArray> for MezmoTaskTransformTrace {
    fn transform(
        self: Box<Self>,
        stream: Pin<Box<dyn Stream<Item = EventArray> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = EventArray> + Send>> {
        let key = self.key.clone();
        let internal = self.internal;
        let stream = stream
            .map(|mut events| {
                for mut event in events.iter_events_mut() {
                    // To retain the u128 precision through the Value boundary, digest it into 16 bytes.
                    // The endianness doesn't matter as long as it's consistent with the post process logic.
                    let trace_start = Value::from(current_time().to_ne_bytes());
                    event
                        .metadata_mut()
                        .value_mut()
                        .insert("__mezmo_trace_start", trace_start);
                }
                events
            })
            .boxed();
        self.inner
            .transform(stream)
            .map(move |mut events| {
                for mut event in events.iter_events_mut() {
                    let metadata = event.metadata_mut();
                    let elapsed = match metadata.value_mut().remove("__mezmo_trace_start", true) {
                        Some(Value::Bytes(mut start)) => {
                            // Ignoring the overflow here is probably fine for tracing because i64::MAX nanoseconds is
                            // approximately 2,562,047 hours. We're more likely to have a vector release or pod roll
                            // before we would need to worry about silent precision loss.
                            (current_time() - start.get_u128_ne()) as i64
                        }
                        _ => -1,
                    };
                    add_trace_data(&key, internal, elapsed, metadata);
                }
                events
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ComponentKey, DataType, TransformOutput};
    use crate::event::{Event, LogEvent};
    use crate::transforms::{SyncTransform, TaskTransform, TransformOutputsBuf};
    use assay::assay;
    use futures::Stream;
    use futures_util::stream;
    use std::collections::HashMap;
    use std::pin::Pin;
    use vector_lib::event::{EventArray, EventMetadata};
    use vrl::core::Value;

    #[test]
    fn test_add_trace_data() {
        let mut metadata = EventMetadata::default();
        assert!(metadata.value().get(MEZMO_TRACE_KEY).is_none());

        add_trace_data("key-1", false, 1000, &mut metadata);
        let trace_arr = metadata
            .value()
            .get(MEZMO_TRACE_KEY)
            .expect("should have MEZMO_TRACE_KEY element");
        let trace_arr = trace_arr
            .as_array()
            .expect("MEZMO_TRACE_KEY value should be an array");
        assert_eq!(
            "[{\"elapsed\":1000,\"id\":\"key-1\",\"internal\":false}]",
            serde_json::to_string(&trace_arr)
                .expect("mezmo trace data should be json serializable")
        );

        add_trace_data("key-2", true, 2000, &mut metadata);
        let trace_arr = metadata
            .value()
            .get(MEZMO_TRACE_KEY)
            .expect("should have MEZMO_TRACE_KEY element");
        let trace_arr = trace_arr
            .as_array()
            .expect("MEZMO_TRACE_KEY value should be an array");
        assert_eq!(
            "[{\"elapsed\":1000,\"id\":\"key-1\",\"internal\":false},{\"elapsed\":2000,\"id\":\"key-2\",\"internal\":true}]",
            serde_json::to_string(&trace_arr)
                .expect("mezmo trace data should be json serializable")
        );
    }

    #[derive(Clone)]
    struct TestSyncTransform;
    impl SyncTransform for TestSyncTransform {
        fn transform(&mut self, event: Event, output: &mut TransformOutputsBuf) {
            output.push(None, event);
        }
    }

    fn new_output_buf() -> TransformOutputsBuf {
        TransformOutputsBuf::new_with_capacity(
            vec![
                TransformOutput::new(DataType::all(), HashMap::new()),
                TransformOutput::new(DataType::all(), HashMap::new()).with_port("dropped"),
            ],
            1,
        )
    }

    #[assay(env = [("MEZMO_EVENT_TRACE_ENABLED", "false")])]
    fn sync_transform_trace_disabled() {
        let key = ComponentKey::from("v1:filter:transform:node-1:pipeline-abc:acct1");
        let xform = Box::new(TestSyncTransform);
        let mut xform = MezmoSyncTransformTrace::maybe_wrap(key, xform);

        let mut outputs = new_output_buf();
        let event = LogEvent::from("test-event");
        xform.transform(event.into(), &mut outputs);

        let actual = outputs.drain().next().expect("output should have an event");
        let actual = actual.metadata().value().get(MEZMO_TRACE_KEY);
        assert!(actual.is_none());
    }

    #[assay(env = [("MEZMO_EVENT_TRACE_ENABLED", "true")])]
    fn sync_transform_trace_wrapper() {
        let key = ComponentKey::from("v1:filter:transform:node-1:pipeline-abc:acct1");
        let xform = Box::new(TestSyncTransform);
        let mut xform = MezmoSyncTransformTrace::maybe_wrap(key, xform);

        let mut outputs = new_output_buf();
        let event = LogEvent::from("test-event");
        xform.transform(event.into(), &mut outputs);

        let actual = outputs.drain().next().expect("output should have an event");
        let actual = actual
            .metadata()
            .value()
            .get(MEZMO_TRACE_KEY)
            .expect("should have trace object");
        let actual = actual
            .as_array()
            .expect("trace object should be an array value");
        assert_eq!(1, actual.len());
        assert!(actual[0].get("elapsed").is_some_and(Value::is_integer));
        assert!(actual[0]
            .get("id")
            .is_some_and(|v| v.as_str().expect("node is a string") == "node-1"));
    }

    #[derive(Clone)]
    struct TestTaskTransform;
    impl TaskTransform<EventArray> for TestTaskTransform {
        fn transform(
            self: Box<Self>,
            task: Pin<Box<dyn Stream<Item = EventArray> + Send>>,
        ) -> Pin<Box<dyn Stream<Item = EventArray> + Send>> {
            task
        }
    }

    #[assay(env = [("MEZMO_EVENT_TRACE_ENABLED", "false")])]
    async fn task_transform_trace_disabled() {
        let key = ComponentKey::from("v1:filter:transform:node-1:pipeline-abc:acct1");
        let xform = Box::new(TestTaskTransform);
        let xform = MezmoTaskTransformTrace::maybe_wrap(key, xform);

        let input = stream::once(async { EventArray::from(LogEvent::from("test-event")) }).boxed();
        let mut output = xform.transform(input).collect::<Vec<EventArray>>().await;

        let actual = match output.pop().expect("one batch should have been processed") {
            EventArray::Logs(array) => array,
            _ => panic!("unexpected event types in unit test"),
        };

        let metadata = actual.first().expect("batch should have an event");
        assert!(metadata.value().get("__mezmo_trace_start").is_none());

        let trace = metadata.value().get(MEZMO_TRACE_KEY);
        assert!(trace.is_none());
    }

    #[assay(env = [("MEZMO_EVENT_TRACE_ENABLED", "true")])]
    async fn task_transform_trace_wrapper() {
        let key = ComponentKey::from("v1:filter:transform:node-1:pipeline-abc:acct1");
        let xform = Box::new(TestTaskTransform);
        let xform = MezmoTaskTransformTrace::maybe_wrap(key, xform);

        let input = stream::once(async { EventArray::from(LogEvent::from("test-event")) }).boxed();
        let mut output = xform.transform(input).collect::<Vec<EventArray>>().await;

        let actual = match output.pop().expect("one batch should have been processed") {
            EventArray::Logs(array) => array,
            _ => panic!("unexpected event types in unit test"),
        };
        let metadata = actual
            .first()
            .expect("batch should have an event")
            .metadata();
        assert!(metadata.value().get("__mezmo_trace_start").is_none());

        let trace = metadata
            .value()
            .get(MEZMO_TRACE_KEY)
            .expect("should have a trace object");
        let trace = trace
            .as_array()
            .expect("trace object should be an array value");
        assert_eq!(1, trace.len());
        assert!(trace[0]
            .get("elapsed")
            .is_some_and(|v| v.as_integer().expect("elapsed is an integer") > -1));
        assert!(trace[0]
            .get("id")
            .is_some_and(|v| v.as_str().expect("node is a string") == "node-1"));
    }
}
