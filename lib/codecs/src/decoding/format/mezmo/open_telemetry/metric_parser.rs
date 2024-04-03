use chrono::{NaiveDateTime, TimeZone, Utc};
use smallvec::SmallVec;
use std::borrow::Cow;

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
            from_f64_or_zero, IntoTagValue, IntoValue, MetricTags, MetricTagsAccessor,
            MetricToLogEvent, MetricValueAccessor, MetricValuePairs, MetricValueSerializable,
            MezmoMetric,
        },
        Event, LogEvent, MetricKind, Value,
    },
};

use crate::decoding::format::mezmo::open_telemetry::{DeserializerError, OpenTelemetryKeyValue};

const NANO_RATIO: i64 = 1_000_000_000;
const METRIC_TIMESTAMP_KEY: &str = "message.value.value.time_unix_nano";

#[derive(Debug, Default, PartialEq)]
pub struct GaugeMetricValue<'a> {
    pub value: NumberDataPointOneOfValue,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub flags: u32,
}

impl<'a> GaugeMetricValue<'a> {
    fn new(
        gauge_metric: NumberDataPoint<'a>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
    ) -> Self {
        GaugeMetricValue {
            value: NumberDataPointOneOfValue {
                value: gauge_metric.value,
            },
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: gauge_metric.exemplars,
            },
            start_time_unix_nano: gauge_metric.start_time_unix_nano,
            time_unix_nano: gauge_metric.time_unix_nano,
            flags: gauge_metric.flags,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for GaugeMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 7>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("gauge"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (&"value" as &dyn ToString, &self.value as &dyn IntoValue),
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
                    &"start_time_unix_nano" as &dyn ToString,
                    &self.start_time_unix_nano as &dyn IntoValue,
                ),
                (
                    &"time_unix_nano" as &dyn ToString,
                    &self.time_unix_nano as &dyn IntoValue,
                ),
                (&"flags" as &dyn ToString, &self.flags as &dyn IntoValue),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct GaugeMetricMetadata<'a> {
    pub resource: ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> GaugeMetricMetadata<'a> {
    fn new(
        gauge_metric: NumberDataPoint<'a>,
        resource: ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        GaugeMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: gauge_metric.attributes,
            },
        }
    }
}

impl<'a> MetricValueAccessor<'a> for GaugeMetricMetadata<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    &self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SumMetricValue<'a> {
    pub value: NumberDataPointOneOfValue,
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub flags: u32,
    pub is_monotonic: bool,
    pub aggregation_temporality: i32,
}

impl<'a> SumMetricValue<'a> {
    fn new(
        sum_metric: NumberDataPoint<'a>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
        is_monotonic: bool,
    ) -> Self {
        SumMetricValue {
            value: NumberDataPointOneOfValue {
                value: sum_metric.value,
            },
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: sum_metric.exemplars,
            },
            start_time_unix_nano: sum_metric.start_time_unix_nano,
            time_unix_nano: sum_metric.time_unix_nano,
            flags: sum_metric.flags,
            is_monotonic,
            aggregation_temporality: aggregation_temporality as i32,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        if self.aggregation_temporality
            == AggregationTemporality::AGGREGATION_TEMPORALITY_CUMULATIVE as i32
        {
            &MetricKind::Incremental
        } else {
            &MetricKind::Absolute
        }
    }
}

impl<'a> MetricValueAccessor<'a> for SumMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 9>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("sum"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (&"value" as &dyn ToString, &self.value as &dyn IntoValue),
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
                    &"start_time_unix_nano" as &dyn ToString,
                    &self.start_time_unix_nano as &dyn IntoValue,
                ),
                (
                    &"time_unix_nano" as &dyn ToString,
                    &self.time_unix_nano as &dyn IntoValue,
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
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SumMetricMetadata<'a> {
    pub resource: ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> SumMetricMetadata<'a> {
    fn new(
        sum_metric: NumberDataPoint<'a>,
        resource: ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        SumMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: sum_metric.attributes,
            },
        }
    }
}

impl<'a> MetricValueAccessor<'a> for SumMetricMetadata<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    &self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct HistogramMetricValue<'a> {
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: f64,
    pub bucket_counts: Cow<'a, [u64]>,
    pub explicit_bounds: Cow<'a, [f64]>,
    pub flags: u32,
    pub min: f64,
    pub max: f64,
    pub aggregation_temporality: i32,
}

impl<'a> HistogramMetricValue<'a> {
    fn new(
        histogram_metric: HistogramDataPoint<'a>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
    ) -> Self {
        HistogramMetricValue {
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: histogram_metric.exemplars,
            },
            start_time_unix_nano: histogram_metric.start_time_unix_nano,
            time_unix_nano: histogram_metric.time_unix_nano,
            count: histogram_metric.count,
            sum: histogram_metric.sum,
            bucket_counts: histogram_metric.bucket_counts,
            explicit_bounds: histogram_metric.explicit_bounds,
            flags: histogram_metric.flags,
            min: histogram_metric.min,
            max: histogram_metric.max,
            aggregation_temporality: aggregation_temporality as i32,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        if self.aggregation_temporality
            == AggregationTemporality::AGGREGATION_TEMPORALITY_CUMULATIVE as i32
        {
            &MetricKind::Incremental
        } else {
            &MetricKind::Absolute
        }
    }
}

impl<'a> MetricValueAccessor<'a> for HistogramMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 13>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("histogram"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
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
                    &"start_time_unix_nano" as &dyn ToString,
                    &self.start_time_unix_nano as &dyn IntoValue,
                ),
                (
                    &"time_unix_nano" as &dyn ToString,
                    &self.time_unix_nano as &dyn IntoValue,
                ),
                (&"count" as &dyn ToString, &self.count as &dyn IntoValue),
                (&"sum" as &dyn ToString, &self.sum as &dyn IntoValue),
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
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct HistogramMetricMetadata<'a> {
    pub resource: ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> HistogramMetricMetadata<'a> {
    fn new(
        histogram_metric: HistogramDataPoint<'a>,
        resource: ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        HistogramMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: histogram_metric.attributes,
            },
        }
    }
}

impl<'a> MetricValueAccessor<'a> for HistogramMetricMetadata<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    &self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ExponentialHistogramMetricValue<'a> {
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub exemplars: ExemplarsMetricValue<'a>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
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

impl<'a> ExponentialHistogramMetricValue<'a> {
    fn new(
        exp_histogram_metric: ExponentialHistogramDataPoint<'a>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
        aggregation_temporality: AggregationTemporality,
    ) -> Self {
        ExponentialHistogramMetricValue {
            description,
            unit,
            exemplars: ExemplarsMetricValue {
                exemplars: exp_histogram_metric.exemplars,
            },
            start_time_unix_nano: exp_histogram_metric.start_time_unix_nano,
            time_unix_nano: exp_histogram_metric.time_unix_nano,
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

    fn kind(&'a self) -> &'a MetricKind {
        if self.aggregation_temporality
            == AggregationTemporality::AGGREGATION_TEMPORALITY_CUMULATIVE as i32
        {
            &MetricKind::Incremental
        } else {
            &MetricKind::Absolute
        }
    }
}

impl<'a> MetricValueAccessor<'a> for ExponentialHistogramMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 16>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("exponential_histogram"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
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
                    &"start_time_unix_nano" as &dyn ToString,
                    &self.start_time_unix_nano as &dyn IntoValue,
                ),
                (
                    &"time_unix_nano" as &dyn ToString,
                    &self.time_unix_nano as &dyn IntoValue,
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
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ExponentialHistogramMetricMetadata<'a> {
    pub resource: ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> ExponentialHistogramMetricMetadata<'a> {
    fn new(
        exp_histogram_metric: ExponentialHistogramDataPoint<'a>,
        resource: ResourceMetricValue<'a>,
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

impl<'a> MetricValueAccessor<'a> for ExponentialHistogramMetricMetadata<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    &self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SummaryMetricValue<'a> {
    pub description: Cow<'a, str>,
    pub unit: Cow<'a, str>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: f64,
    pub quantile_values: QuantileValuesMetricValue,
    pub flags: u32,
}

impl<'a> SummaryMetricValue<'a> {
    fn new(
        summary_metric: SummaryDataPoint<'a>,
        description: Cow<'a, str>,
        unit: Cow<'a, str>,
    ) -> Self {
        SummaryMetricValue {
            description,
            unit,
            start_time_unix_nano: summary_metric.start_time_unix_nano,
            time_unix_nano: summary_metric.time_unix_nano,
            count: summary_metric.count,
            sum: summary_metric.sum,
            quantile_values: QuantileValuesMetricValue {
                quantile_values: summary_metric.quantile_values,
            },
            flags: summary_metric.flags,
        }
    }

    fn kind(&'a self) -> &'a MetricKind {
        &MetricKind::Absolute
    }
}

impl<'a> MetricValueAccessor<'a> for SummaryMetricValue<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 8>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        Some(Cow::from("summary"))
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"description" as &dyn ToString,
                    &self.description as &dyn IntoValue,
                ),
                (&"unit" as &dyn ToString, &self.unit as &dyn IntoValue),
                (
                    &"start_time_unix_nano" as &dyn ToString,
                    &self.start_time_unix_nano as &dyn IntoValue,
                ),
                (
                    &"time_unix_nano" as &dyn ToString,
                    &self.time_unix_nano as &dyn IntoValue,
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
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SummaryMetricMetadata<'a> {
    pub resource: ResourceMetricValue<'a>,
    pub scope: ScopeMetricValue<'a>,
    pub attributes: OpenTelemetryKeyValue<'a>,
}

impl<'a> SummaryMetricMetadata<'a> {
    fn new(
        summary_metric: SummaryDataPoint<'a>,
        resource: ResourceMetricValue<'a>,
        scope: ScopeMetricValue<'a>,
    ) -> Self {
        SummaryMetricMetadata {
            resource,
            scope,
            attributes: OpenTelemetryKeyValue {
                attributes: summary_metric.attributes,
            },
        }
    }
}

impl<'a> MetricValueAccessor<'a> for SummaryMetricMetadata<'_> {
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    fn value(&'a self) -> MetricValueSerializable<'_, Self::ArrIter, Self::ObjIter> {
        MetricValueSerializable::Object(MetricValuePairs {
            elements: [
                (
                    &"resource" as &dyn ToString,
                    &self.resource as &dyn IntoValue,
                ),
                (&"scope" as &dyn ToString, &self.scope as &dyn IntoValue),
                (
                    &"attributes" as &dyn ToString,
                    &self.attributes as &dyn IntoValue,
                ),
            ]
            .into_iter(),
        })
    }
}

#[derive(Debug, Default, PartialEq)]
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

        let name = self.name.as_ref().unwrap().to_string();
        let version = self.version.as_ref().unwrap().to_string();

        Value::Object(
            [
                (
                    "name".into(),
                    if !name.is_empty() {
                        name.into()
                    } else {
                        Value::Null
                    },
                ),
                (
                    "version".into(),
                    if !version.is_empty() {
                        version.into()
                    } else {
                        Value::Null
                    },
                ),
                ("attributes".into(), attributes.to_value()),
                (
                    "dropped_attributes_count".into(),
                    self.dropped_attributes_count.into(),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ResourceMetricValue<'a> {
    pub attributes: OpenTelemetryKeyValue<'a>,
    pub dropped_attributes_count: Option<u32>,
}

impl<'a> ResourceMetricValue<'a> {
    fn new(opt: Option<Resource<'a>>) -> Self {
        match opt {
            Some(resource) => ResourceMetricValue {
                attributes: OpenTelemetryKeyValue {
                    attributes: resource.attributes,
                },
                dropped_attributes_count: Some(resource.dropped_attributes_count),
            },
            None => ResourceMetricValue {
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
        Value::Object(
            [
                ("attributes".into(), self.attributes.to_value()),
                (
                    "dropped_attributes_count".into(),
                    self.dropped_attributes_count.into(),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct QuantileValuesMetricValue {
    pub quantile_values: Vec<SummaryDataPointValueAtQuantile>,
}

impl IntoValue for QuantileValuesMetricValue {
    fn to_value(&self) -> Value {
        Value::Array(
            self.quantile_values
                .iter()
                .map(|quantile_value| {
                    Value::Object(
                        [
                            ("quantile".into(), from_f64_or_zero(quantile_value.quantile)),
                            ("value".into(), from_f64_or_zero(quantile_value.value)),
                        ]
                        .into_iter()
                        .collect(),
                    )
                })
                .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
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

                    Value::Object(
                        [
                            ("filtered_attributes".into(), filtered_attributes.to_value()),
                            ("value".into(), exemplar_value),
                            ("time_unix_nano".into(), exemplar.time_unix_nano.into()),
                            (
                                "span_id".into(),
                                Value::from(faster_hex::hex_string(&exemplar.span_id)),
                            ),
                            (
                                "trace_id".into(),
                                Value::from(faster_hex::hex_string(&exemplar.trace_id)),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )
                })
                .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
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
        Value::Object(
            [
                ("offset".into(), self.offset.into()),
                (
                    "bucket_counts".into(),
                    Value::Array(
                        self.bucket_counts
                            .iter()
                            .map(|count| Value::from(*count))
                            .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
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

    fn tags(&'a self) -> MetricTags<'a, Self::Iter> {
        MetricTags {
            tags: self
                .tags
                .iter()
                .filter_map(|key_value| match &key_value.value {
                    Some(AnyValue {
                        value: AnyValueOneOfvalue::string_value(val),
                    }) if val.is_empty() => None,
                    Some(ref any_value) => Some((
                        &key_value.key as &'a dyn ToString,
                        &any_value.value as &'a dyn IntoTagValue,
                    )),
                    None => None,
                }),
        }
    }
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

    for resource_metric in metric_request.resource_metrics {
        let tags = MetricTagsWrapper {
            tags: &resource_metric.resource.clone().unwrap().attributes,
        };

        for scope_metric in resource_metric.scope_metrics {
            for metric in scope_metric.metrics {
                match metric.data {
                    MetricOneOfdata::gauge(gauge) => gauge
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value = GaugeMetricValue::new(
                                data_point.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                            );

                            let metric_metadata = GaugeMetricMetadata::new(
                                data_point.clone(),
                                ResourceMetricValue::new(resource_metric.resource.clone()),
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
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
                            let metric_value = SumMetricValue::new(
                                data_point.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                sum.aggregation_temporality,
                                sum.is_monotonic,
                            );

                            let metric_metadata = SumMetricMetadata::new(
                                data_point.clone(),
                                ResourceMetricValue::new(resource_metric.resource.clone()),
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
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
                            let metric_value = HistogramMetricValue::new(
                                data_point.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                histogram.aggregation_temporality,
                            );

                            let metric_metadata = HistogramMetricMetadata::new(
                                data_point.clone(),
                                ResourceMetricValue::new(resource_metric.resource.clone()),
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
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
                            let metric_value = ExponentialHistogramMetricValue::new(
                                data_point.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                                exp_histogram.aggregation_temporality,
                            );

                            let metric_metadata = ExponentialHistogramMetricMetadata::new(
                                data_point.clone(),
                                ResourceMetricValue::new(resource_metric.resource.clone()),
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
                                        value: &metric_value,
                                    }
                                }
                                .to_log_event(),
                            ));
                        })
                        .collect(),
                    MetricOneOfdata::summary(summary) => summary
                        .data_points
                        .iter()
                        .map(|data_point| {
                            let metric_value = SummaryMetricValue::new(
                                data_point.clone(),
                                metric.description.clone(),
                                metric.unit.clone(),
                            );

                            let metric_metadata = SummaryMetricMetadata::new(
                                data_point.clone(),
                                ResourceMetricValue::new(resource_metric.resource.clone()),
                                ScopeMetricValue::new(scope_metric.scope.clone()),
                            );

                            out.push(make_event(
                                {
                                    MezmoMetric {
                                        name: metric.name.clone(),
                                        namespace: None,
                                        kind: metric_value.kind(),
                                        tags: Some(&tags),
                                        user_metadata: Some(&metric_metadata),
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
    if let Some(timestamp_key) = log_schema().timestamp_key() {
        let metric_timestamp_target = (lookup::PathPrefix::Event, METRIC_TIMESTAMP_KEY);
        let timestamp = match log_event.get(metric_timestamp_target) {
            Some(ts) => {
                let ts = ts.as_integer().unwrap();
                let ms: i64 = ts / NANO_RATIO;
                let nanos: u32 = (ts % NANO_RATIO) as u32;
                Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_opt(ms, nanos)
                        .expect("timestamp should be a valid timestamp"),
                )
            }
            None => Utc::now(),
        };

        log_event.insert((lookup::PathPrefix::Event, timestamp_key), timestamp);
    }

    log_event.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::ops::Deref;

    use opentelemetry_rs::opentelemetry::metrics::{
        AnyValue, AnyValueOneOfvalue, ExportMetricsServiceRequest, KeyValue, Metric,
        MetricOneOfdata,
    };
    use std::borrow::Cow;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_gauge() {
        use opentelemetry_rs::opentelemetry::metrics::{
            Exemplar, ExemplarOneOfvalue, Gauge, InstrumentationScope, NumberDataPoint,
            NumberDataPointOneOfvalue, Resource, ResourceMetrics, ScopeMetrics,
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
                        version: Cow::from(""),
                        attributes: vec![key_value_str.clone(), key_value_empty_str.clone()],
                        dropped_attributes_count: 10,
                    }),
                    metrics: vec![Metric {
                        name: Cow::from("test_name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("123.[psi]"),
                        data: MetricOneOfdata::gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    key_value_str.clone(),
                                    key_value_empty_str.clone(),
                                ],
                                start_time_unix_nano: 1_579_134_612_000_000_011,
                                time_unix_nano: 1_579_134_612_000_000_011,
                                value: NumberDataPointOneOfvalue::as_int(10),
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
                ("name".into(), "test_name".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "gauge".into()),
                        (
                            "value".into(),
                            Value::Object(BTreeMap::from([
                                ("value".into(), Value::Integer(10)),
                                ("description".into(), "test_description".into()),
                                ("unit".into(), "123.[psi]".into()),
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
                                            "time_unix_nano".into(),
                                            Value::Integer(1_579_134_612_000_000_011)
                                        ),
                                        ("trace_id".into(), "74657374".into()),
                                        ("value".into(), Value::Integer(10)),
                                    ]))]))
                                ),
                                ("flags".into(), Value::Integer(1)),
                                (
                                    "start_time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                (
                                    "time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                            ]))
                        ),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
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
                        ("version".into(), Value::Null),
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
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn otlp_metrics_deserialize_sum() {
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
                        name: Cow::from("test_name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("123.[psi]"),
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
                ("kind".into(), "absolute".into()),
                ("name".into(), "test_name".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "sum".into()),
                        (
                            "value".into(),
                            Value::Object(BTreeMap::from([
                                ("description".into(), "test_description".into()),
                                ("unit".into(), "123.[psi]".into()),
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
                                            "time_unix_nano".into(),
                                            Value::Integer(1_579_134_612_000_000_011)
                                        ),
                                        ("trace_id".into(), "74657374".into()),
                                        ("value".into(), Value::Integer(10)),
                                    ]))]))
                                ),
                                ("flags".into(), Value::Integer(1)),
                                (
                                    "start_time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                (
                                    "time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                ("value".into(), from_f64_or_zero(10_f64)),
                                ("aggregation_temporality".into(), Value::Integer(0)),
                                ("is_monotonic".into(), Value::Boolean(true)),
                            ]))
                        ),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
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
                        name: Cow::from("test_name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("123.[psi]"),
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
                                bucket_counts: Cow::from(vec![1, 2, 3]),
                                explicit_bounds: Cow::from(vec![1.3_f64, 5.9_f64]),
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
                ("kind".into(), "incremental".into()),
                ("name".into(), "test_name".into()),
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
                                ("description".into(), "test_description".into()),
                                ("unit".into(), "123.[psi]".into()),
                                (
                                    "bucket_counts".into(),
                                    Value::Array(Vec::from([
                                        Value::Integer(1),
                                        Value::Integer(2),
                                        Value::Integer(3),
                                    ]))
                                ),
                                ("count".into(), Value::Integer(10)),
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
                                            "time_unix_nano".into(),
                                            Value::Integer(1_579_134_612_000_000_011)
                                        ),
                                        ("trace_id".into(), "74657374".into()),
                                        ("value".into(), from_f64_or_zero(10.5)),
                                    ]))]))
                                ),
                                (
                                    "explicit_bounds".into(),
                                    Value::Array(Vec::from([
                                        from_f64_or_zero(1.3),
                                        from_f64_or_zero(5.9),
                                    ]))
                                ),
                                ("flags".into(), Value::Integer(1)),
                                (
                                    "start_time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                (
                                    "time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                ("max".into(), from_f64_or_zero(9.9)),
                                ("min".into(), from_f64_or_zero(0.1)),
                                ("sum".into(), from_f64_or_zero(3.7)),
                                ("aggregation_temporality".into(), Value::Integer(2)),
                            ]))
                        ),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
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
                        name: Cow::from("test_name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("123.[psi]"),
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
                ("name".into(), "test_name".into()),
                (
                    "tags".into(),
                    Value::Object(BTreeMap::from([("foo".into(), "bar".into()),]))
                ),
                (
                    "value".into(),
                    Value::Object(BTreeMap::from([
                        ("type".into(), "exponential_histogram".into()),
                        (
                            "value".into(),
                            Value::Object(BTreeMap::from([
                                ("description".into(), "test_description".into()),
                                ("unit".into(), "123.[psi]".into()),
                                ("count".into(), Value::Integer(10)),
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
                                            "time_unix_nano".into(),
                                            Value::Integer(1_579_134_612_000_000_011)
                                        ),
                                        ("trace_id".into(), "74657374".into()),
                                        ("value".into(), Value::Integer(10)),
                                    ]))]))
                                ),
                                ("flags".into(), Value::Integer(1)),
                                ("max".into(), from_f64_or_zero(9.9)),
                                ("min".into(), from_f64_or_zero(0.1)),
                                (
                                    "positive".into(),
                                    Value::Object(BTreeMap::from([
                                        (
                                            "bucket_counts".into(),
                                            Value::Array(Vec::from([
                                                Value::Integer(1_579_134_612_000_000_011),
                                                Value::Integer(9_223_372_036_854_775_807),
                                            ]))
                                        ),
                                        ("offset".into(), Value::Integer(1)),
                                    ]))
                                ),
                                (
                                    "negative".into(),
                                    Value::Object(BTreeMap::from([
                                        // TODO This should be Vec<u64> but Value::Integer is i64
                                        //  All u64 fields should be converted into Value::Float
                                        (
                                            "bucket_counts".into(),
                                            Value::Array(Vec::from([
                                                Value::Integer(1_579_134_612_000_000_011),
                                                Value::Integer(9_223_372_036_854_775_807),
                                            ]))
                                        ),
                                        ("offset".into(), Value::Integer(1)),
                                    ]))
                                ),
                                ("scale".into(), Value::Integer(10)),
                                ("sum".into(), from_f64_or_zero(3.7)),
                                (
                                    "start_time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                (
                                    "time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                ("zero_count".into(), Value::Integer(12)),
                                ("zero_threshold".into(), from_f64_or_zero(3.3)),
                                ("aggregation_temporality".into(), Value::Integer(2)),
                            ]))
                        ),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
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
                        name: Cow::from("test_name"),
                        description: Cow::from("test_description"),
                        unit: Cow::from("123.[psi]"),
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
                ("name".into(), "test_name".into()),
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
                                ("description".into(), "test_description".into()),
                                ("unit".into(), "123.[psi]".into()),
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
                                    "start_time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                                (
                                    "time_unix_nano".into(),
                                    Value::Integer(1_579_134_612_000_000_011)
                                ),
                            ]))
                        ),
                    ]))
                ),
            ]))
        );

        assert_eq!(
            *metrics[0]
                .clone()
                .into_log()
                .value()
                .get("metadata")
                .unwrap()
                .deref(),
            Value::Object(BTreeMap::from([
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

        assert_eq!(*metric_type, Value::from("sum"));
    }
}
