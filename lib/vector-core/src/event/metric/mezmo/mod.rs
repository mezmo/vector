mod vector;

use crate::config::log_schema;
use opentelemetry_rs::opentelemetry::common::AnyValueOneOfvalue as OpenTelemetryMetricAnyValue;
use opentelemetry_rs::opentelemetry::common::ArrayValue as OpenTelemetryMetricArrayValue;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Deref;

use crate::event::{
    metric::{MetricKind, TagValue},
    LogEvent, Value,
};

pub use vector::{from_metric, to_metric, TransformError};

pub fn from_f64_or_zero(value: f64) -> Value {
    use ordered_float::NotNan;
    NotNan::new(value).map_or_else(
        |_| Value::Float(NotNan::new(0.0).expect("0.0 is not NaN")),
        Value::Float,
    )
}

#[derive(Debug)]
pub struct MezmoMetric<'a, 'b, 'c, 'd, 'e, 'f, 'j, T, U, V, M, A> {
    pub name: Cow<'a, str>,
    pub namespace: Option<Cow<'b, str>>,
    pub kind: &'c T,
    pub tags: Option<&'d U>,
    pub value: &'e V,
    pub user_metadata: Option<&'f M>,
    pub arbitrary_data: Option<&'j A>,
}

pub trait MetricKindAccessor {
    fn kind(&self) -> Option<MetricKind>;
}

impl MetricKindAccessor for MetricKind {
    fn kind(&self) -> Option<MetricKind> {
        Some(*self)
    }
}

impl<T> MetricKindAccessor for &T
where
    T: ?Sized + MetricKindAccessor,
{
    fn kind(&self) -> Option<MetricKind> {
        (**self).kind()
    }
}

impl<T> MetricKindAccessor for Option<&T>
where
    T: ?Sized + MetricKindAccessor,
{
    fn kind(&self) -> Option<MetricKind> {
        (self).and_then(MetricKindAccessor::kind)
    }
}

pub struct MetricTags<'a, T>
where
    T: Iterator<Item = (&'a dyn ToString, &'a dyn IntoTagValue)> + Clone + 'a,
{
    pub tags: T,
}

impl<'a, T> MetricTags<'a, T>
where
    T: Iterator<Item = (&'a dyn ToString, &'a dyn IntoTagValue)> + Clone + 'a,
{
    fn to_value(&self) -> Value {
        Value::Object(
            self.tags
                .clone()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_tag_value().as_option().into()))
                .collect::<BTreeMap<_, _>>(),
        )
    }
}

pub trait MetricTagsAccessor<'a> {
    type Iter: Iterator<Item = (&'a dyn ToString, &'a dyn IntoTagValue)> + Clone;
    fn tags(&'a self) -> MetricTags<'a, Self::Iter>;
}

pub trait IntoTagValue {
    fn to_tag_value(&self) -> TagValue;
}

impl<'a, T> IntoTagValue for &'a T
where
    TagValue: From<&'a T>,
    T: Sized + 'a,
{
    fn to_tag_value(&self) -> TagValue {
        TagValue::from(self)
    }
}

impl<'a> IntoTagValue for Cow<'a, str> {
    fn to_tag_value(&self) -> TagValue {
        TagValue::from(self.clone().into_owned())
    }
}

impl IntoTagValue for String {
    fn to_tag_value(&self) -> TagValue {
        TagValue::from(self.clone())
    }
}

impl<'a> IntoTagValue for &'a str {
    fn to_tag_value(&self) -> TagValue {
        TagValue::from(self.to_owned())
    }
}

impl IntoTagValue for OpenTelemetryMetricAnyValue<'_> {
    fn to_tag_value(&self) -> TagValue {
        match self.clone() {
            OpenTelemetryMetricAnyValue::string_value(val) => val.into(),
            OpenTelemetryMetricAnyValue::bool_value(val) => u32::from(val).to_string().into(),
            OpenTelemetryMetricAnyValue::int_value(val) => val.to_string().into(),
            OpenTelemetryMetricAnyValue::double_value(val) => val.to_string().into(),
            OpenTelemetryMetricAnyValue::bytes_value(val) => {
                String::from_utf8_lossy(&val[..]).into()
            }
            OpenTelemetryMetricAnyValue::array_value(val) => val.to_tag_value(),

            // NOTE Tag is supposed to be a scalar type, array and struct cannot be converted
            // We may need to serialize them if exists
            // OpenTelemetryMetricAnyValue::kvlist_value(kv_list) => ...
            _ => TagValue::Bare,
        }
    }
}

impl IntoTagValue for OpenTelemetryMetricArrayValue<'_> {
    fn to_tag_value(&self) -> TagValue {
        self.values
            .iter()
            .map(|any_value| match &any_value.value {
                OpenTelemetryMetricAnyValue::string_value(val) => val.to_string(),
                OpenTelemetryMetricAnyValue::bool_value(val) => u32::from(*val).to_string(),
                OpenTelemetryMetricAnyValue::int_value(val) => val.to_string(),
                OpenTelemetryMetricAnyValue::double_value(val) => val.to_string(),
                OpenTelemetryMetricAnyValue::bytes_value(val) => {
                    String::from_utf8_lossy(&val[..]).into()
                }
                OpenTelemetryMetricAnyValue::array_value(val) => match val.to_tag_value() {
                    TagValue::Value(val) => val,
                    TagValue::Bare => String::new(),
                },
                // NOTE Tag is supposed to be a scalar type, array and struct cannot be converted
                // We may need to serialize them if exists
                // OpenTelemetryMetricAnyValue::kvlist_value(kv_list) => ...
                _ => String::new(),
            })
            .collect::<Vec<String>>()
            .join(", ")
            .into()
    }
}

impl<'a, T> IntoValue for MetricTags<'a, T>
where
    T: Iterator<Item = (&'a dyn ToString, &'a dyn IntoTagValue)> + Clone + 'a,
{
    fn to_value(&self) -> Value {
        Value::Object(
            self.tags
                .clone()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_tag_value().as_option().into()))
                .collect::<BTreeMap<_, _>>(),
        )
    }
}

pub trait IntoValue {
    fn to_value(&self) -> Value;
}

impl IntoValue for BTreeMap<String, Value> {
    fn to_value(&self) -> Value {
        Value::from(self.clone())
    }
}

impl IntoValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl<T> IntoValue for Vec<T>
where
    T: IntoValue,
{
    fn to_value(&self) -> Value {
        Value::Array(self.iter().map(IntoValue::to_value).collect())
    }
}

impl<'a, T> IntoValue for &'a [T]
where
    T: IntoValue,
{
    fn to_value(&self) -> Value {
        Value::Array(self.iter().map(IntoValue::to_value).collect())
    }
}

impl<T> IntoValue for [T]
where
    T: IntoValue,
{
    fn to_value(&self) -> Value {
        Value::Array(self.iter().map(IntoValue::to_value).collect())
    }
}

impl IntoValue for f64 {
    fn to_value(&self) -> Value {
        from_f64_or_zero(*self)
    }
}

impl IntoValue for u64 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl IntoValue for u32 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl IntoValue for i32 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl IntoValue for bool {
    fn to_value(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl<'a> IntoValue for Cow<'a, [u64]> {
    fn to_value(&self) -> Value {
        Value::Array(
            self.clone()
                .as_ref()
                .deref()
                .iter()
                .map(|val| Value::from(*val))
                .collect::<Vec<Value>>(),
        )
    }
}

impl<'a> IntoValue for Cow<'a, [f64]> {
    fn to_value(&self) -> Value {
        Value::Array(
            self.clone()
                .as_ref()
                .deref()
                .iter()
                .map(|val| from_f64_or_zero(*val))
                .collect::<Vec<Value>>(),
        )
    }
}

pub struct MetricValueArray<T> {
    pub elements: T,
}

pub struct MetricValuePairs<T> {
    pub elements: T,
}

impl<'a, T> IntoValue for MetricValuePairs<T>
where
    T: Iterator<Item = (&'a dyn ToString, &'a dyn IntoValue)> + Clone,
{
    fn to_value(&self) -> Value {
        Value::Object(
            self.elements
                .clone()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_value()))
                .collect::<BTreeMap<_, _>>(),
        )
    }
}

impl<'a, T> IntoValue for MetricValueArray<T>
where
    T: Iterator<Item = &'a dyn IntoValue> + Clone,
{
    fn to_value(&self) -> Value {
        Value::Array(
            self.elements
                .clone()
                .into_iter()
                .map(IntoValue::to_value)
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a> IntoValue for Cow<'a, str> {
    fn to_value(&self) -> Value {
        Value::from(self.clone())
    }
}

pub enum MetricValueSerializable<'s, T, U> {
    Single(&'s dyn IntoValue),
    Array(MetricValueArray<T>),
    Object(MetricValuePairs<U>),
}

pub trait MetricValueAccessor<'a> {
    type ArrIter: Iterator<Item = &'a dyn IntoValue> + Clone;
    type ObjIter: Iterator<Item = (&'a dyn ToString, &'a dyn IntoValue)> + Clone;
    fn metric_type(&'a self) -> Option<Cow<'a, str>>;
    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter>;
}

impl<'a, M: MetricValueAccessor<'a> + ?Sized> MetricValueAccessor<'a> for &M {
    type ArrIter = M::ArrIter;
    type ObjIter = M::ObjIter;
    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        (**self).metric_type()
    }
    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        (**self).value()
    }
}

impl<'a, M: MetricValueAccessor<'a> + ?Sized> MetricValueAccessor<'a> for Box<M> {
    type ArrIter = M::ArrIter;
    type ObjIter = M::ObjIter;
    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        (**self).metric_type()
    }
    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        (**self).value()
    }
}

pub trait MetricArbitraryAccessor<'a> {
    type ObjIter: Iterator<Item = (&'a dyn ToString, &'a dyn IntoValue)> + Clone;
    fn value(&'a self) -> MetricValuePairs<Self::ObjIter>;
}

impl<'a, M: MetricArbitraryAccessor<'a> + ?Sized> MetricArbitraryAccessor<'a> for &M {
    type ObjIter = M::ObjIter;
    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        (**self).value()
    }
}

pub trait MetricToLogEvent {
    fn to_log_event(&self) -> LogEvent;
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'j, T, U, V, M, A> MetricToLogEvent
    for MezmoMetric<'a, 'b, 'c, 'd, 'e, 'f, 'j, T, U, V, M, A>
where
    T: MetricKindAccessor,
    U: MetricTagsAccessor<'d>,
    V: MetricValueAccessor<'e>,
    M: MetricArbitraryAccessor<'f>,
    A: MetricArbitraryAccessor<'j>,
{
    fn to_log_event(&self) -> LogEvent {
        let value = match self.value.value() {
            MetricValueSerializable::Single(value) => value.to_value(),
            MetricValueSerializable::Array(value_elements) => value_elements.to_value(),
            MetricValueSerializable::Object(value_elements) => value_elements.to_value(),
        };
        let mut value = if let Some(metric_type) = self.value.metric_type() {
            Value::Object(
                [
                    ("type".to_string(), Value::from(metric_type)),
                    ("value".to_string(), value),
                ]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            )
        } else {
            Value::Object(
                [("value".to_string(), value)]
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        };

        if let Some(arbitrary_data) = self.arbitrary_data {
            if let Value::Object(data) = arbitrary_data.value().to_value() {
                for (key, val) in data.iter() {
                    value.insert(key.as_str(), val.clone());
                }
            }
        }

        let mut values = BTreeMap::<String, Value>::new();
        values.insert("name".to_owned(), self.name.clone().into());
        if let Some(namespace) = &self.namespace {
            values.insert("namespace".to_owned(), namespace.clone().into());
        }

        if let Some(kind) = self.kind.kind() {
            values.insert(
                "kind".to_owned(),
                match kind {
                    MetricKind::Absolute => "absolute",
                    MetricKind::Incremental => "incremental",
                }
                .into(),
            );
        };

        if let Some(tags) = self.tags {
            values.insert("tags".to_owned(), tags.tags().to_value());
        }

        values.insert("value".to_owned(), value);

        let mut event = BTreeMap::<String, Value>::new();

        event.insert("message".to_owned(), Value::Object(values));

        if let Some(user_metadata) = self.user_metadata {
            event.insert(
                log_schema().user_metadata_key().to_string(),
                user_metadata.value().to_value(),
            );
        }

        LogEvent::from_map(event, Default::default())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::BTreeMap;

    use vrl::value::Value;

    use crate::event::LogEvent;

    use super::{
        IntoTagValue, IntoValue, MetricArbitraryAccessor, MetricTags, MetricTagsAccessor,
        MetricToLogEvent, MetricValueAccessor, MetricValuePairs, MetricValueSerializable,
    };

    impl<'a> MetricTagsAccessor<'a> for String {
        type Iter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoTagValue), 1>;

        fn tags(&'a self) -> MetricTags<'a, Self::Iter> {
            MetricTags {
                tags: [(&"key" as &dyn ToString, self as &dyn IntoTagValue)].into_iter(),
            }
        }
    }

    impl IntoValue for String {
        fn to_value(&self) -> Value {
            Value::from(self.clone())
        }
    }

    impl IntoValue for &str {
        fn to_value(&self) -> Value {
            Value::from((*self).to_string())
        }
    }

    impl<'a> MetricValueAccessor<'a> for String {
        type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
        type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 0>;

        fn metric_type(&'a self) -> Option<Cow<'a, str>> {
            Some(Cow::from("str"))
        }
        fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
            MetricValueSerializable::Single(self as &dyn IntoValue)
        }
    }

    struct DummyValue<'a> {
        value: &'a str,
    }

    impl<'a> MetricValueAccessor<'a> for DummyValue<'a> {
        type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
        type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

        fn metric_type(&'a self) -> Option<Cow<'a, str>> {
            Some(Cow::from("dummy"))
        }
        fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
            MetricValueSerializable::Object(MetricValuePairs {
                elements: [
                    (&"complex" as &dyn ToString, &self.value as &dyn IntoValue),
                    (&"with" as &dyn ToString, &"multiple" as &dyn IntoValue),
                    (&"kv" as &dyn ToString, &"pairs" as &dyn IntoValue),
                ]
                .into_iter(),
            })
        }
    }

    struct DummyArbitrary<'a> {
        value: &'a str,
    }

    impl<'a> MetricArbitraryAccessor<'a> for DummyArbitrary<'a> {
        type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

        fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
            MetricValuePairs {
                elements: [
                    (&"complex" as &dyn ToString, &self.value as &dyn IntoValue),
                    (&"with" as &dyn ToString, &"multiple" as &dyn IntoValue),
                    (&"kv" as &dyn ToString, &"pairs" as &dyn IntoValue),
                ]
                .into_iter(),
            }
        }
    }

    #[test]
    fn mezmo_metric() {
        use super::MezmoMetric;
        use crate::event::metric::MetricKind;

        let tag = "test".to_string();
        let metric = MezmoMetric {
            name: Cow::from("test"),
            namespace: Some(Cow::from("ns")),
            kind: &MetricKind::Absolute,
            tags: Some(&tag),
            user_metadata: Some(&DummyArbitrary { value: "object" }),
            arbitrary_data: Some(&DummyArbitrary { value: "object" }),
            value: &String::new(),
        };
        let log_event = metric.to_log_event();

        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                     "name": "test",
                     "namespace": "ns",
                     "kind": "absolute",
                     "tags": { "key": "test" },
                     "value": {
                        "type": "str",
                        "value": "",
                        "complex": "object",
                        "with": "multiple",
                        "kv": "pairs"
                     }
                },
                "metadata": {
                     "complex": "object",
                     "with": "multiple",
                     "kv": "pairs"
                 }
            }"#,
        )
        .unwrap()
        .into();

        assert_eq!(log_event, expected, "{log_event:#?} vs \n{expected:#?}");

        let metric = MezmoMetric {
            name: Cow::from("test"),
            namespace: Some(Cow::from("ns")),
            kind: &MetricKind::Absolute,
            tags: None::<&String>,
            user_metadata: None::<&DummyArbitrary>,
            arbitrary_data: None::<&DummyArbitrary>,
            value: &DummyValue { value: "object" },
        };
        let log_event = metric.to_log_event();

        let expected: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(
            r#"{
                "message": {
                     "name": "test",
                     "namespace": "ns",
                     "kind": "absolute",
                     "value": {
                         "type": "dummy",
                         "value": {
                             "complex": "object",
                             "with": "multiple",
                             "kv": "pairs"
                         }
                     }
                }
            }"#,
        )
        .unwrap()
        .into();

        assert_eq!(log_event, expected, "{log_event:#?} vs \n{expected:#?}");
    }
}
