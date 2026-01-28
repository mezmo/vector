use std::borrow::Cow;

use vector_lib::lookup::lookup_v2::ConfigTargetPath;
use vector_lib::transform::SyncTransform;

use mezmo::{user_log_warn, user_trace::MezmoUserLog, MezmoContext};

use crate::{
    config::log_schema,
    event::{Event, MaybeAsLogMut, ObjectMap, Value},
    internal_events::MezmoDatadogAgentParserError,
};

use logs::DatadogLogEvent;
use metrics::{DatadogMetricEvent, DatadogSketchEvent};
use traces::DatadogTraceEvent;

mod common;
mod config;
mod logs;
mod metrics;
mod traces;

pub use config::{EventTypeValues, MezmoDatadogAgentParserConfig};

pub const LOGS_OUTPUT: &str = "logs";
pub const METRICS_OUTPUT: &str = "metrics";
pub const TRACES_OUTPUT: &str = "traces";
pub const UNMATCHED_OUTPUT: &str = "_unmatched";

#[derive(Clone, Debug)]
pub struct MezmoDatadogAgentParser {
    event_type_path: ConfigTargetPath,
    event_type_values: EventTypeValues,
    payload_version_path: ConfigTargetPath,
    parse_log_tags: bool,
    split_metric_namespace: bool,
    strip_event_type: bool,
    strip_payload_version: bool,
    reroute_unmatched: bool,
    mezmo_ctx: Option<MezmoContext>,
}

impl MezmoDatadogAgentParser {
    pub fn new(config: &MezmoDatadogAgentParserConfig, mezmo_ctx: Option<MezmoContext>) -> Self {
        Self {
            event_type_path: config.event_type_path.clone(),
            event_type_values: config.event_type_values.clone(),
            payload_version_path: config.payload_version_path.clone(),
            parse_log_tags: config.parse_log_tags,
            split_metric_namespace: config.split_metric_namespace,
            strip_event_type: config.strip_event_type,
            strip_payload_version: config.strip_payload_version,
            reroute_unmatched: config.reroute_unmatched,
            mezmo_ctx,
        }
    }

    fn get_event_type(&self, event: &Event) -> Option<String> {
        event
            .maybe_as_log()
            .and_then(|log| log.get(&self.event_type_path.0))
            .and_then(|v| v.as_str())
            .map(Cow::into_owned)
    }

    fn get_payload_version(&self, event: &Event) -> Option<String> {
        event
            .maybe_as_log()
            .and_then(|log| log.get(&self.payload_version_path.0))
            .and_then(|v| v.as_str())
            .map(Cow::into_owned)
    }

    fn handle_transform_event<T>(
        &self,
        event: Event,
        output: &mut vector_lib::transform::TransformOutputsBuf,
        output_name: &'static str,
        event_type_name: &'static str,
        user_log_event_type: &'static str,
    ) where
        T: TransformDatadogEvent,
    {
        match T::transform(event, self) {
            Ok(events) => {
                for event in events {
                    output.push(Some(output_name), event);
                }
            }
            Err(err) => {
                user_log_warn!(
                    self.mezmo_ctx.clone(),
                    format!(
                        "Failed to transform {}: {}",
                        user_log_event_type, err.message
                    )
                );
                emit!(MezmoDatadogAgentParserError {
                    error: &err.message,
                    event_type: Some(event_type_name)
                });
                if self.reroute_unmatched {
                    output.push(Some(UNMATCHED_OUTPUT), *err.input);
                }
            }
        }
    }

    pub(super) fn strip_fields(&self, event: &mut Event) {
        if let Some(log) = event.maybe_as_log_mut() {
            if self.strip_event_type {
                log.remove(&self.event_type_path.0);
            }
            if self.strip_payload_version {
                log.remove(&self.payload_version_path.0);
            }
        }
    }

    pub(super) fn build_events_from_payloads(
        &self,
        mut event: Event,
        payloads: Vec<(ObjectMap, Option<Value>)>,
    ) -> Result<Vec<Event>, (String, Box<Event>)> {
        if payloads.is_empty() {
            return Ok(Vec::new());
        }

        let Some(message_path) = log_schema().message_key_target_path() else {
            return Err(("Missing message key".into(), Box::new(event)));
        };

        if event.maybe_as_log().is_none() {
            return Err(("Event is not a log".into(), Box::new(event)));
        };

        self.strip_fields(&mut event);
        event.as_mut_log().remove(message_path);

        let mut results: Vec<Event> = Vec::with_capacity(payloads.len() + 1);
        for payload in payloads {
            let mut new_event = event.clone();
            let log = new_event.as_mut_log();
            let (message, timestamp) = payload;
            log.insert(message_path, Value::Object(message));

            if let Some(timestamp) = timestamp {
                if let Some(timestamp_path) = log_schema().timestamp_key_target_path() {
                    log.insert(timestamp_path, timestamp);
                }
            }

            results.push(new_event);
        }

        Ok(results)
    }
}

impl SyncTransform for MezmoDatadogAgentParser {
    fn transform(&mut self, event: Event, output: &mut vector_lib::transform::TransformOutputsBuf) {
        let event_type = self.get_event_type(&event);

        match event_type.as_deref() {
            Some(t) if t == self.event_type_values.log => {
                self.handle_transform_event::<DatadogLogEvent>(
                    event,
                    output,
                    LOGS_OUTPUT,
                    "log",
                    "log",
                );
            }
            Some(t) if t == self.event_type_values.metric => {
                self.handle_transform_event::<DatadogMetricEvent>(
                    event,
                    output,
                    METRICS_OUTPUT,
                    "metric",
                    "metric",
                );
            }
            Some(t) if t == self.event_type_values.sketch => {
                self.handle_transform_event::<DatadogSketchEvent>(
                    event,
                    output,
                    METRICS_OUTPUT,
                    "sketch",
                    "metric",
                );
            }
            Some(t) if t == self.event_type_values.trace => {
                self.handle_transform_event::<DatadogTraceEvent>(
                    event,
                    output,
                    TRACES_OUTPUT,
                    "trace",
                    "trace",
                );
            }
            _ => {
                if self.reroute_unmatched {
                    output.push(Some(UNMATCHED_OUTPUT), event);
                }
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct TransformDatadogEventError {
    message: String,
    // Reduce memory used by Result enums using this error
    input: Box<Event>,
}

impl TransformDatadogEventError {
    fn from(input: Event, message: &str) -> Self {
        Self {
            input: Box::new(input),
            message: message.to_string(),
        }
    }
}

trait TransformDatadogEvent {
    fn transform(
        event: Event,
        parser: &MezmoDatadogAgentParser,
    ) -> Result<Vec<Event>, TransformDatadogEventError>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::config::{log_schema, DataType, TransformOutput};
    use crate::event::{LogEvent, Value};
    use crate::transforms::SyncTransform;
    use vector_lib::transform::TransformOutputsBuf;

    fn build_outputs_buf() -> (Vec<&'static str>, TransformOutputsBuf) {
        let output_names = vec![LOGS_OUTPUT, METRICS_OUTPUT, TRACES_OUTPUT, UNMATCHED_OUTPUT];
        let buf = TransformOutputsBuf::new_with_capacity(
            output_names
                .iter()
                .map(|output_name| {
                    TransformOutput::new(DataType::Log, HashMap::new())
                        .with_port(output_name.to_owned())
                })
                .collect(),
            1,
        );
        (output_names, buf)
    }

    fn build_event(event_type: &str, message: Value) -> Event {
        let mut log = LogEvent::default();
        log.insert(log_schema().message_key_target_path().unwrap(), message);
        let event_type_path = log_schema()
            .metadata_key_target_path()
            .expect("metadata key must exist")
            .with_field_appended("x-mezmo-dd-event-type");
        log.insert(&event_type_path, Value::from(event_type));
        Event::Log(log)
    }

    #[test]
    fn routes_events_to_expected_outputs() {
        let config = MezmoDatadogAgentParserConfig::default();
        let mut parser = MezmoDatadogAgentParser::new(&config, None);
        let (output_names, mut outputs) = build_outputs_buf();

        let log_event = build_event(
            "log",
            serde_json::json!({
                "status": "ok"
            })
            .into(),
        );
        parser.transform(log_event, &mut outputs);

        let metric_event = build_event(
            "metric",
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "metric": "system.cpu.usage",
                "type": 3,
                "points": [{"timestamp": 1234567890, "value": 42.5}],
                "tags": [],
                "resources": []
            })
            .into(),
        );
        parser.transform(metric_event, &mut outputs);

        let sketch_event = build_event(
            "sketch",
            serde_json::json!({
                "metric": "system.cpu.sketch",
                "tags": ["env:prod"],
                "host": "testhost",
                "dogsketches": [
                    {
                        "cnt": 12,
                        "min": 1.0,
                        "max": 9.0,
                        "sum": 15.0,
                        "avg": 4.5,
                        "k": [1, 2],
                        "n": [3, 4],
                        "ts": 1234567890
                    }
                ]
            })
            .into(),
        );
        parser.transform(sketch_event, &mut outputs);

        let trace_event = build_event(
            "trace",
            serde_json::json!({
                "mezmo_payload_version": "v2",
                "chunks": []
            })
            .into(),
        );
        parser.transform(trace_event, &mut outputs);

        let unmatched_event = build_event("unknown", serde_json::json!({}).into());
        parser.transform(unmatched_event, &mut outputs);

        for output_name in output_names {
            let events: Vec<_> = outputs.drain_named(output_name).collect();
            match output_name {
                LOGS_OUTPUT => assert_eq!(events.len(), 1),
                METRICS_OUTPUT => assert_eq!(events.len(), 2),
                TRACES_OUTPUT => assert_eq!(events.len(), 1),
                UNMATCHED_OUTPUT => assert_eq!(events.len(), 1),
                _ => unreachable!("unexpected output"),
            }
        }
    }

    #[test]
    fn build_events_from_payloads_sets_message_and_timestamp() {
        let config = MezmoDatadogAgentParserConfig::default();
        let parser = MezmoDatadogAgentParser::new(&config, None);

        let mut log = LogEvent::default();
        log.insert(
            log_schema().message_key_target_path().unwrap(),
            Value::Object(ObjectMap::new()),
        );
        let event = Event::Log(log);

        let timestamp = Value::Timestamp(Utc.timestamp_opt(1234567890, 0).single().unwrap());
        let mut first = ObjectMap::new();
        first.insert("message".into(), Value::from("first"));
        let mut second = ObjectMap::new();
        second.insert("message".into(), Value::from("second"));

        let results = parser
            .build_events_from_payloads(
                event,
                vec![
                    (first, Some(timestamp.clone())),
                    (second, Some(timestamp.clone())),
                ],
            )
            .expect("build events");

        assert_eq!(results.len(), 2);
        for result in results {
            let log = result.as_log();
            let message = log
                .get(log_schema().message_key_target_path().unwrap())
                .and_then(Value::as_object)
                .expect("message object");
            assert!(matches!(
                message.get("message").and_then(Value::as_str),
                Some(_)
            ));

            let parsed_timestamp = log
                .get(log_schema().timestamp_key_target_path().unwrap())
                .and_then(Value::as_timestamp)
                .expect("timestamp should be set");
            assert_eq!(
                *parsed_timestamp,
                Utc.timestamp_opt(1234567890, 0).single().unwrap()
            );
        }
    }
}
