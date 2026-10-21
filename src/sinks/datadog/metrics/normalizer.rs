use vector_lib::{
    event::{
        EventMetadata, Metric, MetricKind, MetricValue, Value,
        metric::{
            Bucket, MetricArbitrary, MetricData, MetricSketch,
            mezmo::{
                ExponentialHistogram, mark_otel_sum_absent, otel_metric_context, otel_sum_reported,
            },
        },
    },
    metrics::AgentDDSketch,
};

use crate::sinks::util::buffer::metrics::{
    CumulativePoint, MetricNormalize, MetricSet, track_cumulative,
};

#[derive(Default)]
pub(crate) struct DatadogMetricsNormalizer;

impl MetricNormalize for DatadogMetricsNormalizer {
    fn normalize(&mut self, state: &mut MetricSet, metric: Metric) -> Option<Metric> {
        // We primarily care about making sure that counters are incremental, and that gauges are
        // always absolute.  For other metric kinds, we want them to be incremental.
        match &metric.value() {
            // We always send counters as incremental and gauges as absolute.  Realistically, any
            // system sending an incremental gauge update is kind of doing it wrong, but alas.
            MetricValue::Counter { .. } if is_otel_cumulative(&metric) => {
                normalize_otel_counter(state, metric)
            }
            MetricValue::Counter { .. } => state.make_incremental(metric),
            MetricValue::Gauge { .. } => state.make_absolute(metric),
            // We convert distributions and aggregated histograms to sketches internally. We can't
            // send absolute sketches to Datadog, though, so we incrementalize them first.
            MetricValue::Distribution { .. } => state
                .make_incremental(metric)
                .filter(|metric| !metric.value().is_empty())
                .and_then(|metric| AgentDDSketch::transform_to_sketch(metric).ok()),
            MetricValue::AggregatedHistogram { .. } => normalize_histogram(state, metric),
            // Sketches cannot be subtracted from one another, so we treat them as implicitly
            // incremental, and just update the metric type.
            MetricValue::Sketch { .. } => Some(metric.into_incremental()),
            // Otherwise, ensure that it's incremental.
            _ => state.make_incremental(metric),
        }
    }
}

fn is_otel_cumulative(metric: &Metric) -> bool {
    metric.kind() == MetricKind::Absolute && otel_metric_context(metric).is_some()
}

/// The generic subtraction cannot tell a restarted series from a counter that went backwards,
/// and subtracts the previous process's total from a restarted one that has already passed it.
/// OpenTelemetry marks restarts with the start time.
fn normalize_otel_counter(state: &mut MetricSet, metric: Metric) -> Option<Metric> {
    let baseline = baseline(&metric, metric.value().clone(), MetricArbitrary::default());
    match track_cumulative(state, &metric, baseline)? {
        CumulativePoint::Unrelated => None,
        CumulativePoint::Restarted(_) => Some(metric.into_incremental()),
        CumulativePoint::Continued(previous) => {
            // A counter that went backwards without a new start time leaves no delta to report.
            let mut value = metric.value().clone();
            value
                .subtract(&previous.value)
                .then(|| metric.with_value(value).into_incremental())
        }
    }
}

fn normalize_histogram(state: &mut MetricSet, metric: Metric) -> Option<Metric> {
    if otel_metric_context(&metric).is_some() {
        return normalize_otel_histogram(state, metric);
    }
    state
        .make_incremental(metric)
        .filter(|metric| !metric.value().is_empty())
        .and_then(|metric| AgentDDSketch::transform_to_sketch(metric).ok())
}

/// The bucket layout an OpenTelemetry histogram arrived with.
enum OtelBuckets {
    Explicit(Vec<Bucket>),
    Exponential(ExponentialHistogram),
}

impl OtelBuckets {
    fn of(metric: &Metric) -> Option<Self> {
        if let Some(histogram) = ExponentialHistogram::from_metric(metric) {
            return Some(Self::Exponential(histogram));
        }
        Self::from_data(metric.data())
    }

    /// A baseline keeps no user metadata, so the exponential fields alone identify it.
    fn from_data(data: &MetricData) -> Option<Self> {
        if let Some(histogram) = ExponentialHistogram::from_arbitrary(&data.arbitrary) {
            return Some(Self::Exponential(histogram));
        }
        match &data.value {
            MetricValue::AggregatedHistogram { buckets, .. } => {
                Some(Self::Explicit(buckets.clone()))
            }
            _ => None,
        }
    }

    /// `None` when the layouts differ or `previous` has more observations in some bucket, since
    /// neither leaves a delta to report.
    fn checked_sub(&self, previous: &Self) -> Option<Self> {
        match (self, previous) {
            (Self::Explicit(current), Self::Explicit(previous)) => {
                if current.len() != previous.len() {
                    return None;
                }
                current
                    .iter()
                    .zip(previous)
                    .map(|(current, previous)| {
                        if current.upper_limit != previous.upper_limit {
                            return None;
                        }
                        Some(Bucket {
                            upper_limit: current.upper_limit,
                            count: current.count.checked_sub(previous.count)?,
                        })
                    })
                    .collect::<Option<_>>()
                    .map(Self::Explicit)
            }
            (Self::Exponential(current), Self::Exponential(previous)) => {
                current.checked_sub(previous).map(Self::Exponential)
            }
            _ => None,
        }
    }

    /// The explicit buckets a baseline carries in its value, and the exponential fields it
    /// carries in `arbitrary`.
    fn baseline_parts(&self) -> (Vec<Bucket>, MetricArbitrary) {
        match self {
            Self::Explicit(buckets) => (buckets.clone(), MetricArbitrary::default()),
            Self::Exponential(histogram) => (Vec::new(), histogram.to_arbitrary()),
        }
    }

    /// Exponential buckets are spread over their own value range, as in the Datadog agent.
    ///
    /// Reference: datadog-agent `pkg/opentelemetry-mapping-go/otlp/metrics/default_mapper.go`
    /// (`exponentialHistogramToDDSketch`) and `pkg/util/quantile/ddsketch.go`
    fn to_sketch(&self, stats: &ExactStats) -> Option<AgentDDSketch> {
        let mut sketch = AgentDDSketch::with_agent_defaults();
        match self {
            // Without a finite bound, interpolation would place every value at negative
            // infinity.
            Self::Explicit(buckets)
                if buckets
                    .iter()
                    .all(|bucket| bucket.upper_limit == f64::INFINITY) =>
            {
                insert_bucketless(&mut sketch, stats)?;
            }
            Self::Explicit(buckets) => sketch.insert_interpolate_buckets(buckets.clone()).ok()?,
            Self::Exponential(histogram) => {
                for (lower, upper, count) in histogram.bucket_ranges() {
                    // A boundary beyond the f64 range has no bin to land in. The agent rejects
                    // these too.
                    if !lower.is_finite() || !upper.is_finite() {
                        return None;
                    }
                    sketch.insert_interpolate_bucket(lower, upper, u32::try_from(count).ok()?);
                }
                if histogram.zero_count > 0 {
                    sketch.insert_n(0.0, u32::try_from(histogram.zero_count).ok()?);
                }
            }
        }
        Some(sketch)
    }
}

fn normalize_otel_histogram(state: &mut MetricSet, metric: Metric) -> Option<Metric> {
    let buckets = OtelBuckets::of(&metric)?;
    let (metric, buckets, extremes_apply) = match metric.kind() {
        MetricKind::Incremental => (metric, buckets, true),
        MetricKind::Absolute => cumulative_otel_delta(state, metric, buckets)?,
    };
    let stats = otel_exact_stats(&metric, extremes_apply)?;
    if stats.count == 0 {
        return None;
    }
    let sketch = buckets.to_sketch(&stats)?;
    Some(with_exact_stats(metric, sketch, &stats))
}

/// OpenTelemetry reads a histogram without buckets, or with only the overflow bucket, as one
/// bucket holding every value. Only an exact min and max, or the sum, say where its values lie.
/// With the sum alone every value is placed at the mean, which keeps the count and sum exact.
/// Without either there is no distribution to send, and an empty sketch would carry impossible
/// extremes.
///
/// Reference: <https://opentelemetry.io/docs/specs/otel/metrics/data-model/#histogram>
fn insert_bucketless(sketch: &mut AgentDDSketch, stats: &ExactStats) -> Option<()> {
    match (stats.count, stats.sum, stats.min, stats.max) {
        (1, Some(sum), _, _) => sketch.insert(sum),
        // Bounds out of order would leave the sketch no bins to spread the count over.
        (count, _, Some(min), Some(max)) if min.is_finite() && max.is_finite() && min <= max => {
            sketch.insert_interpolate_bucket(min, max, count);
        }
        (count, Some(sum), min, max) if min.zip(max).is_none() => {
            let mean = sum / f64::from(count);
            if !mean.is_finite() {
                return None;
            }
            sketch.insert_n(mean, count);
        }
        _ => return None,
    }
    Some(())
}

/// Subtracts the previous point of the series in the native layout, which for exponential
/// histograms keeps them even when the SDK has lowered the scale since the previous point. The
/// Datadog agent drops cumulative exponential histograms instead. Returns the delta and whether
/// the point's min and max describe it.
fn cumulative_otel_delta(
    state: &mut MetricSet,
    metric: Metric,
    buckets: OtelBuckets,
) -> Option<(Metric, OtelBuckets, bool)> {
    let MetricValue::AggregatedHistogram { count, sum, .. } = *metric.value() else {
        return None;
    };
    let baseline = histogram_baseline(&metric, &buckets, count, sum);
    let previous = match track_cumulative(state, &metric, baseline)? {
        CumulativePoint::Unrelated => return None,
        // The full point covers only the time since the restart.
        CumulativePoint::Restarted(_) => return Some((metric.into_incremental(), buckets, true)),
        CumulativePoint::Continued(previous) => previous,
    };

    let MetricValue::AggregatedHistogram {
        count: previous_count,
        sum: previous_sum,
        ..
    } = previous.value
    else {
        return None;
    };
    // Fewer observations without a new start time leave no way to know what the delta is, so
    // the point only becomes the baseline for the next one.
    let delta = buckets.checked_sub(&OtelBuckets::from_data(&previous)?)?;
    let value = MetricValue::AggregatedHistogram {
        buckets: match &delta {
            OtelBuckets::Explicit(buckets) => buckets.clone(),
            OtelBuckets::Exponential(_) => Vec::new(),
        },
        count: count.checked_sub(previous_count)?,
        sum: sum - previous_sum,
    };
    let mut arbitrary = metric.arbitrary_value().clone();
    if !otel_sum_reported(&previous.arbitrary) {
        mark_otel_sum_absent(&mut arbitrary);
    }
    let metric = metric
        .with_value(value)
        .with_arbitrary(arbitrary)
        .into_incremental();
    Some((metric, delta, false))
}

fn histogram_baseline(metric: &Metric, buckets: &OtelBuckets, count: u64, sum: f64) -> Metric {
    let (buckets, mut arbitrary) = buckets.baseline_parts();
    if !otel_sum_reported(metric.arbitrary_value()) {
        mark_otel_sum_absent(&mut arbitrary);
    }
    let value = MetricValue::AggregatedHistogram {
        buckets,
        count,
        sum,
    };
    baseline(metric, value, arbitrary)
}

/// Only what the next delta needs, without the event's finalizers.
fn baseline(metric: &Metric, value: MetricValue, arbitrary: MetricArbitrary) -> Metric {
    let data = MetricData {
        time: metric.data().time,
        kind: MetricKind::Absolute,
        value,
        arbitrary,
    };
    Metric::from_parts(metric.series().clone(), data, EventMetadata::default())
}

struct ExactStats {
    count: u32,
    sum: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
}

/// OpenTelemetry histograms carry an exact count and sum, and for delta points an exact min and
/// max, that a sketch built from bucket boundaries only approximates. The Datadog agent's own
/// OTLP mapping uses the exact values; a cumulative min or max covers more than the delta, so it
/// is left out.
///
/// Reference: datadog-agent `pkg/opentelemetry-mapping-go/otlp/metrics/default_mapper.go`
fn otel_exact_stats(metric: &Metric, extremes_apply: bool) -> Option<ExactStats> {
    let MetricValue::AggregatedHistogram { count, sum, .. } = metric.value() else {
        return None;
    };
    let arbitrary = metric.arbitrary_value();
    let extreme = |key: &str| match arbitrary.value.get(key) {
        Some(Value::Float(value)) if extremes_apply => Some(value.into_inner()),
        _ => None,
    };
    Some(ExactStats {
        count: u32::try_from(*count).ok()?,
        sum: otel_sum_reported(arbitrary).then_some(*sum),
        min: extreme("min"),
        max: extreme("max"),
    })
}

fn with_exact_stats(metric: Metric, mut sketch: AgentDDSketch, stats: &ExactStats) -> Metric {
    sketch.set_exact_stats(stats.count, stats.sum, stats.min, stats.max);
    metric.with_value(MetricValue::Sketch {
        sketch: MetricSketch::AgentDDSketch(sketch),
    })
}

#[cfg(test)]
mod tests {
    use vector_lib::{
        event::{Metric, MetricKind, MetricValue, metric::MetricSketch},
        metrics::AgentDDSketch,
    };

    use super::DatadogMetricsNormalizer;
    use crate::test_util::metrics::{
        assert_normalize, generate_f64s, get_aggregated_histogram, get_distribution, tests,
    };

    fn get_sketch<N, S, V>(name: N, samples: S, kind: MetricKind) -> Metric
    where
        N: Into<String>,
        S: IntoIterator<Item = V>,
        V: Into<f64>,
    {
        let samples = samples.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut ddsketch = AgentDDSketch::with_agent_defaults();
        ddsketch.insert_many(&samples);

        Metric::new(
            name,
            kind,
            MetricValue::Sketch {
                sketch: MetricSketch::AgentDDSketch(ddsketch),
            },
        )
    }

    #[test]
    fn absolute_counter() {
        tests::absolute_counter_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn incremental_counter() {
        tests::incremental_counter_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn mixed_counter() {
        tests::mixed_counter_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn absolute_gauge() {
        tests::absolute_gauge_normalize_to_absolute(DatadogMetricsNormalizer);
    }

    #[test]
    fn incremental_gauge() {
        tests::incremental_gauge_normalize_to_absolute(DatadogMetricsNormalizer);
    }

    #[test]
    fn mixed_gauge() {
        tests::mixed_gauge_normalize_to_absolute(DatadogMetricsNormalizer);
    }

    #[test]
    fn absolute_set() {
        tests::absolute_set_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn incremental_set() {
        tests::incremental_set_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn mixed_set() {
        tests::mixed_set_normalize_to_incremental(DatadogMetricsNormalizer);
    }

    #[test]
    fn absolute_distribution() {
        let samples1 = generate_f64s(1, 100);

        let mut samples2 = samples1.clone();
        samples2.extend(generate_f64s(75, 125));

        let sketch_samples = generate_f64s(101, 125);

        let distributions = vec![
            get_distribution(samples1, MetricKind::Absolute),
            get_distribution(samples2, MetricKind::Absolute),
        ];

        let expected_sketches = vec![
            None,
            Some(get_sketch(
                distributions[1].name(),
                sketch_samples,
                MetricKind::Incremental,
            )),
        ];

        assert_normalize(DatadogMetricsNormalizer, distributions, expected_sketches);
    }

    #[test]
    fn incremental_distribution() {
        let samples1 = generate_f64s(1, 100);
        let samples2 = generate_f64s(75, 125);
        let sketch1_samples = samples1.clone();
        let sketch2_samples = samples2.clone();

        let distributions = vec![
            get_distribution(samples1, MetricKind::Incremental),
            get_distribution(samples2, MetricKind::Incremental),
        ];

        let expected_sketches = vec![
            Some(get_sketch(
                distributions[0].name(),
                sketch1_samples,
                MetricKind::Incremental,
            )),
            Some(get_sketch(
                distributions[1].name(),
                sketch2_samples,
                MetricKind::Incremental,
            )),
        ];

        assert_normalize(DatadogMetricsNormalizer, distributions, expected_sketches);
    }

    #[test]
    fn mixed_distribution() {
        let samples1 = generate_f64s(1, 100);
        let samples2 = generate_f64s(75, 125);
        let samples3 = generate_f64s(75, 187);
        let samples4 = generate_f64s(22, 45);
        let samples5 = generate_f64s(1, 100);
        let sketch1_samples = samples1.clone();
        let sketch3_samples = generate_f64s(126, 187);
        let sketch4_samples = samples4.clone();
        let sketch5_samples = samples5.clone();

        let distributions = vec![
            get_distribution(samples1, MetricKind::Incremental),
            get_distribution(samples2, MetricKind::Absolute),
            get_distribution(samples3, MetricKind::Absolute),
            get_distribution(samples4, MetricKind::Incremental),
            get_distribution(samples5, MetricKind::Incremental),
        ];

        let expected_sketches = vec![
            Some(get_sketch(
                distributions[0].name(),
                sketch1_samples,
                MetricKind::Incremental,
            )),
            None,
            Some(get_sketch(
                distributions[2].name(),
                sketch3_samples,
                MetricKind::Incremental,
            )),
            Some(get_sketch(
                distributions[3].name(),
                sketch4_samples,
                MetricKind::Incremental,
            )),
            Some(get_sketch(
                distributions[4].name(),
                sketch5_samples,
                MetricKind::Incremental,
            )),
        ];

        assert_normalize(DatadogMetricsNormalizer, distributions, expected_sketches);
    }

    #[test]
    fn absolute_aggregated_histogram() {
        let samples1 = generate_f64s(1, 100);
        let samples2 = generate_f64s(1, 125);
        let sketch_samples = generate_f64s(101, 125);

        let agg_histograms = vec![
            get_aggregated_histogram(samples1, MetricKind::Absolute),
            get_aggregated_histogram(samples2, MetricKind::Absolute),
        ];

        let expected_sketches = vec![
            None,
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
        ];

        assert_normalize(DatadogMetricsNormalizer, agg_histograms, expected_sketches);
    }

    #[test]
    fn incremental_aggregated_histogram() {
        let samples1 = generate_f64s(1, 100);
        let samples2 = generate_f64s(1, 125);
        let sketch1_samples = samples1.clone();
        let sketch2_samples = samples2.clone();

        let agg_histograms = vec![
            get_aggregated_histogram(samples1, MetricKind::Incremental),
            get_aggregated_histogram(samples2, MetricKind::Incremental),
        ];

        let expected_sketches = vec![
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch1_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch2_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
        ];

        assert_normalize(DatadogMetricsNormalizer, agg_histograms, expected_sketches);
    }

    #[test]
    fn mixed_aggregated_histogram() {
        let samples1 = generate_f64s(1, 100);
        let samples2 = generate_f64s(75, 125);
        let samples3 = generate_f64s(75, 187);
        let samples4 = generate_f64s(22, 45);
        let samples5 = generate_f64s(1, 100);
        let sketch1_samples = samples1.clone();
        let sketch3_samples = generate_f64s(126, 187);
        let sketch4_samples = samples4.clone();
        let sketch5_samples = samples5.clone();

        let agg_histograms = vec![
            get_aggregated_histogram(samples1, MetricKind::Incremental),
            get_aggregated_histogram(samples2, MetricKind::Absolute),
            get_aggregated_histogram(samples3, MetricKind::Absolute),
            get_aggregated_histogram(samples4, MetricKind::Incremental),
            get_aggregated_histogram(samples5, MetricKind::Incremental),
        ];

        let expected_sketches = vec![
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch1_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
            None,
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch3_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch4_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
            Some(
                AgentDDSketch::transform_to_sketch(get_aggregated_histogram(
                    sketch5_samples,
                    MetricKind::Incremental,
                ))
                .unwrap(),
            ),
        ];

        assert_normalize(DatadogMetricsNormalizer, agg_histograms, expected_sketches);
    }
}
