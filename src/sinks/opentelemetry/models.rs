use chrono::SecondsFormat;
use std::{ops::SubAssign, str::FromStr};

use std::collections::HashMap;
use vector_lib::{
    config::log_schema,
    event::{Event, LogEvent, Value},
    lookup::PathPrefix,
};

use super::{
    logs::model::OpentelemetryLogsModel, metrics::model::OpentelemetryMetricsModel,
    traces::model::OpentelemetryTracesModel,
};
use opentelemetry::{
    logs::AnyValue as OtlpAnyValue,
    trace::{SpanId, TraceFlags, TraceId, TraceState},
    Array as OtlpArray, InstrumentationLibrary, Key, KeyValue, StringValue, Value as OtlpValue,
};

use opentelemetry_sdk::Resource;
use std::{
    borrow::Cow,
    time::{Duration, SystemTime},
};

pub fn value_to_otlp_any_value(value: Value) -> OtlpAnyValue {
    match &value {
        Value::Bytes(bytes) => OtlpAnyValue::from(String::from_utf8_lossy(bytes).into_owned()),
        Value::Integer(int) => OtlpAnyValue::from(*int),
        Value::Float(float) => OtlpAnyValue::from(*float.as_ref()),
        Value::Boolean(bool) => OtlpAnyValue::from(*bool),
        Value::Timestamp(timestamp) => {
            OtlpAnyValue::from(timestamp.to_rfc3339_opts(SecondsFormat::AutoSi, true))
        }
        Value::Array(val_list) => val_list
            .iter()
            .map(|val| value_to_otlp_any_value(val.clone()))
            .collect::<OtlpAnyValue>(),
        Value::Object(object) => OtlpAnyValue::Map(
            object
                .iter()
                .map(|(key, value)| {
                    (
                        key.to_string().into(),
                        value_to_otlp_any_value(value.clone()),
                    )
                })
                .collect::<HashMap<Key, OtlpAnyValue>>(),
        ),
        Value::Null => OtlpAnyValue::from(""),
        _ => OtlpAnyValue::from(""),
    }
}

pub fn value_to_otlp_value(value: Value) -> OtlpValue {
    match &value {
        Value::Bytes(bytes) => OtlpValue::from(String::from_utf8_lossy(bytes).into_owned()),
        Value::Integer(int) => OtlpValue::I64(*int),
        Value::Float(float) => OtlpValue::F64(*float.as_ref()),
        Value::Boolean(bool) => OtlpValue::Bool(*bool),
        Value::Array(val_list) => OtlpValue::Array(value_to_otlp_array(val_list.to_vec())),
        Value::Timestamp(timestamp) => {
            OtlpValue::from(timestamp.to_rfc3339_opts(SecondsFormat::AutoSi, true))
        }
        Value::Null => OtlpValue::from(""),
        _ => OtlpValue::from(""),
        // Other value types: Regex, Object are not supported by the OtlpValue enum.
    }
}

pub fn value_to_otlp_array(values: Vec<Value>) -> OtlpArray {
    let mut string_values: Vec<StringValue> = vec![];

    for val in values.iter() {
        string_values.push(match &val {
            Value::Bytes(bytes) => String::from_utf8_lossy(bytes).into_owned().into(),
            Value::Integer(int) => int.to_string().into(),
            Value::Float(float) => float.to_string().into(),
            Value::Boolean(bool) => bool.to_string().into(),
            Value::Timestamp(timestamp) => timestamp
                .to_rfc3339_opts(SecondsFormat::AutoSi, true)
                .into(),
            Value::Null => "".to_string().into(),
            _ => "".to_string().into(),
            // Other value types: Array Regex, Object are not supported by the OtlpArray enum.
        });
    }

    string_values.into()
}

pub fn value_to_system_time(value: &Value) -> SystemTime {
    match value {
        Value::Timestamp(time) => {
            let mut now = SystemTime::now();
            let now_unix = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();

            let diff = now_unix
                - Duration::from_nanos(
                    time.to_owned()
                        .timestamp_nanos_opt()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                );

            now.sub_assign(diff);
            now
        }
        Value::Integer(time) => {
            let mut now = SystemTime::now();
            let now_unix = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let diff = now_unix - Duration::from_millis((*time).try_into().unwrap());
            now.sub_assign(diff);
            now
        }
        _ => SystemTime::now(),
    }
}

#[derive(Debug)]
pub struct OpentelemetryResource {
    pub attributes: OpentelemetryAttributes,
    pub schema_url: Cow<'static, str>,
}

pub struct OpentelemetryTraceId(TraceId);

impl From<Option<&Value>> for OpentelemetryTraceId {
    fn from(bytes: Option<&Value>) -> Self {
        match bytes {
            Some(Value::Bytes(bytes)) => {
                let mut trace_id = [0; 16];
                match faster_hex::hex_decode(bytes, &mut trace_id) {
                    Ok(_) => Self(TraceId::from_bytes(trace_id)),
                    Err(_) => Self(TraceId::INVALID),
                }
            }
            _ => Self(TraceId::INVALID),
        }
    }
}

impl From<OpentelemetryTraceId> for TraceId {
    fn from(trace_id: OpentelemetryTraceId) -> TraceId {
        trace_id.0
    }
}

pub struct OpentelemetrySpanId(SpanId);

impl From<Option<&Value>> for OpentelemetrySpanId {
    fn from(bytes: Option<&Value>) -> Self {
        match bytes {
            Some(Value::Bytes(bytes)) => {
                let mut span_id = [0; 8];
                match faster_hex::hex_decode(bytes, &mut span_id) {
                    Ok(_) => Self(SpanId::from_bytes(span_id)),
                    Err(_) => Self(SpanId::INVALID),
                }
            }
            _ => Self(SpanId::INVALID),
        }
    }
}

impl From<OpentelemetrySpanId> for SpanId {
    fn from(span_id: OpentelemetrySpanId) -> Self {
        span_id.0
    }
}

pub struct OpentelemetryTraceState(TraceState);

impl From<Option<&Value>> for OpentelemetryTraceState {
    fn from(bytes: Option<&Value>) -> Self {
        match bytes {
            Some(Value::Bytes(bytes)) => {
                let str = String::from_utf8_lossy(bytes);
                Self(TraceState::from_str(&str).unwrap_or_default())
            }
            _ => Self(TraceState::NONE),
        }
    }
}

impl From<OpentelemetryTraceState> for TraceState {
    fn from(state: OpentelemetryTraceState) -> Self {
        state.0
    }
}

pub struct OpentelemetryTraceFlags(TraceFlags);

impl From<Option<&Value>> for OpentelemetryTraceFlags {
    fn from(bytes: Option<&Value>) -> Self {
        match bytes {
            Some(Value::Integer(flag)) => Self(TraceFlags::new(
                u8::try_from(*flag).unwrap_or(TraceFlags::NOT_SAMPLED.to_u8()),
            )),
            _ => Self(TraceFlags::NOT_SAMPLED),
        }
    }
}

impl From<OpentelemetryTraceFlags> for TraceFlags {
    fn from(flags: OpentelemetryTraceFlags) -> Self {
        flags.0
    }
}

#[derive(Default, Debug)]
pub struct OpentelemetryAttributes(Vec<KeyValue>);

impl From<Option<&Value>> for OpentelemetryAttributes {
    fn from(value: Option<&Value>) -> Self {
        match value {
            Some(Value::Object(obj)) => Self(
                obj.iter()
                    .map(|(key, value)| {
                        KeyValue::new(key.to_string(), value_to_otlp_value(value.clone()))
                    })
                    .collect(),
            ),
            _ => Self(vec![]),
        }
    }
}

impl From<OpentelemetryAttributes> for Vec<KeyValue> {
    fn from(attrs: OpentelemetryAttributes) -> Self {
        attrs.0
    }
}

impl From<&LogEvent> for OpentelemetryResource {
    fn from(log: &LogEvent) -> Self {
        let mut attributes = vec![];
        let mut schema_url = Cow::from("");
        if let Some(metadata) = log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
            if let Some(Value::Object(obj)) = metadata.get("resource.attributes") {
                for (key, value) in obj.iter() {
                    attributes.push(KeyValue::new(
                        key.to_string(),
                        value_to_otlp_value(value.clone()),
                    ));
                }
            }

            if let Some(Value::Bytes(bytes)) = metadata.get("resource.schema_url") {
                schema_url = String::from_utf8_lossy(bytes).into_owned().into();
            }
        }

        OpentelemetryResource {
            attributes: OpentelemetryAttributes(attributes),
            schema_url,
        }
    }
}

impl From<OpentelemetryResource> for Resource {
    fn from(val: OpentelemetryResource) -> Self {
        Resource::from_schema_url(Into::<Vec<KeyValue>>::into(val.attributes), val.schema_url)
    }
}

#[derive(Debug)]
pub struct OpentelemetryScope {
    pub name: Cow<'static, str>,
    pub version: Option<Cow<'static, str>>,
    pub schema_url: Option<Cow<'static, str>>,
    pub attributes: OpentelemetryAttributes,
}

impl From<&LogEvent> for OpentelemetryScope {
    fn from(log: &LogEvent) -> Self {
        let mut name = Cow::from("");
        let mut version = None;
        let mut schema_url = None;
        let mut attributes = vec![];

        if let Some(metadata) = log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
            if let Some(scope) = metadata.get("scope") {
                name = if let Some(Value::Bytes(val)) = scope.get("name") {
                    Cow::from(String::from_utf8_lossy(val).into_owned())
                } else {
                    Cow::from("")
                };

                version = if let Some(Value::Bytes(val)) = scope.get("version") {
                    Some(Cow::from(String::from_utf8_lossy(val).into_owned()))
                } else {
                    None
                };

                schema_url = if let Some(Value::Bytes(val)) = scope.get("schema_url") {
                    Some(Cow::from(String::from_utf8_lossy(val).into_owned()))
                } else {
                    None
                };

                if let Some(Value::Object(obj)) = scope.get("attributes") {
                    for (key, value) in obj.iter() {
                        attributes.push(KeyValue::new(
                            key.to_string(),
                            value_to_otlp_value(value.clone()),
                        ));
                    }
                }
            }
        }

        Self {
            name,
            version,
            schema_url,
            attributes: OpentelemetryAttributes(attributes),
        }
    }
}

impl From<OpentelemetryScope> for InstrumentationLibrary {
    fn from(val: OpentelemetryScope) -> Self {
        InstrumentationLibrary::new(
            val.name,
            val.version,
            val.schema_url,
            Some(val.attributes.into()),
        )
    }
}

pub enum OpentelemetryModel {
    Logs(OpentelemetryLogsModel),
    Metrics(OpentelemetryMetricsModel),
    Traces(OpentelemetryTracesModel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpentelemetryModelType {
    Logs,
    Metrics,
    Traces,
}

pub trait OpentelemetryModelMatch {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType>
    where
        Self: Sized;
}

#[cfg(test)]
mod test {

    use super::*;
    use chrono::{NaiveDateTime, TimeZone, Utc};

    #[test]
    fn test_value_to_system_time_timestamp() {
        let value = Value::Timestamp(
            Utc.from_utc_datetime(
                &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                    .expect("timestamp should be a valid timestamp"),
            ),
        );

        let expected =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(1_579_134_612_000_000_011);
        assert_eq!(value_to_system_time(&value), expected);
    }

    #[test]
    fn test_value_to_system_time_int() {
        let value = Value::Integer(1_579_134_612);

        let expected = SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1_579_134_612);
        assert_eq!(value_to_system_time(&value), expected);
    }

    #[test]
    fn test_value_to_system_time_invalid_default_now() {
        let value = Value::from("invalid".to_string());

        assert!(matches!(value_to_system_time(&value), SystemTime { .. }));
    }
}
