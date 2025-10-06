use std::collections::HashSet;
use std::mem::size_of;

use bytes::{Bytes, BytesMut};
use chrono::{DateTime, Utc};
use dyn_clone::DynClone;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use vector_lib::configurable::configurable_component;

use crate::event::{KeyString, LogEvent, Value};
use vector_lib::usage_metrics::value_size;

/// Strategies for merging events.
#[configurable_component]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "proptest", derive(proptest_derive::Arbitrary))]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// Discard all but the first value found.
    Discard,

    /// Discard all but the last value found.
    ///
    /// Works as a way to coalesce by not retaining `null`.
    Retain,

    /// Sum all numeric values.
    Sum,

    /// Keep the maximum numeric value seen.
    Max,

    /// Keep the minimum numeric value seen.
    Min,

    /// Append each value to an array.
    Array,

    /// Concatenate each string value, delimited with a space.
    Concat,

    /// Concatenate each string value, delimited with a newline.
    ConcatNewline,

    /// Concatenate each string, without a delimiter.
    ConcatRaw,

    /// Keep the shortest array seen.
    ShortestArray,

    /// Keep the longest array seen.
    LongestArray,

    /// Create a flattened array of all unique values.
    FlatUnique,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscardMerger {
    v: Value,
    size_estimate: usize,
}

impl DiscardMerger {
    fn new(v: Value) -> Self {
        let size_estimate = value_size(&v);
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for DiscardMerger {
    fn add(&mut self, _v: Value) -> Result<(), String> {
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), self.v);
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainMerger {
    v: Value,
    size_estimate: usize,
}

impl RetainMerger {
    #[allow(clippy::missing_const_for_fn)] // const cannot run destructor
    fn new(v: Value) -> Self {
        let size_estimate = value_size(&v);
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for RetainMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if Value::Null != v {
            self.v = v;
            self.size_estimate = value_size(&self.v);
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), self.v);
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcatMerger {
    v: BytesMut,
    join_by: Option<Vec<u8>>,
    size_estimate: usize,
}

impl ConcatMerger {
    fn new(v: Bytes, join_by: Option<char>) -> Self {
        // We need to get the resulting bytes for this character in case it's actually a multi-byte character.
        let join_by = join_by.map(|c| c.to_string().into_bytes());

        let v = BytesMut::from(&v[..]);
        let size_estimate = v.len();
        Self {
            v,
            join_by,
            size_estimate,
        }
    }
}

impl ReduceValueMerger for ConcatMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if let Value::Bytes(b) = v {
            if let Some(buf) = self.join_by.as_ref() {
                self.v.extend(&buf[..]);
            }
            self.v.extend_from_slice(&b);
            self.size_estimate = self.v.len();
            Ok(())
        } else {
            Err(format!(
                "expected string value, found: '{}'",
                v.to_string_lossy()
            ))
        }
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Bytes(self.v.into()));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcatArrayMerger {
    v: Vec<Value>,
    size_estimate: usize,
}

impl ConcatArrayMerger {
    fn new(v: Vec<Value>) -> Self {
        let size_estimate = v.iter().map(value_size).sum::<usize>();
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for ConcatArrayMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if let Value::Array(a) = v {
            self.size_estimate += a.iter().map(value_size).sum::<usize>();
            self.v.extend_from_slice(&a);
        } else {
            self.size_estimate += value_size(&v);
            self.v.push(v);
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Array(self.v));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayMerger {
    v: Vec<Value>,
    size_estimate: usize,
}

impl ArrayMerger {
    fn new(v: Value) -> Self {
        let size_estimate = value_size(&v);
        Self {
            v: vec![v],
            size_estimate,
        }
    }
}

impl ReduceValueMerger for ArrayMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        self.size_estimate += value_size(&v);
        self.v.push(v);
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Array(self.v));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongestArrayMerger {
    v: Vec<Value>,
    size_estimate: usize,
}

impl LongestArrayMerger {
    fn new(v: Vec<Value>) -> Self {
        let size_estimate = v.iter().map(value_size).sum::<usize>();
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for LongestArrayMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if let Value::Array(a) = v {
            if a.len() > self.v.len() {
                self.size_estimate = a.iter().map(value_size).sum::<usize>();
                self.v = a;
            }
            Ok(())
        } else {
            Err(format!(
                "expected array value, found: '{}'",
                v.to_string_lossy()
            ))
        }
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Array(self.v));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortestArrayMerger {
    v: Vec<Value>,
    size_estimate: usize,
}

impl ShortestArrayMerger {
    fn new(v: Vec<Value>) -> Self {
        let size_estimate = v.iter().map(value_size).sum::<usize>();
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for ShortestArrayMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if let Value::Array(a) = v {
            if a.len() < self.v.len() {
                self.size_estimate = a.iter().map(value_size).sum::<usize>();
                self.v = a;
            }
            Ok(())
        } else {
            Err(format!(
                "expected array value, found: '{}'",
                v.to_string_lossy()
            ))
        }
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Array(self.v));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[allow(clippy::mutable_key_type)]
#[derive(Debug, Clone, Serialize)]
pub struct FlatUniqueMerger {
    v: HashSet<Value>,
    size_estimate: usize,
}

// Manual Deserialize implementation to work around clippy mutable_key_type warning
impl<'de> serde::Deserialize<'de> for FlatUniqueMerger {
    #[allow(clippy::mutable_key_type)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct FlatUniqueMergerVisitor;

        impl<'de> Visitor<'de> for FlatUniqueMergerVisitor {
            type Value = FlatUniqueMerger;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct FlatUniqueMerger")
            }

            #[allow(clippy::mutable_key_type)]
            fn visit_map<V>(self, mut map: V) -> Result<FlatUniqueMerger, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut v = None;
                let mut size_estimate = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "v" => {
                            if v.is_some() {
                                return Err(de::Error::duplicate_field("v"));
                            }
                            v = Some(map.next_value()?);
                        }
                        "size_estimate" => {
                            if size_estimate.is_some() {
                                return Err(de::Error::duplicate_field("size_estimate"));
                            }
                            size_estimate = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                let v = v.ok_or_else(|| de::Error::missing_field("v"))?;
                let size_estimate =
                    size_estimate.ok_or_else(|| de::Error::missing_field("size_estimate"))?;
                Ok(FlatUniqueMerger { v, size_estimate })
            }
        }

        const FIELDS: &[&str] = &["v", "size_estimate"];
        deserializer.deserialize_struct("FlatUniqueMerger", FIELDS, FlatUniqueMergerVisitor)
    }
}

#[allow(clippy::mutable_key_type)] // false positive due to bytes::Bytes
fn insert_value(h: &mut HashSet<Value>, v: Value) -> usize {
    let mut size_estimate_delta = 0;
    // Do size estimates for every value, and if the insert succeeds, add that size to the delta.
    // Doing value_size for each value (even if not inserted) is cheaper and faster than inserting v.clone().
    match v {
        Value::Object(m) => {
            for (_, v) in m {
                let val_size = value_size(&v);
                if h.insert(v) {
                    size_estimate_delta += val_size
                }
            }
        }
        Value::Array(vec) => {
            for v in vec {
                let val_size = value_size(&v);
                if h.insert(v) {
                    size_estimate_delta += val_size
                }
            }
        }
        _ => {
            let val_size = value_size(&v);
            if h.insert(v) {
                size_estimate_delta += val_size
            }
        }
    }
    size_estimate_delta
}

impl FlatUniqueMerger {
    #[allow(clippy::mutable_key_type)] // false positive due to bytes::Bytes
    fn new(v: Value) -> Self {
        let mut h = HashSet::default();
        let size_estimate = insert_value(&mut h, v);
        Self {
            v: h,
            size_estimate,
        }
    }
}

impl ReduceValueMerger for FlatUniqueMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        self.size_estimate += insert_value(&mut self.v, v);
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(k.as_str(), Value::Array(self.v.into_iter().collect()));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampWindowMerger {
    started: DateTime<Utc>,
    latest: DateTime<Utc>,
    size_estimate: usize,
}

impl TimestampWindowMerger {
    const fn new(v: DateTime<Utc>) -> Self {
        Self {
            started: v,
            latest: v,
            size_estimate: size_of::<DateTime<Utc>>() * 2,
        }
    }
}

impl ReduceValueMerger for TimestampWindowMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        if let Value::Timestamp(ts) = v {
            self.latest = ts
        } else {
            return Err(format!(
                "expected timestamp value, found: {}",
                v.to_string_lossy()
            ));
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        v.insert(format!("{k}_end").as_str(), Value::Timestamp(self.latest));
        v.insert(k.as_str(), Value::Timestamp(self.started));
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum NumberMergerValue {
    Int(i64),
    Float(NotNan<f64>),
}

impl From<i64> for NumberMergerValue {
    fn from(v: i64) -> Self {
        NumberMergerValue::Int(v)
    }
}

impl From<NotNan<f64>> for NumberMergerValue {
    fn from(v: NotNan<f64>) -> Self {
        NumberMergerValue::Float(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNumbersMerger {
    v: NumberMergerValue,
    size_estimate: usize,
}

impl AddNumbersMerger {
    const fn new(v: NumberMergerValue) -> Self {
        let size_estimate = match v {
            NumberMergerValue::Float(_) => size_of::<NotNan<f64>>(),
            NumberMergerValue::Int(_) => size_of::<i64>(),
        };
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for AddNumbersMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        // Try and keep max precision with integer values, but once we've
        // received a float downgrade to float precision.
        match v {
            Value::Integer(i) => match self.v {
                NumberMergerValue::Int(j) => self.v = NumberMergerValue::Int(i + j),
                NumberMergerValue::Float(j) => {
                    self.v = NumberMergerValue::Float(NotNan::new(i as f64).unwrap() + j)
                }
            },
            Value::Float(f) => match self.v {
                NumberMergerValue::Int(j) => self.v = NumberMergerValue::Float(f + j as f64),
                NumberMergerValue::Float(j) => self.v = NumberMergerValue::Float(f + j),
            },
            _ => {
                return Err(format!(
                    "expected numeric value, found: '{}'",
                    v.to_string_lossy()
                ));
            }
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        match self.v {
            NumberMergerValue::Float(f) => v.insert(k.as_str(), Value::Float(f)),
            NumberMergerValue::Int(i) => v.insert(k.as_str(), Value::Integer(i)),
        };
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxNumberMerger {
    v: NumberMergerValue,
    size_estimate: usize,
}

impl MaxNumberMerger {
    const fn new(v: NumberMergerValue) -> Self {
        let size_estimate = match v {
            NumberMergerValue::Float(_) => size_of::<NotNan<f64>>(),
            NumberMergerValue::Int(_) => size_of::<i64>(),
        };
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for MaxNumberMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        // Try and keep max precision with integer values, but once we've
        // received a float downgrade to float precision.
        match v {
            Value::Integer(i) => {
                match self.v {
                    NumberMergerValue::Int(i2) => {
                        if i > i2 {
                            self.v = NumberMergerValue::Int(i);
                        }
                    }
                    NumberMergerValue::Float(f2) => {
                        let f = NotNan::new(i as f64).unwrap();
                        if f > f2 {
                            self.v = NumberMergerValue::Float(f);
                        }
                    }
                };
            }
            Value::Float(f) => {
                let f2 = match self.v {
                    NumberMergerValue::Int(i2) => NotNan::new(i2 as f64).unwrap(),
                    NumberMergerValue::Float(f2) => f2,
                };
                if f > f2 {
                    self.v = NumberMergerValue::Float(f);
                }
            }
            _ => {
                return Err(format!(
                    "expected numeric value, found: '{}'",
                    v.to_string_lossy()
                ));
            }
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        match self.v {
            NumberMergerValue::Float(f) => v.insert(k.as_str(), Value::Float(f)),
            NumberMergerValue::Int(i) => v.insert(k.as_str(), Value::Integer(i)),
        };
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinNumberMerger {
    v: NumberMergerValue,
    size_estimate: usize,
}

impl MinNumberMerger {
    const fn new(v: NumberMergerValue) -> Self {
        let size_estimate = match v {
            NumberMergerValue::Float(_) => size_of::<NotNan<f64>>(),
            NumberMergerValue::Int(_) => size_of::<i64>(),
        };
        Self { v, size_estimate }
    }
}

impl ReduceValueMerger for MinNumberMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        // Try and keep max precision with integer values, but once we've
        // received a float downgrade to float precision.
        match v {
            Value::Integer(i) => {
                match self.v {
                    NumberMergerValue::Int(i2) => {
                        if i < i2 {
                            self.v = NumberMergerValue::Int(i);
                        }
                    }
                    NumberMergerValue::Float(f2) => {
                        let f = NotNan::new(i as f64).unwrap();
                        if f < f2 {
                            self.v = NumberMergerValue::Float(f);
                        }
                    }
                };
            }
            Value::Float(f) => {
                let f2 = match self.v {
                    NumberMergerValue::Int(i2) => NotNan::new(i2 as f64).unwrap(),
                    NumberMergerValue::Float(f2) => f2,
                };
                if f < f2 {
                    self.v = NumberMergerValue::Float(f);
                }
            }
            _ => {
                return Err(format!(
                    "expected numeric value, found: '{}'",
                    v.to_string_lossy()
                ));
            }
        }
        Ok(())
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        match self.v {
            NumberMergerValue::Float(f) => v.insert(k.as_str(), Value::Float(f)),
            NumberMergerValue::Int(i) => v.insert(k.as_str(), Value::Integer(i)),
        };
        Ok(())
    }

    fn size_estimate(&self) -> usize {
        self.size_estimate
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub trait ReduceValueMerger: std::fmt::Debug + Send + Sync + DynClone {
    fn add(&mut self, v: Value) -> Result<(), String>;
    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String>;
    fn size_estimate(&self) -> usize;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Serializable wrapper for ReduceValueMerger trait objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableReduceValueMerger {
    Discard(DiscardMerger),
    Retain(RetainMerger),
    Concat(ConcatMerger),
    ConcatArray(ConcatArrayMerger),
    Array(ArrayMerger),
    LongestArray(LongestArrayMerger),
    ShortestArray(ShortestArrayMerger),
    FlatUnique(FlatUniqueMerger),
    TimestampWindow(TimestampWindowMerger),
    AddNumbers(AddNumbersMerger),
    MaxNumber(MaxNumberMerger),
    MinNumber(MinNumberMerger),
}

impl SerializableReduceValueMerger {
    /// Convert to boxed trait object
    pub fn into_boxed_merger(self) -> Box<dyn ReduceValueMerger> {
        match self {
            SerializableReduceValueMerger::Discard(m) => Box::new(m),
            SerializableReduceValueMerger::Retain(m) => Box::new(m),
            SerializableReduceValueMerger::Concat(m) => Box::new(m),
            SerializableReduceValueMerger::ConcatArray(m) => Box::new(m),
            SerializableReduceValueMerger::Array(m) => Box::new(m),
            SerializableReduceValueMerger::LongestArray(m) => Box::new(m),
            SerializableReduceValueMerger::ShortestArray(m) => Box::new(m),
            SerializableReduceValueMerger::FlatUnique(m) => Box::new(m),
            SerializableReduceValueMerger::TimestampWindow(m) => Box::new(m),
            SerializableReduceValueMerger::AddNumbers(m) => Box::new(m),
            SerializableReduceValueMerger::MaxNumber(m) => Box::new(m),
            SerializableReduceValueMerger::MinNumber(m) => Box::new(m),
        }
    }

    /// Convert from boxed trait object
    pub fn from_boxed_merger(merger: Box<dyn ReduceValueMerger>) -> Option<Self> {
        // Use the as_any method for downcasting
        let any_ref = merger.as_any();

        if let Some(m) = any_ref.downcast_ref::<DiscardMerger>() {
            Some(SerializableReduceValueMerger::Discard(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<RetainMerger>() {
            Some(SerializableReduceValueMerger::Retain(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ConcatMerger>() {
            Some(SerializableReduceValueMerger::Concat(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ConcatArrayMerger>() {
            Some(SerializableReduceValueMerger::ConcatArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ArrayMerger>() {
            Some(SerializableReduceValueMerger::Array(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<LongestArrayMerger>() {
            Some(SerializableReduceValueMerger::LongestArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ShortestArrayMerger>() {
            Some(SerializableReduceValueMerger::ShortestArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<FlatUniqueMerger>() {
            Some(SerializableReduceValueMerger::FlatUnique(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<TimestampWindowMerger>() {
            Some(SerializableReduceValueMerger::TimestampWindow(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<AddNumbersMerger>() {
            Some(SerializableReduceValueMerger::AddNumbers(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<MaxNumberMerger>() {
            Some(SerializableReduceValueMerger::MaxNumber(m.clone()))
        } else {
            any_ref
                .downcast_ref::<MinNumberMerger>()
                .map(|m| SerializableReduceValueMerger::MinNumber(m.clone()))
        }
    }

    /// Convert from Any reference (for easier downcasting)
    pub fn from_any_ref(any_ref: &dyn std::any::Any) -> Option<Self> {
        if let Some(m) = any_ref.downcast_ref::<DiscardMerger>() {
            Some(SerializableReduceValueMerger::Discard(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<RetainMerger>() {
            Some(SerializableReduceValueMerger::Retain(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ConcatMerger>() {
            Some(SerializableReduceValueMerger::Concat(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ConcatArrayMerger>() {
            Some(SerializableReduceValueMerger::ConcatArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ArrayMerger>() {
            Some(SerializableReduceValueMerger::Array(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<LongestArrayMerger>() {
            Some(SerializableReduceValueMerger::LongestArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<ShortestArrayMerger>() {
            Some(SerializableReduceValueMerger::ShortestArray(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<FlatUniqueMerger>() {
            Some(SerializableReduceValueMerger::FlatUnique(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<TimestampWindowMerger>() {
            Some(SerializableReduceValueMerger::TimestampWindow(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<AddNumbersMerger>() {
            Some(SerializableReduceValueMerger::AddNumbers(m.clone()))
        } else if let Some(m) = any_ref.downcast_ref::<MaxNumberMerger>() {
            Some(SerializableReduceValueMerger::MaxNumber(m.clone()))
        } else {
            any_ref
                .downcast_ref::<MinNumberMerger>()
                .map(|m| SerializableReduceValueMerger::MinNumber(m.clone()))
        }
    }
}

impl ReduceValueMerger for SerializableReduceValueMerger {
    fn add(&mut self, v: Value) -> Result<(), String> {
        match self {
            SerializableReduceValueMerger::Discard(m) => m.add(v),
            SerializableReduceValueMerger::Retain(m) => m.add(v),
            SerializableReduceValueMerger::Concat(m) => m.add(v),
            SerializableReduceValueMerger::ConcatArray(m) => m.add(v),
            SerializableReduceValueMerger::Array(m) => m.add(v),
            SerializableReduceValueMerger::LongestArray(m) => m.add(v),
            SerializableReduceValueMerger::ShortestArray(m) => m.add(v),
            SerializableReduceValueMerger::FlatUnique(m) => m.add(v),
            SerializableReduceValueMerger::TimestampWindow(m) => m.add(v),
            SerializableReduceValueMerger::AddNumbers(m) => m.add(v),
            SerializableReduceValueMerger::MaxNumber(m) => m.add(v),
            SerializableReduceValueMerger::MinNumber(m) => m.add(v),
        }
    }

    fn insert_into(self: Box<Self>, k: KeyString, v: &mut LogEvent) -> Result<(), String> {
        match *self {
            SerializableReduceValueMerger::Discard(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::Retain(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::Concat(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::ConcatArray(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::Array(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::LongestArray(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::ShortestArray(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::FlatUnique(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::TimestampWindow(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::AddNumbers(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::MaxNumber(m) => Box::new(m).insert_into(k, v),
            SerializableReduceValueMerger::MinNumber(m) => Box::new(m).insert_into(k, v),
        }
    }

    fn size_estimate(&self) -> usize {
        match self {
            SerializableReduceValueMerger::Discard(m) => m.size_estimate(),
            SerializableReduceValueMerger::Retain(m) => m.size_estimate(),
            SerializableReduceValueMerger::Concat(m) => m.size_estimate(),
            SerializableReduceValueMerger::ConcatArray(m) => m.size_estimate(),
            SerializableReduceValueMerger::Array(m) => m.size_estimate(),
            SerializableReduceValueMerger::LongestArray(m) => m.size_estimate(),
            SerializableReduceValueMerger::ShortestArray(m) => m.size_estimate(),
            SerializableReduceValueMerger::FlatUnique(m) => m.size_estimate(),
            SerializableReduceValueMerger::TimestampWindow(m) => m.size_estimate(),
            SerializableReduceValueMerger::AddNumbers(m) => m.size_estimate(),
            SerializableReduceValueMerger::MaxNumber(m) => m.size_estimate(),
            SerializableReduceValueMerger::MinNumber(m) => m.size_estimate(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

dyn_clone::clone_trait_object!(ReduceValueMerger);

impl From<Value> for Box<dyn ReduceValueMerger> {
    fn from(v: Value) -> Self {
        match v {
            Value::Integer(i) => Box::new(AddNumbersMerger::new(i.into())),
            Value::Float(f) => Box::new(AddNumbersMerger::new(f.into())),
            Value::Timestamp(ts) => Box::new(TimestampWindowMerger::new(ts)),
            Value::Object(_) => Box::new(DiscardMerger::new(v)),
            Value::Null => Box::new(DiscardMerger::new(v)),
            Value::Boolean(_) => Box::new(DiscardMerger::new(v)),
            Value::Bytes(_) => Box::new(DiscardMerger::new(v)),
            Value::Regex(_) => Box::new(DiscardMerger::new(v)),
            Value::Array(_) => Box::new(DiscardMerger::new(v)),
        }
    }
}

pub(crate) fn get_value_merger(
    v: Value,
    m: &MergeStrategy,
) -> Result<Box<dyn ReduceValueMerger>, String> {
    match m {
        MergeStrategy::Sum => match v {
            Value::Integer(i) => Ok(Box::new(AddNumbersMerger::new(i.into()))),
            Value::Float(f) => Ok(Box::new(AddNumbersMerger::new(f.into()))),
            _ => Err(format!(
                "expected number value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::Max => match v {
            Value::Integer(i) => Ok(Box::new(MaxNumberMerger::new(i.into()))),
            Value::Float(f) => Ok(Box::new(MaxNumberMerger::new(f.into()))),
            _ => Err(format!(
                "expected number value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::Min => match v {
            Value::Integer(i) => Ok(Box::new(MinNumberMerger::new(i.into()))),
            Value::Float(f) => Ok(Box::new(MinNumberMerger::new(f.into()))),
            _ => Err(format!(
                "expected number value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::Concat => match v {
            Value::Bytes(b) => Ok(Box::new(ConcatMerger::new(b, Some(' ')))),
            Value::Array(a) => Ok(Box::new(ConcatArrayMerger::new(a))),
            _ => Err(format!(
                "expected string or array value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::ConcatNewline => match v {
            Value::Bytes(b) => Ok(Box::new(ConcatMerger::new(b, Some('\n')))),
            _ => Err(format!(
                "expected string value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::ConcatRaw => match v {
            Value::Bytes(b) => Ok(Box::new(ConcatMerger::new(b, None))),
            _ => Err(format!(
                "expected string value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::Array => Ok(Box::new(ArrayMerger::new(v))),
        MergeStrategy::ShortestArray => match v {
            Value::Array(a) => Ok(Box::new(ShortestArrayMerger::new(a))),
            _ => Err(format!(
                "expected array value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::LongestArray => match v {
            Value::Array(a) => Ok(Box::new(LongestArrayMerger::new(a))),
            _ => Err(format!(
                "expected array value, found: '{}'",
                v.to_string_lossy()
            )),
        },
        MergeStrategy::Discard => Ok(Box::new(DiscardMerger::new(v))),
        MergeStrategy::Retain => Ok(Box::new(RetainMerger::new(v))),
        MergeStrategy::FlatUnique => Ok(Box::new(FlatUniqueMerger::new(v))),
    }
}

#[cfg(test)]
mod test {
    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::event::{LogEvent, Value};

    #[test]
    fn initial_values() {
        assert!(get_value_merger("foo".into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger("foo".into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger("foo".into(), &MergeStrategy::Concat).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::ConcatNewline).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::ConcatRaw).is_ok());
        assert!(get_value_merger("foo".into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(42.into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Sum).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Min).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Max).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(42.into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(42.into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(42.into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(42.into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(42.into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(42.into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(42.into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::Sum).is_ok());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::Min).is_ok());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::Max).is_ok());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(4.2.into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(true.into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(true.into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(true.into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(true.into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(true.into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(Utc::now().into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::LongestArray).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::ShortestArray).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::Concat).is_ok());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(json!([]).into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(json!({}).into(), &MergeStrategy::FlatUnique).is_ok());

        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Discard).is_ok());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Retain).is_ok());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Sum).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Max).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Min).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Array).is_ok());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::LongestArray).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::ShortestArray).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::Concat).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::ConcatNewline).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::ConcatRaw).is_err());
        assert!(get_value_merger(json!(null).into(), &MergeStrategy::FlatUnique).is_ok());
    }

    #[test]
    fn merging_values() {
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::Discard),
            Ok("foo".into())
        );
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::Retain),
            Ok("bar".into())
        );
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::Array),
            Ok(json!(["foo", "bar"]).into())
        );
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::Concat),
            Ok("foo bar".into())
        );
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::ConcatNewline),
            Ok("foo\nbar".into())
        );
        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::ConcatRaw),
            Ok("foobar".into())
        );
        assert!(merge("foo".into(), 42.into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), 4.2.into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), true.into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), Utc::now().into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), json!({}).into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), json!([]).into(), &MergeStrategy::Concat).is_err());
        assert!(merge("foo".into(), json!(null).into(), &MergeStrategy::Concat).is_err());

        assert_eq!(
            merge("foo".into(), "bar".into(), &MergeStrategy::ConcatNewline),
            Ok("foo\nbar".into())
        );

        assert_eq!(
            merge(21.into(), 21.into(), &MergeStrategy::Sum),
            Ok(42.into())
        );
        assert_eq!(
            merge(41.into(), 42.into(), &MergeStrategy::Max),
            Ok(42.into())
        );
        assert_eq!(
            merge(42.into(), 41.into(), &MergeStrategy::Max),
            Ok(42.into())
        );
        assert_eq!(
            merge(42.into(), 43.into(), &MergeStrategy::Min),
            Ok(42.into())
        );
        assert_eq!(
            merge(43.into(), 42.into(), &MergeStrategy::Min),
            Ok(42.into())
        );

        assert_eq!(
            merge(2.1.into(), 2.1.into(), &MergeStrategy::Sum),
            Ok(4.2.into())
        );
        assert_eq!(
            merge(4.1.into(), 4.2.into(), &MergeStrategy::Max),
            Ok(4.2.into())
        );
        assert_eq!(
            merge(4.2.into(), 4.1.into(), &MergeStrategy::Max),
            Ok(4.2.into())
        );
        assert_eq!(
            merge(4.2.into(), 4.3.into(), &MergeStrategy::Min),
            Ok(4.2.into())
        );
        assert_eq!(
            merge(4.3.into(), 4.2.into(), &MergeStrategy::Min),
            Ok(4.2.into())
        );

        assert_eq!(
            merge(
                json!([4_i64]).into(),
                json!([2_i64]).into(),
                &MergeStrategy::Concat
            ),
            Ok(json!([4_i64, 2_i64]).into())
        );
        assert_eq!(
            merge(json!([]).into(), 42_i64.into(), &MergeStrategy::Concat),
            Ok(json!([42_i64]).into())
        );

        assert_eq!(
            merge(
                json!([34_i64]).into(),
                json!([42_i64, 43_i64]).into(),
                &MergeStrategy::ShortestArray
            ),
            Ok(json!([34_i64]).into())
        );
        assert_eq!(
            merge(
                json!([34_i64]).into(),
                json!([42_i64, 43_i64]).into(),
                &MergeStrategy::LongestArray
            ),
            Ok(json!([42_i64, 43_i64]).into())
        );

        let v = merge(34_i64.into(), 43_i64.into(), &MergeStrategy::FlatUnique).unwrap();
        match v.clone() {
            Value::Array(v) => {
                let v: Vec<_> = v
                    .into_iter()
                    .map(|i| {
                        if let Value::Integer(i) = i {
                            i
                        } else {
                            panic!("Bad value");
                        }
                    })
                    .collect();
                assert_eq!(v.iter().filter(|i| **i == 34i64).count(), 1);
                assert_eq!(v.iter().filter(|i| **i == 43i64).count(), 1);
            }
            _ => {
                panic!("Not array");
            }
        }
        let v = merge(v, 34_i32.into(), &MergeStrategy::FlatUnique).unwrap();
        if let Value::Array(v) = v {
            let v: Vec<_> = v
                .into_iter()
                .map(|i| {
                    if let Value::Integer(i) = i {
                        i
                    } else {
                        panic!("Bad value");
                    }
                })
                .collect();
            assert_eq!(v.iter().filter(|i| **i == 34i64).count(), 1);
            assert_eq!(v.iter().filter(|i| **i == 43i64).count(), 1);
        } else {
            panic!("Not array");
        }
    }

    fn merge(initial: Value, additional: Value, strategy: &MergeStrategy) -> Result<Value, String> {
        let mut merger = get_value_merger(initial, strategy)?;
        merger.add(additional)?;
        let mut output = LogEvent::default();
        merger.insert_into("out".into(), &mut output)?;
        Ok(output.remove("out").unwrap())
    }

    #[test]
    fn mezmo_sum_size_estimate() {
        let mut m = get_value_merger(3587.into(), &MergeStrategy::Sum).unwrap();
        assert_eq!(m.size_estimate(), 8, "size of int is correct");

        m.add(2.1.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added float, but size remains unchanged"
        );

        m.add(655360.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added int; but size remains unchanged"
        );
    }

    #[test]
    fn mezmo_max_size_estimate() {
        let mut m = get_value_merger(3587.into(), &MergeStrategy::Max).unwrap();
        assert_eq!(m.size_estimate(), 8, "size of int is correct");

        m.add(3588.8.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added float, but size remains unchanged"
        );

        m.add(655360.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added int; but size remains unchanged"
        );
    }

    #[test]
    fn mezmo_min_size_estimate() {
        let mut m = get_value_merger(3587.into(), &MergeStrategy::Min).unwrap();
        assert_eq!(m.size_estimate(), 8, "size of int is correct");

        m.add(2.1.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added float, but size remains unchanged"
        );

        m.add(1.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            8,
            "added int; but size remains unchanged"
        );
    }

    #[test]
    fn mezmo_concat_size_estimate() {
        let mut m = get_value_merger("hello".into(), &MergeStrategy::Concat).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");
        m.add("somethinglonger".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            21,
            "value updated; concats the longer value, space-delimited"
        );

        let mut m = get_value_merger(
            vec![Value::from("hello"), Value::from("there")].into(),
            &MergeStrategy::Concat,
        )
        .unwrap();
        assert_eq!(m.size_estimate(), 10, "size of array elements is correct");
        m.add("onemore".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            17,
            "value updated; concats the added value"
        );

        let mut m = get_value_merger(json!([1, 2, 3]).into(), &MergeStrategy::Concat).unwrap();
        assert_eq!(
            m.size_estimate(),
            24,
            "size of integer array elements is correct"
        );
        m.add(4.into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            32,
            "value updated; concats the added value"
        );
    }

    #[test]
    fn mezmo_concat_newline_size_estimate() {
        let mut m = get_value_merger("hello".into(), &MergeStrategy::ConcatNewline).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");

        m.add("line2".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            11,
            "value updated; concats the second value, newline-delimited"
        );

        m.add("line3".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            17,
            "value updated; concats the third value, newline-delimited"
        );
    }

    #[test]
    fn mezmo_concat_raw_size_estimate() {
        let mut m = get_value_merger("hello".into(), &MergeStrategy::ConcatRaw).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");

        m.add("line2".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            10,
            "value updated; concats the second value WITHOUT a delimiter"
        );

        m.add("line3".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            15,
            "value updated; concats the third value, no delimiter"
        );
    }

    #[test]
    fn mezmo_array_size_estimate() {
        let mut m = get_value_merger("hello".into(), &MergeStrategy::Array).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");

        m.add("second".into()).unwrap();
        assert_eq!(m.size_estimate(), 11, "Concats a string to the array");

        m.add(4.into()).unwrap();
        assert_eq!(m.size_estimate(), 19, "Concats an integer to the array");

        m.add(5.12.into()).unwrap();
        assert_eq!(m.size_estimate(), 27, "Concats an float to the array");
    }

    #[test]
    fn mezmo_shortest_array_size_estimate() {
        let mut m =
            get_value_merger(json!([1, 2, 3, 4]).into(), &MergeStrategy::ShortestArray).unwrap();
        assert_eq!(
            m.size_estimate(),
            32,
            "size of the initial array is correct"
        );

        m.add(json!([1, 2, 3, 4, 5]).into()).unwrap();
        assert_eq!(m.size_estimate(), 32, "Longer array is ignored");

        m.add(json!([1, 2, 3]).into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            24,
            "Shorter array overwrites the original array"
        );

        m.add(json!(["something really big", "something else really big"]).into())
            .unwrap();
        assert_eq!(
            m.size_estimate(),
            45,
            "Shorter array overwrites, even though its size is greater"
        );
    }

    #[test]
    fn mezmo_longest_array_size_estimate() {
        let mut m = get_value_merger(json!([1, 2]).into(), &MergeStrategy::LongestArray).unwrap();
        assert_eq!(
            m.size_estimate(),
            16,
            "size of the initial array is correct"
        );

        m.add(json!([1, 2, 3]).into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            24,
            "Longer array overwrites the original array"
        );

        m.add(json!([1, 2]).into()).unwrap();
        assert_eq!(m.size_estimate(), 24, "Shorter array is ignored");

        m.add(
            json!([
                "something really big",      // len: 20
                "something else really big", // len: 25
                "boo",                       // len: 3
                "hiss",                      // len: 4
                3587                         // len: 8
            ])
            .into(),
        )
        .unwrap();
        assert_eq!(
            m.size_estimate(),
            60,
            "Longer array overwrites with mixed data types"
        );
    }

    #[test]
    fn mezmo_discard_size_estimate() {
        let m = get_value_merger(35.into(), &MergeStrategy::Discard).unwrap();
        assert_eq!(m.size_estimate(), 8, "size of integer is correct");

        let mut m = get_value_merger("hello".into(), &MergeStrategy::Discard).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");
        m.add("somethinglonger".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            5,
            "value unchanged; discarded longer value"
        );

        // NOTE: Because this ValueMerger doesn't need to unwrap arrays, its size will also include
        // the `BASE_ARRAY_SIZE` of 8 bytes when doing `size_of(Value::Array)`
        let mut m = get_value_merger(
            vec![Value::from("hello"), Value::from(35)].into(),
            &MergeStrategy::Discard,
        )
        .unwrap();
        assert_eq!(
            m.size_estimate(),
            21,
            "size of array elements is correct; includes BASE_ARRAY_SIZE"
        );
        m.add("short".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            21,
            "value unchanged; discarded shorter value"
        );
    }

    #[test]
    fn mezmo_retain_size_estimate() {
        let m = get_value_merger(35.into(), &MergeStrategy::Retain).unwrap();
        assert_eq!(m.size_estimate(), 8, "size of integer is correct");

        let mut m = get_value_merger("hello".into(), &MergeStrategy::Retain).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of string is correct");
        m.add("somethinglonger".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            15,
            "value updated; retains the longer value"
        );

        let mut m = get_value_merger(
            vec![Value::from("hello"), Value::from(5)].into(),
            &MergeStrategy::Retain,
        )
        .unwrap();
        assert_eq!(m.size_estimate(), 21, "size of array elements is correct");
        m.add("short".into()).unwrap();
        assert_eq!(
            m.size_estimate(),
            5,
            "value updated; retains the shorter value"
        );
    }

    #[test]
    fn mezmo_flat_unique_size_estimate() {
        let mut m = get_value_merger("first".into(), &MergeStrategy::FlatUnique).unwrap();
        assert_eq!(m.size_estimate(), 5, "size of the initial set is correct");

        m.add("first".into()).unwrap();
        assert_eq!(m.size_estimate(), 5, "Duplicate value ignored");

        m.add(json!(["second", "third"]).into()).unwrap();
        assert_eq!(m.size_estimate(), 16, "New values added via an array");

        m.add(json!({"key1": "second", "key2": "third", "key3": "fourth", "key4": "fifth"}).into())
            .unwrap();
        assert_eq!(
            m.size_estimate(),
            27,
            "New values added via object; Ignores duplicate values alread in the set"
        );
    }

    #[test]
    fn mezmo_datetime_size_estimate() {
        let m = TimestampWindowMerger::new(Utc::now());
        assert_eq!(m.size_estimate(), 24, "size includes 2 timestamps");
    }
}
