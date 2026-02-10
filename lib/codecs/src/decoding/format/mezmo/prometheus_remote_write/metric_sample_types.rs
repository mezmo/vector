use std::borrow::Cow;

use vrl::value::Value;

use vector_core::event::metric::mezmo::{
    IntoValue, MetricValueAccessor, MetricValuePairs, MetricValueSerializable, from_f64_or_zero,
};

#[derive(Debug, Default, PartialEq)]
pub struct SummaryQuantileValue {
    pub quantile: f64,
    pub value: f64,
}

impl IntoValue for SummaryQuantileValue {
    fn to_value(&self) -> Value {
        Value::Object(
            [
                ("quantile".into(), from_f64_or_zero(self.quantile)),
                ("value".into(), from_f64_or_zero(self.value)),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SummaryMetricValue {
    pub quantiles: Vec<SummaryQuantileValue>,
    pub sum: f64,
    pub count: u64,
}

impl<'a> MetricValueAccessor<'a> for SummaryMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("summary"))
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"quantiles" as &dyn ToString,
                    &self.quantiles as &dyn IntoValue,
                ),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct HistogramBucketValue {
    pub upper_limit: f64,
    pub count: u64,
}

impl IntoValue for HistogramBucketValue {
    fn to_value(&self) -> Value {
        Value::Object(
            [
                ("upper_limit".into(), from_f64_or_zero(self.upper_limit)),
                ("count".into(), self.count.into()),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct HistogramMetricValue {
    pub buckets: Vec<HistogramBucketValue>,
    pub sum: f64,
    pub count: u64,
}

impl<'a> MetricValueAccessor<'a> for HistogramMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("histogram"))
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (&"bucket" as &dyn ToString, &self.buckets as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug)]
pub struct Counter;
#[derive(Debug)]
pub struct Gauge;
#[derive(Debug)]
pub struct Untyped;

#[derive(Debug, Default, PartialEq)]
pub struct BasicMetricValue<T> {
    pub value: f64,
    _t: std::marker::PhantomData<T>,
}

impl<T> BasicMetricValue<T> {
    pub fn new(value: f64) -> Self {
        BasicMetricValue {
            value,
            _t: std::marker::PhantomData {},
        }
    }
}

impl<T> IntoValue for BasicMetricValue<T> {
    fn to_value(&self) -> Value {
        from_f64_or_zero(self.value)
    }
}

impl<'a> MetricValueAccessor<'a> for BasicMetricValue<Counter> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("count"))
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Single(&self.value as &dyn IntoValue)
    }
}

impl<'a> MetricValueAccessor<'a> for BasicMetricValue<Gauge> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("gauge"))
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Single(&self.value as &dyn IntoValue)
    }
}

impl<'a> MetricValueAccessor<'a> for BasicMetricValue<Untyped> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Single(&self.value as &dyn IntoValue)
    }
}
