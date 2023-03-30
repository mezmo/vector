use std::collections::BTreeMap;

use ::value::Value;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Deserializer, Value as JsonValue};
use vrl::prelude::*;

/// Parse Splunk HEC events from JSON string into an array of values
fn parse_splunk_hec(value: Value) -> Resolved {
    let bytes = &value.try_bytes()?;
    let string = String::from_utf8_lossy(bytes);
    let stream = Deserializer::from_str(&string).into_iter::<JsonValue>();
    let mut values = vec![];

    for json in stream {
        let json = json.map_err(|e| format!("unable to parse json: {e}"))?;
        let mut object = BTreeMap::new();

        match json.get("event") {
            Some(event) => {
                let event = event.into();
                object.insert("event".to_string(), event);
            }
            None => return Err(r#"missing "event" field"#.into()),
        }

        insert_string_value(&json, "host", &mut object)?;
        insert_string_value(&json, "source", &mut object)?;
        insert_string_value(&json, "sourcetype", &mut object)?;
        insert_string_value(&json, "index", &mut object)?;

        match json.get("time") {
            Some(JsonValue::Number(time)) => {
                object.insert("time".to_string(), Value::Timestamp(to_timestamp(time)?));
            }
            Some(JsonValue::String(time)) => {
                let time = time
                    .parse::<serde_json::Number>()
                    .map_err(|e| format!("invalid time format: {e}"))?;
                object.insert("time".to_string(), Value::Timestamp(to_timestamp(&time)?));
            }
            None => (), // "time" is optional
            _ => return Err(r#""time" is invalid type"#.into()),
        }

        if let Some(fields) = json.get("fields") {
            if !fields.is_object() {
                return Err(r#""fields" is not an object"#.into());
            }
            object.insert("fields".to_string(), fields.into());
        }

        values.push(Value::Object(object));
    }

    Ok(Value::Array(values))
}

/// Converts a JSON number to a timestamp using the same method used by the Splunk HEC source.
fn to_timestamp(number: &serde_json::Number) -> Result<DateTime<Utc>> {
    if let Some(time) = number.as_i64() {
        parse_timestamp(time).ok_or_else(|| r#""time" is invalid date format"#.into())
    } else if let Some(time) = number.as_f64() {
        Utc.timestamp_opt(
            time.floor() as i64,
            (time.fract() * 1000.0 * 1000.0 * 1000.0) as u32,
        )
        .single()
        .ok_or_else(|| r#""time" is invalid date format"#.into())
    } else {
        Err(r#""time" is invalid date format"#.into())
    }
}

/// Checks for an optional string value and inserts into into the final object
/// if present and is a string.
fn insert_string_value(
    json: &JsonValue,
    name: &str,
    object: &mut BTreeMap<String, Value>,
) -> Result<()> {
    if let Some(value) = json.get(name) {
        let value = value
            .as_str()
            .ok_or_else(|| format!(r#""{name}" is not a string"#))?;
        object.insert(name.to_string(), Value::from(value));
    }
    Ok(())
}

/// Functionality originally from the Splunk HEC source
///
/// Parse a `i64` unix timestamp that can either be in seconds, milliseconds or
/// nanoseconds.
///
/// This attempts to parse timestamps based on what cutoff range they fall into.
/// For seconds to be parsed the timestamp must be less than the unix epoch of
/// the year `2400`. For this to parse milliseconds the time must be smaller
/// than the year `10,000` in unix epoch milliseconds. If the value is larger
/// than both we attempt to parse it as nanoseconds.
///
/// Returns `None` if `t` is negative.
fn parse_timestamp(t: i64) -> Option<DateTime<Utc>> {
    // Utc.ymd(2400, 1, 1).and_hms(0,0,0).timestamp();
    const SEC_CUTOFF: i64 = 13_569_465_600;
    // Utc.ymd(10_000, 1, 1).and_hms(0,0,0).timestamp_millis();
    const MILLISEC_CUTOFF: i64 = 253_402_300_800_000;

    // Timestamps can't be negative!
    if t < 0 {
        return None;
    }

    if t < SEC_CUTOFF {
        Utc.timestamp_opt(t, 0).single()
    } else if t < MILLISEC_CUTOFF {
        Utc.timestamp_millis_opt(t).single()
    } else {
        Some(Utc.timestamp_nanos(t))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParseSplunkHec;

impl Function for ParseSplunkHec {
    fn identifier(&self) -> &'static str {
        "parse_splunk_hec"
    }

    fn summary(&self) -> &'static str {
        "parse a string to a Splunk HEC log events"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            kind: kind::BYTES,
            required: true,
        }]
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "valid",
            source: r#"parse_splunk_hec!(s'{ "event": "event1" }')"#,
            result: Ok(r#"[{ "event": "event1" }]"#),
        }]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");

        Ok(ParseSplunkHecFn { value }.as_expr())
    }
}

#[derive(Debug, Clone)]
struct ParseSplunkHecFn {
    value: Box<dyn Expression>,
}

impl FunctionExpression for ParseSplunkHecFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let value = self.value.resolve(ctx)?;
        parse_splunk_hec(value)
    }

    fn type_def(&self, _: &state::TypeState) -> TypeDef {
        type_def()
    }
}

fn type_def() -> TypeDef {
    TypeDef::array(Collection::any()).fallible()
}

#[cfg(test)]
mod tests {
    use super::*;

    test_function![
        parse_splunk_hec => ParseSplunkHec;

        valid {
            args: func_args![ value: r#"{
                "time": 1426279439,
                "event": "abc",
                "index": "main",
                "host": "localhost",
                "source": "some_source",
                "sourcetype": "some_sourcetype" }"# ],
            want: Ok(value!([{
                time: (Utc.timestamp(1_426_279_439, 0)),
                event: "abc",
                index: "main",
                host: "localhost",
                source: "some_source",
                sourcetype: "some_sourcetype",
            }])),
            tdef: type_def(),
        }

        multiple_events {
            args: func_args![ value: r#"{"event": "abc"} {"event": "def"} {"event": "xyz"}"# ],
            want: Ok(value!([
                { event: "abc" },
                { event: "def" },
                { event: "xyz" }
            ])),
            tdef: type_def(),
        }

        event_complex {
            args: func_args![ value: r#"{"event": {"f1": "abc", "f2": 123}}"# ],
            want: Ok(value!([{
                event: (value!({f1: "abc", f2: 123}))
            }])),
            tdef: type_def(),
        }

        event_fields {
            args: func_args![ value: r#"{"event": "abc", "fields": {"f1": "abc", "f2": "def"}}"# ],
            want: Ok(value!([
                { event: "abc", fields: { f1: "abc", f2: "def" } },
            ])),
            tdef: type_def(),
        }

        time_as_str {
            args: func_args![ value: r#"{"event": "abc", "time": "1426279439"}"# ],
            want: Ok(value!([
                { event: "abc", time: (Utc.timestamp(1_426_279_439, 0)) },
            ])),
            tdef: type_def(),
        }

        no_event {
            args: func_args![ value: r#"{"index": "main"}"# ],
            want: Err(r#"missing "event" field"#),
            tdef: type_def(),
        }

        host_invalid_type {
            args: func_args![ value: r#"{"event": "abc", "host": 123}"# ],
            want: Err(r#""host" is not a string"#),
            tdef: type_def(),
        }

        index_invalid_type {
            args: func_args![ value: r#"{"event": "abc", "index": 123}"# ],
            want: Err(r#""index" is not a string"#),
            tdef: type_def(),
        }

        source_invalid_type {
            args: func_args![ value: r#"{"event": "abc", "source": 123}"# ],
            want: Err(r#""source" is not a string"#),
            tdef: type_def(),
        }

        sourcetype_invalid_type {
            args: func_args![ value: r#"{"event": "abc", "sourcetype": 123}"# ],
            want: Err(r#""sourcetype" is not a string"#),
            tdef: type_def(),
        }

        time_invalid_type_str {
            args: func_args![ value: r#"{"event": "abc", "time": "18446744073709551615"}"# ],
            want: Err(r#""time" is invalid date format"#),
            tdef: type_def(),
        }

        time_invalid_type_max_u64 {
            args: func_args![ value: r#"{"event": "abc", "time": 18446744073709551616}"# ],
            want: Err(r#""time" is invalid date format"#),
            tdef: type_def(),
        }

        fields_invalid_type {
            args: func_args![ value: r#"{"event": "abc", "fields": "not_valid"}"# ],
            want: Err(r#""fields" is not an object"#),
            tdef: type_def(),
        }
    ];
}
