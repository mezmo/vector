use chrono::SecondsFormat;
use std::ops::SubAssign;

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
    logs::AnyValue as OtlpAnyValue, Array as OtlpArray, InstrumentationLibrary, Key, KeyValue,
    StringValue, Value as OtlpValue,
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
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            let diff = now
                - Duration::from_nanos(
                    time.to_owned()
                        .timestamp_nanos_opt()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                );

            let mut ts = SystemTime::now();
            ts.sub_assign(diff);
            ts
        }
        Value::Integer(time) => {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let diff = now - Duration::from_millis((*time).try_into().unwrap());
            let mut ts = SystemTime::now();
            ts.sub_assign(diff);
            ts
        }
        _ => SystemTime::now(),
    }
}

#[derive(Debug)]
pub struct OpentelemetryResource {
    pub attributes: Vec<KeyValue>,
}

impl From<&LogEvent> for OpentelemetryResource {
    fn from(log: &LogEvent) -> Self {
        let mut attributes = vec![];

        if let Some(metadata) = log.get((PathPrefix::Event, log_schema().user_metadata_key())) {
            if let Some(Value::Object(obj)) = metadata.get("resource") {
                for (key, value) in obj.iter() {
                    attributes.push(KeyValue::new(
                        key.to_string(),
                        value_to_otlp_value(value.clone()),
                    ));
                }
            }
        }

        OpentelemetryResource { attributes }
    }
}

impl From<OpentelemetryResource> for Resource {
    fn from(val: OpentelemetryResource) -> Self {
        Resource::new(val.attributes)
    }
}

#[derive(Debug)]
pub struct OpentelemetryScope {
    pub name: Cow<'static, str>,
    pub version: Option<Cow<'static, str>>,
    pub schema_url: Option<Cow<'static, str>>,
    pub attributes: Vec<KeyValue>,
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
            attributes,
        }
    }
}

impl From<OpentelemetryScope> for InstrumentationLibrary {
    fn from(val: OpentelemetryScope) -> Self {
        InstrumentationLibrary::new(val.name, val.version, val.schema_url, Some(val.attributes))
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
