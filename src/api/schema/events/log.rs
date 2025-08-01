use std::borrow::Cow;

use async_graphql::Object;
use chrono::{DateTime, Utc};
use vector_lib::config::log_schema;
use vector_lib::encode_logfmt;
use vector_lib::event;
use vector_lib::tap::topology::TapOutput;

use super::EventEncodingType;

#[derive(Debug, Clone)]
pub struct Log {
    output: TapOutput,
    event: event::LogEvent,
}

impl Log {
    pub const fn new(output: TapOutput, event: event::LogEvent) -> Self {
        Self { output, event }
    }

    pub fn get_message(&self) -> Option<Cow<'_, str>> {
        Some(
            self.event
                .get(log_schema().message_key_target_path().unwrap())?
                .to_string_lossy(),
        )
    }

    pub fn get_user_metadata(&self) -> Option<Cow<'_, str>> {
        Some(
            self.event
                .get(log_schema().user_metadata_key())?
                .to_string_lossy(),
        )
    }

    pub fn get_timestamp(&self) -> Option<&DateTime<Utc>> {
        // Mezmo: this is wrong upstream, it should use the `log_schema` field, but it's
        // not affecting anything since we've settled on the  default
        // NOTE: DO NOT switch to using `event_path!` as done elsewhere.
        self.event.get("timestamp")?.as_timestamp()
    }
}

#[Object]
/// Log event with fields for querying log data
impl Log {
    /// Id of the component associated with the log event
    async fn component_id(&self) -> &str {
        self.output.output_id.component.id()
    }

    /// Type of component associated with the log event
    async fn component_type(&self) -> &str {
        self.output.component_type.as_ref()
    }

    /// Kind of component associated with the log event
    async fn component_kind(&self) -> &str {
        self.output.component_kind
    }

    /// Log message
    async fn message(&self) -> Option<String> {
        self.get_message().map(Into::into)
    }

    /// Log user-facing metadata
    async fn user_metadata(&self) -> Option<String> {
        self.get_user_metadata().map(Into::into)
    }

    /// Log timestamp
    async fn timestamp(&self) -> Option<&DateTime<Utc>> {
        self.get_timestamp()
    }

    /// Log event as an encoded string format
    async fn string(&self, encoding: EventEncodingType) -> String {
        match encoding {
            EventEncodingType::Json => serde_json::to_string(&self.event)
                .expect("JSON serialization of log event failed. Please report."),
            EventEncodingType::Yaml => serde_yaml::to_string(&self.event)
                .expect("YAML serialization of log event failed. Please report."),
            EventEncodingType::Logfmt => encode_logfmt::encode_value(self.event.value())
                .expect("logfmt serialization of log event failed. Please report."),
        }
    }

    /// Get JSON field data on the log event, by field name
    async fn json(&self, field: String) -> Option<String> {
        // NOTE: DO NOT switch to using `event_path!` as done elsewhere. This breaks nested lookups.
        self.event.get(field.as_str()).map(|field| {
            serde_json::to_string(field)
                .expect("JSON serialization of trace event field failed. Please report.")
        })
    }
}
