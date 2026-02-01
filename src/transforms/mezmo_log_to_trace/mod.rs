pub mod config;

use crate::{
    config::log_schema,
    event::{Event, EventMetadata, TraceEvent, Value},
    internal_events::MezmoLogToTraceEventDropped,
    transforms::{FunctionTransform, OutputBuffer},
};
use config::LogToTraceConfig;
use mezmo::MezmoContext;
use vrl::path::TargetPath;

#[derive(Debug, Clone)]
pub struct LogToTrace {
    #[allow(dead_code)]
    config: LogToTraceConfig,
    #[allow(dead_code)]
    mezmo_ctx: Option<MezmoContext>,
}

impl LogToTrace {
    pub const fn new(config: LogToTraceConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        Self { config, mezmo_ctx }
    }
}

impl FunctionTransform for LogToTrace {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        match event {
            Event::Trace(trace) => {
                output.push(Event::Trace(trace));
            }
            Event::Log(log) => {
                let (value, metadata) = log.into_parts();

                let message_path = match log_schema().message_key_target_path() {
                    Some(path) => path.value_path(),
                    None => {
                        emit!(MezmoLogToTraceEventDropped {
                            reason: "The trace event is missing the message key",
                        });
                        return;
                    }
                };

                let message_obj = match value.get(message_path).and_then(Value::as_object) {
                    Some(value) => value.clone(),
                    None => {
                        emit!(MezmoLogToTraceEventDropped {
                            reason: "The trace event must have a 'message' field of type object",
                        });
                        return;
                    }
                };

                let metadata = if self.config.exclude_metadata {
                    EventMetadata::default()
                } else {
                    metadata
                };
                let trace = TraceEvent::from_parts(message_obj, metadata);
                output.push(Event::Trace(trace));
            }
            _ => {
                emit!(MezmoLogToTraceEventDropped {
                    reason: "Event is not a log or trace",
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json;
    use vrl::event_path;

    use super::*;
    use vector_lib::event::TraceEvent;

    use crate::event::{LogEvent, ObjectMap, Value};

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<LogToTraceConfig>();
    }

    #[test]
    fn passes_through_trace() {
        let mut trace = TraceEvent::default();
        trace.insert(event_path!("env"), "production");
        let timestamp = Utc.timestamp_opt(1_234_567_890, 0).single().unwrap();
        let timestamp_path = log_schema().timestamp_key_target_path().unwrap();
        trace.insert(timestamp_path, Value::Timestamp(timestamp));
        let metadata = trace.metadata().clone();

        let mut transform = LogToTrace::new(LogToTraceConfig::default(), None);
        let mut output = OutputBuffer::with_capacity(1);
        transform.transform(&mut output, Event::Trace(trace));

        let mut events = output.into_events();
        let event = events.next().expect("transformed event");
        assert!(events.next().is_none());

        let trace = event.as_trace();
        assert_eq!(trace.metadata(), &metadata);
        assert_eq!(
            trace
                .get(event_path!("env"))
                .and_then(Value::as_str)
                .as_deref(),
            Some("production")
        );
        assert_eq!(
            trace.get(timestamp_path),
            Some(&Value::Timestamp(timestamp))
        );
    }

    #[test]
    fn converts_log_to_trace() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "env": "production",
                "spans": []
            }),
        );
        let metadata = log.metadata().clone();

        let mut transform = LogToTrace::new(LogToTraceConfig::default(), None);
        let mut output = OutputBuffer::with_capacity(1);
        transform.transform(&mut output, Event::Log(log));

        let mut events = output.into_events();
        let event = events.next().expect("transformed event");
        assert!(events.next().is_none());

        let trace = event.as_trace();
        assert_eq!(trace.metadata(), &metadata);
        assert_eq!(
            trace
                .get(event_path!("env"))
                .and_then(Value::as_str)
                .as_deref(),
            Some("production")
        );
        assert_eq!(
            trace
                .get(event_path!("spans"))
                .and_then(Value::as_array)
                .map(|arr| arr.len()),
            Some(0)
        );
    }

    #[test]
    fn excludes_metadata_when_configured() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            serde_json::json!({
                "env": "production",
                "spans": []
            }),
        );
        let mut metadata_value = ObjectMap::new();
        metadata_value.insert("account".into(), Value::from("west"));
        *log.metadata_mut().value_mut() = Value::Object(metadata_value.clone());

        let mut transform = LogToTrace::new(
            LogToTraceConfig {
                exclude_metadata: true,
            },
            None,
        );
        let mut output = OutputBuffer::with_capacity(1);
        transform.transform(&mut output, Event::Log(log));

        let mut events = output.into_events();
        let event = events.next().expect("transformed event");
        assert!(events.next().is_none());

        let trace = event.as_trace();
        assert_eq!(trace.metadata().value(), &Value::Object(ObjectMap::new()));
        assert_ne!(trace.metadata().value(), &Value::Object(metadata_value));
    }

    #[test]
    fn rejects_log_without_message_object() {
        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            Value::from("not-an-object"),
        );

        let mut transform = LogToTrace::new(LogToTraceConfig::default(), None);
        let mut output = OutputBuffer::with_capacity(1);
        transform.transform(&mut output, Event::Log(log));

        let mut events = output.into_events();
        assert!(events.next().is_none());
    }
}
