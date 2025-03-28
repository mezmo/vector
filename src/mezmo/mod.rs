#![allow(missing_docs)]

use vector_lib::config::log_schema;
use vector_lib::event::LogEvent;

pub mod config;
pub mod event_trace;
#[cfg(feature = "component-persistence")]
pub mod persistence;
#[cfg(feature = "api-client")]
pub mod remote_task_execution;
pub mod user_trace;

#[macro_export]
macro_rules! mezmo_env_config {
    ($key:expr, $default:expr) => {
        match std::env::var($key) {
            Ok(value) => match value.parse() {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        error = %err,
                        "Unable to parse {value} for config key {}, using default value of {}",
                        $key, $default
                    );
                    $default
                }
          },
          Err(err) => {
                debug!(
                    error = %err,
                    "{} was not set, using default value of {}",
                    $key, $default
                );
                $default
            }
        }
    };
}

/// This function moves whatever is in the LogEvent's `message` property into
/// the root of a new LogEvent message.
pub fn reshape_log_event_by_message(log: &mut LogEvent) {
    let message_key = log_schema().message_key_target_path().unwrap();
    if log.get(message_key).is_some() {
        log.rename_key(message_key, ".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::vrl::value::Value;
    use vector_lib::btreemap;
    use vector_lib::config::log_schema;
    use vector_lib::event::LogEvent;

    #[test]
    fn reshaping_logevent_works_even_if_message_is_not_an_object() {
        let message_key = log_schema().message_key().unwrap().to_string();
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
        let message_key = log_schema().message_key().unwrap().to_string();
        let message_key_path = log_schema().message_key_target_path().unwrap();

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
        assert_eq!(
            event.get(message_key_path),
            None,
            "message property is now gone"
        );
    }

    #[test]
    fn reshaping_successful_and_trashes_other_root_level_properties() {
        let mut event = LogEvent::default();
        let message_key = log_schema().message_key().unwrap().to_string();

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
