#![deny(missing_docs)]

use chrono::{DateTime, Utc};
use core::fmt::Debug;
use std::collections::BTreeMap;
use vector_lib::codecs::encoding::Serializer;

use ordered_float::NotNan;
use serde::{Deserialize, Deserializer};
use vector_lib::configurable::configurable_component;
use vector_lib::event::{LogEvent, MaybeAsLogMut};
use vector_lib::lookup::lookup_v2::ConfigValuePath;
use vector_lib::lookup::{event_path, PathPrefix};
use vector_lib::schema::meaning;
use vrl::path::OwnedValuePath;
use vrl::value::Value;

use crate::{event::Event, mezmo::reshape_log_event_by_message, serde::is_default};

/// Transformations to prepare an event for serialization.
#[configurable_component(no_deser)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Transformer {
    /// List of fields that are included in the encoded event.
    #[serde(default, skip_serializing_if = "is_default")]
    only_fields: Option<Vec<ConfigValuePath>>,

    /// List of fields that are excluded from the encoded event.
    #[serde(default, skip_serializing_if = "is_default")]
    except_fields: Option<Vec<ConfigValuePath>>,

    /// Format used for timestamp fields.
    #[serde(default, skip_serializing_if = "is_default")]
    timestamp_format: Option<TimestampFormat>,

    /// Should we do custom reshaping for Mezmo sinks?
    #[serde(skip_serializing)]
    should_mezmo_reshape: bool,
}

impl<'de> Deserialize<'de> for Transformer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct TransformerInner {
            #[serde(default)]
            only_fields: Option<Vec<OwnedValuePath>>,
            #[serde(default)]
            except_fields: Option<Vec<OwnedValuePath>>,
            #[serde(default)]
            timestamp_format: Option<TimestampFormat>,
        }

        let inner: TransformerInner = Deserialize::deserialize(deserializer)?;
        Self::new(
            inner
                .only_fields
                .map(|v| v.iter().map(|p| ConfigValuePath(p.clone())).collect()),
            inner
                .except_fields
                .map(|v| v.iter().map(|p| ConfigValuePath(p.clone())).collect()),
            inner.timestamp_format,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Transformer {
    /// Creates a new `Transformer`.
    ///
    /// Returns `Err` if `only_fields` and `except_fields` fail validation, i.e. are not mutually
    /// exclusive.
    pub fn new(
        only_fields: Option<Vec<ConfigValuePath>>,
        except_fields: Option<Vec<ConfigValuePath>>,
        timestamp_format: Option<TimestampFormat>,
    ) -> Result<Self, crate::Error> {
        Self::validate_fields(only_fields.as_ref(), except_fields.as_ref())?;

        Ok(Self {
            only_fields,
            except_fields,
            timestamp_format,
            should_mezmo_reshape: false,
        })
    }

    /// Creates a new `Transformer` with custom Mezmo reshape logic.
    /// The env var must be set to "1", and the encoding must be JSON or ndjson. If there is no
    /// Serializer used, we will default to doing the reshape.
    /// - LOG-22403 - added Protobuf serializer for vector-native OpenTelemetry Sink
    pub fn new_with_mezmo_reshape(
        transformer: Transformer,
        serializer: Option<&Serializer>,
    ) -> Self {
        let env_var_is_set = match std::env::var("MEZMO_RESHAPE_MESSAGE") {
            Ok(env_var) => env_var == "1",
            _ => false,
        };

        let should_mezmo_reshape = match (env_var_is_set, serializer) {
            (false, _) => false,
            (true, Some(serializer)) => match serializer {
                // For now, only explicit json and ndjson encodings are supported. May need to add more later.
                Serializer::Json(_) | Serializer::NativeJson(_) | Serializer::Protobuf(_) => true,
                _ => false,
            },
            // Lack of a serializer means we should reshape, ie things like elasticsearch don't use one
            (true, _) => true,
        };

        Self {
            only_fields: transformer.only_fields,
            except_fields: transformer.except_fields,
            timestamp_format: transformer.timestamp_format,
            should_mezmo_reshape,
        }
    }

    /// Get the `Transformer`'s `only_fields`.
    #[cfg(test)]
    pub const fn only_fields(&self) -> &Option<Vec<ConfigValuePath>> {
        &self.only_fields
    }

    /// Get the `Transformer`'s `except_fields`.
    pub const fn except_fields(&self) -> &Option<Vec<ConfigValuePath>> {
        &self.except_fields
    }

    /// Get the `Transformer`'s `timestamp_format`.
    pub const fn timestamp_format(&self) -> &Option<TimestampFormat> {
        &self.timestamp_format
    }

    /// Check if `except_fields` and `only_fields` items are mutually exclusive.
    ///
    /// If an error is returned, the entire encoding configuration should be considered inoperable.
    fn validate_fields(
        only_fields: Option<&Vec<ConfigValuePath>>,
        except_fields: Option<&Vec<ConfigValuePath>>,
    ) -> crate::Result<()> {
        if let (Some(only_fields), Some(except_fields)) = (only_fields, except_fields) {
            if except_fields
                .iter()
                .any(|f| only_fields.iter().any(|v| v == f))
            {
                return Err(
                    "`except_fields` and `only_fields` should be mutually exclusive.".into(),
                );
            }
        }
        Ok(())
    }

    /// Prepare an event for serialization by the given transformation rules.
    pub fn transform(&self, event: &mut Event) {
        // Rules are currently applied to logs only.
        if let Some(log) = event.maybe_as_log_mut() {
            if self.should_mezmo_reshape {
                reshape_log_event_by_message(log);
            }
            // Ordering in here should not matter.
            self.apply_except_fields(log);
            self.apply_only_fields(log);
            self.apply_timestamp_format(log);
        }
    }

    fn apply_only_fields(&self, log: &mut LogEvent) {
        if let Some(only_fields) = self.only_fields.as_ref() {
            let mut old_value = std::mem::replace(log.value_mut(), Value::Object(BTreeMap::new()));

            for field in only_fields {
                if let Some(value) = old_value.remove(field, true) {
                    log.insert((PathPrefix::Event, field), value);
                }
            }

            // We may need the service field to apply tags to emitted metrics after the log message has been pruned. If there
            // is a service meaning, we move this value to `dropped_fields` in the metadata.
            // If the field is still in the new log message after pruning it will have been removed from `old_value` above.
            let service_path = log
                .metadata()
                .schema_definition()
                .meaning_path(meaning::SERVICE);
            if let Some(service_path) = service_path {
                let mut new_log = LogEvent::from(old_value);
                if let Some(service) = new_log.remove(service_path) {
                    log.metadata_mut()
                        .add_dropped_field(meaning::SERVICE.into(), service);
                }
            }
        }
    }

    fn apply_except_fields(&self, log: &mut LogEvent) {
        if let Some(except_fields) = self.except_fields.as_ref() {
            for field in except_fields {
                let value_path = &field.0;
                let value = log.remove((PathPrefix::Event, value_path));

                let service_path = log
                    .metadata()
                    .schema_definition()
                    .meaning_path(meaning::SERVICE);
                // If we are removing the service field we need to store this in a `dropped_fields` list as we may need to
                // refer to this later when emitting metrics.
                if let (Some(v), Some(service_path)) = (value, service_path) {
                    if service_path.path == *value_path {
                        log.metadata_mut()
                            .add_dropped_field(meaning::SERVICE.into(), v);
                    }
                }
            }
        }
    }

    fn format_timestamps<F, T>(&self, log: &mut LogEvent, extract: F)
    where
        F: Fn(&DateTime<Utc>) -> T,
        T: Into<Value>,
    {
        if log.value().is_object() {
            let mut unix_timestamps = Vec::new();
            for (k, v) in log.all_event_fields().expect("must be an object") {
                if let Value::Timestamp(ts) = v {
                    unix_timestamps.push((k.clone(), extract(ts).into()));
                }
            }
            for (k, v) in unix_timestamps {
                log.parse_path_and_insert(k, v).unwrap();
            }
        } else {
            // root is not an object
            let timestamp = if let Value::Timestamp(ts) = log.value() {
                Some(extract(ts))
            } else {
                None
            };
            if let Some(ts) = timestamp {
                log.insert(event_path!(), ts.into());
            }
        }
    }

    fn apply_timestamp_format(&self, log: &mut LogEvent) {
        if let Some(timestamp_format) = self.timestamp_format.as_ref() {
            match timestamp_format {
                TimestampFormat::Unix => self.format_timestamps(log, |ts| ts.timestamp()),
                TimestampFormat::UnixMs => self.format_timestamps(log, |ts| ts.timestamp_millis()),
                TimestampFormat::UnixUs => self.format_timestamps(log, |ts| ts.timestamp_micros()),
                TimestampFormat::UnixNs => self.format_timestamps(log, |ts| {
                    ts.timestamp_nanos_opt().expect("Timestamp out of range")
                }),
                TimestampFormat::UnixFloat => self.format_timestamps(log, |ts| {
                    NotNan::new(ts.timestamp_micros() as f64 / 1e6).unwrap()
                }),
                // RFC3339 is the default serialization of a timestamp.
                TimestampFormat::Rfc3339 => (),
            }
        }
    }

    /// Set the `except_fields` value.
    ///
    /// Returns `Err` if the new `except_fields` fail validation, i.e. are not mutually exclusive
    /// with `only_fields`.
    #[cfg(test)]
    pub fn set_except_fields(
        &mut self,
        except_fields: Option<Vec<ConfigValuePath>>,
    ) -> crate::Result<()> {
        Self::validate_fields(self.only_fields.as_ref(), except_fields.as_ref())?;
        self.except_fields = except_fields;
        Ok(())
    }
}

#[configurable_component]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// The format in which a timestamp should be represented.
pub enum TimestampFormat {
    /// Represent the timestamp as a Unix timestamp.
    Unix,

    /// Represent the timestamp as a RFC 3339 timestamp.
    Rfc3339,

    /// Represent the timestamp as a Unix timestamp in milliseconds.
    UnixMs,

    /// Represent the timestamp as a Unix timestamp in microseconds
    UnixUs,

    /// Represent the timestamp as a Unix timestamp in nanoseconds.
    UnixNs,

    /// Represent the timestamp as a Unix timestamp in floating point.
    UnixFloat,
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use vector_lib::btreemap;
    use vector_lib::codecs::encoding::{
        format::{
            AvroSerializerConfig, GelfSerializer, JsonSerializer, JsonSerializerOptions,
            LogfmtSerializer, NativeJsonSerializer, RawMessageSerializer, TextSerializer,
        },
        Serializer,
    };
    use vector_lib::config::{log_schema, LogNamespace};
    use vector_lib::lookup::path::parse_target_path;
    use vrl::value::Kind;

    use crate::config::schema;

    use super::*;
    use assay::assay;
    use std::{collections::BTreeMap, sync::Arc};
    use vector_lib::codecs::MetricTagValues;

    #[test]
    fn serialize() {
        let string =
            r#"{"only_fields":["a.b[0]"],"except_fields":["ignore_me"],"timestamp_format":"unix"}"#;

        let transformer = serde_json::from_str::<Transformer>(string).unwrap();

        let serialized = serde_json::to_string(&transformer).unwrap();

        assert_eq!(string, serialized);
    }

    #[test]
    fn serialize_empty() {
        let string = "{}";

        let transformer = serde_json::from_str::<Transformer>(string).unwrap();

        let serialized = serde_json::to_string(&transformer).unwrap();

        assert_eq!(string, serialized);
    }

    #[test]
    fn deserialize_and_transform_except() {
        let transformer: Transformer =
            toml::from_str(r#"except_fields = ["a.b.c", "b", "c[0].y", "d.z", "e"]"#).unwrap();
        let mut log = LogEvent::default();
        {
            log.insert("a", 1);
            log.insert("a.b", 1);
            log.insert("a.b.c", 1);
            log.insert("a.b.d", 1);
            log.insert("b[0]", 1);
            log.insert("b[1].x", 1);
            log.insert("c[0].x", 1);
            log.insert("c[0].y", 1);
            log.insert("d.z", 1);
            log.insert("e.a", 1);
            log.insert("e.b", 1);
        }
        let mut event = Event::from(log);
        transformer.transform(&mut event);
        assert!(!event.as_mut_log().contains("a.b.c"));
        assert!(!event.as_mut_log().contains("b"));
        assert!(!event.as_mut_log().contains("b[1].x"));
        assert!(!event.as_mut_log().contains("c[0].y"));
        assert!(!event.as_mut_log().contains("d.z"));
        assert!(!event.as_mut_log().contains("e.a"));

        assert!(event.as_mut_log().contains("a.b.d"));
        assert!(event.as_mut_log().contains("c[0].x"));
    }

    #[test]
    fn deserialize_and_transform_only() {
        let transformer: Transformer =
            toml::from_str(r#"only_fields = ["a.b.c", "b", "c[0].y", "\"g.z\""]"#).unwrap();
        let mut log = LogEvent::default();
        {
            log.insert("a", 1);
            log.insert("a.b", 1);
            log.insert("a.b.c", 1);
            log.insert("a.b.d", 1);
            log.insert("b[0]", 1);
            log.insert("b[1].x", 1);
            log.insert("c[0].x", 1);
            log.insert("c[0].y", 1);
            log.insert("d.y", 1);
            log.insert("d.z", 1);
            log.insert("e[0]", 1);
            log.insert("e[1]", 1);
            log.insert("\"f.z\"", 1);
            log.insert("\"g.z\"", 1);
            log.insert("h", BTreeMap::new());
            log.insert("i", Vec::<Value>::new());
        }
        let mut event = Event::from(log);
        transformer.transform(&mut event);
        assert!(event.as_mut_log().contains("a.b.c"));
        assert!(event.as_mut_log().contains("b"));
        assert!(event.as_mut_log().contains("b[1].x"));
        assert!(event.as_mut_log().contains("c[0].y"));
        assert!(event.as_mut_log().contains("\"g.z\""));

        assert!(!event.as_mut_log().contains("a.b.d"));
        assert!(!event.as_mut_log().contains("c[0].x"));
        assert!(!event.as_mut_log().contains("d"));
        assert!(!event.as_mut_log().contains("e"));
        assert!(!event.as_mut_log().contains("f"));
        assert!(!event.as_mut_log().contains("h"));
        assert!(!event.as_mut_log().contains("i"));
    }

    #[test]
    fn deserialize_and_transform_timestamp() {
        let mut base = Event::Log(LogEvent::from("Demo"));
        let timestamp = base
            .as_mut_log()
            .get((PathPrefix::Event, log_schema().timestamp_key().unwrap()))
            .unwrap()
            .clone();
        let timestamp = timestamp.as_timestamp().unwrap();
        base.as_mut_log()
            .insert("another", Value::Timestamp(*timestamp));

        let cases = [
            ("unix", Value::from(timestamp.timestamp())),
            ("unix_ms", Value::from(timestamp.timestamp_millis())),
            ("unix_us", Value::from(timestamp.timestamp_micros())),
            (
                "unix_ns",
                Value::from(timestamp.timestamp_nanos_opt().unwrap()),
            ),
            (
                "unix_float",
                Value::from(timestamp.timestamp_micros() as f64 / 1e6),
            ),
        ];
        for (fmt, expected) in cases {
            let config: String = format!(r#"timestamp_format = "{fmt}""#);
            let transformer: Transformer = toml::from_str(&config).unwrap();
            let mut event = base.clone();
            transformer.transform(&mut event);
            let log = event.as_mut_log();

            for actual in [
                // original key
                log.get((PathPrefix::Event, log_schema().timestamp_key().unwrap()))
                    .unwrap(),
                // second key
                log.get("another").unwrap(),
            ] {
                // type matches
                assert_eq!(expected.kind_str(), actual.kind_str());
                // value matches
                assert_eq!(&expected, actual);
            }
        }
    }

    #[test]
    fn exclusivity_violation() {
        let config: std::result::Result<Transformer, _> = toml::from_str(indoc! {r#"
            except_fields = ["Doop"]
            only_fields = ["Doop"]
        "#});
        assert!(config.is_err())
    }

    #[test]
    fn deny_unknown_fields() {
        // We're only checking this explicitly because of our custom deserializer arrangement to
        // make it possible to throw the exclusivity error during deserialization, to ensure that we
        // enforce this on the top-level `Transformer` type even though it has to be applied at the
        // intermediate deserialization stage, on `TransformerInner`.
        let config: std::result::Result<Transformer, _> = toml::from_str(indoc! {r#"
            onlyfields = ["Doop"]
        "#});
        assert!(config.is_err())
    }

    #[test]
    fn mezmo_reshaping_env_var_must_be_set() {
        let transformer = Transformer::new_with_mezmo_reshape(Transformer::default(), None);
        let mut log_event = LogEvent::default();
        let path = format!("{}.nope", log_schema().message_key().unwrap());
        log_event.insert(path.as_str(), "This will not be reshaped");

        let mut event = Event::from(log_event);
        transformer.transform(&mut event);

        let expected = event.clone();

        assert_eq!(
            event, expected,
            "ENV var was not set, so the messsage was not reshaped"
        );
    }

    #[assay(
        env = [
          ("MEZMO_RESHAPE_MESSAGE", "0"),
        ]
      )]
    fn mezmo_reshaping_env_var_must_be_set_to_1() {
        let transformer = Transformer::new_with_mezmo_reshape(Transformer::default(), None);
        let mut log_event = LogEvent::default();
        let path = format!("{}.nope", log_schema().message_key().unwrap());
        log_event.insert(path.as_str(), "This will not be reshaped");

        let mut event = Event::from(log_event);
        transformer.transform(&mut event);

        let expected = event.clone();

        assert_eq!(
            event, expected,
            "ENV var was not set, so the messgae was not reshaped"
        );
    }

    #[assay(
        env = [
          ("MEZMO_RESHAPE_MESSAGE", "1"),
        ]
      )]
    fn mezmo_reshaping_based_on_serializer() {
        let mut transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(JsonSerializer::new(
                MetricTagValues::Single,
                JsonSerializerOptions::default(),
            ))),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, true,
            "JSON serializer should reshape"
        );

        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(NativeJsonSerializer {})),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, true,
            "ndjson serializer should reshape"
        );

        let schema = indoc! {r#"
            {
                "type": "record",
                "name": "Log",
                "fields": [
                    {
                        "name": "foo",
                        "type": ["string"]
                    }
                ]
            }
        "#}
        .to_owned();
        let avro = AvroSerializerConfig::new(schema).build().unwrap();
        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(avro)),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, false,
            "Avro serializer should NOT reshape"
        );

        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(GelfSerializer::new())),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, false,
            "Gelf serializer should NOT reshape"
        );

        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(RawMessageSerializer {})),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, false,
            "Raw message serializer should NOT reshape"
        );

        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(TextSerializer::new(
                MetricTagValues::Single,
            ))),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, false,
            "Text serializer should NOT reshape"
        );

        transformer = Transformer::new_with_mezmo_reshape(
            Transformer::default(),
            Some(&Serializer::from(LogfmtSerializer {})),
        );
        assert_eq!(
            transformer.should_mezmo_reshape, false,
            "Log format serializer should NOT reshape"
        );
    }

    #[assay(
        env = [
          ("MEZMO_RESHAPE_MESSAGE", "1"),
        ]
      )]
    fn mezmo_reshaping_successful_reshape_message() {
        let transformer = Transformer::new_with_mezmo_reshape(Transformer::default(), None);
        let mut log_event = LogEvent::default();
        let message_key = log_schema().message_key().unwrap().to_string();

        log_event.insert(format!("{message_key}.one").as_str(), 1);
        log_event.insert(format!("{message_key}.two").as_str(), 2);
        log_event.insert(format!("{message_key}.three.four").as_str(), 4);

        let mut event = Event::from(log_event);
        transformer.transform(&mut event);

        let expected_log = LogEvent::from(btreemap! {
            "one" => 1,
            "two" => 2,
            "three" => btreemap! {
                "four" => 4
            },
        });
        let expected = Event::from(expected_log);

        assert_eq!(event, expected, "message payload was reshaped");
        assert_eq!(
            event
                .as_log()
                .get(log_schema().message_key_target_path().unwrap()),
            None,
            "message property is now gone"
        );
    }

    #[test]
    fn only_fields_with_service() {
        let transformer: Transformer = toml::from_str(r#"only_fields = ["message"]"#).unwrap();
        let mut log = LogEvent::default();
        {
            log.insert("message", 1);
            log.insert("thing.service", "carrot");
        }

        let schema = schema::Definition::new_with_default_metadata(
            Kind::object(btreemap! {
                "thing" => Kind::object(btreemap! {
                    "service" => Kind::bytes(),
                })
            }),
            [LogNamespace::Vector],
        );

        let schema = schema.with_meaning(parse_target_path("thing.service").unwrap(), "service");

        let mut event = Event::from(log);

        event
            .metadata_mut()
            .set_schema_definition(&Arc::new(schema));

        transformer.transform(&mut event);
        assert!(event.as_mut_log().contains("message"));

        // Event no longer contains the service field.
        assert!(!event.as_mut_log().contains("thing.service"));

        // But we can still get the service by meaning.
        assert_eq!(
            &Value::from("carrot"),
            event.as_log().get_by_meaning("service").unwrap()
        );
    }

    #[test]
    fn except_fields_with_service() {
        let transformer: Transformer =
            toml::from_str(r#"except_fields = ["thing.service"]"#).unwrap();
        let mut log = LogEvent::default();
        {
            log.insert("message", 1);
            log.insert("thing.service", "carrot");
        }

        let schema = schema::Definition::new_with_default_metadata(
            Kind::object(btreemap! {
                "thing" => Kind::object(btreemap! {
                    "service" => Kind::bytes(),
                })
            }),
            [LogNamespace::Vector],
        );

        let schema = schema.with_meaning(parse_target_path("thing.service").unwrap(), "service");

        let mut event = Event::from(log);

        event
            .metadata_mut()
            .set_schema_definition(&Arc::new(schema));

        transformer.transform(&mut event);
        assert!(event.as_mut_log().contains("message"));

        // Event no longer contains the service field.
        assert!(!event.as_mut_log().contains("thing.service"));

        // But we can still get the service by meaning.
        assert_eq!(
            &Value::from("carrot"),
            event.as_log().get_by_meaning("service").unwrap()
        );
    }
}
