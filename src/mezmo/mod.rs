#![allow(missing_docs)]

use ::vrl::value::Value;
use snafu::Snafu;
use std::convert::Infallible;
use std::str::FromStr;

use vector_core::config::log_schema;
use vector_core::event::LogEvent;

pub mod callsite;
pub mod config;
pub mod macros;
#[allow(dead_code)]
pub mod user_trace;
pub mod vrl;

#[derive(Debug, Snafu)]
pub enum ContextParseError {
    #[snafu(display("{id} is not in user-facing component identifier format."))]
    NotUserIdentifier { id: String },
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ContextIdentifier {
    Value { id: String },
    Shared,
}

impl FromStr for ContextIdentifier {
    type Err = Infallible;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id == "shared" {
            Ok(Self::Shared)
        } else {
            Ok(Self::Value { id: id.to_owned() })
        }
    }
}

impl From<&ContextIdentifier> for Value {
    fn from(ctx: &ContextIdentifier) -> Self {
        match ctx {
            ContextIdentifier::Value { id } => Value::from(id.as_str()),
            ContextIdentifier::Shared => Value::from("shared"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ComponentKind {
    Source,
    Transform,
    Sink,
}

impl From<&ComponentKind> for Value {
    fn from(kind: &ComponentKind) -> Self {
        match kind {
            ComponentKind::Source => Value::from("source"),
            ComponentKind::Sink => Value::from("sink"),
            ComponentKind::Transform => Value::from("transform"),
        }
    }
}

/// Container for Mezmo-specific information that may be injected into a source, transform
/// or sink context as long as the identifier for that component follows our naming pattern
/// used when we generate pipeline components.
#[derive(Debug, Clone)]
pub struct MezmoContext {
    id: String,
    account_id: ContextIdentifier,
    pipeline_id: ContextIdentifier,
    component_id: String,
    component_kind: ComponentKind,
    internal: bool,
}

impl TryFrom<String> for MezmoContext {
    type Error = ContextParseError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        let mut parts: Vec<&str> = id.split(':').collect();
        if parts.len() == 4 && parts[3] == "shared" {
            // Format: {version}:{kind}:{component_id}:shared
            // Example: 'v1:internal_source:metrics:shared'
            //
            // To normalize the internal component id scheme into a user-defined id scheme,
            // we need to insert the type field and then insert the account_id, which should be
            // the same "shared" string as the pipeline_id.
            parts.insert(1, "");
            parts.push("shared");
        }

        if parts.len() != 6 || parts[0] != "v1" {
            return Err(Self::Error::NotUserIdentifier { id });
        }

        // Format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
        // Example: v1:mezmo:sink:ef757476-43a5-4e0d-b998-3db35dbde001:1515707f-f668-4ca1-8493-969e5b13e781:800e5a08-3e67-431c-bbf0-14aa94beafcc
        let account_id = ContextIdentifier::from_str(parts[5]).unwrap();
        let pipeline_id = ContextIdentifier::from_str(parts[4]).unwrap();
        let component_id = parts[3].to_owned();
        let internal = parts[2].starts_with("internal_");
        let component_kind = match parts[2] {
            "source" | "internal_source" => ComponentKind::Source,
            "sink" | "internal_sink" => ComponentKind::Sink,
            "transform" | "internal_transform" => ComponentKind::Transform,
            _ => return Err(Self::Error::NotUserIdentifier { id }),
        };

        Ok(Self {
            id,
            account_id,
            pipeline_id,
            component_id,
            component_kind,
            internal,
        })
    }
}

/// This function moves whatever is in the LogEvent's `message` property into
/// the root of a new LogEvent message.
pub fn reshape_log_event_by_message(log: &mut LogEvent) {
    let message_key = log_schema().message_key();
    if log.get(message_key).is_some() {
        log.rename_key(message_key, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vector_common::btreemap;
    use vector_core::config::log_schema;
    use vector_core::event::LogEvent;
    use vrl::value::Value;

    #[test]
    fn test_mezmo_context_try_from_shared_component() {
        let res = MezmoContext::try_from("v1:internal_source:http:shared".to_owned()).unwrap();
        assert_eq!(res.account_id, ContextIdentifier::Shared);
        assert_eq!(res.pipeline_id, ContextIdentifier::Shared);
        assert_eq!(res.component_kind, ComponentKind::Source);
        assert_eq!(res.component_id, "http".to_owned());
        assert!(res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_internal_component() {
        let res = MezmoContext::try_from(
            "v1:kafka:internal_source:component_id:pipeline_id:account_id".to_owned(),
        )
        .unwrap();
        assert!(matches!(res.account_id, ContextIdentifier::Value { id } if id == "account_id"));
        assert!(matches!(res.pipeline_id, ContextIdentifier::Value { id } if id == "pipeline_id"));
        assert_eq!(res.component_kind, ComponentKind::Source);
        assert_eq!(res.component_id, "component_id".to_owned());
        assert!(res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_user_component() {
        let res =
            MezmoContext::try_from("v1:mezmo:sink:component_id:pipeline_id:account_id".to_owned())
                .unwrap();
        assert!(matches!(res.account_id, ContextIdentifier::Value { id } if id == "account_id"));
        assert!(matches!(res.pipeline_id, ContextIdentifier::Value { id } if id == "pipeline_id"));
        assert_eq!(res.component_kind, ComponentKind::Sink);
        assert_eq!(res.component_id, "component_id".to_owned());
        assert!(!res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_invalid_case_1() {
        let invalid_formats = vec![
            "not even close to a valid format",
            "v1: this just happens to start with a valid prefix",
            "url:mezmo:sink:component_id:pipeline_id:account_id",
            "v2:mezmo:sink:component_id:pipeline_id:account_id",
            "",
        ];

        for case in invalid_formats {
            let res = MezmoContext::try_from(case.to_owned());
            assert!(
                matches!(res, Err(ContextParseError::NotUserIdentifier { id: _ })),
                "{} was not rejected as NotUserIdentifier",
                case
            );
        }
    }

    #[test]
    fn reshaping_logevent_works_even_if_message_is_not_an_object() {
        let message_key = log_schema().message_key();
        let mut event = LogEvent::from(btreemap! {
            message_key => "this is not an object event"
        });
        reshape_log_event_by_message(&mut event);

        assert_eq!(
            event,
            LogEvent::from(Value::from("this is not an object event")),
            "reshaping was done on a non-object message"
        );
    }

    #[test]
    fn reshaping_ignored_if_no_message_property() {
        let mut event = LogEvent::default();
        event.insert(".", "This value does not have a message key");
        reshape_log_event_by_message(&mut event);

        let mut expected = LogEvent::default();
        expected.insert(".", "This value does not have a message key");
        assert_eq!(
            event, expected,
            "payload not reshaped because there was no message property"
        );
    }

    #[test]
    fn reshaping_successful_reshape_message() {
        let mut event = LogEvent::default();
        let message_key = log_schema().message_key();

        event.insert(format!("{}.one", message_key).as_str(), 1);
        event.insert(format!("{}.two", message_key).as_str(), 2);
        event.insert(format!("{}.three.four", message_key).as_str(), 4);

        reshape_log_event_by_message(&mut event);

        let expected = LogEvent::from(btreemap! {
            "one" => 1,
            "two" => 2,
            "three" => btreemap! {
                "four" => 4
            },
        });

        assert_eq!(event, expected, "message payload was reshaped");
        assert_eq!(event.get(message_key), None, "message property is now gone");
    }

    #[test]
    fn reshaping_successful_and_trashes_other_root_level_properties() {
        let mut event = LogEvent::default();
        let message_key = log_schema().message_key();

        event.insert("trash1", "nope");
        event.insert("trash2", true);
        event.insert(format!("{}.one", message_key).as_str(), 1);
        event.insert(format!("{}.two", message_key).as_str(), 2);

        reshape_log_event_by_message(&mut event);

        let expected = LogEvent::from(btreemap! {
            "one" => 1,
            "two" => 2,
        });

        assert_eq!(
            event, expected,
            "Other root properties were trashed upon reshaping"
        );
    }
}
