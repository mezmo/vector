//! Converts cumulative metrics into the deltas the New Relic Metric API expects.
//!
//! The Metric API has no cumulative counter type, and its `count` and `summary` types require
//! `interval.ms`. So each delta carries the time since the previous point of the same series.
//!
//! Reference: https://docs.newrelic.com/docs/data-apis/understand-data/metric-data/metric-data-type/

use std::num::NonZeroU32;

use chrono::Utc;
use vector_lib::event::{
    EventMetadata, MetricKind, MetricValue,
    metric::{
        Metric, MetricArbitrary, MetricData, MetricSeries,
        mezmo::{mark_otel_sum_absent, otel_metric_context, otel_sum_reported},
    },
};

use crate::sinks::util::buffer::metrics::{
    CumulativePoint, MetricNormalize, MetricSet, NormalizerSettings, track_cumulative,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NewRelicNormalizerSettings;

impl NormalizerSettings for NewRelicNormalizerSettings {
    const MAX_EVENTS: Option<usize> = None;
    const MAX_BYTES: Option<usize> = None;
    const TIME_TO_LIVE: Option<u64> = Some(180);
}

#[derive(Default)]
pub(super) struct NewRelicMetricsNormalizer;

impl MetricNormalize for NewRelicMetricsNormalizer {
    fn normalize(&mut self, state: &mut MetricSet, metric: Metric) -> Option<Metric> {
        if metric.kind() == MetricKind::Incremental || !needs_delta(&metric) {
            return Some(metric);
        }

        let metric = if metric.timestamp().is_none() {
            metric.with_timestamp(Some(Utc::now()))
        } else {
            metric
        };
        let baseline = baseline(baseline_series(&metric), &metric);
        let point = track_cumulative(state, &metric, baseline)?;
        to_delta(metric, point)
    }
}

/// OpenTelemetry identifies a series by its instrumentation scope too, which `Metric::series`
/// leaves out. The scope goes in the namespace of the cache key, where no tag can collide with it.
fn baseline_series(metric: &Metric) -> MetricSeries {
    let mut series = metric.series().clone();
    if let Some(context) = otel_metric_context(metric) {
        let scope = (
            context.scope_name,
            context.scope_version,
            context.scope_attributes,
        );
        series.name.namespace = Some(format!("{:?}{scope:?}", series.name.namespace));
    }
    series
}

/// Only the time, the value and whether it has a sum are needed to derive the next delta.
fn baseline(series: MetricSeries, metric: &Metric) -> Metric {
    let mut arbitrary = MetricArbitrary::default();
    if !otel_sum_reported(metric.arbitrary_value()) {
        mark_otel_sum_absent(&mut arbitrary);
    }
    let data = MetricData {
        time: metric.data().time,
        kind: metric.kind(),
        value: metric.value().clone(),
        arbitrary,
    };
    Metric::from_parts(series, data, EventMetadata::default())
}

/// Absolute counters from other sources keep their existing `gauge` treatment so their type in
/// New Relic does not change. Histograms and summaries had no representation before, so every
/// source gets them.
fn needs_delta(metric: &Metric) -> bool {
    match metric.value() {
        MetricValue::Counter { .. } => otel_metric_context(metric).is_some(),
        MetricValue::AggregatedHistogram { .. } | MetricValue::AggregatedSummary { .. } => true,
        _ => false,
    }
}

fn to_delta(metric: Metric, point: CumulativePoint) -> Option<Metric> {
    let time = metric.timestamp()?;
    let (metric, since) = match point {
        CumulativePoint::Unrelated => return None,
        // After a reset the full value covers exactly the time since the new start, so its
        // extremes still apply.
        CumulativePoint::Restarted(start) => (metric, start),
        CumulativePoint::Continued(previous) => {
            // A value that went backwards without a new start time leaves no way to know the
            // interval it covers, so it only becomes the baseline for the next point.
            let value = delta_value(metric.value(), &previous.value)?;
            let mut arbitrary = without_cumulative_extremes(&metric);
            if !otel_sum_reported(&previous.arbitrary) {
                mark_otel_sum_absent(&mut arbitrary);
            }
            let metric = metric.with_value(value).with_arbitrary(arbitrary);
            (metric, previous.time.timestamp?)
        }
    };
    let interval_ms = u32::try_from((time - since).num_milliseconds())
        .ok()
        .and_then(NonZeroU32::new)?;
    Some(
        metric
            .with_interval_ms(Some(interval_ms))
            .into_incremental(),
    )
}

/// New Relic's `summary` type carries only count, sum, min and max, so histograms and summaries
/// subtract count and sum alone. Bucket layouts need not match, which matters for exponential
/// histograms whose range grows with new observations. Quantiles are dropped because the 0 and 1
/// quantiles become the summary's min and max, which would describe the whole cumulative window.
fn delta_value(current: &MetricValue, previous: &MetricValue) -> Option<MetricValue> {
    match (current, previous) {
        (
            MetricValue::AggregatedHistogram { count, sum, .. },
            MetricValue::AggregatedHistogram {
                count: previous_count,
                sum: previous_sum,
                ..
            },
        ) if count >= previous_count => Some(MetricValue::AggregatedHistogram {
            buckets: Vec::new(),
            count: count - previous_count,
            sum: sum - previous_sum,
        }),
        (
            MetricValue::AggregatedSummary { count, sum, .. },
            MetricValue::AggregatedSummary {
                count: previous_count,
                sum: previous_sum,
                ..
            },
        ) if count >= previous_count => Some(MetricValue::AggregatedSummary {
            quantiles: Vec::new(),
            count: count - previous_count,
            sum: sum - previous_sum,
        }),
        (MetricValue::AggregatedHistogram { .. } | MetricValue::AggregatedSummary { .. }, _) => {
            None
        }
        _ => {
            let mut delta = current.clone();
            delta.subtract(previous).then_some(delta)
        }
    }
}

/// A cumulative histogram's `min` and `max` describe its whole window, not the delta.
fn without_cumulative_extremes(metric: &Metric) -> MetricArbitrary {
    let mut arbitrary = metric.arbitrary_value().clone();
    arbitrary.value.remove("min");
    arbitrary.value.remove("max");
    arbitrary
}
