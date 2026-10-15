//! Conversion of the native OpenTelemetry metric envelope into a Vector metric.
//!
//! ingestion-service writes one log event per OTLP data point:
//!
//! ```text
//! message: {resource: {attributes}, scope: {name?, version?, attributes}, type: "metric",
//!           record: {metric: {name, description?, unit?},
//!                    dataPoint: {type, attributes, timeUnixNano, startTimeUnixNano?, flags?,
//!                                ...type specific fields..., exemplars?,
//!                                droppedAttributesCount}}}
//! ```
//!
//! int64 and uint64 fields arrive as strings because JSON numbers cannot hold them exactly.
//!
//! The `arbitrary` layout matches the one written by the OTLP codec
//! (`lib/codecs/src/decoding/format/mezmo/open_telemetry/metric_parser.rs`), so the Mezmo
//! OpenTelemetry sink reads metrics from either origin the same way.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use ordered_float::NotNan;

use crate::{
    config::log_schema,
    event::{
        KeyString, LogEvent, Value,
        metric::{
            Bucket, Metric, MetricArbitrary, MetricData, MetricKind, MetricName, MetricSeries,
            MetricTags, MetricTime, MetricValue, Quantile, mezmo::from_f64_or_zero,
        },
    },
};

use super::TransformError;
use super::exponential::{self, ExponentialBuckets, ExponentialHistogram};
use super::vector::event_timestamp;

const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MILLI: u64 = 1_000_000;
const MAX_EXACT_F64_INTEGER: u64 = 1 << 53;
const FLAG_NO_RECORDED_VALUE: u64 = 1;

const TEMPORALITY_DELTA: i64 = 1;
const TEMPORALITY_CUMULATIVE: i64 = 2;
const TEMPORALITY_DELTA_NAME: &str = "AGGREGATION_TEMPORALITY_DELTA";
const TEMPORALITY_CUMULATIVE_NAME: &str = "AGGREGATION_TEMPORALITY_CUMULATIVE";

type Object = BTreeMap<KeyString, Value>;

const SUM: &str = "sum";
const START_TIME: &str = "start_time_unix";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Temporality {
    Delta,
    Cumulative,
}

impl Temporality {
    const fn kind(self) -> MetricKind {
        match self {
            Temporality::Delta => MetricKind::Incremental,
            Temporality::Cumulative => MetricKind::Absolute,
        }
    }

    const fn as_otlp(self) -> i64 {
        match self {
            Temporality::Delta => TEMPORALITY_DELTA,
            Temporality::Cumulative => TEMPORALITY_CUMULATIVE,
        }
    }
}

struct Converted {
    value: MetricValue,
    kind: MetricKind,
    temporality: Option<Temporality>,
    original_type: &'static str,
}

/// An object in the envelope, along with its path from the message root so errors can name
/// the exact field a user's pipeline removed or rewrote.
struct Fields<'a> {
    map: &'a Object,
    path: String,
}

impl<'a> Fields<'a> {
    fn path_of(&self, name: &str) -> String {
        if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{}.{name}", self.path)
        }
    }

    fn optional(&self, name: &str) -> Option<&'a Value> {
        self.map
            .get(name)
            .filter(|value| !matches!(value, Value::Null))
    }

    fn required(&self, name: &str) -> Result<&'a Value, TransformError> {
        match self.map.get(name) {
            None => Err(TransformError::FieldNotFound {
                field: self.path_of(name),
            }),
            Some(Value::Null) => Err(TransformError::FieldNull {
                field: self.path_of(name),
            }),
            Some(value) => Ok(value),
        }
    }

    fn required_object(&self, name: &str) -> Result<Fields<'a>, TransformError> {
        as_fields(self.required(name)?, self.path_of(name))
    }

    fn optional_object(&self, name: &str) -> Result<Option<Fields<'a>>, TransformError> {
        self.optional(name)
            .map(|value| as_fields(value, self.path_of(name)))
            .transpose()
    }

    fn required_string(&self, name: &str) -> Result<String, TransformError> {
        parse_string(self.required(name)?, self.path_of(name))
    }

    fn optional_string(&self, name: &str) -> Result<Option<String>, TransformError> {
        self.optional(name)
            .map(|value| parse_string(value, self.path_of(name)))
            .transpose()
    }

    fn optional_bool(&self, name: &str) -> Result<Option<bool>, TransformError> {
        self.optional(name)
            .map(|value| {
                value
                    .as_boolean()
                    .ok_or_else(|| TransformError::FieldInvalidType {
                        field: self.path_of(name),
                    })
            })
            .transpose()
    }

    fn required_u64(&self, name: &str) -> Result<u64, TransformError> {
        parse_u64(self.required(name)?, self.path_of(name))
    }

    fn optional_u64(&self, name: &str) -> Result<Option<u64>, TransformError> {
        self.optional(name)
            .map(|value| parse_u64(value, self.path_of(name)))
            .transpose()
    }

    fn optional_i64(&self, name: &str) -> Result<Option<i64>, TransformError> {
        self.optional(name)
            .map(|value| parse_i64(value, self.path_of(name)))
            .transpose()
    }

    fn required_f64(&self, name: &str) -> Result<f64, TransformError> {
        parse_f64(self.required(name)?, self.path_of(name))
    }

    fn optional_f64(&self, name: &str) -> Result<Option<f64>, TransformError> {
        self.optional(name)
            .map(|value| parse_f64(value, self.path_of(name)))
            .transpose()
    }

    /// Absent arrays are empty: ingestion-service omits empty repeated fields.
    fn array<T>(
        &self,
        name: &str,
        parse: impl Fn(&'a Value, String) -> Result<T, TransformError>,
    ) -> Result<Vec<T>, TransformError> {
        let Some(value) = self.optional(name) else {
            return Ok(Vec::new());
        };
        let path = self.path_of(name);
        value
            .as_array()
            .ok_or_else(|| TransformError::FieldInvalidType {
                field: path.clone(),
            })?
            .iter()
            .enumerate()
            .map(|(index, item)| parse(item, format!("{path}[{index}]")))
            .collect()
    }

    fn cloned_or_empty_object(&self, name: &str) -> Value {
        self.optional(name)
            .cloned()
            .unwrap_or_else(|| Value::Object(Object::new()))
    }
}

fn as_fields(value: &Value, path: String) -> Result<Fields<'_>, TransformError> {
    match value.as_object() {
        Some(map) => Ok(Fields { map, path }),
        None => Err(TransformError::FieldInvalidType { field: path }),
    }
}

fn parse_string(value: &Value, field: String) -> Result<String, TransformError> {
    value
        .as_str()
        .map(Cow::into_owned)
        .ok_or(TransformError::FieldInvalidType { field })
}

fn parse_u64(value: &Value, field: String) -> Result<u64, TransformError> {
    match value {
        Value::Integer(integer) => {
            u64::try_from(*integer).map_err(|_| TransformError::ParseIntOverflow { field })
        }
        Value::Bytes(_) => parse_string(value, field.clone())?
            .parse::<u64>()
            .map_err(|_| TransformError::ParseIntOverflow { field }),
        _ => Err(TransformError::FieldInvalidType { field }),
    }
}

fn parse_i64(value: &Value, field: String) -> Result<i64, TransformError> {
    match value {
        Value::Integer(integer) => Ok(*integer),
        Value::Bytes(_) => parse_string(value, field.clone())?
            .parse::<i64>()
            .map_err(|_| TransformError::FieldInvalidValue {
                field,
                reason: "expected an integer".into(),
            }),
        _ => Err(TransformError::FieldInvalidType { field }),
    }
}

/// An OTLP double. JSON writes a whole double without a fraction, so it arrives as an integer
/// that is exact even beyond 2^53, and OTLP/JSON writes non-finite doubles as "NaN", "Infinity"
/// and "-Infinity".
fn parse_f64(value: &Value, field: String) -> Result<f64, TransformError> {
    match value {
        Value::Float(float) => Ok(float.into_inner()),
        #[allow(clippy::cast_precision_loss)]
        Value::Integer(integer) => Ok(*integer as f64),
        Value::Bytes(_) => parse_string(value, field.clone())?
            .parse::<f64>()
            .map_err(|_| TransformError::FieldInvalidValue {
                field,
                reason: "expected a number".into(),
            }),
        _ => Err(TransformError::FieldInvalidType { field }),
    }
}

#[derive(Clone, Copy)]
enum ValueType {
    Int,
    Double,
}

/// ingestion-service collapses the `value` oneof of number data points and exemplars into `value`
/// and `valueType`, writing an int64 as a string and a double as a JSON number.
fn value_type(fields: &Fields<'_>, value: &Value) -> Result<ValueType, TransformError> {
    match fields.optional_string("valueType")?.as_deref() {
        Some("int") => Ok(ValueType::Int),
        Some("double") => Ok(ValueType::Double),
        Some(_) => Err(TransformError::FieldInvalidValue {
            field: fields.path_of("valueType"),
            reason: "expected \"int\" or \"double\"".into(),
        }),
        // Without a type, any whole number is read as an int so precision loss is not silent.
        None => Ok(match value {
            Value::Integer(_) => ValueType::Int,
            Value::Bytes(_)
                if value
                    .as_str()
                    .is_some_and(|text| text.parse::<i64>().is_ok()) =>
            {
                ValueType::Int
            }
            _ => ValueType::Double,
        }),
    }
}

/// Vector holds every gauge and counter as a double, so an int beyond 2^53 is rejected rather
/// than rounded.
fn required_number(fields: &Fields<'_>) -> Result<f64, TransformError> {
    let value = fields.required("value")?;
    let field = fields.path_of("value");
    match value_type(fields, value)? {
        ValueType::Int => integer_to_f64(parse_i64(value, field.clone())?, field),
        ValueType::Double => parse_f64(value, field),
    }
}

fn integer_to_f64(integer: i64, field: String) -> Result<f64, TransformError> {
    if integer.unsigned_abs() > MAX_EXACT_F64_INTEGER {
        return Err(TransformError::NumberTruncation { field });
    }
    #[allow(clippy::cast_precision_loss)]
    Ok(integer as f64)
}

fn nanos_to_datetime(nanos: u64, field: String) -> Result<DateTime<Utc>, TransformError> {
    #[allow(clippy::cast_possible_truncation)]
    let subsec_nanos = (nanos % NANOS_PER_SECOND) as u32;
    i64::try_from(nanos / NANOS_PER_SECOND)
        .ok()
        .and_then(|seconds| DateTime::from_timestamp(seconds, subsec_nanos))
        .ok_or_else(|| TransformError::FieldInvalidValue {
            field,
            reason: "timestamp is out of range".into(),
        })
}

fn message(log: &LogEvent) -> Option<&Value> {
    log.get(log_schema().message_key_target_path()?)
}

/// The OpenTelemetry context a metric converted from OTLP carries in `arbitrary`, for sinks that
/// can place it somewhere the `Metric` itself cannot.
pub struct OtelMetricContext<'a> {
    pub scope_name: Option<Cow<'a, str>>,
    pub scope_version: Option<Cow<'a, str>>,
    pub scope_attributes: Option<&'a Object>,
    pub data_point_attributes: Option<&'a Object>,
    pub unit: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
}

/// Returns `None` unless the metric came from OTLP, either through this conversion or the OTLP
/// codec, which share the `arbitrary` layout. Empty strings are treated as absent.
pub fn otel_metric_context(metric: &Metric) -> Option<OtelMetricContext<'_>> {
    let arbitrary = &metric.arbitrary_value().value;
    let metadata = arbitrary
        .get(log_schema().user_metadata_key())?
        .as_object()?;
    let is_otlp = metadata
        .get("data_provider")
        .and_then(Value::as_str)
        .is_some_and(|provider| provider == "otlp");
    if !is_otlp {
        return None;
    }

    let scope = metadata.get("scope").and_then(Value::as_object);
    Some(OtelMetricContext {
        scope_name: non_empty_str(scope.and_then(|scope| scope.get("name"))),
        scope_version: non_empty_str(scope.and_then(|scope| scope.get("version"))),
        scope_attributes: scope
            .and_then(|scope| scope.get("attributes"))
            .and_then(Value::as_object),
        data_point_attributes: metadata.get("attributes").and_then(Value::as_object),
        unit: non_empty_str(arbitrary.get("unit")),
        description: non_empty_str(arbitrary.get("description")),
    })
}

/// Vector's histogram value always has a sum, but an OpenTelemetry histogram may omit it, so
/// this conversion records the absence as a null `sum` in `arbitrary`. The OTLP codec cannot
/// tell an absent sum from 0, because its protobuf library drops field presence, so its sums
/// count as reported.
pub fn otel_sum_reported(arbitrary: &MetricArbitrary) -> bool {
    !matches!(arbitrary.value.get(SUM), Some(Value::Null))
}

pub fn mark_otel_sum_absent(arbitrary: &mut MetricArbitrary) {
    arbitrary.value.insert(SUM.into(), Value::Null);
}

/// The start of the window a cumulative point covers, as written by this conversion and by the
/// OTLP codec.
pub fn otel_start_time(arbitrary: &MetricArbitrary) -> Option<DateTime<Utc>> {
    match arbitrary.value.get(START_TIME) {
        Some(Value::Timestamp(start)) => Some(*start),
        _ => None,
    }
}

pub fn set_otel_start_time(arbitrary: &mut MetricArbitrary, start: DateTime<Utc>) {
    arbitrary
        .value
        .insert(START_TIME.into(), Value::Timestamp(start));
}

fn insert_sum(arbitrary: &mut Object, sum: Option<f64>) {
    arbitrary.insert(SUM.into(), sum.map_or(Value::Null, from_f64_or_zero));
}

fn non_empty_str(value: Option<&Value>) -> Option<Cow<'_, str>> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

/// Decided by shape alone. The OpenTelemetry source sets `%mezmo_native_otel` on every signal,
/// and the flag survives processors that rewrite the message, such as event-to-metric, so it
/// tells where an event came from rather than whether it is still an envelope.
pub(super) fn is_native_otel(log: &LogEvent) -> bool {
    let Some(Value::Object(message)) = message(log) else {
        return false;
    };
    let is_metric = message
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|envelope_type| envelope_type == "metric");
    is_metric && matches!(message.get("record"), Some(Value::Object(_)))
}

/// # Errors
///
/// Will return `Err` if a required envelope field is missing or has the wrong type
/// `None` for a point flagged as recording no value, which OpenTelemetry uses for explicitly
/// missing data, like a Prometheus staleness marker. A Vector metric cannot express the absence,
/// and any value would be read as a measurement.
///
/// Reference: <https://opentelemetry.io/docs/specs/otel/metrics/data-model/#no-recorded-value>
pub(super) fn to_metric(log: &LogEvent) -> Result<Option<Metric>, TransformError> {
    let message_key = log_schema()
        .message_key()
        .map(ToString::to_string)
        .unwrap_or_default();
    let message = message(log).ok_or_else(|| TransformError::FieldNotFound {
        field: message_key.clone(),
    })?;
    // Errors name fields relative to the message, matching the flat-shape path.
    let envelope = Fields {
        map: message
            .as_object()
            .ok_or(TransformError::FieldInvalidType { field: message_key })?,
        path: String::new(),
    };

    let record = envelope.required_object("record")?;
    let metric = record.required_object("metric")?;
    let data_point = record.required_object("dataPoint")?;
    if data_point.optional_u64("flags")?.unwrap_or(0) & FLAG_NO_RECORDED_VALUE != 0 {
        return Ok(None);
    }
    let resource = envelope.optional_object("resource")?;

    let name = metric.required_string("name")?;

    let time_unix_nano = data_point.required_u64("timeUnixNano")?;
    let start_time_unix_nano = data_point
        .optional_u64("startTimeUnixNano")?
        .filter(|nanos| *nanos > 0);
    let timestamp = if time_unix_nano > 0 {
        nanos_to_datetime(time_unix_nano, data_point.path_of("timeUnixNano"))?
    } else {
        event_timestamp(log)
    };
    let start_time = start_time_unix_nano
        .map(|nanos| nanos_to_datetime(nanos, data_point.path_of("startTimeUnixNano")))
        .transpose()?;

    let mut arbitrary = Object::new();
    let converted = convert_value(&data_point, &mut arbitrary)?;

    let interval_ms = match (converted.temporality, start_time_unix_nano) {
        (Some(Temporality::Delta), Some(start)) if time_unix_nano > start => {
            u32::try_from((time_unix_nano - start) / NANOS_PER_MILLI)
                .ok()
                .and_then(NonZeroU32::new)
        }
        _ => None,
    };

    insert_envelope_arbitrary(
        &mut arbitrary,
        &envelope,
        &metric,
        &data_point,
        &converted,
        timestamp,
        start_time,
    )?;

    let resource_attributes = match &resource {
        Some(resource) => resource.optional_object("attributes")?,
        None => None,
    };
    let tags = build_tags(
        resource_attributes.as_ref(),
        data_point.optional_object("attributes")?.as_ref(),
    );

    Ok(Some(Metric::from_parts(
        MetricSeries {
            name: MetricName {
                name,
                namespace: None,
            },
            tags,
        },
        MetricData {
            time: MetricTime {
                timestamp: Some(timestamp),
                interval_ms,
            },
            kind: converted.kind,
            value: converted.value,
            arbitrary: MetricArbitrary { value: arbitrary },
        },
        log.metadata().clone(),
    )))
}

fn convert_value(
    data_point: &Fields<'_>,
    arbitrary: &mut Object,
) -> Result<Converted, TransformError> {
    match data_point.required_string("type")?.as_str() {
        "gauge" => Ok(Converted {
            value: MetricValue::Gauge {
                value: required_number(data_point)?,
            },
            kind: MetricKind::Absolute,
            temporality: None,
            original_type: "gauge",
        }),
        "sum" => convert_sum(data_point, arbitrary),
        "histogram" => convert_histogram(data_point, arbitrary),
        "exponentialHistogram" => convert_exponential_histogram(data_point, arbitrary),
        "summary" => convert_summary(data_point),
        other => Err(TransformError::InvalidMetricType {
            type_name: other.to_string(),
        }),
    }
}

fn insert_envelope_arbitrary(
    arbitrary: &mut Object,
    envelope: &Fields<'_>,
    metric: &Fields<'_>,
    data_point: &Fields<'_>,
    converted: &Converted,
    timestamp: DateTime<Utc>,
    start_time: Option<DateTime<Utc>>,
) -> Result<(), TransformError> {
    if let Some(temporality) = converted.temporality {
        arbitrary.insert(
            "aggregation_temporality".into(),
            temporality.as_otlp().into(),
        );
    }
    arbitrary.insert("name".into(), metric.required_string("name")?.into());
    arbitrary.insert(
        "description".into(),
        metric
            .optional_string("description")?
            .unwrap_or_default()
            .into(),
    );
    arbitrary.insert(
        "unit".into(),
        metric.optional_string("unit")?.unwrap_or_default().into(),
    );
    arbitrary.insert("time_unix".into(), Value::Timestamp(timestamp));
    if let Some(start_time) = start_time {
        arbitrary.insert(START_TIME.into(), Value::Timestamp(start_time));
    }
    arbitrary.insert(
        "flags".into(),
        data_point.optional_u64("flags")?.unwrap_or(0).into(),
    );
    arbitrary.insert(
        "dropped_attributes_count".into(),
        data_point
            .optional_u64("droppedAttributesCount")?
            .unwrap_or(0)
            .into(),
    );
    arbitrary.insert(
        "exemplars".into(),
        convert_exemplars(data_point, timestamp)?,
    );
    arbitrary.insert(
        log_schema().user_metadata_key().into(),
        Value::Object(Object::from([
            ("original_type".into(), converted.original_type.into()),
            ("data_provider".into(), "otlp".into()),
            (
                "resource".into(),
                envelope.cloned_or_empty_object("resource"),
            ),
            ("scope".into(), envelope.cloned_or_empty_object("scope")),
            (
                "attributes".into(),
                data_point.cloned_or_empty_object("attributes"),
            ),
        ])),
    );
    Ok(())
}

fn convert_temporality(data_point: &Fields<'_>) -> Result<Temporality, TransformError> {
    let field = data_point.path_of("aggregationTemporality");
    let value = data_point.required("aggregationTemporality")?;
    match value {
        Value::Integer(TEMPORALITY_DELTA) => Ok(Temporality::Delta),
        Value::Integer(TEMPORALITY_CUMULATIVE) => Ok(Temporality::Cumulative),
        Value::Integer(_) => Err(invalid_temporality(field)),
        Value::Bytes(_) => match parse_string(value, field.clone())?.as_str() {
            TEMPORALITY_DELTA_NAME => Ok(Temporality::Delta),
            TEMPORALITY_CUMULATIVE_NAME => Ok(Temporality::Cumulative),
            _ => Err(invalid_temporality(field)),
        },
        _ => Err(TransformError::FieldInvalidType { field }),
    }
}

fn invalid_temporality(field: String) -> TransformError {
    TransformError::FieldInvalidValue {
        field,
        reason: format!(
            "expected {TEMPORALITY_DELTA} ({TEMPORALITY_DELTA_NAME}) or {TEMPORALITY_CUMULATIVE} \
             ({TEMPORALITY_CUMULATIVE_NAME})"
        ),
    }
}

fn convert_sum(
    data_point: &Fields<'_>,
    arbitrary: &mut Object,
) -> Result<Converted, TransformError> {
    let temporality = convert_temporality(data_point)?;
    // ingestion-service only writes isMonotonic when it is true.
    let is_monotonic = data_point.optional_bool("isMonotonic")?.unwrap_or(false);
    let value = required_number(data_point)?;
    arbitrary.insert("is_monotonic".into(), is_monotonic.into());
    Ok(Converted {
        value: if is_monotonic {
            MetricValue::Counter { value }
        } else {
            MetricValue::Gauge { value }
        },
        kind: temporality.kind(),
        temporality: Some(temporality),
        original_type: "sum",
    })
}

fn convert_histogram(
    data_point: &Fields<'_>,
    arbitrary: &mut Object,
) -> Result<Converted, TransformError> {
    let temporality = convert_temporality(data_point)?;
    let count = data_point.required_u64("count")?;
    let sum = data_point.optional_f64("sum")?;
    let bucket_counts = data_point.array("bucketCounts", parse_u64)?;
    let explicit_bounds = data_point.array("explicitBounds", parse_f64)?;

    // OTLP: bucket_counts is empty, or has one more entry than explicit_bounds for the
    // overflow bucket above the last bound.
    if !bucket_counts.is_empty() && bucket_counts.len() != explicit_bounds.len() + 1 {
        return Err(TransformError::FieldInvalidValue {
            field: data_point.path_of("bucketCounts"),
            reason: format!(
                "expected {} bucket counts for {} explicit bounds, found {}",
                explicit_bounds.len() + 1,
                explicit_bounds.len(),
                bucket_counts.len()
            ),
        });
    }
    let buckets = explicit_bounds
        .iter()
        .copied()
        .chain(std::iter::once(f64::INFINITY))
        .zip(bucket_counts.iter().copied())
        .map(|(upper_limit, count)| Bucket { upper_limit, count })
        .collect();

    insert_min_max(data_point, arbitrary)?;
    insert_sum(arbitrary, sum);
    arbitrary.insert(
        "bucket_counts".into(),
        Value::Array(bucket_counts.into_iter().map(Value::from).collect()),
    );
    arbitrary.insert(
        "explicit_bounds".into(),
        Value::Array(explicit_bounds.into_iter().map(from_f64_or_zero).collect()),
    );

    Ok(Converted {
        value: MetricValue::AggregatedHistogram {
            buckets,
            count,
            sum: sum.unwrap_or(0.0),
        },
        kind: temporality.kind(),
        temporality: Some(temporality),
        original_type: "histogram",
    })
}

fn parse_exponential_buckets(
    data_point: &Fields<'_>,
    name: &str,
) -> Result<ExponentialBuckets, TransformError> {
    let Some(buckets) = data_point.optional_object(name)? else {
        return Ok(ExponentialBuckets::default());
    };
    Ok(ExponentialBuckets {
        offset: buckets.optional_i64("offset")?.unwrap_or(0),
        counts: buckets.array("bucketCounts", parse_u64)?,
    })
}

/// Vector has no exponential histogram type, so the buckets are re-expressed with explicit
/// upper limits. The original fields are kept in `arbitrary` for the OpenTelemetry sink and for
/// sinks that can use the native buckets.
fn convert_exponential_histogram(
    data_point: &Fields<'_>,
    arbitrary: &mut Object,
) -> Result<Converted, TransformError> {
    let temporality = convert_temporality(data_point)?;
    let count = data_point.required_u64("count")?;
    let sum = data_point.optional_f64("sum")?;
    let scale = data_point.optional_i64("scale")?.unwrap_or(0);
    let histogram = ExponentialHistogram {
        scale: i32::try_from(scale).map_err(|_| TransformError::FieldInvalidValue {
            field: data_point.path_of("scale"),
            reason: "expected a 32-bit integer".into(),
        })?,
        zero_count: data_point.optional_u64("zeroCount")?.unwrap_or(0),
        zero_threshold: data_point.optional_f64("zeroThreshold")?.unwrap_or(0.0),
        positive: parse_exponential_buckets(data_point, "positive")?,
        negative: parse_exponential_buckets(data_point, "negative")?,
    };

    insert_min_max(data_point, arbitrary)?;
    arbitrary.insert("count".into(), count.into());
    insert_sum(arbitrary, sum);
    histogram.insert_into(arbitrary);

    Ok(Converted {
        value: MetricValue::AggregatedHistogram {
            buckets: histogram.buckets(),
            count,
            sum: sum.unwrap_or(0.0),
        },
        kind: temporality.kind(),
        temporality: Some(temporality),
        original_type: exponential::ORIGINAL_TYPE,
    })
}

fn convert_summary(data_point: &Fields<'_>) -> Result<Converted, TransformError> {
    let quantiles = data_point.array("quantileValues", |value, path| {
        // Protobuf decoding omits zero-valued fields, so an absent quantile or value is 0.
        let quantile = as_fields(value, path)?;
        Ok(Quantile {
            quantile: quantile.optional_f64("quantile")?.unwrap_or(0.0),
            value: quantile.optional_f64("value")?.unwrap_or(0.0),
        })
    })?;

    Ok(Converted {
        value: MetricValue::AggregatedSummary {
            quantiles,
            count: data_point.required_u64("count")?,
            sum: data_point.required_f64("sum")?,
        },
        kind: MetricKind::Absolute,
        temporality: None,
        original_type: "summary",
    })
}

fn insert_min_max(data_point: &Fields<'_>, arbitrary: &mut Object) -> Result<(), TransformError> {
    for name in ["min", "max"] {
        if let Some(value) = data_point.optional_f64(name)? {
            arbitrary.insert(name.into(), from_f64_or_zero(value));
        }
    }
    Ok(())
}

fn convert_exemplars(
    data_point: &Fields<'_>,
    data_point_time: DateTime<Utc>,
) -> Result<Value, TransformError> {
    let exemplars = data_point.array("exemplars", |value, path| {
        let exemplar = as_fields(value, path)?;
        // An exemplar is a sample of the measurements, so one without a value has nothing for a
        // destination to show. `Value` cannot hold NaN.
        let Some(value) = exemplar_value(&exemplar)? else {
            return Ok(None);
        };
        let time = match exemplar.optional_u64("timeUnixNano")? {
            Some(nanos) if nanos > 0 => nanos_to_datetime(nanos, exemplar.path_of("timeUnixNano"))?,
            _ => data_point_time,
        };
        Ok(Some(Value::Object(Object::from([
            (
                "filtered_attributes".into(),
                exemplar.cloned_or_empty_object("filteredAttributes"),
            ),
            ("value".into(), value),
            ("time_unix".into(), Value::Timestamp(time)),
            (
                "trace_id".into(),
                exemplar
                    .optional_string("traceId")?
                    .unwrap_or_default()
                    .into(),
            ),
            (
                "span_id".into(),
                exemplar
                    .optional_string("spanId")?
                    .unwrap_or_default()
                    .into(),
            ),
        ]))))
    })?;
    Ok(Value::Array(exemplars.into_iter().flatten().collect()))
}

fn exemplar_value(exemplar: &Fields<'_>) -> Result<Option<Value>, TransformError> {
    let Some(value) = exemplar.optional("value") else {
        return Ok(None);
    };
    let field = exemplar.path_of("value");
    Ok(match value_type(exemplar, value)? {
        ValueType::Int => Some(Value::Integer(parse_i64(value, field)?)),
        ValueType::Double => NotNan::new(parse_f64(value, field)?).ok().map(Value::Float),
    })
}

/// Resource attributes first so a data point attribute with the same key wins.
fn build_tags(
    resource: Option<&Fields<'_>>,
    data_point: Option<&Fields<'_>>,
) -> Option<MetricTags> {
    let mut tags = MetricTags::default();
    for attributes in [resource, data_point].into_iter().flatten() {
        for (key, value) in attributes.map {
            if let Some(value) = attribute_tag_value(value) {
                tags.replace(key.to_string(), value);
            }
        }
    }
    (!tags.is_empty()).then_some(tags)
}

/// Renders an OpenTelemetry attribute value as a tag value: strings as-is, numbers and booleans
/// as their literal text, arrays and maps as JSON.
pub fn attribute_tag_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bytes(_) => value.as_str().map(Cow::into_owned),
        Value::Integer(integer) => Some(integer.to_string()),
        Value::Float(float) => Some(float.to_string()),
        Value::Boolean(boolean) => Some(boolean.to_string()),
        Value::Timestamp(timestamp) => Some(timestamp.to_rfc3339()),
        Value::Array(_) | Value::Object(_) | Value::Regex(_) => serde_json::to_string(value).ok(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::num::NonZeroU32;

    use chrono::{DateTime, TimeZone, Utc};
    use serde_json::json;
    use vrl::value::{KeyString, Value};

    use crate::event::{
        LogEvent,
        metric::{Bucket, Metric, MetricKind, MetricValue, Quantile},
    };

    use super::super::{otel_metric_context, otel_sum_reported, to_metric};

    // 2026-10-15T12:00:00Z and one minute earlier.
    const TIME_NANOS: &str = "1792065600000000000";
    const START_NANOS: &str = "1792065540000000000";

    fn time() -> DateTime<Utc> {
        Utc.timestamp_opt(1_792_065_600, 0).unwrap()
    }

    fn start_time() -> DateTime<Utc> {
        Utc.timestamp_opt(1_792_065_540, 0).unwrap()
    }

    fn log_from(event: serde_json::Value) -> LogEvent {
        serde_json::from_value::<BTreeMap<KeyString, Value>>(event)
            .unwrap()
            .into()
    }

    fn envelope_message(data_point: serde_json::Value) -> serde_json::Value {
        let mut message = json!({
            "resource": {"attributes": {"service.name": "checkout", "host.name": "node-1"}},
            "scope": {"name": "io.opentelemetry.http", "version": "1.2.0", "attributes": {"lib": "otel"}},
            "type": "metric",
            "record": {
                "metric": {"name": "http.server.duration", "description": "Request time", "unit": "ms"},
            },
        });
        message["record"]["dataPoint"] = data_point;
        message
    }

    fn envelope(data_point: serde_json::Value) -> LogEvent {
        log_from(json!({"message": envelope_message(data_point)}))
    }

    fn flagged(mut log: LogEvent) -> LogEvent {
        log.metadata_mut()
            .value_mut()
            .insert("mezmo_native_otel", Value::Boolean(true));
        log
    }

    fn convert(data_point: serde_json::Value) -> Metric {
        to_metric(&flagged(envelope(data_point))).unwrap().unwrap()
    }

    fn convert_error(data_point: serde_json::Value) -> String {
        to_metric(&flagged(envelope(data_point)))
            .unwrap_err()
            .to_string()
    }

    fn arbitrary(metric: &Metric, key: &str) -> Value {
        metric
            .arbitrary_value()
            .value()
            .get(key)
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn user_metadata(metric: &Metric, key: &str) -> Value {
        arbitrary(metric, "metadata")
            .as_object()
            .and_then(|metadata| metadata.get(key).cloned())
            .unwrap_or(Value::Null)
    }

    fn json_value(value: serde_json::Value) -> Value {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn gauge_double() {
        let metric = convert(json!({
            "type": "gauge", "attributes": {}, "timeUnixNano": TIME_NANOS,
            "value": 1.5, "valueType": "double", "droppedAttributesCount": 0,
        }));

        assert_eq!(metric.name(), "http.server.duration");
        assert_eq!(metric.namespace(), None);
        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.value(), &MetricValue::Gauge { value: 1.5 });
        assert_eq!(metric.timestamp(), Some(time()));
        assert_eq!(metric.interval_ms(), None);
        assert_eq!(user_metadata(&metric, "original_type"), "gauge".into());
        assert_eq!(arbitrary(&metric, "aggregation_temporality"), Value::Null);
    }

    #[test]
    fn gauge_int_is_parsed_from_string() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": "-42", "valueType": "int",
        }));

        assert_eq!(metric.value(), &MetricValue::Gauge { value: -42.0 });
    }

    #[test]
    fn int_beyond_f64_precision_is_an_error() {
        let error = convert_error(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS,
            "value": "9007199254740993", "valueType": "int",
        }));

        assert_eq!(
            error,
            "Field 'record.dataPoint.value' was truncated during parsing"
        );
    }

    #[test]
    fn non_finite_double_strings_are_accepted() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": "Infinity", "valueType": "double",
        }));

        assert_eq!(
            metric.value(),
            &MetricValue::Gauge {
                value: f64::INFINITY
            }
        );
    }

    #[test]
    fn points_without_recorded_value_are_skipped() {
        for data_point in [
            json!({"type": "gauge", "timeUnixNano": TIME_NANOS, "flags": 1}),
            json!({
                "type": "sum", "timeUnixNano": TIME_NANOS, "flags": 3, "value": "0",
                "valueType": "int", "aggregationTemporality": 2, "isMonotonic": true,
            }),
            json!({
                "type": "histogram", "timeUnixNano": TIME_NANOS, "flags": 1, "count": "0",
                "aggregationTemporality": 2,
            }),
        ] {
            assert_eq!(to_metric(&flagged(envelope(data_point))).unwrap(), None);
        }
    }

    #[test]
    fn whole_doubles_beyond_exact_int_range_are_accepted() {
        let gauge = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS,
            "value": 10_000_000_000_000_000_i64, "valueType": "double",
        }));
        let histogram = convert(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "count": "1",
            "sum": 10_000_000_000_000_000_i64, "aggregationTemporality": 1,
        }));

        assert_eq!(gauge.value(), &MetricValue::Gauge { value: 1e16 });
        assert_eq!(
            histogram.value(),
            &MetricValue::AggregatedHistogram {
                buckets: vec![],
                count: 1,
                sum: 1e16
            }
        );
    }

    #[test]
    fn untyped_whole_number_beyond_exact_int_range_is_an_error() {
        let error = convert_error(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 9_007_199_254_740_993_i64,
        }));

        assert_eq!(
            error,
            "Field 'record.dataPoint.value' was truncated during parsing"
        );
    }

    #[test]
    fn unknown_value_type_is_an_error() {
        let error = convert_error(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "float",
        }));

        assert_eq!(
            error,
            r#"Field 'record.dataPoint.valueType' value is not valid: expected "int" or "double""#
        );
    }

    #[test]
    fn exemplar_values_follow_their_value_type() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "exemplars": [
                {"value": "Infinity", "valueType": "double"},
                {"value": 10_000_000_000_000_000_i64, "valueType": "double"},
                {"value": "7", "valueType": "int"},
                {"value": "NaN", "valueType": "double"},
                {"timeUnixNano": "0"},
            ],
        }));

        let Value::Array(exemplars) = arbitrary(&metric, "exemplars") else {
            panic!("expected exemplars");
        };
        let values = exemplars
            .iter()
            .map(|exemplar| exemplar.as_object().unwrap()["value"].clone())
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                Value::from(f64::INFINITY),
                Value::from(1e16),
                Value::Integer(7)
            ]
        );
    }

    #[test]
    fn monotonic_delta_sum_is_incremental_counter_with_interval() {
        let metric = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "value": "7", "valueType": "int", "aggregationTemporality": 1, "isMonotonic": true,
        }));

        assert_eq!(metric.kind(), MetricKind::Incremental);
        assert_eq!(metric.value(), &MetricValue::Counter { value: 7.0 });
        assert_eq!(metric.interval_ms(), NonZeroU32::new(60_000));
        assert_eq!(arbitrary(&metric, "aggregation_temporality"), 1.into());
        assert_eq!(arbitrary(&metric, "is_monotonic"), true.into());
        assert_eq!(user_metadata(&metric, "original_type"), "sum".into());
    }

    #[test]
    fn monotonic_cumulative_sum_is_absolute_counter_without_interval() {
        let metric = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "value": 1200.5, "valueType": "double", "aggregationTemporality": 2, "isMonotonic": true,
        }));

        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.value(), &MetricValue::Counter { value: 1200.5 });
        assert_eq!(metric.interval_ms(), None);
        assert_eq!(arbitrary(&metric, "aggregation_temporality"), 2.into());
    }

    #[test]
    fn non_monotonic_delta_sum_is_incremental_gauge() {
        let metric = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "value": -3.0, "valueType": "double", "aggregationTemporality": 1,
        }));

        assert_eq!(metric.kind(), MetricKind::Incremental);
        assert_eq!(metric.value(), &MetricValue::Gauge { value: -3.0 });
        assert_eq!(metric.interval_ms(), NonZeroU32::new(60_000));
        assert_eq!(arbitrary(&metric, "is_monotonic"), false.into());
    }

    #[test]
    fn non_monotonic_cumulative_sum_is_absolute_gauge() {
        let metric = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "value": 10, "valueType": "double",
            "aggregationTemporality": 2, "isMonotonic": false,
        }));

        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.value(), &MetricValue::Gauge { value: 10.0 });
    }

    #[test]
    fn delta_without_start_time_has_no_interval() {
        let metric = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "aggregationTemporality": 1, "isMonotonic": true,
        }));

        assert_eq!(metric.interval_ms(), None);
        assert_eq!(arbitrary(&metric, "start_time_unix"), Value::Null);
    }

    #[test]
    fn temporality_enum_names_are_accepted() {
        let delta = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "aggregationTemporality": "AGGREGATION_TEMPORALITY_DELTA",
        }));
        let cumulative = convert(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "aggregationTemporality": "AGGREGATION_TEMPORALITY_CUMULATIVE",
        }));

        assert_eq!(delta.kind(), MetricKind::Incremental);
        assert_eq!(cumulative.kind(), MetricKind::Absolute);
    }

    #[test]
    fn unspecified_temporality_is_an_error() {
        let unspecified = convert_error(json!({
            "type": "sum", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "aggregationTemporality": 0,
        }));
        let missing = convert_error(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "count": "0",
        }));

        assert_eq!(
            unspecified,
            "Field 'record.dataPoint.aggregationTemporality' value is not valid: expected 1 \
             (AGGREGATION_TEMPORALITY_DELTA) or 2 (AGGREGATION_TEMPORALITY_CUMULATIVE)"
        );
        assert_eq!(
            missing,
            "Required field 'record.dataPoint.aggregationTemporality' not found in the log event"
        );
    }

    #[test]
    fn histogram_adds_overflow_bucket() {
        let metric = convert(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "count": "6", "sum": 42.5, "min": 0.5, "max": 30,
            "bucketCounts": ["1", "2", "3"], "explicitBounds": [5, 10.5],
            "aggregationTemporality": 2,
        }));

        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedHistogram {
                buckets: vec![
                    Bucket {
                        upper_limit: 5.0,
                        count: 1
                    },
                    Bucket {
                        upper_limit: 10.5,
                        count: 2
                    },
                    Bucket {
                        upper_limit: f64::INFINITY,
                        count: 3
                    },
                ],
                count: 6,
                sum: 42.5,
            }
        );
        assert_eq!(
            arbitrary(&metric, "bucket_counts"),
            json_value(json!([1, 2, 3]))
        );
        assert_eq!(
            arbitrary(&metric, "explicit_bounds"),
            json_value(json!([5.0, 10.5]))
        );
        assert_eq!(arbitrary(&metric, "min"), json_value(json!(0.5)));
        assert_eq!(arbitrary(&metric, "max"), json_value(json!(30.0)));
        assert_eq!(user_metadata(&metric, "original_type"), "histogram".into());
    }

    #[test]
    fn delta_histogram_is_incremental() {
        let metric = convert(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "count": "0", "aggregationTemporality": 1,
        }));

        assert_eq!(metric.kind(), MetricKind::Incremental);
        assert_eq!(metric.interval_ms(), NonZeroU32::new(60_000));
        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedHistogram {
                buckets: vec![],
                count: 0,
                sum: 0.0,
            }
        );
        assert_eq!(arbitrary(&metric, "bucket_counts"), json_value(json!([])));
    }

    #[test]
    fn absent_histogram_sum_is_marked() {
        let without = |kind: &str| {
            convert(json!({
                "type": kind, "timeUnixNano": TIME_NANOS, "count": "1",
                "aggregationTemporality": 1,
            }))
        };
        let with_zero = convert(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "count": "1", "sum": 0,
            "aggregationTemporality": 1,
        }));

        assert!(!otel_sum_reported(without("histogram").arbitrary_value()));
        assert!(!otel_sum_reported(
            without("exponentialHistogram").arbitrary_value()
        ));
        assert!(otel_sum_reported(with_zero.arbitrary_value()));
    }

    #[test]
    fn histogram_bucket_count_mismatch_is_an_error() {
        let error = convert_error(json!({
            "type": "histogram", "timeUnixNano": TIME_NANOS, "count": "3",
            "bucketCounts": ["1", "2"], "explicitBounds": [5, 10], "aggregationTemporality": 2,
        }));

        assert_eq!(
            error,
            "Field 'record.dataPoint.bucketCounts' value is not valid: expected 3 bucket counts \
             for 2 explicit bounds, found 2"
        );
    }

    #[test]
    fn exponential_histogram_is_approximated() {
        // Scale 0 means base 2: bucket i holds (2^i, 2^(i+1)].
        let metric = convert(json!({
            "type": "exponentialHistogram", "timeUnixNano": TIME_NANOS,
            "count": "10", "sum": 12.5, "min": -3, "max": 7.5, "scale": 0,
            "zeroCount": "1", "zeroThreshold": 0.001,
            "positive": {"offset": 0, "bucketCounts": ["2", "4"]},
            "negative": {"offset": 1, "bucketCounts": ["3"]},
            "aggregationTemporality": 1,
        }));

        assert_eq!(metric.kind(), MetricKind::Incremental);
        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedHistogram {
                // Empty buckets mark where the negative, zero and first positive buckets start.
                buckets: vec![
                    Bucket {
                        upper_limit: -4.0,
                        count: 0
                    },
                    Bucket {
                        upper_limit: -2.0,
                        count: 3
                    },
                    Bucket {
                        upper_limit: -0.001,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 0.001,
                        count: 1
                    },
                    Bucket {
                        upper_limit: 1.0,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 2.0,
                        count: 2
                    },
                    Bucket {
                        upper_limit: 4.0,
                        count: 4
                    },
                ],
                count: 10,
                sum: 12.5,
            }
        );
        assert_eq!(
            user_metadata(&metric, "original_type"),
            "exponential_histogram".into()
        );
        assert_eq!(arbitrary(&metric, "scale"), 0.into());
        assert_eq!(arbitrary(&metric, "count"), 10.into());
        assert_eq!(arbitrary(&metric, "sum"), json_value(json!(12.5)));
        assert_eq!(arbitrary(&metric, "zero_count"), 1.into());
        assert_eq!(
            arbitrary(&metric, "zero_threshold"),
            json_value(json!(0.001))
        );
        assert_eq!(
            arbitrary(&metric, "positive"),
            json_value(json!({"offset": 0, "bucket_counts": [2, 4]}))
        );
        assert_eq!(
            arbitrary(&metric, "negative"),
            json_value(json!({"offset": 1, "bucket_counts": [3]}))
        );
        assert_eq!(arbitrary(&metric, "min"), json_value(json!(-3.0)));
    }

    #[test]
    fn exponential_histogram_honors_scale() {
        // Scale 1 means base sqrt(2).
        let metric = convert(json!({
            "type": "exponentialHistogram", "timeUnixNano": TIME_NANOS, "count": "1", "scale": 1,
            "zeroCount": "0", "positive": {"offset": 2, "bucketCounts": ["1"]},
            "aggregationTemporality": 2,
        }));

        let MetricValue::AggregatedHistogram { buckets, .. } = metric.value() else {
            panic!("expected a histogram");
        };
        assert_eq!(
            buckets[..2],
            [
                Bucket {
                    upper_limit: 0.0,
                    count: 0
                },
                Bucket {
                    upper_limit: 2.0,
                    count: 0
                }
            ]
        );
        assert!((buckets[2].upper_limit - 2f64.powf(1.5)).abs() < 1e-12);
    }

    #[test]
    fn summary_is_absolute() {
        let metric = convert(json!({
            "type": "summary", "timeUnixNano": TIME_NANOS, "count": "4", "sum": 10.0,
            "quantileValues": [{"value": 0.5}, {"quantile": 0.99, "value": 4.0}],
        }));

        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedSummary {
                quantiles: vec![
                    Quantile {
                        quantile: 0.0,
                        value: 0.5
                    },
                    Quantile {
                        quantile: 0.99,
                        value: 4.0
                    },
                ],
                count: 4,
                sum: 10.0,
            }
        );
        assert_eq!(user_metadata(&metric, "original_type"), "summary".into());
    }

    #[test]
    fn unknown_data_point_type_is_an_error() {
        let error = convert_error(json!({"type": "bogus", "timeUnixNano": TIME_NANOS}));

        assert_eq!(error, "Metric type 'bogus' is not supported");
    }

    #[test]
    fn tags_merge_resource_and_data_point_attributes() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1, "valueType": "double",
            "attributes": {
                "host.name": "override",
                "http.status_code": 200,
                "ratio": 0.25,
                "retry": false,
                "empty": "",
                "list": ["a", 1],
                "map": {"k": "v"},
            },
        }));

        let tags: BTreeMap<String, String> = metric
            .tags()
            .unwrap()
            .iter_single()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        let expected: BTreeMap<String, String> = [
            ("service.name", "checkout"),
            ("host.name", "override"),
            ("http.status_code", "200"),
            ("ratio", "0.25"),
            ("retry", "false"),
            ("empty", ""),
            ("list", r#"["a",1]"#),
            ("map", r#"{"k":"v"}"#),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
        assert_eq!(tags, expected);
    }

    #[test]
    fn no_attributes_means_no_tags() {
        let log = log_from(json!({"message": {
            "type": "metric",
            "record": {
                "metric": {"name": "up"},
                "dataPoint": {"type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1},
            },
        }}));

        let metric = to_metric(&log).unwrap().unwrap();

        assert_eq!(metric.tags(), None);
        assert_eq!(arbitrary(&metric, "description"), "".into());
        assert_eq!(arbitrary(&metric, "unit"), "".into());
        assert_eq!(user_metadata(&metric, "resource"), json_value(json!({})));
        assert_eq!(user_metadata(&metric, "scope"), json_value(json!({})));
    }

    #[test]
    fn arbitrary_keeps_everything_the_metric_has_no_field_for() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "startTimeUnixNano": START_NANOS,
            "value": 1, "valueType": "double", "flags": 2, "droppedAttributesCount": 2,
            "attributes": {"http.status_code": 200},
            "exemplars": [
                {
                    "filteredAttributes": {"user": "u1"}, "timeUnixNano": START_NANOS,
                    "value": "3", "valueType": "int",
                    "traceId": "5b8efff798038103d269b633813fc60c", "spanId": "eee19b7ec3c1b174",
                },
                {"filteredAttributes": {}, "timeUnixNano": "0", "value": 0.5, "valueType": "double"},
            ],
        }));

        assert_eq!(arbitrary(&metric, "name"), "http.server.duration".into());
        assert_eq!(arbitrary(&metric, "description"), "Request time".into());
        assert_eq!(arbitrary(&metric, "unit"), "ms".into());
        assert_eq!(arbitrary(&metric, "flags"), 2.into());
        assert_eq!(arbitrary(&metric, "dropped_attributes_count"), 2.into());
        assert_eq!(arbitrary(&metric, "time_unix"), Value::Timestamp(time()));
        assert_eq!(
            arbitrary(&metric, "start_time_unix"),
            Value::Timestamp(start_time())
        );
        assert_eq!(
            arbitrary(&metric, "exemplars"),
            Value::Array(vec![
                Value::Object(BTreeMap::from([
                    (
                        "filtered_attributes".into(),
                        json_value(json!({"user": "u1"}))
                    ),
                    ("value".into(), 3.into()),
                    ("time_unix".into(), Value::Timestamp(start_time())),
                    ("trace_id".into(), "5b8efff798038103d269b633813fc60c".into()),
                    ("span_id".into(), "eee19b7ec3c1b174".into()),
                ])),
                Value::Object(BTreeMap::from([
                    ("filtered_attributes".into(), json_value(json!({}))),
                    ("value".into(), json_value(json!(0.5))),
                    ("time_unix".into(), Value::Timestamp(time())),
                    ("trace_id".into(), "".into()),
                    ("span_id".into(), "".into()),
                ])),
            ])
        );
        assert_eq!(user_metadata(&metric, "data_provider"), "otlp".into());
        assert_eq!(
            user_metadata(&metric, "resource"),
            json_value(json!({"attributes": {"service.name": "checkout", "host.name": "node-1"}}))
        );
        assert_eq!(
            user_metadata(&metric, "scope"),
            json_value(json!({
                "name": "io.opentelemetry.http", "version": "1.2.0", "attributes": {"lib": "otel"}
            }))
        );
        assert_eq!(
            user_metadata(&metric, "attributes"),
            json_value(json!({"http.status_code": 200}))
        );
    }

    #[test]
    fn zero_time_falls_back_to_event_timestamp() {
        let event_time = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let mut log = envelope(json!({
            "type": "gauge", "timeUnixNano": "0", "value": 1, "valueType": "double",
        }));
        log.insert("timestamp", Value::Timestamp(event_time));

        let metric = to_metric(&log).unwrap().unwrap();

        assert_eq!(metric.timestamp(), Some(event_time));
    }

    #[test]
    fn envelope_shape_alone_is_detected() {
        let log = envelope(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 2, "valueType": "double",
        }));

        let metric = to_metric(&log).unwrap().unwrap();

        assert_eq!(metric.name(), "http.server.duration");
    }

    #[test]
    fn flagged_flat_metric_takes_flat_path() {
        // Event-to-metric rewrote an OTel log into a flat metric; the source flag remains.
        let log = flagged(log_from(json!({"message": {
            "name": "flat", "kind": "absolute", "value": {"type": "gauge", "value": 1},
        }})));

        let metric = to_metric(&log).unwrap().unwrap();

        assert_eq!(metric.name(), "flat");
    }

    #[test]
    fn non_metric_envelope_without_flag_takes_flat_path() {
        let mut message = envelope_message(json!({}));
        message["type"] = json!("log");
        let log = log_from(json!({"message": message}));

        let error = to_metric(&log).unwrap_err().to_string();

        assert_eq!(error, "Required field 'name' not found in the log event");
    }

    #[test]
    fn missing_property_names_the_field() {
        let mut message = envelope_message(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1,
        }));
        message["record"]["metric"]
            .as_object_mut()
            .unwrap()
            .remove("name");
        let log = flagged(log_from(json!({"message": message})));

        let error = to_metric(&log).unwrap_err().to_string();

        assert_eq!(
            error,
            "Required field 'record.metric.name' not found in the log event"
        );
    }

    #[test]
    fn wrongly_typed_property_names_the_field() {
        let error = convert_error(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": true,
        }));

        assert_eq!(error, "Field 'record.dataPoint.value' type is not valid");
    }

    #[test]
    fn context_reads_back_otel_fields() {
        let metric = convert(json!({
            "type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1, "attributes": {"a": "b"},
        }));

        let context = otel_metric_context(&metric).unwrap();

        assert_eq!(context.scope_name.as_deref(), Some("io.opentelemetry.http"));
        assert_eq!(context.scope_version.as_deref(), Some("1.2.0"));
        assert_eq!(
            context.scope_attributes.cloned(),
            json_value(json!({"lib": "otel"})).as_object().cloned()
        );
        assert_eq!(
            context.data_point_attributes.cloned(),
            json_value(json!({"a": "b"})).as_object().cloned()
        );
        assert_eq!(context.unit.as_deref(), Some("ms"));
        assert_eq!(context.description.as_deref(), Some("Request time"));
    }

    #[test]
    fn context_treats_empty_strings_as_absent() {
        let log = log_from(json!({"message": {
            "type": "metric",
            "record": {
                "metric": {"name": "up"},
                "dataPoint": {"type": "gauge", "timeUnixNano": TIME_NANOS, "value": 1},
            },
        }}));

        let metric = to_metric(&log).unwrap().unwrap();
        let context = otel_metric_context(&metric).unwrap();

        assert_eq!(context.scope_name, None);
        assert_eq!(context.unit, None);
        assert_eq!(context.description, None);
    }

    #[test]
    fn flat_metric_has_no_context() {
        let log = log_from(json!({"message": {
            "name": "flat", "kind": "absolute", "value": {"type": "gauge", "value": 1},
        }}));

        assert!(otel_metric_context(&to_metric(&log).unwrap().unwrap()).is_none());
    }
}
