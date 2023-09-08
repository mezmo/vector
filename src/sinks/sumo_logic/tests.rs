use std::collections::HashMap;

use vector_buffers::EventCount;
use vector_core::event::{Event, LogEvent};
use vrl::value::Value;

use super::{
    config::{SumoLogicCredentials, SumoLogicSinkConfig},
    models::SumoLogsModel,
};

#[test]
fn generate_config() {
    crate::test_util::test_generate_config::<SumoLogicSinkConfig>();
}

#[test]
fn test_sink_log_event() {
    let mut map = HashMap::<String, Value>::new();
    map.insert("message_0".to_owned(), Value::from("value_0".to_owned()));
    map.insert("message_1".to_owned(), Value::from("value_1".to_owned()));
    map.insert("message_2".to_owned(), Value::from("value_2".to_owned()));

    let event = Event::Log(LogEvent::from(map));
    assert_eq!(event.event_count(), 1);

    let model = SumoLogsModel::try_from(vec![event]).expect("Failed mapping logs into model");
    let logs = model.0[0].get("logs").expect("Logs data store not present");

    assert!(logs[0].get("message_0").is_some());
    assert!(logs[0].get("message_1").is_some());
    assert!(logs[0].get("message_2").is_some());
    assert_eq!(
        logs[0].get("message_0").unwrap().to_string_lossy(),
        "value_0".to_owned()
    );
    assert_eq!(
        logs[0].get("message_1").unwrap().to_string_lossy(),
        "value_1".to_owned()
    );
    assert_eq!(
        logs[0].get("message_2").unwrap().to_string_lossy(),
        "value_2".to_owned()
    );
}

#[test]
fn test_invalid_uri() {
    let uri = String::from("invalid/blah");
    let creds = SumoLogicCredentials {
        endpoint: uri.into(),
    };

    assert!(creds.build_uri().is_err());
}
