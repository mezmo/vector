use std::collections::HashMap;

use vector_lib::buffers::EventCount;
use vector_lib::event::{Event, LogEvent};
use vrl::value::{KeyString, Value};

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
    let mut map = HashMap::<KeyString, Value>::new();
    map.insert("message_0".into(), Value::from("value_0".to_owned()));
    map.insert("message_1".into(), Value::from("value_1".to_owned()));
    map.insert("message_2".into(), Value::from("value_2".to_owned()));

    let event = Event::Log(LogEvent::from(map));
    assert_eq!(event.event_count(), 1);

    let model = SumoLogsModel::try_from(vec![event]).expect("Failed mapping logs into model");
    let logs = model.0[0].get("logs").expect("Logs data store not present");

    assert!(logs[0].contains_key("message_0"));
    assert!(logs[0].contains_key("message_1"));
    assert!(logs[0].contains_key("message_2"));
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
