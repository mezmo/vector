use lookup::lookup_v2::{parse_target_path, OptionalValuePath};
use lookup::{owned_value_path, OwnedTargetPath, OwnedValuePath};
use once_cell::sync::{Lazy, OnceCell};
use vector_config::configurable_component;

static LOG_SCHEMA: OnceCell<LogSchema> = OnceCell::new();
static LOG_SCHEMA_DEFAULT: Lazy<LogSchema> = Lazy::new(LogSchema::default);

/// Loads Log Schema from configurations and sets global schema. Once this is
/// done, configurations can be correctly loaded using configured log schema
/// defaults.
///
/// # Errors
///
/// This function will fail if the `builder` fails.
///
/// # Panics
///
/// If deny is set, will panic if schema has already been set.
pub fn init_log_schema(log_schema: LogSchema, deny_if_set: bool) {
    assert!(
        !(LOG_SCHEMA.set(log_schema).is_err() && deny_if_set),
        "Couldn't set schema"
    );
}

/// Components should use global `LogSchema` returned by this function.  The
/// returned value can differ from `LogSchema::default()` which is unchanging.
pub fn log_schema() -> &'static LogSchema {
    LOG_SCHEMA.get().unwrap_or(&LOG_SCHEMA_DEFAULT)
}

/// Log schema.
///
/// A log schema is used by Vector not only to uniformly process the fields of an event, but also to
/// specify which fields should hold specific data that is also set by Vector once an event is
/// flowing through a topology.
#[configurable_component]
#[derive(Clone, Debug, Eq, PartialEq)]
#[serde(default)]
pub struct LogSchema {
    /// The name of the event field to treat as the event message.
    ///
    /// This would be the field that holds the raw message, such as a raw log line.
    #[serde(default = "LogSchema::default_message_key")]
    message_key: String,

    /// The name of the event field to treat as the event timestamp.
    #[serde(default = "LogSchema::default_timestamp_key")]
    timestamp_key: OptionalValuePath,

    /// The name of the event field to treat as the host which sent the message.
    ///
    /// This field will generally represent a real host, or container, that generated the message,
    /// but is somewhat source-dependent.
    #[serde(default = "LogSchema::default_host_key")]
    host_key: String,

    /// The name of the event field to set the source identifier in.
    ///
    /// This field will be set by the Vector source that the event was created in.
    #[serde(default = "LogSchema::default_source_type_key")]
    source_type_key: String,

    /// The name of the event field to set the internal event metadata in.
    ///
    /// Generally, this field will be set by Vector to hold event-specific metadata, such as
    /// annotations by the `remap` transform when an error or abort is encountered.
    /// Mezmo: this field is separate from our own `metadata` field, which is used to carry
    /// user-facing event metadata. The internal event metadata field is not exposed to users
    /// as it may (now, or in the future from upstream changes) contain information that should
    /// not be available within the pipeline.
    #[serde(default = "LogSchema::default_metadata_key")]
    metadata_key: String,

    /// The name of the event field for user-facing event metadata
    ///
    /// This field is intended to be populated with metadata from our own processing, either
    /// from the edge as request metadata, or from kafka (headers). It can contain values
    /// that may be referenced in field templates.
    #[serde(default = "LogSchema::default_user_metadata_key")]
    user_metadata_key: String,

    /// The name of the event field for Mezmo's internal annotations
    #[serde(default = "LogSchema::default_annotations_key")]
    annotations_key: String,
}

impl Default for LogSchema {
    fn default() -> Self {
        LogSchema {
            message_key: Self::default_message_key(),
            timestamp_key: Self::default_timestamp_key(),
            host_key: Self::default_host_key(),
            source_type_key: Self::default_source_type_key(),
            metadata_key: Self::default_metadata_key(),
            user_metadata_key: Self::default_user_metadata_key(),
            annotations_key: Self::default_annotations_key(),
        }
    }
}

impl LogSchema {
    fn default_message_key() -> String {
        String::from("message")
    }

    fn default_timestamp_key() -> OptionalValuePath {
        OptionalValuePath {
            path: Some(owned_value_path!("timestamp")),
        }
    }

    fn default_host_key() -> String {
        String::from("host")
    }

    fn default_source_type_key() -> String {
        String::from("source_type")
    }

    fn default_metadata_key() -> String {
        String::from("internal_metadata")
    }

    fn default_user_metadata_key() -> String {
        String::from("metadata")
    }

    fn default_annotations_key() -> String {
        String::from("annotations")
    }

    pub fn message_key(&self) -> &str {
        &self.message_key
    }

    /// Returns an `OwnedTargetPath` of the message key.
    /// This parses the path and will panic if it is invalid.
    ///
    /// This should only be used where the result will either be cached,
    /// or performance isn't critical, since this requires parsing / memory allocation.
    pub fn owned_message_path(&self) -> OwnedTargetPath {
        parse_target_path(self.message_key()).expect("valid message key")
    }

    pub fn timestamp_key(&self) -> Option<&OwnedValuePath> {
        self.timestamp_key.path.as_ref()
    }

    pub fn host_key(&self) -> &str {
        &self.host_key
    }

    pub fn source_type_key(&self) -> &str {
        &self.source_type_key
    }

    pub fn metadata_key(&self) -> &str {
        &self.metadata_key
    }

    pub fn user_metadata_key(&self) -> &str {
        &self.user_metadata_key
    }

    pub fn annotations_key(&self) -> &str {
        &self.annotations_key
    }

    pub fn set_message_key(&mut self, v: String) {
        self.message_key = v;
    }

    pub fn set_timestamp_key(&mut self, v: Option<OwnedValuePath>) {
        self.timestamp_key = OptionalValuePath { path: v };
    }

    pub fn set_host_key(&mut self, v: String) {
        self.host_key = v;
    }

    pub fn set_source_type_key(&mut self, v: String) {
        self.source_type_key = v;
    }

    pub fn set_metadata_key(&mut self, v: String) {
        self.metadata_key = v;
    }

    pub fn set_user_metadata_key(&mut self, v: String) {
        self.user_metadata_key = v;
    }

    /// Merge two `LogSchema` instances together.
    ///
    /// # Errors
    ///
    /// This function will fail when the `LogSchema` to be merged contains
    /// conflicting keys.
    pub fn merge(&mut self, other: &LogSchema) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if *other != *LOG_SCHEMA_DEFAULT {
            // If the set value is the default, override it. If it's already overridden, error.
            if self.host_key() != LOG_SCHEMA_DEFAULT.host_key()
                && self.host_key() != other.host_key()
            {
                errors.push("conflicting values for 'log_schema.host_key' found".to_owned());
            } else {
                self.set_host_key(other.host_key().to_string());
            }
            if self.message_key() != LOG_SCHEMA_DEFAULT.message_key()
                && self.message_key() != other.message_key()
            {
                errors.push("conflicting values for 'log_schema.message_key' found".to_owned());
            } else {
                self.set_message_key(other.message_key().to_string());
            }
            if self.timestamp_key() != LOG_SCHEMA_DEFAULT.timestamp_key()
                && self.timestamp_key() != other.timestamp_key()
            {
                errors.push("conflicting values for 'log_schema.timestamp_key' found".to_owned());
            } else {
                self.set_timestamp_key(other.timestamp_key().cloned());
            }
            if self.source_type_key() != LOG_SCHEMA_DEFAULT.source_type_key()
                && self.source_type_key() != other.source_type_key()
            {
                errors.push("conflicting values for 'log_schema.source_type_key' found".to_owned());
            } else {
                self.set_source_type_key(other.source_type_key().to_string());
            }
            if self.metadata_key() != LOG_SCHEMA_DEFAULT.metadata_key()
                && self.metadata_key() != other.metadata_key()
            {
                errors.push("conflicting values for 'log_schema.metadata_key' found".to_owned());
            } else {
                self.set_metadata_key(other.metadata_key().to_string());
            }
            if self.user_metadata_key() != LOG_SCHEMA_DEFAULT.user_metadata_key()
                && self.user_metadata_key() != other.user_metadata_key()
            {
                errors
                    .push("conflicting values for 'log_schema.user_metadata_key' found".to_owned());
            } else {
                self.set_user_metadata_key(other.user_metadata_key().to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn partial_log_schema() {
        let toml = r#"
            message_key = "message"
            timestamp_key = "timestamp"
            metadata_key = "not_used_metadata"
            user_metadata_key = "meta"
        "#;
        toml::from_str::<LogSchema>(toml).unwrap();
    }
}
