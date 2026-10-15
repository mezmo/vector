//! Tracks the previous point of each cumulative OpenTelemetry series, for sinks that only accept
//! deltas.
//!
//! Reference: <https://opentelemetry.io/docs/specs/otel/metrics/data-model/#resets-and-gaps>

use chrono::{DateTime, Utc};
use vector_lib::event::metric::{
    Metric, MetricData,
    mezmo::{otel_start_time, set_otel_start_time},
};

use super::MetricSet;

/// How a cumulative point relates to the previous point of its series.
#[derive(Debug, PartialEq)]
pub enum CumulativePoint {
    /// The series continues from this point, so the delta is the difference from it.
    Continued(MetricData),
    /// The series restarted at this time, after the previous point, so the whole point is new.
    Restarted(DateTime<Utc>),
    /// Nothing usable precedes the point. Either the series is new, or it restarted at a time
    /// that leaves no way to tell which of its observations the previous point already counted.
    /// The point only becomes the baseline for the next one.
    Unrelated,
}

/// Stores `baseline` as the latest point of its series, and classifies `metric` against the
/// point it replaces. The start time of `metric` is added to `baseline`, which otherwise should
/// hold only what the caller needs to derive the next delta: keeping the event's finalizers would
/// hold its acknowledgement until the series is next updated or evicted.
///
/// Returns `None`, leaving the stored point in place, when `metric` is no newer than it. A late
/// or repeated point would otherwise count its observations a second time in the next delta.
///
/// The Datadog agent applies the same rules to cumulative OTLP sums.
///
/// Reference: datadog-agent `pkg/opentelemetry-mapping-go/otlp/metrics/ttlcache.go`
/// (`isNotFirstPoint`, `putAndGetMonotonic`)
pub fn track_cumulative(
    state: &mut MetricSet,
    metric: &Metric,
    baseline: Metric,
) -> Option<CumulativePoint> {
    let previous = state
        .peek(baseline.series())
        .map(|entry| entry.data.clone());
    let time = metric.timestamp();
    if let (Some(previous_time), Some(time)) = (
        previous
            .as_ref()
            .and_then(|previous| previous.time.timestamp),
        time,
    ) && previous_time >= time
    {
        return None;
    }

    let start = otel_start_time(metric.arbitrary_value());
    let baseline = match start {
        Some(start) => {
            let mut arbitrary = baseline.arbitrary_value().clone();
            set_otel_start_time(&mut arbitrary, start);
            baseline.with_arbitrary(arbitrary)
        }
        None => baseline,
    };
    state.insert_update(baseline);

    let Some(previous) = previous else {
        return Some(CumulativePoint::Unrelated);
    };
    let Some(start) = start else {
        return Some(CumulativePoint::Continued(previous));
    };
    // A start equal to the point's own time marks a reset at an unknown time.
    if time == Some(start) {
        return Some(CumulativePoint::Unrelated);
    }
    if previous
        .time
        .timestamp
        .is_some_and(|previous_time| start >= previous_time)
    {
        return Some(CumulativePoint::Restarted(start));
    }
    // Every point of one cumulative series shares the start of its first observation, so a
    // different one is a reset, even when it is too early to say what is new.
    if otel_start_time(&previous.arbitrary).is_some_and(|previous_start| previous_start != start) {
        return Some(CumulativePoint::Unrelated);
    }
    Some(CumulativePoint::Continued(previous))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use vector_lib::event::{EventMetadata, MetricKind, MetricValue, metric::MetricArbitrary};

    use super::*;

    fn at(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).unwrap()
    }

    fn point(seconds: i64, start: Option<i64>, value: f64) -> Metric {
        let mut arbitrary = MetricArbitrary::default();
        if let Some(start) = start {
            set_otel_start_time(&mut arbitrary, at(start));
        }
        Metric::new(
            "requests",
            MetricKind::Absolute,
            MetricValue::Counter { value },
        )
        .with_timestamp(Some(at(seconds)))
        .with_arbitrary(arbitrary)
    }

    fn track(state: &mut MetricSet, metric: &Metric) -> Option<CumulativePoint> {
        let (series, data, _) = metric.clone().into_parts();
        let baseline = Metric::from_parts(
            series,
            MetricData {
                arbitrary: MetricArbitrary::default(),
                ..data
            },
            EventMetadata::default(),
        );
        track_cumulative(state, metric, baseline)
    }

    fn continued_from(seconds: i64, start: Option<i64>, value: f64) -> Option<CumulativePoint> {
        let (_, data, _) = point(seconds, start, value).into_parts();
        Some(CumulativePoint::Continued(data))
    }

    #[test]
    fn first_point_is_unrelated() {
        let mut state = MetricSet::default();

        assert_eq!(
            track(&mut state, &point(100, Some(0), 1.0)),
            Some(CumulativePoint::Unrelated)
        );
    }

    #[test]
    fn same_start_continues() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, Some(0), 1.0));

        assert_eq!(
            track(&mut state, &point(160, Some(0), 3.0)),
            continued_from(100, Some(0), 1.0)
        );
    }

    #[test]
    fn missing_start_continues() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, None, 1.0));

        assert_eq!(
            track(&mut state, &point(160, None, 3.0)),
            continued_from(100, None, 1.0)
        );
    }

    #[test]
    fn start_after_previous_point_is_a_restart() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, Some(0), 1.0));

        assert_eq!(
            track(&mut state, &point(160, Some(130), 3.0)),
            Some(CumulativePoint::Restarted(at(130)))
        );
    }

    #[test]
    fn start_equal_to_point_time_is_unrelated() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, Some(0), 1.0));

        assert_eq!(
            track(&mut state, &point(160, Some(160), 3.0)),
            Some(CumulativePoint::Unrelated)
        );
    }

    #[test]
    fn changed_start_before_previous_point_is_unrelated() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, Some(0), 1.0));

        assert_eq!(
            track(&mut state, &point(160, Some(50), 3.0)),
            Some(CumulativePoint::Unrelated)
        );
        assert_eq!(
            track(&mut state, &point(220, Some(50), 4.0)),
            continued_from(160, Some(50), 3.0)
        );
    }

    #[test]
    fn late_point_keeps_the_baseline() {
        let mut state = MetricSet::default();
        track(&mut state, &point(100, Some(0), 1.0));

        assert_eq!(track(&mut state, &point(90, Some(0), 5.0)), None);
        assert_eq!(track(&mut state, &point(100, Some(0), 5.0)), None);
        assert_eq!(
            track(&mut state, &point(160, Some(0), 3.0)),
            continued_from(100, Some(0), 1.0)
        );
    }
}
