use crate::{
    config::{
        schema::Definition, DataType, Input, LogNamespace, OutputId, TransformConfig,
        TransformContext,
    },
    event::Event,
    transforms::{TaskTransform, Transform},
};
use futures::StreamExt;
use vector_common::btreemap;
use vector_config::configurable_component;
use vector_core::{
    config::{log_schema, TransformOutput},
    usage_metrics::value_size,
};

use vrl::value::Value;

use std::collections::HashMap;
use std::future::ready;

/// Configuration for the `mezmo_log_classification` transform.
#[configurable_component(transform("mezmo_log_classification"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LogClassificationConfig {
    /// When a [[LogEvent]] ".message" property is an object, look for matches in these fields.
    /// Fields are evaluated in the order they are defined in the configuration, and the
    /// first matched field will classify the event. Note that these fields are relative to
    /// the message field rather than the root of the event.
    line_fields: Option<Vec<String>>,
}

impl_generate_config_from_default!(LogClassificationConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_log_classification")]
impl TransformConfig for LogClassificationConfig {
    async fn build(&self, _context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::event_task(LogClassification::new(self)))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: enrichment::TableRegistry,
        _: &[(OutputId, Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Log, HashMap::new())]
    }
}

pub struct LogClassification {
    parser: fgrok_parser::GrokMatcher,
    line_fields: Vec<String>,
}

impl LogClassification {
    pub fn new(config: &LogClassificationConfig) -> Self {
        LogClassification {
            parser: fgrok_parser::GrokMatcher::default(),
            line_fields: config.line_fields.clone().unwrap_or_default(),
        }
    }

    fn transform_one(&mut self, mut event: Event) -> Option<Event> {
        let log = event.as_mut_log();

        if let Some(message) = log.get(log_schema().message_key()) {
            let mut matches = vec![];
            let mut message_key = log_schema().message_key().to_string();

            // For object messages, look for matches in any of the line_fields in order.
            // Otherwise just look for matches in the message (string).
            // NOTE: array values for `message` are not explicitly handled here, as it is
            // expected the events are already unrolled when hitting this transform.
            if message.is_object() {
                for line_field in self.line_fields.iter() {
                    let value = message.get(line_field.as_str());
                    if let Some(value) = value {
                        // Avoid checking the string representation of objects/arrays
                        // that are deeply nested to avoid false-positives.
                        if value.is_array() || value.is_object() {
                            continue;
                        }

                        let line = value.to_string_lossy();
                        matches = self.parser.classify_all(&line);

                        // First field with matches wins
                        if !matches.is_empty() {
                            message_key = line_field.to_string();
                            break;
                        }
                    }
                }
            } else if message.is_bytes() {
                matches = self.parser.classify_all(&message.to_string_lossy())
            };

            // If there is no matches, classify as UNDEFINED
            if matches.is_empty() {
                matches = vec!["UNDEFINED".to_string()];
            }

            log.insert(
                (log_schema().annotations_key().to_string() + ".classification").as_str(),
                Value::Object(btreemap!(
                    "event_count".to_string() => Value::Integer(1),
                    "event_types".to_string() => Value::Object(matches.into_iter().map(|m| (m.to_string(), Value::Integer(1))).collect()),
                    "total_bytes".to_string() => Value::Integer(value_size(message) as i64),
                ))
            );
            log.insert(
                (log_schema().annotations_key().to_string() + ".message_key").as_str(),
                Value::Bytes(message_key.into()),
            );
        }

        Some(event)
    }
}

impl TaskTransform<Event> for LogClassification {
    fn transform(
        self: Box<Self>,
        task: std::pin::Pin<Box<dyn futures_util::Stream<Item = Event> + Send>>,
    ) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Event> + Send>> {
        let mut inner = self;
        Box::pin(task.filter_map(move |v| ready(inner.transform_one(v))))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use vector_core::event::Value;

    use super::*;
    use crate::event::{Event, LogEvent};
    use crate::test_util::components::assert_transform_compliance;
    use crate::transforms::test::create_topology;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<LogClassificationConfig>();
    }

    fn make_expected_annotations(
        input_event: &Event,
        message_key: String,
        matches: Vec<String>,
    ) -> Value {
        let mut annotations = BTreeMap::new();

        let message = input_event
            .as_log()
            .get(log_schema().message_key())
            .expect("message always exists in the presence of annotations");

        annotations.insert("message_key".to_string(), Value::Bytes(message_key.into()));
        annotations.insert("classification".to_string(), Value::Object(btreemap!(
            "event_count" => Value::Integer(1),
            "event_types" => Value::Object(matches.into_iter().map(|m| (m.to_string(), Value::Integer(1))).collect()),
            "total_bytes" => Value::Integer(value_size(message) as i64),
        )));
        Value::Object(annotations)
    }

    async fn do_transform(config: LogClassificationConfig, event: Event) -> Option<Event> {
        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let (topology, mut out) = create_topology(ReceiverStream::new(rx), config).await;
            tx.send(event).await.unwrap();
            let result = tokio::time::timeout(Duration::from_secs(5), out.recv())
                .await
                .unwrap_or(None);
            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
            result
        })
        .await
    }

    #[tokio::test]
    async fn event_with_string_message() {
        let line = r#"47.29.201.179 - - [28/Feb/2019:13:17:10 +0000] "GET /?p=1 HTTP/2.0" 200 5316 "https://domain1.com/?p=1" "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/72.0.3626.119 Safari/537.36" "2.75"#;
        let message_key = "message".to_string();
        let event = Event::Log(LogEvent::from(Value::Object(
            btreemap!(message_key.clone() => Value::Bytes(line.into())),
        )));

        let config = LogClassificationConfig { line_fields: None };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let annotations = make_expected_annotations(
            &event,
            message_key.clone(),
            vec![
                "HTTPD_COMBINEDLOG".to_string(),
                "HTTPD_COMMONLOG".to_string(),
            ],
        );

        // line is retained
        assert_eq!(
            output.as_log().get(message_key.as_str()),
            Some(Value::Bytes(line.into())).as_ref()
        );

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }

    #[tokio::test]
    async fn event_with_array_message() {
        let event = Event::Log(LogEvent::from(btreemap! {
            "message" => vec![
                r#"47.29.201.179 - - [28/Feb/2019:13:17:10 +0000] "GET /?p=1 HTTP/2.0" 200 5316 "https://domain1.com/?p=1" "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/72.0.3626.119 Safari/537.36" "2.75"#,
                r#"<161>2 2023-11-07T14:20:52.042-05:00 walker.net jeralddamore 948 ID430 - Authentication failed from 163.27.187.39 (163.27.187.39): Permission denied in replay cache code"#
            ]
        }));

        let config = LogClassificationConfig { line_fields: None };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let annotations =
            make_expected_annotations(&event, "message".to_string(), vec!["UNDEFINED".to_string()]);

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }

    #[tokio::test]
    async fn event_with_object_message_no_line_fields() {
        let event = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "key1" => "value1",
                "key2" => "value2",
                "key3" => "value3"
            }
        }));

        let config = LogClassificationConfig { line_fields: None };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let annotations =
            make_expected_annotations(&event, "message".to_string(), vec!["UNDEFINED".to_string()]);

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }

    #[tokio::test]
    async fn event_with_configured_line_fields_no_matches() {
        let event = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "key1" => "value1",
                "key2" => "value2",
                "key3" => "value3"
            }
        }));

        let config = LogClassificationConfig {
            line_fields: Some(vec![
                ".key1".to_string(),
                ".key2".to_string(),
                ".key3".to_string(),
            ]),
        };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let annotations =
            make_expected_annotations(&event, "message".to_string(), vec!["UNDEFINED".to_string()]);

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }

    #[tokio::test]
    async fn event_with_object_message_and_configured_line_fields() {
        let event = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "foo" => "bar",
                "apache" => r#"47.29.201.179 - - [28/Feb/2019:13:17:10 +0000] "GET /?p=1 HTTP/2.0" 200 5316 "https://domain1.com/?p=1" "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/72.0.3626.119 Safari/537.36" "2.75"#,
                "syslog" => r#"<161>2 2023-11-07T14:20:52.042-05:00 walker.net jeralddamore 948 ID430 - Authentication failed from 163.27.187.39 (163.27.187.39): Permission denied in replay cache code"#
            }
        }));

        let config = LogClassificationConfig {
            // First match wins, apache is not detected
            line_fields: Some(vec![".syslog".to_string(), ".apache".to_string()]),
        };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let annotations = make_expected_annotations(
            &event,
            ".syslog".to_string(),
            vec![
                "MCOLLECTIVEAUDIT".to_string(),
                "SYSLOG5424LINE".to_string(),
                "SYSLOGLINE".to_string(),
            ],
        );

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }

    #[tokio::test]
    async fn does_not_clobber_existing_annotations() {
        let event = Event::Log(LogEvent::from(btreemap! {
            "message" => r#"47.29.201.179 - - [28/Feb/2019:13:17:10 +0000] "GET /?p=1 HTTP/2.0" 200 5316 "https://domain1.com/?p=1" "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/72.0.3626.119 Safari/537.36" "2.75"#,
            "annotations" => btreemap! {
                "foo" => "bar",
            }
        }));

        let config = LogClassificationConfig { line_fields: None };
        let output = do_transform(config, event.clone().into()).await.unwrap();

        let mut annotations = make_expected_annotations(
            &event,
            "message".to_string(),
            vec![
                "HTTPD_COMBINEDLOG".to_string(),
                "HTTPD_COMMONLOG".to_string(),
            ],
        );

        annotations.insert("foo", Value::Bytes("bar".into()));

        assert_eq!(
            output.as_log().get(log_schema().annotations_key()),
            Some(&annotations)
        );
    }
}
