use rand::Rng;

use once_cell::sync::Lazy;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::HashMap;

use opentelemetry_rs::opentelemetry::metrics::{
    AggregationTemporality, AnyValue, AnyValueOneOfvalue, Exemplar, ExemplarOneOfvalue,
    ExponentialHistogramDataPoint, ExponentialHistogramDataPointBuckets,
    ExportMetricsServiceRequest, HistogramDataPoint, InstrumentationScope, MetricOneOfdata,
    NumberDataPoint, NumberDataPointOneOfvalue, Resource, SummaryDataPoint,
    SummaryDataPointValueAtQuantile, Validate,
};

use opentelemetry_rs::opentelemetry::common::KeyValue;

use vector_core::{
    config::log_schema,
    event::{
        metric::mezmo::{
            from_f64_or_zero, IntoTagValue, IntoValue, MetricArbitraryAccessor, MetricTags,
            MetricTagsAccessor, MetricToLogEvent, MetricValueAccessor, MetricValuePairs,
            MetricValueSerializable, MezmoMetric,
        },
        Event, LogEvent, MetricKind, Value,
    },
};

use vector_common::btreemap;

use crate::decoding::format::mezmo::open_telemetry::{
    nano_to_timestamp, DeserializerError, OpenTelemetryKeyValue,
};

const METRIC_TIMESTAMP_KEY: &str = "message.value.time_unix";
static UNIT_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    vec![
        // Time
        ("d", "days"),
        ("h", "hours"),
        ("min", "minutes"),
        ("s", "seconds"),
        ("ms", "milliseconds"),
        ("us", "microseconds"),
        ("ns", "nanoseconds"),
        // Bytes
        ("By", "bytes"),
        ("KiBy", "kibibytes"),
        ("MiBy", "mebibytes"),
        ("GiBy", "gibibytes"),
        ("TiBy", "tibibytes"),
        ("KBy", "kilobytes"),
        ("MBy", "megabytes"),
        ("GBy", "gigabytes"),
        ("TBy", "terabytes"),
        // SI
        ("m", "meters"),
        ("V", "volts"),
        ("A", "amperes"),
        ("J", "joules"),
        ("W", "watts"),
        ("g", "grams"),
        // Misc
        ("Cel", "celsius"),
        ("Hz", "hertz"),
        ("1", ""),
        ("%", "percent"),
    ]
    .into_iter()
    .collect()
});
static PER_UNIT_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    vec![
        ("s", "second"),
        ("m", "minute"),
        ("h", "hour"),
        ("d", "day"),
        ("w", "week"),
        ("mo", "month"),
        ("y", "year"),
    ]
    .into_iter()
    .collect()
});

#[derive(Debug, PartialEq)]
pub struct GaugeMetricValue {
    pub value: NumberDataPointOneOfValue,
}

impl<'a> GaugeMetricValue {
    fn new(gauge_metric: NumberDataPoint<'a>) -> Self {
        GaugeMetricValue {
            value: NumberDataPointOneOfValue {
                value: gauge_metric.value,
            },
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for GaugeMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 0>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("gauge"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Single(&self.value as &dyn IntoValue)
    }
}

#[derive(Debug, PartialEq)]
pub struct GaugeMetricArbitrary<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix: Value,
    pub time_unix: Value,
    pub flags: u32,
}

impl<'a> GaugeMetricArbitrary<'a> {
    fn new(
        gauge_metric: NumberDataPoint<'a>,
        name: Cow<'a, str>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
    ) -> Self {
        GaugeMetricArbitrary {
            name,
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: gauge_metric.exemplars,
            },
            start_time_unix: nano_to_timestamp(gauge_metric.start_time_unix_nano),
            time_unix: nano_to_timestamp(gauge_metric.time_unix_nano),
            flags: gauge_metric.flags,
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for GaugeMetricArbitrary<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 7>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (&"name" as &dyn ToString, &self.name as &dyn IntoValue),
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"exemplars" as &dyn ToString,
                    &self.exemplars as &dyn IntoValue,
                ),
                (
                    &"start_time_unix" as &dyn ToString,
                    &self.start_time_unix as &dyn IntoValue,
                ),
                (
                    &"time_unix" as &dyn ToString,
                    &self.time_unix as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GaugeMetricMetadata<'a> {
    pub resource: &'a ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
    original_type: Cow<'a, str>,
    data_provider: Cow<'a, str>,
}

impl<'a> GaugeMetricMetadata<'a> {
    fn new(
        gauge_metric: NumberDataPoint<'a>,
        resource: &'a ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        GaugeMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: gauge_metric.attributes,
            },
            original_type: Cow::from("gauge"),
            data_provider: Cow::from("otlp"),
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for GaugeMetricMetadata<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 5>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (
                    &"original_type" as &dyn ToString,
                    &self.original_type as &dyn IntoValue,
                ),
                (
                    &"data_provider" as &dyn ToString,
                    &self.data_provider as &dyn IntoValue,
                ),
                (
                    &"resource" as &dyn ToString,
                    self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SumMetricValue {
    pub value: NumberDataPointOneOfValue,
    pub is_monotonic: bool,
}

impl<'a> SumMetricValue {
    fn new(sum_metric: NumberDataPoint<'a>, is_monotonic: bool) -> Self {
        // TODO LOG-19828 It's not clear how to handle aggregation_temporality.delta flag.
        // Based on documentation we have to convert a data point from delta
        // to cumulative.
        // https://opentelemetry.io/docs/specs/otel/metrics/data-model/#sums-delta-to-cumulative

        SumMetricValue {
            value: NumberDataPointOneOfValue {
                value: sum_metric.value,
            },
            is_monotonic,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        if self.is_monotonic {
            &MetricKind::Incremental
        } else {
            &MetricKind::Absolute
        }
    }
}

impl<'a> MetricValueAccessor<'a> for SumMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 0>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        if self.is_monotonic {
            Some(Cow::from("counter"))
        } else {
            Some(Cow::from("gauge"))
        }
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Single(&self.value as &dyn IntoValue)
    }
}

#[derive(Debug, PartialEq)]
pub struct SumMetricArbitrary<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix: Value,
    pub time_unix: Value,
    pub flags: u32,
    pub is_monotonic: bool,
    pub aggregation_temporality: i32,
}

impl<'a> SumMetricArbitrary<'a> {
    fn new(
        sum_metric: NumberDataPoint<'a>,
        name: Cow<'a, str>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
        is_monotonic: bool,
    ) -> Self {
        SumMetricArbitrary {
            name,
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: sum_metric.exemplars,
            },
            start_time_unix: nano_to_timestamp(sum_metric.start_time_unix_nano),
            time_unix: nano_to_timestamp(sum_metric.time_unix_nano),
            flags: sum_metric.flags,
            is_monotonic,
            aggregation_temporality: aggregation_temporality as i32,
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for SumMetricArbitrary<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 9>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (&"name" as &dyn ToString, &self.name as &dyn IntoValue),
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"exemplars" as &dyn ToString,
                    &self.exemplars as &dyn IntoValue,
                ),
                (
                    &"start_time_unix" as &dyn ToString,
                    &self.start_time_unix as &dyn IntoValue,
                ),
                (
                    &"time_unix" as &dyn ToString,
                    &self.time_unix as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
                (
                    &"aggregation_temporality" as &dyn ToString,
                    &self.aggregation_temporality as &dyn IntoValue,
                ),
                (
                    &"is_monotonic" as &dyn ToString,
                    &self.is_monotonic as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SumMetricMetadata<'a> {
    pub resource: &'a ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
    original_type: Cow<'a, str>,
    data_provider: Cow<'a, str>,
}

impl<'a> SumMetricMetadata<'a> {
    fn new(
        sum_metric: NumberDataPoint<'a>,
        resource: &'a ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        SumMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: sum_metric.attributes,
            },
            original_type: Cow::from("sum"),
            data_provider: Cow::from("otlp"),
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for SumMetricMetadata<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 5>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (
                    &"original_type" as &dyn ToString,
                    &self.original_type as &dyn IntoValue,
                ),
                (
                    &"data_provider" as &dyn ToString,
                    &self.data_provider as &dyn IntoValue,
                ),
                (
                    &"resource" as &dyn ToString,
                    self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HistogramMetricValue {
    pub count: u64,
    pub sum: f64,
    pub buckets: Vec<HistogramBucketValue>,
}

impl<'a> HistogramMetricValue {
    fn new(histogram_metric: HistogramDataPoint<'a>) -> Self {
        let bucket_counts = histogram_metric.bucket_counts.into_owned();
        let explicit_bounds = histogram_metric.explicit_bounds.into_owned();

        let buckets: Vec<HistogramBucketValue> = bucket_counts
            .into_iter()
            .zip(explicit_bounds.into_iter())
            .map(|(count, upper_limit)| HistogramBucketValue { count, upper_limit })
            .collect();

        // TODO LOG-19828 It's not clear how to handle aggregation_temporality.delta flag.
        // Based on documentation we have to convert a data point from delta
        // to cumulative.
        // https://opentelemetry.io/docs/specs/otel/metrics/data-model/#histogram

        HistogramMetricValue {
            count: histogram_metric.count,
            sum: histogram_metric.sum,
            buckets,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for HistogramMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("histogram"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (&"buckets" as &dyn ToString, &self.buckets as &dyn IntoValue),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct HistogramMetricArbitrary<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix: Value,
    pub time_unix: Value,
    pub bucket_counts: Cow<'a, [u64]>,
    pub explicit_bounds: Cow<'a, [f64]>,
    pub flags: u32,
    pub min: f64,
    pub max: f64,
    pub aggregation_temporality: i32,
}

impl<'a> HistogramMetricArbitrary<'a> {
    fn new(
        histogram_metric: HistogramDataPoint<'a>,
        name: Cow<'a, str>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
    ) -> Self {
        HistogramMetricArbitrary {
            name,
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: histogram_metric.exemplars,
            },
            start_time_unix: nano_to_timestamp(histogram_metric.start_time_unix_nano),
            time_unix: nano_to_timestamp(histogram_metric.time_unix_nano),
            bucket_counts: histogram_metric.bucket_counts,
            explicit_bounds: histogram_metric.explicit_bounds,
            flags: histogram_metric.flags,
            min: histogram_metric.min,
            max: histogram_metric.max,
            aggregation_temporality: aggregation_temporality as i32,
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for HistogramMetricArbitrary<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 12>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (&"name" as &dyn ToString, &self.name as &dyn IntoValue),
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"exemplars" as &dyn ToString,
                    &self.exemplars as &dyn IntoValue,
                ),
                (
                    &"start_time_unix" as &dyn ToString,
                    &self.start_time_unix as &dyn IntoValue,
                ),
                (
                    &"time_unix" as &dyn ToString,
                    &self.time_unix as &dyn IntoValue,
                ),
                (
                    &"bucket_counts" as &dyn ToString,
                    &self.bucket_counts as &dyn IntoValue,
                ),
                (
                    &"explicit_bounds" as &dyn ToString,
                    &self.explicit_bounds as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
                (&"min" as &dyn ToString, &self.min as &dyn IntoValue),
                (&"max" as &dyn ToString, &self.max as &dyn IntoValue),
                (
                    &"aggregation_temporality" as &dyn ToString,
                    &self.aggregation_temporality as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HistogramMetricMetadata<'a> {
    pub resource: &'a ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
    original_type: Cow<'a, str>,
    data_provider: Cow<'a, str>,
}

impl<'a> HistogramMetricMetadata<'a> {
    fn new(
        histogram_metric: HistogramDataPoint<'a>,
        resource: &'a ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        HistogramMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: histogram_metric.attributes,
            },
            original_type: Cow::from("histogram"),
            data_provider: Cow::from("otlp"),
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for HistogramMetricMetadata<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 5>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (
                    &"original_type" as &dyn ToString,
                    &self.original_type as &dyn IntoValue,
                ),
                (
                    &"data_provider" as &dyn ToString,
                    &self.data_provider as &dyn IntoValue,
                ),
                (
                    &"resource" as &dyn ToString,
                    self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ExponentialHistogramMetricValue<'a> {
    pub exp_histogram_metric: ExponentialHistogramDataPoint<'a>,
}

impl<'a> ExponentialHistogramMetricValue<'a> {
    fn new(exp_histogram_metric: ExponentialHistogramDataPoint<'a>) -> Self {
        // TODO LOG-19828 It's not clear how to handle aggregation_temporality.delta flag.
        // Based on documentation we have to convert a data point from delta
        // to cumulative.

        ExponentialHistogramMetricValue {
            exp_histogram_metric,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for ExponentialHistogramMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 0>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("exponential_histogram"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [].into_iter(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ExponentialHistogramMetricArbitrary<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix: Value,
    pub time_unix: Value,
    pub count: u64,
    pub sum: f64,
    pub scale: i32,
    pub zero_count: u64,
    pub positive: DataPointBucketsMetricValue,
    pub negative: DataPointBucketsMetricValue,
    pub flags: u32,
    pub min: f64,
    pub max: f64,
    pub zero_threshold: f64,
    pub aggregation_temporality: i32,
}

impl<'a> ExponentialHistogramMetricArbitrary<'a> {
    fn new(
        exp_histogram_metric: ExponentialHistogramDataPoint<'a>,
        name: Cow<'a, str>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
    ) -> Self {
        ExponentialHistogramMetricArbitrary {
            name,
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: exp_histogram_metric.exemplars,
            },
            start_time_unix: nano_to_timestamp(exp_histogram_metric.start_time_unix_nano),
            time_unix: nano_to_timestamp(exp_histogram_metric.time_unix_nano),
            count: exp_histogram_metric.count,
            sum: exp_histogram_metric.sum,
            scale: exp_histogram_metric.scale,
            zero_count: exp_histogram_metric.zero_count,
            positive: DataPointBucketsMetricValue::new(exp_histogram_metric.positive),
            negative: DataPointBucketsMetricValue::new(exp_histogram_metric.negative),
            flags: exp_histogram_metric.flags,
            min: exp_histogram_metric.min,
            max: exp_histogram_metric.max,
            zero_threshold: exp_histogram_metric.zero_threshold,
            aggregation_temporality: aggregation_temporality as i32,
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for ExponentialHistogramMetricArbitrary<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 17>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (&"name" as &dyn ToString, &self.name as &dyn IntoValue),
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"exemplars" as &dyn ToString,
                    &self.exemplars as &dyn IntoValue,
                ),
                (
                    &"start_time_unix" as &dyn ToString,
                    &self.start_time_unix as &dyn IntoValue,
                ),
                (
                    &"time_unix" as &dyn ToString,
                    &self.time_unix as &dyn IntoValue,
                ),
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (&"scale" as &dyn ToString, &self.scale as &dyn IntoValue),
                (
                    &"zero_count" as &dyn ToString,
                    &self.zero_count as &dyn IntoValue,
                ),
                (
                    &"positive" as &dyn ToString,
                    &self.positive as &dyn IntoValue,
                ),
                (
                    &"negative" as &dyn ToString,
                    &self.negative as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
                (&"min" as &dyn ToString, &self.min as &dyn IntoValue),
                (&"max" as &dyn ToString, &self.max as &dyn IntoValue),
                (
                    &"zero_threshold" as &dyn ToString,
                    &self.zero_threshold as &dyn IntoValue,
                ),
                (
                    &"aggregation_temporality" as &dyn ToString,
                    &self.aggregation_temporality as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ExponentialHistogramMetricMetadata<'a> {
    pub resource: &'a ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> ExponentialHistogramMetricMetadata<'a> {
    fn new(
        exp_histogram_metric: ExponentialHistogramDataPoint<'a>,
        resource: &'a ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        ExponentialHistogramMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: exp_histogram_metric.attributes,
            },
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for ExponentialHistogramMetricMetadata<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SummaryMetricValue {
    pub count: u64,
    pub sum: f64,
    pub quantile_values: QuantileValuesMetricValue,
}

impl<'a> SummaryMetricValue {
    fn new(summary_metric: SummaryDataPoint<'a>) -> Self {
        SummaryMetricValue {
            count: summary_metric.count,
            sum: summary_metric.sum,
            quantile_values: QuantileValuesMetricValue(summary_metric.quantile_values),
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for SummaryMetricValue {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("summary"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (
                    &"quantiles" as &dyn ToString,
                    &self.quantile_values as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct SummaryMetricArbitrary<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub start_time_unix: Value,
    pub time_unix: Value,
    pub count: u64,
    pub sum: f64,
    pub quantile_values: QuantileValuesMetricValue,
    pub flags: u32,
}

impl<'a> SummaryMetricArbitrary<'a> {
    fn new(
        summary_metric: SummaryDataPoint<'a>,
        name: Cow<'a, str>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
    ) -> Self {
        SummaryMetricArbitrary {
            name,
            description,
            unit,
            start_time_unix: nano_to_timestamp(summary_metric.start_time_unix_nano),
            time_unix: nano_to_timestamp(summary_metric.time_unix_nano),
            count: summary_metric.count,
            sum: summary_metric.sum,
            quantile_values: QuantileValuesMetricValue(summary_metric.quantile_values),
            flags: summary_metric.flags,
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for SummaryMetricArbitrary<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 9>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (&"name" as &dyn ToString, &self.name as &dyn IntoValue),
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"start_time_unix" as &dyn ToString,
                    &self.start_time_unix as &dyn IntoValue,
                ),
                (
                    &"time_unix" as &dyn ToString,
                    &self.time_unix as &dyn IntoValue,
                ),
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
                (
                    &"quantile_values" as &dyn ToString,
                    &self.quantile_values as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SummaryMetricMetadata<'a> {
    pub resource: &'a ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
    original_type: Cow<'a, str>,
    data_provider: Cow<'a, str>,
}

impl<'a> SummaryMetricMetadata<'a> {
    fn new(
        summary_metric: SummaryDataPoint<'a>,
        resource: &'a ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        SummaryMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: summary_metric.attributes,
            },
            original_type: Cow::from("summary"),
            data_provider: Cow::from("otlp"),
        }
    }
}

impl<'a> MetricArbitraryAccessor<'a> for SummaryMetricMetadata<'_> {
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 5>;

    fn value(&'a self) -> MetricValuePairs<Self::ObjIter> {
        MetricValuePairs {
            elements: [
                (
                    &"original_type" as &dyn ToString,
                    &self.original_type as &dyn IntoValue,
                ),
                (
                    &"data_provider" as &dyn ToString,
                    &self.data_provider as &dyn IntoValue,
                ),
                (
                    &"resource" as &dyn ToString,
                    self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ScopeMetricValue<'a> {
    pub name: Option<String>,
    pub version: Option<String>,
    pub attributes: Option<Vec<KeyValue<'a>>>,
    pub dropped_attributes_count: Option<u32>,
}

impl<'a> ScopeMetricValue<'a> {
    fn new(opt: Option<InstrumentationScope<'a>>) -> Self {
        match opt {
            Some(scope) => ScopeMetricValue {
                name: Some(scope.name.to_string()),
                version: Some(scope.version.to_string()),
                attributes: Some(scope.attributes),
                dropped_attributes_count: Some(scope.dropped_attributes_count),
            },
            None => ScopeMetricValue {
                name: None,
                version: None,
                attributes: Some(Vec::new()),
                dropped_attributes_count: None,
            },
        }
    }
}

impl IntoValue for ScopeMetricValue<'_> {
    fn to_value(&self) -> Value {
        let attributes = OpenTelemetryKeyValue {
            attributes: self.attributes.as_ref().unwrap().clone(),
        };

        let name = if let Some(name) = self.name.as_ref() {
            if !name.to_string().is_empty() {
                Value::from(name.to_string())
            } else {
                Value::Null
            }
        } else {
            Value::Null
        };

        let version = if let Some(version) = self.version.as_ref() {
            if !version.to_string().is_empty() {
                Value::from(version.to_string())
            } else {
                Value::Null
            }
        } else {
            Value::Null
        };

        Value::Object(btreemap! {
            "name" => name,
            "version" => version,
            "attributes" => attributes.to_value(),
            "dropped_attributes_count" => self.dropped_attributes_count,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ResourceMetricValue<'a> {
    pub uniq_id: &'a Value,
    pub attributes: OpenTelemetryKeyValue<'a>,
    pub dropped_attributes_count: Option<u32>,
}

impl<'a> ResourceMetricValue<'a> {
    fn new(opt: Option<Resource<'a>>, uniq_id: &'a Value) -> Self {
        match opt {
            Some(resource) => ResourceMetricValue {
                uniq_id,
                attributes: OpenTelemetryKeyValue {
                    attributes: resource.attributes,
                },
                dropped_attributes_count: Some(resource.dropped_attributes_count),
            },
            None => ResourceMetricValue {
                uniq_id,
                attributes: OpenTelemetryKeyValue {
                    attributes: Vec::new(),
                },
                dropped_attributes_count: None,
            },
        }
    }
}

impl IntoValue for ResourceMetricValue<'_> {
    fn to_value(&self) -> Value {
        Value::Object(btreemap! {
            "uniq_id" => self.uniq_id.clone(),
            "attributes" => self.attributes.to_value(),
            "dropped_attributes_count" => self.dropped_attributes_count,
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct HistogramBucketValue {
    pub upper_limit: f64,
    pub count: u64,
}

impl IntoValue for HistogramBucketValue {
    fn to_value(&self) -> Value {
        Value::Object(
            [
                ("upper_limit".to_owned(), from_f64_or_zero(self.upper_limit)),
                ("count".to_owned(), self.count.into()),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct QuantileValuesMetricValue(Vec<SummaryDataPointValueAtQuantile>);

impl IntoValue for QuantileValuesMetricValue {
    fn to_value(&self) -> Value {
        Value::Array(
            self.0
                .iter()
                .map(|quantile_value| {
                    Value::Object(btreemap! {
                        "quantile" => from_f64_or_zero(quantile_value.quantile),
                        "value" => from_f64_or_zero(quantile_value.value),
                    })
                })
                .collect(),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct ExemplarsMetricValue<'a> {
    pub exemplars: Vec<Exemplar<'a>>,
}

impl IntoValue for ExemplarsMetricValue<'_> {
    fn to_value(&self) -> Value {
        Value::Array(
            self.exemplars
                .iter()
                .map(|exemplar| {
                    let exemplar_value: Value = match exemplar.value {
                        ExemplarOneOfvalue::as_int(value) => Value::Integer(value),
                        ExemplarOneOfvalue::as_double(value) => from_f64_or_zero(value),
                        ExemplarOneOfvalue::None => Value::Null,
                    };

                    let filtered_attributes = OpenTelemetryKeyValue {
                        attributes: exemplar.filtered_attributes.clone(),
                    };

                    Value::Object(btreemap! {
                        "filtered_attributes" => filtered_attributes.to_value(),
                        "value" => exemplar_value,
                        "time_unix" => nano_to_timestamp(exemplar.time_unix_nano),
                        "span_id" => faster_hex::hex_string(&exemplar.span_id),
                        "trace_id" => faster_hex::hex_string(&exemplar.trace_id),
                    })
                })
                .collect(),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct DataPointBucketsMetricValue {
    pub offset: Option<i32>,
    pub bucket_counts: Vec<u64>,
}

impl DataPointBucketsMetricValue {
    fn new(opt: Option<ExponentialHistogramDataPointBuckets>) -> Self {
        match opt {
            Some(buckets) => DataPointBucketsMetricValue {
                offset: Some(buckets.offset),
                bucket_counts: buckets.bucket_counts,
            },
            None => DataPointBucketsMetricValue {
                offset: None,
                bucket_counts: Vec::new(),
            },
        }
    }
}

impl IntoValue for DataPointBucketsMetricValue {
    fn to_value(&self) -> Value {
        Value::Object(btreemap! {
            "offset" => self.offset,
            "bucket_counts" => Value::Array(
                self.bucket_counts
                    .iter()
                    .map(|count| Value::from(*count))
                    .collect(),
            ),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct NumberDataPointOneOfValue {
    pub value: NumberDataPointOneOfvalue,
}

impl IntoValue for NumberDataPointOneOfValue {
    fn to_value(&self) -> Value {
        match self.value {
            NumberDataPointOneOfvalue::as_int(value) => Value::Integer(value),
            NumberDataPointOneOfvalue::as_double(value) => from_f64_or_zero(value),
            NumberDataPointOneOfvalue::None => Value::Null,
        }
    }
}

#[derive(Debug)]
struct MetricTagsWrapper<'a> {
    tags: &'a [KeyValue<'a>],
}

impl<'a> MetricTagsAccessor<'a> for MetricTagsWrapper<'a> {
    type Iter = std::iter::FilterMap<
        std::slice::Iter<'a, KeyValue<'a>>,
        fn(&'a KeyValue<'a>) -> Option<(&'a dyn ToString, &'a dyn IntoTagValue)>,
    >;

    fn tags(&self) -> MetricTags<'a, Self::Iter> {
        MetricTags {
            tags: self
                .tags
                .iter()
                .filter_map(|key_value| match &key_value.value {
                    Some(AnyValue {
                        value: AnyValueOneOfvalue::string_value(val),
                    }) if val.is_empty() => None,
                    Some(AnyValue {
                        value: AnyValueOneOfvalue::string_value(val),
                    }) => Some((
                        &key_value.key as &'a dyn ToString,
                        val as &'a dyn IntoTagValue,
                    )),
                    Some(AnyValue {
                        value: AnyValueOneOfvalue::array_value(array_val),
                    }) => Some((
                        &key_value.key as &'a dyn ToString,
                        array_val as &'a dyn IntoTagValue,
                    )),
                    Some(_) => None,
                    None => None,
                }),
        }
    }
}

pub fn sanitize_tags<'a>(
    tags: Vec<KeyValue<'a>>,
    additional_tags: Vec<KeyValue<'a>>,
) -> Vec<KeyValue<'a>> {
    let mut super_tags = tags;
    super_tags.extend(additional_tags.into_iter());

    super_tags
        .into_iter()
        .map(|key_value| {
            let mut key = key_value.key;
            if !key.is_empty() {
                let mut sanitized_key: String = key
                    .chars()
                    .flat_map(|c| {
                        if char::is_alphanumeric(c) {
                            vec![c]
                        } else {
                            vec!['_']
                        }
                    })
                    .collect();

                let first_char = sanitized_key.chars().next().unwrap();
                if first_char.is_ascii_digit() {
                    sanitized_key.insert_str(0, "key_");
                } else if !first_char.is_alphabetic() {
                    sanitized_key.insert_str(0, "key");
                }

                key = Cow::from(sanitized_key);
            }

            KeyValue {
                key,
                value: key_value.value,
            }
        })
        .collect::<Vec<KeyValue>>()
}

pub fn normalize_name<'a>(
    name: Cow<'a, str>,
    unit: Cow<'a, str>,
    m_type: Cow<'a, str>,
    kind: Cow<'a, str>,
) -> Cow<'a, str> {
    // Split metric name in "tokens" (remove all non-alphanumeric)
    let mut name_tokens = name
        .split(|i| !char::is_alphanumeric(i))
        .filter(|&x| !x.is_empty())
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    // Split unit at the '/' if any
    let unit_tokens: Vec<&str> = unit.splitn(2, '/').collect();

    // Main unit
    // Append if not blank, doesn't contain '{}', and is not present in metric name already
    if !unit_tokens.is_empty() {
        let main_unit_otel = unit_tokens[0].trim();

        if !main_unit_otel.is_empty() && !main_unit_otel.contains(['{', '}']) {
            let main_unit_prom = clean_up_string(unit_map_get_or_default(main_unit_otel));
            if !main_unit_prom.is_empty() && !name_tokens.contains(&main_unit_prom) {
                name_tokens.push(main_unit_prom);
            }
        }

        // Per unit
        // Append if not blank, doesn't contain '{}', and is not present in metric name already
        if unit_tokens.len() > 1 && !unit_tokens[1].is_empty() {
            let per_unit_otel = unit_tokens[1].trim();
            if !per_unit_otel.is_empty() && !per_unit_otel.contains(['{', '}']) {
                let per_unit_prom = clean_up_string(per_unit_map_get_or_default(per_unit_otel));
                if !per_unit_prom.is_empty() && !name_tokens.contains(&per_unit_prom) {
                    name_tokens.push("per".to_string());
                    name_tokens.push(per_unit_prom);
                }
            }
        }
    }

    // Append _total for Counters
    if m_type == "counter" && kind == "incremental" {
        if let Some(total_i) = name_tokens.iter().position(|i| *i == "total") {
            name_tokens.remove(total_i);
        }
        name_tokens.push("total".to_string());
    }

    // Append _ratio for metrics with unit "1"
    // Some Otel receivers improperly use unit "1" for counters of objects
    // See https://github.com/open-telemetry/opentelemetry-collector-contrib/issues?q=is%3Aissue+some+metric+units+don%27t+follow+otel+semantic+conventions
    // Until these issues have been fixed, we're appending `_ratio` for gauges ONLY
    // Theoretically, counters could be ratios as well, but it's absurd (for mathematical reasons)
    if unit == "1" && m_type == "gauge" {
        if let Some(ratio_i) = name_tokens.iter().position(|i| *i == "ratio") {
            name_tokens.remove(ratio_i);
        }
        name_tokens.push("ratio".to_string());
    }

    // Build the string from the tokens, separated with underscores
    let mut normalized_name = name_tokens.join("_");

    // Metric name cannot start with a digit, so prefix it with "_" in this case
    if !normalized_name.is_empty() && normalized_name.chars().next().unwrap().is_ascii_digit() {
        normalized_name = "_".to_owned() + &normalized_name
    }

    normalized_name.into()
}

// Clean up specified string so it's Prometheus compliant
fn clean_up_string(string: &str) -> String {
    string
        .split(|i| !char::is_alphanumeric(i))
        .filter(|&x| !x.is_empty())
        .collect::<Vec<&str>>()
        .join("_")
}

// Retrieve the Prometheus "basic" unit corresponding to the specified "basic" unit
// Returns the specified unit if not found in unitMap
fn unit_map_get_or_default(unit: &str) -> &str {
    if let Some(prom_unit) = UNIT_MAP.get(unit) {
        return prom_unit;
    }

    unit
}

// Retrieve the Prometheus "per" unit corresponding to the specified "per" unit
// Returns the specified unit if not found in perUnitMap
fn per_unit_map_get_or_default(per_unit: &str) -> &str {
    if let Some(prom_per_unit) = PER_UNIT_MAP.get(per_unit) {
        return prom_per_unit;
    }

    per_unit
}

pub fn parse_metrics_request(bytes: &[u8]) -> vector_common::Result<SmallVec<[Event; 1]>> {
    let parsed_metrics = ExportMetricsServiceRequest::try_from(bytes)
        .map_err(|e| DeserializerError::ProtobufParseError { source: e })?;

    parsed_metrics
        .validate()
        .map_err(|e| DeserializerError::ProtobufValidationError { source: e })?;

    Ok(to_events(parsed_metrics))
}

#[allow(clippy::too_many_lines)]
pub fn to_events(metric_request: ExportMetricsServiceRequest) -> SmallVec<[Event; 1]> {
    let metric_count = metric_request.resource_metrics.iter().fold(0, |acc, rms| {
        rms.scope_metrics
            .iter()
            .fold(acc, |acc, sms| acc + sms.metrics.len())
    });
    let mut out = SmallVec::with_capacity(metric_count);
    let resource_uniq_id: [u8; 8] = rand::thread_rng().gen();
    let resource_uniq_id: Value = Value::from(faster_hex::hex_string(&resource_uniq_id));

    for resource_metric in metric_request.resource_metrics {
        let recource =
            ResourceMetricValue::new(resource_metric.resource.clone(), &resource_uniq_id);

        for scope_metric in resource_metric.scope_metrics {
            for metric in scope_metric.metrics {
                // Create a uniq ID and put it into a arbitrary or metadata
                // To track group of metrics is in OTLP destination
                match metric.data {
                    MetricOneOfdata::gauge(gauge) => gauge
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value = GaugeMetricValue::new(data_point.clone());

                            let metric_arbitrary = GaugeMetricArbitrary::new(
                                data_point.clone(),
                                metric.name.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                            );

                            let metric_metadata = GaugeMetricMetadata::new(
                                data_point.clone(),
                                &recource,
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            let tags = MetricTagsWrapper {
                                tags: &sanitize_tags(
                                    recource.attributes.attributes.clone(),
                                    data_point.clone().attributes,
                                ),
                            };

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
                                        arbitrary_data: Some(&metric_arbitrary),
                                        value: &metric_value,
                                    }
                                }
                                .to_log_event(),
                            ));
                        })
                        .collect(),
                    MetricOneOfdata::sum(sum) => sum
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value =
                                SumMetricValue::new(data_point.clone(), sum.is_monotonic);

                            let metric_arbitrary = SumMetricArbitrary::new(
                                data_point.clone(),
                                metric.name.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                sum.aggregation_temporality,
                                sum.is_monotonic,
                            );

                            let metric_metadata = SumMetricMetadata::new(
                                data_point.clone(),
                                &recource,
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            let tags = MetricTagsWrapper {
                                tags: &sanitize_tags(
                                    recource.attributes.attributes.clone(),
                                    data_point.clone().attributes,
                                ),
                            };

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
                                        arbitrary_data: Some(&metric_arbitrary),
                                        value: &metric_value,
                                    }
                                }
                                .to_log_event(),
                            ));
                        })
                        .collect(),
                    MetricOneOfdata::histogram(histogram) => histogram
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value = HistogramMetricValue::new(data_point.clone());

                            let metric_arbitrary = HistogramMetricArbitrary::new(
                                data_point.clone(),
                                metric.name.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                histogram.aggregation_temporality,
                            );

                            let metric_metadata = HistogramMetricMetadata::new(
                                data_point.clone(),
                                &recource,
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            let tags = MetricTagsWrapper {
                                tags: &sanitize_tags(
                                    recource.attributes.attributes.clone(),
                                    data_point.clone().attributes,
                                ),
                            };

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
                                        arbitrary_data: Some(&metric_arbitrary),
                                        value: &metric_value,
                                    }
                                }
                                .to_log_event(),
                            ));
                        })
                        .collect(),
                    MetricOneOfdata::exponential_histogram(exp_histogram) => exp_histogram
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value =
                                ExponentialHistogramMetricValue::new(data_point.clone());

                            let metric_arbitrary = ExponentialHistogramMetricArbitrary::new(
                                data_point.clone(),
                                metric.name.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                exp_histogram.aggregation_temporality,
                            );

                            let metric_metadata = ExponentialHistogramMetricMetadata::new(
                                data_point.clone(),
                                &recource,
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            let tags = MetricTagsWrapper {
                                tags: &sanitize_tags(
                                    recource.attributes.attributes.clone(),
                                    data_point.clone().attributes,
                                ),
                            };

                            let _mezmo_metric = MezmoMetric {
                                name: metric.name.clone(),
                                namespace: None,
                                kind: metric_value.kind(),
                                tags: Some(&tags),
                                user_metadata: Some(&metric_metadata),
                                arbitrary_data: Some(&metric_arbitrary),
                                value: &metric_value,
                            };

                            // TODO LOG-19820 Exponential histogram has to be converted to
                            // a native histogram to be able to be handled by any metric sinks.
                            // For now we just skip this exponential histogram metrics.
                            // out.push(make_event({mezmo_metric}.to_log_event()));
                        })
                        .collect(),
                    MetricOneOfdata::summary(summary) => summary
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value = SummaryMetricValue::new(data_point.clone());

                            let metric_arbitrary = SummaryMetricArbitrary::new(
                                data_point.clone(),
                                metric.name.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                            );

                            let metric_metadata = SummaryMetricMetadata::new(
                                data_point.clone(),
                                &recource,
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            let tags = MetricTagsWrapper {
                                tags: &sanitize_tags(
                                    recource.attributes.attributes.clone(),
                                    data_point.clone().attributes,
                                ),
                            };

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
                                        arbitrary_data: Some(&metric_arbitrary),
                                        value: &metric_value,
                                    }
                                }
                                .to_log_event(),
                            ));
                        })
                        .collect(),
                    MetricOneOfdata::None => todo!(),
                };
            }
        }
    }

    out
}

fn make_event(mut log_event: LogEvent) -> Event {
    if let Some(message_key) = log_schema().message_key() {
        if let Some(message) = log_event.get_mut((lookup::PathPrefix::Event, message_key)) {
            let metric_name = match message.get("name").unwrap_or(&Value::Null) {
                Value::Bytes(bytes) => Cow::from(String::from_utf8_lossy(bytes).into_owned()),
                _ => Cow::from(""),
            };

            let metric_unit = match message.get("value.unit").unwrap_or(&Value::Null) {
                Value::Bytes(bytes) => Cow::from(String::from_utf8_lossy(bytes).into_owned()),
                _ => Cow::from(""),
            };

            let metric_kind = match message.get("kind").unwrap_or(&Value::Null) {
                Value::Bytes(bytes) => Cow::from(String::from_utf8_lossy(bytes).into_owned()),
                _ => Cow::from(""),
            };

            let metric_type = match message.get("value.type").unwrap_or(&Value::Null) {
                Value::Bytes(bytes) => Cow::from(String::from_utf8_lossy(bytes).into_owned()),
                _ => Cow::from(""),
            };

            let normalized_name =
                normalize_name(metric_name, metric_unit, metric_type, metric_kind);

            message.insert("name", Value::from(normalized_name));
        }
    }

    if let Some(timestamp_key) = log_schema().timestamp_key() {
        let metric_timestamp_target = (lookup::PathPrefix::Event, METRIC_TIMESTAMP_KEY);

        let timestamp = if let Some(metric_timestamp) = log_event.get(metric_timestamp_target) {
            metric_timestamp.clone()
        } else {
            nano_to_timestamp(0)
        };

        log_event.insert((lookup::PathPrefix::Event, timestamp_key), timestamp);
    }

    log_event.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, TimeZone, Utc};
    use std::collections::BTreeMap;
    use std::ops::Deref;

    use vector_core::event::metric::{mezmo::to_metric, Bucket, Quantile};
    use vector_core::event::{MetricKind, MetricValue};

    use opentelemetry_rs::opentelemetry::metrics::{
        AnyValue, AnyValueOneOfvalue, ExportMetricsServiceRequest, KeyValue, Metric,
        MetricOneOfdata,
    };
    use std::borrow::Cow;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_gauge() {
        use opentelemetry_rs::opentelemetry::metrics::{
            ArrayValue, Exemplar, ExemplarOneOfvalue, Gauge, InstrumentationScope, NumberDataPoint,
            NumberDataPointOneOfvalue, Resource, ResourceMetrics, ScopeMetrics,
        };

        let key_value_num_key = KeyValue {
            key: Cow::from("1foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::array_value(ArrayValue {
                    values: vec![
                        AnyValue {
                            value: AnyValueOneOfvalue::string_value(Cow::from("bar.1")),
                        },
                        AnyValue {
                            value: AnyValueOneOfvalue::string_value(Cow::from("bar.2")),
                        },
                    ],
                }),
            }),
        };

        let key_value_dot_key = KeyValue {
            key: Cow::from("foo.foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar.bar")),
            }),
        };

        let key_value_underscore_key = KeyValue {
            key: Cow::from("_foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("_bar")),
            }),
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![
                        key_value_str.clone(),
                        key_value_num_key.clone(),
                        key_value_dot_key.clone(),
                        key_value_underscore_key.clone(),
                        key_value_empty_str.clone(),
                    ],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from(""),
                        attributes: vec![
                            key_value_str.clone(),
                            key_value_num_key.clone(),
                            key_value_dot_key.clone(),
                            key_value_underscore_key.clone(),
                            key_value_empty_str.clone(),
                        ],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("system.filesystem.usage"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_num_key.clone(),
                                    key_value_dot_key.clone(),
                                    key_value_underscore_key.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                value: NumberDataPointOneOfvalue::as_int(10),
                                exemplars: vec![Exemplar {
                                    filtered_attributes: vec![
                                        key_value_str.clone(),
                                        key_value_num_key.clone(),
                                        key_value_dot_key.clone(),
                                        key_value_underscore_key.clone(),
                                        key_value_empty_str.clone(),
                                    ],
                                    time_unix_nano: 1_579_134_612_000_000_011,
                                    value: ExemplarOneOfvalue::as_int(10),
                                    span_id: Cow::from("test".as_bytes()),
                                    trace_id: Cow::from("test".as_bytes()),
                                }],
                                flags: 1,
                            }],
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("kind".into(), "absolute".into()),
                (
                    "name".into(),
                    "system_filesystem_usage_gibibytes_per_second".into()
                ),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("key_1foo".into(), "bar.1, bar.2".into()),
                        ("foo_foo".into(), "bar.bar".into()),
                        ("key_foo".into(), "_bar".into()),
                    ]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "gauge".into()),
                        ("value".into(), Value::Integer(10)),
                        ("name".into(), "system.filesystem.usage".into()),
                        ("description".into(), "test_description".into()),
                        ("unit".into(), "GiBy/s".into()),
                        (
                            "exemplars".into(),
                            Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                (
                                    "filtered_attributes".into(),
                                    Value::Object(BTreeMap::from([
                                        ("foo".into(), "bar".into()),
                                        (
                                            "1foo".into(),
                                            Value::Array(Vec::from([
                                                "bar.1".into(),
                                                "bar.2".into()
                                            ]))
                                        ),
                                        ("foo.foo".into(), "bar.bar".into()),
                                        ("_foo".into(), "_bar".into()),
                                        ("empty".into(), Value::Null),
                                    ]))
                                ),
                                ("span_id".into(), "74657374".into()),
                                (
                                    "time_unix".into(),
                                    Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(
                                                1_579_134_612_i64,
                                                11_u32
                                            )
                                            .expect("timestamp should be a valid timestamp"),
                                        )
                                    )
                                ),
                                ("trace_id".into(), "74657374".into()),
                                ("value".into(), Value::Integer(10)),
                            ]))]))
                        ),
                        ("flags".into(), Value::Integer(1)),
                        (
                            "start_time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        (
                            "time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                    ]))
                ),
            ]))
        );

        let log = metrics[0].clone().into_log();

        let metadata = log.value().get("metadata").unwrap();

        let uniq_id = metadata.get("resource.uniq_id");

        assert!(uniq_id.is_some());

        assert_eq!(
            *metadata.deref(),
            Value::Object(BTreeMap::from([
                ("original_type".into(), "gauge".into()),
                ("data_provider".into(), "otlp".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                (
                                    "1foo".into(),
                                    Value::Array(Vec::from(["bar.1".into(), "bar.2".into()]))
                                ),
                                ("foo.foo".into(), "bar.bar".into()),
                                ("_foo".into(), "_bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("uniq_id".into(), uniq_id.unwrap().clone()),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                (
                                    "1foo".into(),
                                    Value::Array(Vec::from(["bar.1".into(), "bar.2".into()]))
                                ),
                                ("foo.foo".into(), "bar.bar".into()),
                                ("_foo".into(), "_bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), Value::Null),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        (
                            "1foo".into(),
                            Value::Array(Vec::from(["bar.1".into(), "bar.2".into()]))
                        ),
                        ("foo.foo".into(), "bar.bar".into()),
                        ("_foo".into(), "_bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        let metric =
            to_metric(&metrics[0].clone().into_log()).expect("Failed to convert lot to metric");

        assert_eq!(metric.value(), &MetricValue::Gauge { value: 10.0 });
        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.tags().unwrap().get("foo").unwrap(), "bar");
        assert_eq!(
            metric.timestamp().unwrap(),
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1_579_134_612, 11).unwrap(),)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_monotonic_sum() {
        use opentelemetry_rs::opentelemetry::metrics::{
            AggregationTemporality, Exemplar, ExemplarOneOfvalue, InstrumentationScope,
            NumberDataPoint, NumberDataPointOneOfvalue, Resource, ResourceMetrics, ScopeMetrics,
            Sum,
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test.name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::sum(Sum {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                value: NumberDataPointOneOfvalue::as_double(10_f64),
                                exemplars: vec![Exemplar {
                                    filtered_attributes: vec![
                                        key_value_str.clone(),
                                        key_value_empty_str.clone(),
                                    ],
                                    time_unix_nano: 1_579_134_612_000_000_011,
                                    value: ExemplarOneOfvalue::as_int(10),
                                    span_id: Cow::from("test".as_bytes()),
                                    trace_id: Cow::from("test".as_bytes()),
                                }],
                                flags: 1,
                            }],
                            aggregation_temporality:
                                AggregationTemporality::AGGREGATION_TEMPORALITY_UNSPECIFIED,
                            is_monotonic: true,
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("kind".into(), "incremental".into()),
                ("name".into(), "test_name_gibibytes_per_second_total".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "counter".into()),
                        ("value".into(), from_f64_or_zero(10_f64)),
                        ("name".into(), "test.name".into()),
                        ("description".into(), "test_description".into()),
                        ("unit".into(), "GiBy/s".into()),
                        (
                            "exemplars".into(),
                            Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                (
                                    "filtered_attributes".into(),
                                    Value::Object(BTreeMap::from([
                                        ("foo".into(), "bar".into()),
                                        ("empty".into(), Value::Null),
                                    ]))
                                ),
                                ("span_id".into(), "74657374".into()),
                                (
                                    "time_unix".into(),
                                    Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(
                                                1_579_134_612_i64,
                                                11_u32
                                            )
                                            .expect("timestamp should be a valid timestamp"),
                                        )
                                    )
                                ),
                                ("trace_id".into(), "74657374".into()),
                                ("value".into(), Value::Integer(10)),
                            ]))]))
                        ),
                        ("flags".into(), Value::Integer(1)),
                        (
                            "start_time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        (
                            "time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        ("aggregation_temporality".into(), Value::Integer(0)),
                        ("is_monotonic".into(), Value::Boolean(true)),
                    ]))
                ),
            ]))
        );

        let log = metrics[0].clone().into_log();

        let metadata = log.value().get("metadata").unwrap();

        let uniq_id = metadata.get("resource.uniq_id");

        assert!(uniq_id.is_some());

        assert_eq!(
            *metadata.deref(),
            Value::Object(BTreeMap::from([
                ("original_type".into(), "sum".into()),
                ("data_provider".into(), "otlp".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("uniq_id".into(), uniq_id.unwrap().clone()),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), "1.2.3".into()),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        let metric =
            to_metric(&metrics[0].clone().into_log()).expect("Failed to convert lot to metric");

        assert_eq!(metric.value(), &MetricValue::Counter { value: 10.0 });
        assert_eq!(metric.kind(), MetricKind::Incremental);
        assert_eq!(metric.tags().unwrap().get("foo").unwrap(), "bar");
        assert_eq!(
            metric.timestamp().unwrap(),
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1_579_134_612, 11).unwrap(),)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_non_monotonic_sum() {
        use opentelemetry_rs::opentelemetry::metrics::{
            AggregationTemporality, Exemplar, ExemplarOneOfvalue, InstrumentationScope,
            NumberDataPoint, NumberDataPointOneOfvalue, Resource, ResourceMetrics, ScopeMetrics,
            Sum,
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test.name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::sum(Sum {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                value: NumberDataPointOneOfvalue::as_double(10_f64),
                                exemplars: vec![Exemplar {
                                    filtered_attributes: vec![
                                        key_value_str.clone(),
                                        key_value_empty_str.clone(),
                                    ],
                                    time_unix_nano: 1_579_134_612_000_000_011,
                                    value: ExemplarOneOfvalue::as_int(10),
                                    span_id: Cow::from("test".as_bytes()),
                                    trace_id: Cow::from("test".as_bytes()),
                                }],
                                flags: 1,
                            }],
                            aggregation_temporality:
                                AggregationTemporality::AGGREGATION_TEMPORALITY_UNSPECIFIED,
                            is_monotonic: false,
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("kind".into(), "absolute".into()),
                ("name".into(), "test_name_gibibytes_per_second".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "gauge".into()),
                        ("value".into(), from_f64_or_zero(10_f64)),
                        ("name".into(), "test.name".into()),
                        ("description".into(), "test_description".into()),
                        ("unit".into(), "GiBy/s".into()),
                        (
                            "exemplars".into(),
                            Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                (
                                    "filtered_attributes".into(),
                                    Value::Object(BTreeMap::from([
                                        ("foo".into(), "bar".into()),
                                        ("empty".into(), Value::Null),
                                    ]))
                                ),
                                ("span_id".into(), "74657374".into()),
                                (
                                    "time_unix".into(),
                                    Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(
                                                1_579_134_612_i64,
                                                11_u32
                                            )
                                            .expect("timestamp should be a valid timestamp"),
                                        )
                                    )
                                ),
                                ("trace_id".into(), "74657374".into()),
                                ("value".into(), Value::Integer(10)),
                            ]))]))
                        ),
                        ("flags".into(), Value::Integer(1)),
                        (
                            "start_time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        (
                            "time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        ("aggregation_temporality".into(), Value::Integer(0)),
                        ("is_monotonic".into(), Value::Boolean(false)),
                    ]))
                ),
            ]))
        );

        let log = metrics[0].clone().into_log();

        let metadata = log.value().get("metadata").unwrap();

        let uniq_id = metadata.get("resource.uniq_id");

        assert!(uniq_id.is_some());

        assert_eq!(
            *metadata.deref(),
            Value::Object(BTreeMap::from([
                ("original_type".into(), "sum".into()),
                ("data_provider".into(), "otlp".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("uniq_id".into(), uniq_id.unwrap().clone()),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), "1.2.3".into()),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        let metric =
            to_metric(&metrics[0].clone().into_log()).expect("Failed to convert lot to metric");

        assert_eq!(metric.value(), &MetricValue::Gauge { value: 10.0 });
        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.tags().unwrap().get("foo").unwrap(), "bar");
        assert_eq!(
            metric.timestamp().unwrap(),
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1_579_134_612, 11).unwrap())
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_histogram() {
        use opentelemetry_rs::opentelemetry::metrics::{
            AggregationTemporality, Exemplar, ExemplarOneOfvalue, Histogram, HistogramDataPoint,
            InstrumentationScope, Resource, ResourceMetrics, ScopeMetrics,
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test.name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                count: 10,
                                sum: 3.7_f64,
                                bucket_counts: Cow::from(vec![
                                    214, 6, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                ]),
                                explicit_bounds: Cow::from(vec![
                                    0.005_f64, 0.01_f64, 0.025_f64, 0.05_f64, 0.075_f64, 0.1_f64,
                                    0.25_f64, 0.5_f64, 0.75_f64, 1.0_f64, 2.5_f64, 5.0_f64,
                                    7.5_f64, 10.0_f64,
                                ]),
                                exemplars: vec![Exemplar {
                                    filtered_attributes: vec![
                                        key_value_str.clone(),
                                        key_value_empty_str.clone(),
                                    ],
                                    time_unix_nano: 1_579_134_612_000_000_011,
                                    value: ExemplarOneOfvalue::as_double(10.5_f64),
                                    span_id: Cow::from("test".as_bytes()),
                                    trace_id: Cow::from("test".as_bytes()),
                                }],
                                flags: 1,
                                min: 0.1_f64,
                                max: 9.9_f64,
                            }],
                            aggregation_temporality:
                                AggregationTemporality::AGGREGATION_TEMPORALITY_CUMULATIVE,
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("kind".into(), "absolute".into()),
                ("name".into(), "test_name_gibibytes_per_second".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "histogram".into()),
                        (
                            "value".into(),
                            Value::Object(BTreeMap::from([
                                ("count".into(), Value::Integer(10)),
                                ("sum".into(), from_f64_or_zero(3.7)),
                                (
                                    "buckets".into(),
                                    Value::Array(Vec::from([
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.005,
                                            "count" => 214,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.01,
                                            "count" => 6,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.025,
                                            "count" => 1,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.05,
                                            "count" => 1,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.075,
                                            "count" => 2,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.1,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.25,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.5,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 0.75,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 1.0,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 2.5,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 5.0,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 7.5,
                                            "count" => 0,
                                        }),
                                        Value::Object(btreemap! {
                                            "upper_limit" => 10.0,
                                            "count" => 0,
                                        }),
                                    ]))
                                ),
                            ]))
                        ),
                        ("name".into(), "test.name".into()),
                        ("description".into(), "test_description".into()),
                        ("unit".into(), "GiBy/s".into()),
                        (
                            "bucket_counts".into(),
                            Value::Array(Vec::from([
                                Value::Integer(214),
                                Value::Integer(6),
                                Value::Integer(1),
                                Value::Integer(1),
                                Value::Integer(2),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                            ]))
                        ),
                        (
                            "explicit_bounds".into(),
                            Value::Array(Vec::from([
                                from_f64_or_zero(0.005),
                                from_f64_or_zero(0.01),
                                from_f64_or_zero(0.025),
                                from_f64_or_zero(0.05),
                                from_f64_or_zero(0.075),
                                from_f64_or_zero(0.1),
                                from_f64_or_zero(0.25),
                                from_f64_or_zero(0.5),
                                from_f64_or_zero(0.75),
                                from_f64_or_zero(1.0),
                                from_f64_or_zero(2.5),
                                from_f64_or_zero(5.0),
                                from_f64_or_zero(7.5),
                                from_f64_or_zero(10.0),
                            ]))
                        ),
                        (
                            "exemplars".into(),
                            Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                (
                                    "filtered_attributes".into(),
                                    Value::Object(BTreeMap::from([
                                        ("foo".into(), "bar".into()),
                                        ("empty".into(), Value::Null),
                                    ]))
                                ),
                                ("span_id".into(), "74657374".into()),
                                (
                                    "time_unix".into(),
                                    Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(
                                                1_579_134_612_i64,
                                                11_u32
                                            )
                                            .expect("timestamp should be a valid timestamp"),
                                        )
                                    )
                                ),
                                ("trace_id".into(), "74657374".into()),
                                ("value".into(), from_f64_or_zero(10.5)),
                            ]))]))
                        ),
                        ("flags".into(), Value::Integer(1)),
                        (
                            "start_time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        (
                            "time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        ("max".into(), from_f64_or_zero(9.9)),
                        ("min".into(), from_f64_or_zero(0.1)),
                        ("aggregation_temporality".into(), Value::Integer(2)),
                    ]))
                ),
            ]))
        );

        let log = metrics[0].clone().into_log();

        let metadata = log.value().get("metadata").unwrap();

        let uniq_id = metadata.get("resource.uniq_id");

        assert!(uniq_id.is_some());

        assert_eq!(
            *metadata.deref(),
            Value::Object(BTreeMap::from([
                ("original_type".into(), "histogram".into()),
                ("data_provider".into(), "otlp".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("uniq_id".into(), uniq_id.unwrap().clone()),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), "1.2.3".into()),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        let metric =
            to_metric(&metrics[0].clone().into_log()).expect("Failed to convert lot to metric");

        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedHistogram {
                buckets: vec![
                    Bucket {
                        upper_limit: 0.005,
                        count: 214
                    },
                    Bucket {
                        upper_limit: 0.01,
                        count: 6
                    },
                    Bucket {
                        upper_limit: 0.025,
                        count: 1
                    },
                    Bucket {
                        upper_limit: 0.05,
                        count: 1
                    },
                    Bucket {
                        upper_limit: 0.075,
                        count: 2
                    },
                    Bucket {
                        upper_limit: 0.1,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 0.25,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 0.5,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 0.75,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 1.0,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 2.5,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 5.0,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 7.5,
                        count: 0
                    },
                    Bucket {
                        upper_limit: 10.0,
                        count: 0
                    },
                ],
                count: 10,
                sum: 3.7,
            }
        );
        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.tags().unwrap().get("foo").unwrap(), "bar");
        assert_eq!(
            metric.timestamp().unwrap(),
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1_579_134_612, 11).unwrap(),)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_exponential_histogram() {
        use opentelemetry_rs::opentelemetry::metrics::{
            AggregationTemporality, Exemplar, ExemplarOneOfvalue, ExponentialHistogram,
            ExponentialHistogramDataPoint, ExponentialHistogramDataPointBuckets,
            ExportMetricsServiceRequest, InstrumentationScope, Resource, ResourceMetrics,
            ScopeMetrics,
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test.name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::exponential_histogram(ExponentialHistogram {
                            data_points: vec![ExponentialHistogramDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                count: 10,
                                sum: 3.7_f64,
                                scale: 10,
                                zero_count: 12,
                                positive: Some(ExponentialHistogramDataPointBuckets {
                                    offset: 1,
                                    bucket_counts: vec![
                                        1_579_134_612_000_000_011,
                                        9_223_372_036_854_775_807,
                                    ],
                                }),
                                negative: Some(ExponentialHistogramDataPointBuckets {
                                    offset: 1,
                                    bucket_counts: vec![
                                        1_579_134_612_000_000_011,
                                        9_223_372_036_854_775_807,
                                    ],
                                }),
                                flags: 1,
                                exemplars: vec![Exemplar {
                                    filtered_attributes: vec![
                                        key_value_str.clone(),
                                        key_value_empty_str.clone(),
                                    ],
                                    time_unix_nano: 1_579_134_612_000_000_011,
                                    value: ExemplarOneOfvalue::as_int(10),
                                    span_id: Cow::from("test".as_bytes()),
                                    trace_id: Cow::from("test".as_bytes()),
                                }],
                                min: 0.1_f64,
                                max: 9.9_f64,
                                zero_threshold: 3.3_f64,
                            }],
                            aggregation_temporality:
                                AggregationTemporality::AGGREGATION_TEMPORALITY_CUMULATIVE,
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());
        assert!(metrics.is_empty());

        // TODO LOG-19820 Exponential histogram has to be converted to
        // a native histogram to be able to be handled by any metric sinks.
        // For now we just skip this exponential histogram metrics.
        // assert_eq!(
        //     *metrics[0]
        //         .clone()
        //         .into_log()
        //         .value()
        //         .get("message")
        //         .unwrap()
        //         .deref(),
        //     Value::Object(BTreeMap::from([
        //         ("kind".into(), "absolute".into()),
        //         ("name".into(), "test_name".into()),
        //         (
        //             "tags".into(),
        //             Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
        //         ),
        //         (
        //             "value".into(),
        //             Value::Object(BTreeMap::from([
        //                 ("type".into(), "exponential_histogram".into()),
        //                 ("value".into(), Value::Object(BTreeMap::from([]))),
        //                 ("name".into(), "test.name".into()),
        //                 ("description".into(), "test_description".into()),
        //                 ("unit".into(), "GiBy/s".into()),
        //                 ("count".into(), Value::Integer(10)),
        //                 (
        //                     "exemplars".into(),
        //                     Value::Array(Vec::from([Value::Object(BTreeMap::from([
        //                         (
        //                             "filtered_attributes".into(),
        //                             Value::Object(BTreeMap::from([
        //                                 ("foo".into(), "bar".into()),
        //                                 ("empty".into(), Value::Null),
        //                             ]))
        //                         ),
        //                         ("span_id".into(), "74657374".into()),
        //                         (
        //                             "time_unix_nano".into(),
        //                             Value::Integer(1_579_134_612_000_000_011)
        //                         ),
        //                         ("trace_id".into(), "74657374".into()),
        //                         ("value".into(), Value::Integer(10)),
        //                     ]))]))
        //                 ),
        //                 ("flags".into(), Value::Integer(1)),
        //                 ("max".into(), from_f64_or_zero(9.9)),
        //                 ("min".into(), from_f64_or_zero(0.1)),
        //                 (
        //                     "positive".into(),
        //                     Value::Object(BTreeMap::from([
        //                         (
        //                             "bucket_counts".into(),
        //                             Value::Array(Vec::from([
        //                                 Value::Integer(1_579_134_612_000_000_011),
        //                                 Value::Integer(9_223_372_036_854_775_807),
        //                             ]))
        //                         ),
        //                         ("offset".into(), Value::Integer(1)),
        //                     ]))
        //                 ),
        //                 (
        //                     "negative".into(),
        //                     Value::Object(BTreeMap::from([
        //                         // TODO This should be Vec<u64> but Value::Integer is i64
        //                         //  All u64 fields should be converted into Value::Float
        //                         (
        //                             "bucket_counts".into(),
        //                             Value::Array(Vec::from([
        //                                 Value::Integer(1_579_134_612_000_000_011),
        //                                 Value::Integer(9_223_372_036_854_775_807),
        //                             ]))
        //                         ),
        //                         ("offset".into(), Value::Integer(1)),
        //                     ]))
        //                 ),
        //                 ("scale".into(), Value::Integer(10)),
        //                 ("sum".into(), from_f64_or_zero(3.7)),
        //                 (
        //                     "start_time_unix_nano".into(),
        //                     Value::Integer(1_579_134_612_000_000_011)
        //                 ),
        //                 (
        //                     "time_unix_nano".into(),
        //                     Value::Integer(1_579_134_612_000_000_011)
        //                 ),
        //                 ("zero_count".into(), Value::Integer(12)),
        //                 ("zero_threshold".into(), from_f64_or_zero(3.3)),
        //                 ("aggregation_temporality".into(), Value::Integer(2)),
        //             ]))
        //         ),
        //     ]))
        // );

        // assert_eq!(
        //     *metrics[0]
        //         .clone()
        //         .into_log()
        //         .value()
        //         .get("metadata")
        //         .unwrap()
        //         .deref(),
        //     Value::Object(BTreeMap::from([
        //         (
        //             "resource".into(),
        //             Value::Object(BTreeMap::from([
        //                 (
        //                     "attributes".into(),
        //                     Value::Object(BTreeMap::from([
        //                         ("foo".into(), "bar".into()),
        //                         ("empty".into(), Value::Null),
        //                     ]))
        //                 ),
        //                 ("dropped_attributes_count".into(), Value::Integer(10)),
        //             ]))
        //         ),
        //         (
        //             "scope".into(),
        //             Value::Object(BTreeMap::from([
        //                 (
        //                     "attributes".into(),
        //                     Value::Object(BTreeMap::from([
        //                         ("foo".into(), "bar".into()),
        //                         ("empty".into(), Value::Null),
        //                     ]))
        //                 ),
        //                 ("dropped_attributes_count".into(), Value::Integer(10)),
        //                 ("name".into(), "test_name".into()),
        //                 ("version".into(), "1.2.3".into()),
        //             ]))
        //         ),
        //         (
        //             "attributes".into(),
        //             Value::Object(BTreeMap::from([
        //                 ("foo".into(), "bar".into()),
        //                 ("empty".into(), Value::Null),
        //             ]))
        //         ),
        //     ]))
        // );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_summary() {
        use opentelemetry_rs::opentelemetry::metrics::{
            ExportMetricsServiceRequest, InstrumentationScope, Resource, ResourceMetrics,
            ScopeMetrics, Summary, SummaryDataPoint, SummaryDataPointValueAtQuantile,
        };

        let key_value_str = KeyValue {
            key: Cow::from("foo"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("bar")),
            }),
        };
        let key_value_empty_str = KeyValue {
            key: Cow::from("empty"),
            value: Some(AnyValue {
                value: AnyValueOneOfvalue::string_value(Cow::from("")),
            }),
        };

        let metrics_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                    dropped_attributes_count: 10,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: Cow::from("test_name"),
                        version: Cow::from("1.2.3"),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test.name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("GiBy/s"),
                        data: MetricOneOfdata::summary(Summary {
                            data_points: vec![SummaryDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                count: 10,
                                sum: 3.7_f64,
                                quantile_values: vec![SummaryDataPointValueAtQuantile {
                                    quantile: 1.0_f64,
                                    value: 2.0_f64,
                                }],
                                flags: 1,
                            }],
                        }),
                    }],
                    schema_url: Cow::from("https://some_url.com"),
                }],
                schema_url: Cow::from("https://some_url.com"),
            }],
        };

        let metrics = to_events(metrics_data.clone());

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("message")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
                ("kind".into(), "absolute".into()),
                ("name".into(), "test_name_gibibytes_per_second".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "summary".into()),
                        (
                            "value".into(),
                            Value::Object(BTreeMap::from([
                                ("count".into(), Value::Integer(10)),
                                ("sum".into(), from_f64_or_zero(3.7)),
                                (
                                    "quantiles".into(),
                                    Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                        ("quantile".into(), from_f64_or_zero(1.0)),
                                        ("value".into(), from_f64_or_zero(2.0)),
                                    ]))]))
                                ),
                            ]))
                        ),
                        ("name".into(), "test.name".into()),
                        ("description".into(), "test_description".into()),
                        ("unit".into(), "GiBy/s".into()),
                        ("count".into(), Value::Integer(10)),
                        ("flags".into(), Value::Integer(1)),
                        (
                            "quantile_values".into(),
                            Value::Array(Vec::from([Value::Object(BTreeMap::from([
                                ("quantile".into(), from_f64_or_zero(1.0)),
                                ("value".into(), from_f64_or_zero(2.0)),
                            ]))]))
                        ),
                        ("sum".into(), from_f64_or_zero(3.7)),
                        (
                            "start_time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                        (
                            "time_unix".into(),
                            Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            )
                        ),
                    ]))
                ),
            ]))
        );

        let log = metrics[0].clone().into_log();

        let metadata = log.value().get("metadata").unwrap();

        let uniq_id = metadata.get("resource.uniq_id");

        assert!(uniq_id.is_some());

        assert_eq!(
            *metadata.deref(),
            Value::Object(BTreeMap::from([
                ("original_type".into(), "summary".into()),
                ("data_provider".into(), "otlp".into()),
                (
                    "resource".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("uniq_id".into(), uniq_id.unwrap().clone()),
                    ]))
                ),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        (
                            "attributes".into(),
                            Value::Object(BTreeMap::from([
                                ("foo".into(), "bar".into()),
                                ("empty".into(), Value::Null),
                            ]))
                        ),
                        ("dropped_attributes_count".into(), Value::Integer(10)),
                        ("name".into(), "test_name".into()),
                        ("version".into(), "1.2.3".into()),
                    ]))
                ),
                (
                    "attributes".into(),
                    Value::Object(BTreeMap::from([
                        ("foo".into(), "bar".into()),
                        ("empty".into(), Value::Null),
                    ]))
                ),
            ]))
        );

        let metric =
            to_metric(&metrics[0].clone().into_log()).expect("Failed to convert lot to metric");

        assert_eq!(
            metric.value(),
            &MetricValue::AggregatedSummary {
                quantiles: vec![Quantile {
                    quantile: 1.0,
                    value: 2.0
                },],
                count: 10,
                sum: 3.7,
            }
        );
        assert_eq!(metric.kind(), MetricKind::Absolute);
        assert_eq!(metric.tags().unwrap().get("foo").unwrap(), "bar");
        assert_eq!(
            metric.timestamp().unwrap(),
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1_579_134_612, 11).unwrap(),)
        );
    }

    #[test]
    fn otlp_metrics_deserialize_parse_request() {
        let out: &[u8] = b"\n\xa7\x02\n\xb8\x01\n)\n\x11service.namespace\x12\x14\n\x12opentelemetry-demo\n!\n\x0cservice.name\x12\x11\n\x0fcurrencyservice\n \n\x15telemetry.sdk.version\x12\x07\n\x051.8.2\n%\n\x12telemetry.sdk.name\x12\x0f\n\ropentelemetry\n\x1f\n\x16telemetry.sdk.language\x12\x05\n\x03cpp\x12j\n\x15\n\x0capp_currency\x12\x051.3.0\x12Q\n\x14app_currency_counter:9\n3\x11\xdc\xf9\0xl\x18W\x17\x19\xb7\xa2\xa1\xb3l\x18W\x171\x02\0\0\0\0\0\0\0:\x16\n\rcurrency_code\x12\x05\n\x03USD\x10\x01\x18\x01";

        let metrics = parse_metrics_request(out).expect("Failed to parse");

        assert_eq!(metrics.len(), 1);

        let log = metrics[0].clone().into_log();
        let metric_type = log
            .get("message.value.type")
            .expect("Metric type is missed");

        let is_monotonic = log
            .get("message.value.is_monotonic")
            .expect("Metric is_monotonic is missed");

        assert_eq!(*metric_type, Value::from("counter"));
        assert_eq!(*is_monotonic, Value::from(true));
    }

    #[test]
    fn otlp_metrics_normalize_name_byte() {
        assert_eq!(
            normalize_name(
                Cow::from("system.filesystem.usage"),
                Cow::from("By"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_filesystem_usage_bytes"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_byte_counter() {
        assert_eq!(
            normalize_name(
                Cow::from("system.io"),
                Cow::from("By"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_io_bytes_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("network_transmitted_bytes_total"),
                Cow::from("By"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "network_transmitted_bytes_total"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_white_spaces() {
        assert_eq!(
            normalize_name(
                Cow::from("system.filesystem.usage       "),
                Cow::from("  By\t"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_filesystem_usage_bytes"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_non_standard_unit() {
        assert_eq!(
            normalize_name(
                Cow::from("system.network.dropped"),
                Cow::from("{packets}"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_network_dropped"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_non_standard_unit_counter() {
        assert_eq!(
            normalize_name(
                Cow::from("system.network.dropped"),
                Cow::from("{packets}"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_network_dropped_total"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_broken_unit() {
        assert_eq!(
            normalize_name(
                Cow::from("system.network.dropped"),
                Cow::from("packets"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_network_dropped_packets"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.network.packets.dropped"),
                Cow::from("packets"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_network_packets_dropped"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.network.packets"),
                Cow::from("packets"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_network_packets"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_broken_unit_counter() {
        assert_eq!(
            normalize_name(
                Cow::from("system.network.dropped"),
                Cow::from("packets"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_network_dropped_packets_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.network.packets.dropped"),
                Cow::from("packets"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_network_packets_dropped_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.network.packets"),
                Cow::from("packets"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_network_packets_total"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_ratio() {
        assert_eq!(
            normalize_name(
                Cow::from("hw.gpu.memory.utilization"),
                Cow::from("1"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "hw_gpu_memory_utilization_ratio"
        );
        assert_eq!(
            normalize_name(
                Cow::from("hw.fan.speed_ratio"),
                Cow::from("1"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "hw_fan_speed_ratio"
        );
        assert_eq!(
            normalize_name(
                Cow::from("objects"),
                Cow::from("1"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "objects_total"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_hertz() {
        assert_eq!(
            normalize_name(
                Cow::from("hw.cpu.speed_limit"),
                Cow::from("Hz"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "hw_cpu_speed_limit_hertz"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_per() {
        assert_eq!(
            normalize_name(
                Cow::from("broken.metric.speed"),
                Cow::from("km/h"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "broken_metric_speed_km_per_hour"
        );
        assert_eq!(
            normalize_name(
                Cow::from("astro.light.speed_limit"),
                Cow::from("m/s"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "astro_light_speed_limit_meters_per_second"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_percent() {
        assert_eq!(
            normalize_name(
                Cow::from("broken.metric.success_ratio"),
                Cow::from("%"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "broken_metric_success_ratio_percent"
        );
        assert_eq!(
            normalize_name(
                Cow::from("broken.metric.success_percent"),
                Cow::from("%"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "broken_metric_success_percent"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_empty() {
        assert_eq!(
            normalize_name(
                Cow::from("test.metric.no_unit"),
                Cow::from(""),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "test_metric_no_unit"
        );
        assert_eq!(
            normalize_name(
                Cow::from("test.metric.spaces"),
                Cow::from("   \t  "),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "test_metric_spaces"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_unsupported_runes() {
        assert_eq!(
            normalize_name(
                Cow::from("unsupported.metric.temperature"),
                Cow::from("°F"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "unsupported_metric_temperature_F"
        );
        assert_eq!(
            normalize_name(
                Cow::from("unsupported.metric.weird"),
                Cow::from("+=.:,!* & #"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "unsupported_metric_weird"
        );
        assert_eq!(
            normalize_name(
                Cow::from("unsupported.metric.redundant"),
                Cow::from("__test $/°C"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "unsupported_metric_redundant_test_per_C"
        );
    }

    #[test]
    fn otlp_metrics_normalize_name_otel_receivers() {
        assert_eq!(
            normalize_name(
                Cow::from("active_directory.ds.replication.network.io"),
                Cow::from("By"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "active_directory_ds_replication_network_io_bytes_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("active_directory.ds.replication.sync.object.pending"),
                Cow::from("{objects}"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "active_directory_ds_replication_sync_object_pending_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("active_directory.ds.replication.object.rate"),
                Cow::from("{objects}/s"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "active_directory_ds_replication_object_rate_per_second"
        );
        assert_eq!(
            normalize_name(
                Cow::from("active_directory.ds.name_cache.hit_rate"),
                Cow::from("%"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "active_directory_ds_name_cache_hit_rate_percent"
        );
        assert_eq!(
            normalize_name(
                Cow::from("active_directory.ds.ldap.bind.last_successful.time"),
                Cow::from("ms"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "active_directory_ds_ldap_bind_last_successful_time_milliseconds"
        );
        assert_eq!(
            normalize_name(
                Cow::from("apache.current_connections"),
                Cow::from("connections"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "apache_current_connections"
        );
        assert_eq!(
            normalize_name(
                Cow::from("apache.workers"),
                Cow::from("connections"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "apache_workers_connections"
        );
        assert_eq!(
            normalize_name(
                Cow::from("apache.requests"),
                Cow::from("1"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "apache_requests_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("bigip.virtual_server.request.count"),
                Cow::from("{requests}"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "bigip_virtual_server_request_count_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.cpu.utilization"),
                Cow::from("1"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_cpu_utilization_ratio"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.disk.operation_time"),
                Cow::from("s"),
                Cow::from("counter"),
                Cow::from("incremental")
            ),
            "system_disk_operation_time_seconds_total"
        );
        assert_eq!(
            normalize_name(
                Cow::from("system.cpu.load_average.15m"),
                Cow::from("1"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "system_cpu_load_average_15m_ratio"
        );
        assert_eq!(
            normalize_name(
                Cow::from("memcached.operation_hit_ratio"),
                Cow::from("%"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "memcached_operation_hit_ratio_percent"
        );
        assert_eq!(
            normalize_name(
                Cow::from("mongodbatlas.process.asserts"),
                Cow::from("{assertions}/s"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "mongodbatlas_process_asserts_per_second"
        );
        assert_eq!(
            normalize_name(
                Cow::from("mongodbatlas.process.journaling.data_files"),
                Cow::from("MiBy"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "mongodbatlas_process_journaling_data_files_mebibytes"
        );
        assert_eq!(
            normalize_name(
                Cow::from("mongodbatlas.process.network.io"),
                Cow::from("By/s"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "mongodbatlas_process_network_io_bytes_per_second"
        );
        assert_eq!(
            normalize_name(
                Cow::from("mongodbatlas.process.oplog.rate"),
                Cow::from("GiBy/h"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "mongodbatlas_process_oplog_rate_gibibytes_per_hour"
        );
        assert_eq!(
            normalize_name(
                Cow::from("mongodbatlas.process.db.query_targeting.scanned_per_returned"),
                Cow::from("{scanned}/{returned}"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "mongodbatlas_process_db_query_targeting_scanned_per_returned"
        );
        assert_eq!(
            normalize_name(
                Cow::from("nginx.requests"),
                Cow::from("requests"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "nginx_requests"
        );
        assert_eq!(
            normalize_name(
                Cow::from("nginx.connections_accepted"),
                Cow::from("connections"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "nginx_connections_accepted"
        );
        assert_eq!(
            normalize_name(
                Cow::from("nsxt.node.memory.usage"),
                Cow::from("KBy"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "nsxt_node_memory_usage_kilobytes"
        );
        assert_eq!(
            normalize_name(
                Cow::from("redis.latest_fork"),
                Cow::from("us"),
                Cow::from("gauge"),
                Cow::from("absolute")
            ),
            "redis_latest_fork_microseconds"
        );
    }
}
