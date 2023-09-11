use lookup::lookup_v2::{BorrowedSegment, ValuePath};
use ordered_float::NotNan;
use rand_distr::num_traits::FromPrimitive;
use std::fmt::Debug;
use vector_core::event::metric::MetricTags;
use vector_core::event::{Metric, MetricValue};
use vrl::value::Value;

fn get_tag_value<'a>(
    tags: Option<&MetricTags>,
    iter: impl Iterator<Item = BorrowedSegment<'a>> + Clone,
) -> Option<Value> {
    let mut iter = iter.peekable();
    if let Some(tags) = tags {
        if let Some(BorrowedSegment::Field(tag_name)) = iter.next().as_ref() {
            if iter.peek().is_none() {
                return tags.get(tag_name).map(Value::from);
            }
        }
    }
    None
}

fn get_metric_value<'a>(
    value: &MetricValue,
    iter: impl Iterator<Item = BorrowedSegment<'a>> + Clone,
) -> Option<Value> {
    let mut iter = iter.peekable();
    match value {
        MetricValue::Counter { value } | MetricValue::Gauge { value } => {
            if let Some(BorrowedSegment::Field(tag_name)) = iter.next().as_ref() {
                if iter.peek().is_none() && tag_name == "value" {
                    let value: Option<NotNan<f64>> = NotNan::from_f64(*value);
                    return value.map(Value::from);
                }
            }
        }
        _ => {
            info!(
                "Postgres sink does not support the {} metric type. Value will be skipped.",
                value.as_name()
            );
        }
    }
    None
}

fn get_metric_property(metric: &Metric, key: &str) -> Option<Value> {
    match key {
        "timestamp" => metric.timestamp().map(Value::from),
        "interval_ms" => metric.interval_ms().map(|v| Value::Integer(v.get() as i64)),
        "name" => Some(Value::from(metric.name())),
        "namespace" => metric.namespace().map(Value::from),
        "kind" => Some(Value::from(metric.kind())),
        _ => {
            warn!("Postgres sink does not support accessing field '{key}' on metric data");
            None
        }
    }
}

pub fn get_from_metric<'a>(metric: &'a Metric, key: impl ValuePath<'a> + Debug) -> Option<Value> {
    let mut iter = key.segment_iter().peekable();
    iter.next().and_then(move |path_seg| {
        if let BorrowedSegment::Field(field) = path_seg {
            if field == "tags" {
                return get_tag_value(metric.tags(), iter);
            } else if field == metric.value().as_name() {
                return get_metric_value(metric.value(), iter);
            } else if iter.peek().is_none() {
                return get_metric_property(metric, field.as_ref());
            }
        }

        warn!("Unable to handle path specification for metric data: {key:?}");
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono;
    use std::{collections::BTreeSet, iter, num::NonZeroU32};
    use vector_core::event::MetricKind;
    use vector_core::metric_tags;
    use vrl::value::kind::Kind;

    fn new_metric() -> Metric {
        let ts = chrono::NaiveDate::from_ymd_opt(2022, 10, 1)
            .unwrap()
            .and_hms_opt(13, 51, 30)
            .unwrap();
        let ts = chrono::DateTime::from_utc(ts, chrono::Utc);

        Metric::new(
            "component_request_total",
            MetricKind::Absolute,
            MetricValue::Counter { value: 123.0 },
        )
        .with_timestamp(Some(ts))
        .with_interval_ms(Some(NonZeroU32::new(12345).unwrap()))
        .with_namespace(Some("namespace-a"))
        .with_tags(Some(metric_tags!("component_name" => "abc")))
    }

    #[test]
    fn test_get_tag_value() {
        let metric = new_metric();
        let path_iter = ".component_name".segment_iter();
        let res = get_tag_value(metric.tags(), path_iter);
        assert_eq!(res, Some(Value::from("abc")));
    }

    #[test]
    fn test_get_tag_value_no_match() {
        let metric = new_metric();
        let path_iter = ".component_type".segment_iter();
        let res = get_tag_value(metric.tags(), path_iter);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_tag_value_no_tags() {
        let path_iter = ".component_name".segment_iter();
        let res = get_tag_value(None, path_iter);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_tag_value_addl_segments() {
        let metric = new_metric();
        let path_iter = ".component_name.extra".segment_iter();
        let res = get_tag_value(metric.tags(), path_iter);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_tag_value_empty_iter() {
        let metric = new_metric();
        let res = get_tag_value(metric.tags(), iter::empty());
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_metric_value_counter() {
        let metric_value = MetricValue::Counter { value: 1234.0 };
        let path_iter = ".value".segment_iter();
        let res = get_metric_value(&metric_value, path_iter);
        assert_eq!(res, Some(Value::from(1234.0)));
    }

    #[test]
    fn test_get_metric_value_gauge() {
        let metric_value = MetricValue::Gauge { value: 1234.0 };
        let path_iter = ".value".segment_iter();
        let res = get_metric_value(&metric_value, path_iter);
        assert_eq!(res, Some(Value::from(1234.0)));
    }

    #[test]
    fn test_get_metric_value_empty_iter() {
        let metric_value = MetricValue::Gauge { value: 1234.0 };
        let res = get_metric_value(&metric_value, iter::empty());
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_metric_value_inexact_path() {
        let metric_value = MetricValue::Gauge { value: 1234.0 };
        let path_iter = ".value.extra".segment_iter();
        let res = get_metric_value(&metric_value, path_iter);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_metric_value_unsupported_type() {
        let metric_value = MetricValue::Set {
            values: BTreeSet::new(),
        };
        let path_iter = ".values".segment_iter();
        let res = get_metric_value(&metric_value, path_iter);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_from_metric() {
        let test_properties: Vec<(&str, for<'r> fn(&'r Kind) -> bool, &str)> = vec![
            (".timestamp", Kind::is_timestamp, "t'2022-10-01T13:51:30Z'"),
            (".interval_ms", Kind::is_integer, "12345"),
            (".namespace", Kind::is_bytes, "\"namespace-a\""),
            (".name", Kind::is_bytes, "\"component_request_total\""),
            (".kind", Kind::is_bytes, "\"absolute\""),
        ];
        let metric = new_metric();
        for (path, kind_fn, expected_val) in test_properties {
            let res = get_from_metric(&metric, path).expect("a value");
            assert_eq!(res.to_string(), expected_val);
            assert!(kind_fn(&res.kind()));
        }
    }

    #[test]
    fn test_get_from_metric_unknown_field() {
        let metric = new_metric();
        let res = get_from_metric(&metric, ".unknown");
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_from_metrics_inexact_path() {
        let metric = new_metric();
        let res = get_from_metric(&metric, ".timestamp.extra");
        assert_eq!(res, None);
    }
}
