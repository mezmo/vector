use chrono::{TimeZone, Utc};

use crate::config::log_schema;
use crate::event::{LogEvent, ObjectMap, Value};

#[derive(Copy, Clone, Debug)]
pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Nanoseconds,
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
        Value::Integer(value) => {
            let (seconds, nanoseconds) = match unit {
                TimestampUnit::Seconds => (*value, None),
                TimestampUnit::Milliseconds => (
                    value.div_euclid(1000),
                    (value.rem_euclid(1000) as u32).checked_mul(1_000_000),
                ),
                TimestampUnit::Nanoseconds => (
                    value.div_euclid(1_000_000_000),
                    Some(value.rem_euclid(1_000_000_000) as u32),
                ),
            };
            let nanoseconds = nanoseconds.unwrap_or_default();
            Utc.timestamp_opt(seconds, nanoseconds).single()
        }
        Value::Float(value) => {
            let value = value.into_inner();
            if !value.is_finite() {
                return None;
            }
            let seconds = match unit {
                TimestampUnit::Seconds => value,
                TimestampUnit::Milliseconds => value / 1000.0,
                TimestampUnit::Nanoseconds => value / 1_000_000_000.0,
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

    // trunc handles negative values better than floor. -1.4 is -1, not -2
    let mut secs = value.trunc();
    let mut fractional_secs = value - secs;

    if fractional_secs < 0.0 {
        fractional_secs += 1.0;
        secs -= 1.0;
    }
    let mut nanoseconds = (fractional_secs * 1_000_000_000.0).round();

    if nanoseconds >= 1_000_000_000.0 {
        nanoseconds -= 1_000_000_000.0;
        secs += 1.0;
    }

    if secs < i64::MIN as f64 || secs > i64::MAX as f64 {
        return None;
    }
    Some((secs as i64, nanoseconds as u32))
}
