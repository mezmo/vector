use crate::config::log_schema;
use crate::event::{Event, MaybeAsLogMut, Value};
use bytes::Bytes;

use super::common::{get_message_object, get_message_object_mut, parse_timestamp, TimestampUnit};
use super::MezmoDatadogAgentParser;

/// See: https://github.com/DataDog/agent-payload/blob/master/proto/logs/agent_logs_payload.proto
/// The log timestamp is in milliseconds, not seconds
pub fn transform_log(event: &mut Event, parser: &MezmoDatadogAgentParser) -> Result<(), String> {
    let log = event
        .maybe_as_log_mut()
        .ok_or_else(|| "Event is not a log".to_string())?;

    let parsed_timestamp = {
        let message_obj = get_message_object(log)?;
        message_obj
            .get("timestamp")
            .and_then(|value| parse_timestamp(value, TimestampUnit::Milliseconds))
    };

    let parsed_ddtags = {
        let message_obj = get_message_object(log)?;
        message_obj
            .get("ddtags")
            .and_then(|value| value.as_bytes())
            .map(parse_ddtags)
    };

    if let Some(parsed_ddtags) = parsed_ddtags {
        let message_obj = get_message_object_mut(log)?;
        message_obj.insert("ddtags".into(), parsed_ddtags);
    }

    if let Some(parsed) = parsed_timestamp {
        if let Some(timestamp_path) = log_schema().timestamp_key_target_path() {
            log.insert(timestamp_path, parsed);
        }
    }

    parser.strip_fields(event);

    Ok(())
}

// Mirrors the Datadog agent source parse_ddtags implementation.
fn parse_ddtags(ddtags: &Bytes) -> Value {
    String::from_utf8_lossy(ddtags)
        .split(',')
        .filter_map(|tag_entry| {
            let trimmed = tag_entry.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(Value::Bytes(Bytes::from(trimmed.to_string())))
            }
        })
        .collect::<Vec<Value>>()
        .into()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bytes::Bytes;
    use chrono::{TimeZone, Utc};

    use crate::config::log_schema;
    use crate::event::{Event, EventMetadata, KeyString, LogEvent, Value};
    use crate::transforms::mezmo_datadog_agent_parser::{
        MezmoDatadogAgentParser, MezmoDatadogAgentParserConfig,
    };

    use super::transform_log;

    fn build_event(message: BTreeMap<KeyString, Value>) -> Event {
        Event::Log(LogEvent::from_map(
            [(
                log_schema()
                    .message_key()
                    .expect("message key")
                    .to_string()
                    .into(),
                Value::Object(message),
            )]
            .into_iter()
            .collect(),
            EventMetadata::default(),
        ))
    }

    #[test]
    fn parse_timestamp_and_tags() {
        let mut message = BTreeMap::new();
        message.insert("timestamp".into(), Value::Integer(1_700_000_000_123));
        message.insert(
            "ddtags".into(),
            Value::Bytes(Bytes::from_static(b"env:prod,team:core")),
        );
        message.insert("status".into(), Value::Bytes(Bytes::from_static(b"info")));

        let mut event = build_event(message);
        let parser = MezmoDatadogAgentParser::new(&MezmoDatadogAgentParserConfig::default());

        transform_log(&mut event, &parser).expect("transform should succeed");

        let log = event.as_log();
        let ts_path = log_schema()
            .timestamp_key_target_path()
            .expect("timestamp key");

        let message = log
            .get(log_schema().message_key_target_path().expect("message key"))
            .and_then(|val| val.as_object())
            .expect("message object");

        let message_ts = message
            .get("timestamp")
            .and_then(Value::as_integer)
            .expect("message timestamp");
        let expected_ts = Utc
            .timestamp_millis_opt(message_ts)
            .latest()
            .expect("valid timestamp");

        assert_eq!(log.get(ts_path), Some(&Value::Timestamp(expected_ts)));

        assert_eq!(
            message.get("ddtags"),
            Some(&Value::Array(vec![
                Value::Bytes(Bytes::from_static(b"env:prod")),
                Value::Bytes(Bytes::from_static(b"team:core")),
            ]))
        );
        assert_eq!(message.get("timestamp"), Some(&Value::Integer(message_ts)));
        assert_eq!(
            message.get("status"),
            Some(&Value::Bytes(Bytes::from_static(b"info")))
        );
    }

    #[test]
    fn handles_invalid_timestamp() {
        let mut message = BTreeMap::new();
        message.insert(
            "timestamp".into(),
            Value::Bytes(Bytes::from_static(b"not-a-ts")),
        );

        let mut event = build_event(message);
        let parser = MezmoDatadogAgentParser::new(&MezmoDatadogAgentParserConfig::default());

        transform_log(&mut event, &parser).expect("transform should succeed");

        let log = event.as_log();
        let ts_path = log_schema()
            .timestamp_key_target_path()
            .expect("timestamp key");
        assert!(log.get(ts_path).is_none());
    }
}
