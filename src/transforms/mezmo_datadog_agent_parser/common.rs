use chrono::{TimeZone, Utc};

use crate::config::log_schema;
use crate::event::{LogEvent, ObjectMap, Value};

#[derive(Copy, Clone, Debug)]
pub enum TimestampUnit {
    Seconds,
    Milliseconds,
}

pub fn get_message_object(log: &LogEvent) -> Result<&ObjectMap, String> {
    let message_path = log_schema()
        .message_key_target_path()
        .ok_or_else(|| "Missing message key".to_string())?;

    let message = log
        .get(message_path)
        .ok_or_else(|| "Missing message field".to_string())?;

    match message {
        Value::Object(obj) => Ok(obj),
        _ => Err("Message is not an object".to_string()),
    }
}

pub fn get_message_object_mut(log: &mut LogEvent) -> Result<&mut ObjectMap, String> {
    let message_path = log_schema()
        .message_key_target_path()
        .ok_or_else(|| "Missing message key".to_string())?;

    let message = log
        .get_mut(message_path)
        .ok_or_else(|| "Missing message field".to_string())?;

    match message {
        Value::Object(obj) => Ok(obj),
        _ => Err("Message is not an object".to_string()),
    }
}

pub fn parse_timestamp(value: &Value, unit: TimestampUnit) -> Option<chrono::DateTime<Utc>> {
    match value {
        Value::Timestamp(timestamp) => Some(*timestamp),
        Value::Integer(value) => match unit {
            TimestampUnit::Seconds => Utc.timestamp_opt(*value, 0).single(),
            TimestampUnit::Milliseconds => Utc.timestamp_millis_opt(*value).single(),
        },
        Value::Float(value) => {
            let value = value.into_inner();
            if !value.is_finite() {
                return None;
            }
            let seconds = match unit {
                TimestampUnit::Seconds => value,
                TimestampUnit::Milliseconds => value / 1000.0,
            };
            let (seconds, nanos) = split_float_seconds(seconds)?;
            Utc.timestamp_opt(seconds, nanos).single()
        }

        _ => None,
    }
}

fn split_float_seconds(value: f64) -> Option<(i64, u32)> {
    if value < (i64::MIN as f64) || value > (i64::MAX as f64) {
        return None;
    }

    let secs = value.floor();
    let fract = value - secs;
    let mut nanos = (fract * 1_000_000_000.0).round();

    // Handle rounding overflow (e.g., 0.9999999999 rounding up to 1s)
    let secs = if nanos >= 1_000_000_000.0 {
        nanos -= 1_000_000_000.0;
        secs + 1.0
    } else {
        secs
    };

    Some((secs as i64, nanos as u32))
}
