use futures::StreamExt;
use indoc::indoc;
use vector_lib::finalization::{BatchNotifier, BatchStatus};

use super::config::DatadogMetricsConfig;
use crate::{
    common::datadog,
    config::{SinkConfig, SinkContext},
    extra_context::ExtraContext,
    sinks::{
        datadog::test_utils::{ApiStatus, test_server},
        util::test::load_sink_with_context,
    },
    test_util::{
        addr::next_addr,
        components::{SINK_TAGS, run_and_assert_sink_compliance},
        random_metrics_with_stream,
    },
};

#[tokio::test]
async fn global_options() {
    let config = "";
    let cx = SinkContext {
        extra_context: ExtraContext::single_value(datadog::Options {
            api_key: Some("global-key".to_string().into()),
            ..Default::default()
        }),
        ..SinkContext::default()
    };
    let (mut config, cx) = load_sink_with_context::<DatadogMetricsConfig>(config, cx).unwrap();

    let (_guard, addr) = next_addr();
    // Swap out the endpoint so we can force send it
    // to our local server
    let endpoint = format!("http://{addr}");
    config.local_dd_common.endpoint = Some(endpoint.clone());

    let (sink, _) = config.build(cx).await.unwrap();

    let (rx, _trigger, server) = test_server(addr, ApiStatus::OKv1);
    tokio::spawn(server);

    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (_expected, events) = random_metrics_with_stream(10, Some(batch), None);

    run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;

    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = rx
        .take(1)
        .map(|r| r.0.headers.get("DD-API-KEY").unwrap().clone())
        .collect::<Vec<_>>()
        .await;

    assert!(
        keys.iter()
            .all(|value| value.to_str().unwrap() == "global-key")
    );
}

#[tokio::test]
async fn override_global_options() {
    let config = indoc! {r#"
            default_api_key = "local-key"
        "#};

    // Set a global key option, which should be overridden by the option in the component configuration.
    let cx = SinkContext {
        extra_context: ExtraContext::single_value(datadog::Options {
            api_key: Some("global-key".to_string().into()),
            ..Default::default()
        }),
        ..SinkContext::default()
    };
    let (mut config, cx) = load_sink_with_context::<DatadogMetricsConfig>(config, cx).unwrap();

    let (_guard, addr) = next_addr();
    // Swap out the endpoint so we can force send it
    // to our local server
    let endpoint = format!("http://{addr}");
    config.local_dd_common.endpoint = Some(endpoint.clone());

    let (sink, _) = config.build(cx).await.unwrap();

    let (rx, _trigger, server) = test_server(addr, ApiStatus::OKv1);
    tokio::spawn(server);

    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (_expected, events) = random_metrics_with_stream(10, Some(batch), None);

    run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;

    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = rx
        .take(1)
        .map(|r| r.0.headers.get("DD-API-KEY").unwrap().clone())
        .collect::<Vec<_>>()
        .await;

    assert!(
        keys.iter()
            .all(|value| value.to_str().unwrap() == "local-key")
    );
}

mod otel_metrics {
    use std::collections::BTreeSet;

    use serde_json::json;

    use vector_lib::{
        event::metric::MetricSketch,
        finalization::{BatchNotifier, BatchStatus},
        metrics::AgentDDSketch,
    };

    use super::super::{normalizer::DatadogMetricsNormalizer, sink::add_otel_scope_tags};
    use crate::{
        event::{Metric, MetricKind, MetricValue},
        sinks::util::buffer::metrics::MetricNormalizer,
        test_util::mezmo_otel::{TIME_SECONDS, envelope_metric, nanos},
    };

    const DELTA: u8 = 1;
    const CUMULATIVE: u8 = 2;

    fn normalizer() -> MetricNormalizer<DatadogMetricsNormalizer> {
        MetricNormalizer::default()
    }

    fn data_point(
        kind: &str,
        seconds: i64,
        start: i64,
        temporality: u8,
        fields: serde_json::Value,
    ) -> Metric {
        let mut data_point = json!({
            "type": kind, "timeUnixNano": nanos(seconds), "startTimeUnixNano": nanos(start),
            "aggregationTemporality": temporality,
        });
        data_point
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        add_otel_scope_tags(envelope_metric("latency", data_point))
    }

    fn exponential(seconds: i64, start: i64, temporality: u8, fields: serde_json::Value) -> Metric {
        data_point("exponentialHistogram", seconds, start, temporality, fields)
    }

    fn sketch(metric: Metric) -> AgentDDSketch {
        let MetricValue::Sketch {
            sketch: MetricSketch::AgentDDSketch(sketch),
        } = metric.value()
        else {
            panic!("expected a sketch");
        };
        sketch.clone()
    }

    fn tags(metric: &Metric) -> BTreeSet<String> {
        metric
            .tags()
            .unwrap()
            .iter_single()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect()
    }

    fn expected_tags() -> BTreeSet<String> {
        [
            "service.name:checkout",
            "k8s.pod.uid:7",
            "http.route:/cart",
            "scope.attr:s",
            "instrumentation_scope:io.opentelemetry.http",
            "instrumentation_scope_version:1.2.0",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[test]
    fn otel_scope_becomes_tags() {
        let gauge = envelope_metric(
            "queue.depth",
            json!({"type": "gauge", "timeUnixNano": nanos(TIME_SECONDS), "value": 3.5}),
        );

        assert_eq!(tags(&add_otel_scope_tags(gauge)), expected_tags());
    }

    #[test]
    fn exponential_histogram_becomes_sketch_with_exact_stats() {
        let histogram = envelope_metric(
            "latency",
            json!({
                "type": "exponentialHistogram", "timeUnixNano": nanos(TIME_SECONDS),
                "startTimeUnixNano": nanos(TIME_SECONDS - 60),
                "count": "10", "sum": 12.5, "min": -3, "max": 7.5, "scale": 0,
                "zeroCount": "1", "positive": {"offset": 0, "bucketCounts": ["2", "4"]},
                "negative": {"offset": 1, "bucketCounts": ["3"]},
                "aggregationTemporality": 1,
            }),
        );

        let sketch = normalizer()
            .normalize(add_otel_scope_tags(histogram))
            .unwrap();

        let MetricValue::Sketch {
            sketch: MetricSketch::AgentDDSketch(sketch),
        } = sketch.value()
        else {
            panic!("expected a sketch");
        };
        assert_eq!(
            (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
            (10, Some(12.5), Some(-3.0), Some(7.5))
        );
    }

    #[test]
    fn exponential_buckets_keep_their_lower_bound() {
        let metric = exponential(
            TIME_SECONDS,
            TIME_SECONDS - 60,
            DELTA,
            json!({
                "count": "300", "sum": 450000, "scale": 0,
                "positive": {"offset": 10, "bucketCounts": ["300"]},
            }),
        );

        let sketch = sketch(normalizer().normalize(metric).unwrap());

        // Every observation is in (1024, 2048], and sketch bins are within about 1% of a value.
        assert!(sketch.min().unwrap() >= 1024.0 * 0.99);
        assert!(sketch.quantile(0.01).unwrap() >= 1024.0 * 0.99);
        assert!(sketch.quantile(0.99).unwrap() <= 2048.0 * 1.01);
    }

    #[test]
    fn cumulative_exponential_histogram_survives_scale_change() {
        let start = TIME_SECONDS - 3600;
        let mut normalizer = normalizer();
        let first = exponential(
            TIME_SECONDS,
            start,
            CUMULATIVE,
            json!({
                "count": "3", "sum": 11, "scale": 1,
                "positive": {"offset": 2, "bucketCounts": ["1", "1", "1"]},
            }),
        );
        let second = exponential(
            TIME_SECONDS + 60,
            start,
            CUMULATIVE,
            json!({
                "count": "8", "sum": 40, "scale": 0,
                "positive": {"offset": 1, "bucketCounts": ["3", "5"]},
            }),
        );

        assert!(normalizer.normalize(first).is_none());
        let delta = normalizer.normalize(second).unwrap();

        assert_eq!(delta.kind(), MetricKind::Incremental);
        // Scale 1 indexes 2 to 4 merge into scale 0 indexes 1 and 2, leaving one new
        // observation in (2, 4] and four in (4, 8].
        let sketch = sketch(delta);
        assert_eq!((sketch.count(), sketch.sum()), (5, Some(29.0)));
        assert!(sketch.min().unwrap() >= 2.0 * 0.99);
        assert!(sketch.max().unwrap() <= 8.0 * 1.01);
    }

    #[test]
    fn exponential_reset_sends_full_point() {
        let mut normalizer = normalizer();
        normalizer.normalize(exponential(
            TIME_SECONDS,
            TIME_SECONDS - 3600,
            CUMULATIVE,
            json!({"count": "9", "sum": 90, "scale": 0, "positive": {"offset": 3, "bucketCounts": ["9"]}}),
        ));

        let delta = normalizer
            .normalize(exponential(
                TIME_SECONDS + 60,
                TIME_SECONDS + 30,
                CUMULATIVE,
                json!({
                    "count": "2", "sum": 8, "min": 3, "max": 5, "scale": 0,
                    "positive": {"offset": 1, "bucketCounts": ["1", "1"]},
                }),
            ))
            .unwrap();

        let sketch = sketch(delta);
        assert_eq!(
            (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
            (2, Some(8.0), Some(3.0), Some(5.0))
        );
    }

    #[test]
    fn late_exponential_point_does_not_replace_baseline() {
        let start = TIME_SECONDS - 3600;
        let point = |seconds: i64, count: &str| {
            exponential(
                seconds,
                start,
                CUMULATIVE,
                json!({"count": count, "sum": 1, "scale": 0, "positive": {"offset": 0, "bucketCounts": [count]}}),
            )
        };
        let mut normalizer = normalizer();

        normalizer.normalize(point(TIME_SECONDS, "3"));
        assert!(
            normalizer
                .normalize(point(TIME_SECONDS - 30, "1"))
                .is_none()
        );
        let delta = normalizer.normalize(point(TIME_SECONDS + 60, "5")).unwrap();

        assert_eq!(sketch(delta).count(), 2);
    }

    #[test]
    fn exponential_baseline_does_not_hold_acknowledgements() {
        let (batch, mut receiver) = BatchNotifier::new_with_receiver();
        let point = exponential(
            TIME_SECONDS,
            TIME_SECONDS - 3600,
            CUMULATIVE,
            json!({"count": "1", "sum": 1, "scale": 0, "positive": {"offset": 0, "bucketCounts": ["1"]}}),
        )
        .with_batch_notifier(&batch);
        drop(batch);
        let mut normalizer = normalizer();

        assert!(normalizer.normalize(point).is_none());
        assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));
    }

    /// Both layouts OpenTelemetry reads as one bucket holding every value: no buckets, and only
    /// the overflow bucket.
    fn unbounded(seconds: i64, temporality: u8, fields: serde_json::Value) -> [Metric; 2] {
        [json!([]), json!([fields["count"]])].map(|bucket_counts| {
            let mut fields = fields.clone();
            fields["bucketCounts"] = bucket_counts;
            fields["explicitBounds"] = json!([]);
            data_point(
                "histogram",
                seconds,
                TIME_SECONDS - 3600,
                temporality,
                fields,
            )
        })
    }

    #[test]
    fn unbounded_histogram_spans_exact_extremes() {
        let metrics = unbounded(
            TIME_SECONDS,
            DELTA,
            json!({"count": "3", "sum": 12, "min": 2, "max": 6}),
        );

        for metric in metrics {
            let sketch = sketch(normalizer().normalize(metric).unwrap());

            assert_eq!(
                (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
                (3, Some(12.0), Some(2.0), Some(6.0))
            );
            let median = sketch.quantile(0.5).unwrap();
            assert!((2.0..=6.0).contains(&median), "median {median}");
        }
    }

    #[test]
    fn unbounded_histogram_without_extremes_sits_at_its_mean() {
        let metrics = unbounded(TIME_SECONDS, DELTA, json!({"count": "3", "sum": 12}));

        for metric in metrics {
            let sketch = sketch(normalizer().normalize(metric).unwrap());

            assert_eq!(
                (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
                (3, Some(12.0), Some(4.0), Some(4.0))
            );
            let median = sketch.quantile(0.5).unwrap();
            assert!((median - 4.0).abs() <= 4.0 * 0.01, "median {median}");
        }
    }

    #[test]
    fn unbounded_histogram_without_sum_or_extremes_is_dropped() {
        for metric in unbounded(TIME_SECONDS, DELTA, json!({"count": "3"})) {
            assert!(normalizer().normalize(metric).is_none());
        }
    }

    #[test]
    fn unbounded_histogram_with_inverted_extremes_is_dropped() {
        let metrics = unbounded(
            TIME_SECONDS,
            DELTA,
            json!({"count": "3", "sum": 12, "min": 6, "max": 2}),
        );

        for metric in metrics {
            assert!(normalizer().normalize(metric).is_none());
        }
    }

    #[test]
    fn unbounded_single_observation_is_its_sum() {
        let firsts = unbounded(TIME_SECONDS, CUMULATIVE, json!({"count": "4", "sum": 10}));
        let seconds = unbounded(
            TIME_SECONDS + 60,
            CUMULATIVE,
            json!({"count": "5", "sum": 17}),
        );

        for (first, second) in firsts.into_iter().zip(seconds) {
            let mut normalizer = normalizer();
            normalizer.normalize(first);

            let sketch = sketch(normalizer.normalize(second).unwrap());
            assert_eq!(
                (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
                (1, Some(7.0), Some(7.0), Some(7.0))
            );
        }
    }

    #[test]
    fn unbounded_cumulative_delta_sits_at_its_mean() {
        let firsts = unbounded(TIME_SECONDS, CUMULATIVE, json!({"count": "4", "sum": 10}));
        let seconds = unbounded(
            TIME_SECONDS + 60,
            CUMULATIVE,
            json!({"count": "6", "sum": 22, "min": 1, "max": 9}),
        );

        for (first, second) in firsts.into_iter().zip(seconds) {
            let mut normalizer = normalizer();
            normalizer.normalize(first);

            // The cumulative min and max describe the whole window, not the delta.
            let sketch = sketch(normalizer.normalize(second).unwrap());
            assert_eq!(
                (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
                (2, Some(12.0), Some(6.0), Some(6.0))
            );
        }
    }

    fn explicit(seconds: i64, start: i64, fields: serde_json::Value) -> Metric {
        let mut fields = fields;
        fields["explicitBounds"] = json!([5]);
        data_point("histogram", seconds, start, CUMULATIVE, fields)
    }

    #[test]
    fn missing_sum_keeps_estimated_sum() {
        let metric = exponential(
            TIME_SECONDS,
            TIME_SECONDS - 60,
            DELTA,
            json!({"count": "4", "scale": 0, "positive": {"offset": 0, "bucketCounts": ["4"]}}),
        );

        let sketch = sketch(normalizer().normalize(metric).unwrap());

        // Four observations in (1, 2].
        let sum = sketch.sum().unwrap();
        assert!((4.0 * 0.99..=8.0 * 1.01).contains(&sum), "sum {sum}");
    }

    #[test]
    fn missing_previous_sum_keeps_estimated_sum() {
        let start = TIME_SECONDS - 3600;
        let mut normalizer = normalizer();
        normalizer.normalize(explicit(
            TIME_SECONDS,
            start,
            json!({"count": "2", "bucketCounts": ["2", "0"]}),
        ));

        let delta = normalizer
            .normalize(explicit(
                TIME_SECONDS + 60,
                start,
                json!({"count": "4", "sum": 1000, "bucketCounts": ["4", "0"]}),
            ))
            .unwrap();

        // Two new observations at or below 5, not the whole cumulative sum.
        let sketch = sketch(delta);
        assert_eq!(sketch.count(), 2);
        assert!(sketch.sum().unwrap() <= 10.0 * 1.01);
    }

    #[test]
    fn explicit_histogram_reset_sends_full_point() {
        let mut normalizer = normalizer();
        normalizer.normalize(explicit(
            TIME_SECONDS,
            TIME_SECONDS - 3600,
            json!({"count": "6", "sum": 60, "bucketCounts": ["1", "5"]}),
        ));

        let delta = normalizer
            .normalize(explicit(
                TIME_SECONDS + 60,
                TIME_SECONDS + 30,
                json!({
                    "count": "9", "sum": 70, "min": 1, "max": 20,
                    "bucketCounts": ["3", "6"],
                }),
            ))
            .unwrap();

        let sketch = sketch(delta);
        assert_eq!(
            (sketch.count(), sketch.sum(), sketch.min(), sketch.max()),
            (9, Some(70.0), Some(1.0), Some(20.0))
        );
    }

    fn cumulative_counter(seconds: i64, start: i64, value: &str) -> Metric {
        data_point(
            "sum",
            seconds,
            start,
            CUMULATIVE,
            json!({"value": value, "valueType": "int", "isMonotonic": true}),
        )
    }

    fn counter_value(metric: &Metric) -> f64 {
        assert_eq!(metric.kind(), MetricKind::Incremental);
        let MetricValue::Counter { value } = metric.value() else {
            panic!("expected a counter");
        };
        *value
    }

    #[test]
    fn cumulative_counter_becomes_delta() {
        let start = TIME_SECONDS - 3600;
        let mut normalizer = normalizer();

        assert!(
            normalizer
                .normalize(cumulative_counter(TIME_SECONDS, start, "10"))
                .is_none()
        );
        let delta = normalizer
            .normalize(cumulative_counter(TIME_SECONDS + 60, start, "25"))
            .unwrap();

        assert_eq!(counter_value(&delta), 15.0);
    }

    #[test]
    fn counter_restart_with_lower_value_sends_full_value() {
        let mut normalizer = normalizer();
        normalizer.normalize(cumulative_counter(TIME_SECONDS, TIME_SECONDS - 3600, "100"));

        let delta = normalizer
            .normalize(cumulative_counter(
                TIME_SECONDS + 60,
                TIME_SECONDS + 30,
                "4",
            ))
            .unwrap();

        assert_eq!(counter_value(&delta), 4.0);
    }

    #[test]
    fn counter_restart_with_higher_value_sends_full_value() {
        let mut normalizer = normalizer();
        normalizer.normalize(cumulative_counter(TIME_SECONDS, TIME_SECONDS - 3600, "10"));

        let delta = normalizer
            .normalize(cumulative_counter(
                TIME_SECONDS + 60,
                TIME_SECONDS + 30,
                "40",
            ))
            .unwrap();

        assert_eq!(counter_value(&delta), 40.0);
    }

    #[test]
    fn counter_restart_overlapping_previous_point_only_becomes_baseline() {
        let mut normalizer = normalizer();
        normalizer.normalize(cumulative_counter(TIME_SECONDS, TIME_SECONDS - 3600, "10"));

        assert!(
            normalizer
                .normalize(cumulative_counter(
                    TIME_SECONDS + 60,
                    TIME_SECONDS - 30,
                    "40"
                ))
                .is_none()
        );
        let delta = normalizer
            .normalize(cumulative_counter(
                TIME_SECONDS + 120,
                TIME_SECONDS - 30,
                "45",
            ))
            .unwrap();

        assert_eq!(counter_value(&delta), 5.0);
    }

    #[test]
    fn counter_decrease_without_restart_only_becomes_baseline() {
        let start = TIME_SECONDS - 3600;
        let mut normalizer = normalizer();
        normalizer.normalize(cumulative_counter(TIME_SECONDS, start, "10"));

        assert!(
            normalizer
                .normalize(cumulative_counter(TIME_SECONDS + 60, start, "4"))
                .is_none()
        );
        let delta = normalizer
            .normalize(cumulative_counter(TIME_SECONDS + 120, start, "9"))
            .unwrap();

        assert_eq!(counter_value(&delta), 5.0);
    }

    #[test]
    fn late_counter_point_does_not_replace_baseline() {
        let start = TIME_SECONDS - 3600;
        let mut normalizer = normalizer();
        normalizer.normalize(cumulative_counter(TIME_SECONDS, start, "10"));

        assert!(
            normalizer
                .normalize(cumulative_counter(TIME_SECONDS - 30, start, "5"))
                .is_none()
        );
        let delta = normalizer
            .normalize(cumulative_counter(TIME_SECONDS + 60, start, "12"))
            .unwrap();

        assert_eq!(counter_value(&delta), 2.0);
    }

    #[test]
    fn counter_baseline_does_not_hold_acknowledgements() {
        let (batch, mut receiver) = BatchNotifier::new_with_receiver();
        let point =
            cumulative_counter(TIME_SECONDS, TIME_SECONDS - 3600, "1").with_batch_notifier(&batch);
        drop(batch);
        let mut normalizer = normalizer();

        assert!(normalizer.normalize(point).is_none());
        assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));
    }

    #[test]
    fn non_otel_metric_is_unchanged() {
        let gauge = Metric::new("g", MetricKind::Absolute, MetricValue::Gauge { value: 1.0 });

        assert_eq!(add_otel_scope_tags(gauge.clone()), gauge);
    }

    #[cfg(feature = "transforms-mezmo_log_to_metric")]
    #[tokio::test]
    async fn envelopes_through_log_to_metric_produce_request_bodies() {
        use std::collections::BTreeMap;
        use std::io::Read;

        use bytes::Bytes;
        use flate2::read::ZlibDecoder;
        use futures::StreamExt;
        use indoc::indoc;
        use prost::Message;

        use super::super::config::{DatadogMetricsConfig, SERIES_V1_PATH, SKETCHES_PATH};
        use crate::{
            config::SinkConfig,
            sinks::{
                datadog::test_utils::{ApiStatus, test_server},
                util::test::load_sink,
            },
            test_util::{addr::next_addr, mezmo_otel::envelope},
            transforms::mezmo_log_to_metric,
        };

        #[allow(warnings, clippy::pedantic, clippy::nursery)]
        mod ddmetric_proto {
            include!(concat!(env!("OUT_DIR"), "/datadog.agentpayload.rs"));
        }

        fn decompress(body: &Bytes) -> Bytes {
            let mut payload = Vec::new();
            ZlibDecoder::new(&body[..])
                .read_to_end(&mut payload)
                .unwrap();
            Bytes::from(payload)
        }

        let counter = |seconds: i64, value: &str| {
            envelope(
                "requests",
                json!({
                    "type": "sum", "timeUnixNano": nanos(seconds),
                    "startTimeUnixNano": nanos(TIME_SECONDS - 3600), "value": value,
                    "valueType": "int", "aggregationTemporality": 2, "isMonotonic": true,
                }),
            )
        };
        let metrics = mezmo_log_to_metric::log_to_metric(vec![
            envelope(
                "queue.depth",
                json!({"type": "gauge", "timeUnixNano": nanos(TIME_SECONDS), "value": 3.5}),
            ),
            counter(TIME_SECONDS, "10"),
            envelope(
                "http.server.duration",
                json!({
                    "type": "histogram", "timeUnixNano": nanos(TIME_SECONDS),
                    "startTimeUnixNano": nanos(TIME_SECONDS - 60), "count": "4", "sum": 20,
                    "min": 1, "max": 9, "bucketCounts": ["1", "3"], "explicitBounds": [5],
                    "aggregationTemporality": 1,
                }),
            ),
            counter(TIME_SECONDS + 60, "25"),
        ]);
        assert_eq!(metrics.len(), 4, "every envelope converts to a metric");

        let (mut config, cx) =
            load_sink::<DatadogMetricsConfig>(indoc! {r#"default_api_key = "atoken""#}).unwrap();
        let (_guard, addr) = next_addr();
        config.local_dd_common.endpoint = Some(format!("http://{addr}"));
        let (sink, _) = config.build(cx).await.unwrap();
        let (rx, _trigger, server) = test_server(addr, ApiStatus::OKv1);
        tokio::spawn(server);

        sink.run_events(metrics).await.unwrap();

        let requests: BTreeMap<String, Bytes> = rx
            .take(2)
            .map(|(parts, body)| (parts.uri.path().to_string(), decompress(&body)))
            .collect()
            .await;

        // Series go to the v1 JSON API unless VECTOR_TEMP_USE_DD_METRICS_SERIES_V2_API is set.
        let series: serde_json::Value = serde_json::from_slice(&requests[SERIES_V1_PATH]).unwrap();
        let series: BTreeMap<String, (serde_json::Value, serde_json::Value, BTreeSet<String>)> =
            series["series"]
                .as_array()
                .unwrap()
                .iter()
                .map(|serie| {
                    let tags = serie["tags"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|tag| tag.as_str().unwrap().to_string())
                        .collect();
                    (
                        serie["metric"].as_str().unwrap().to_string(),
                        (serie["type"].clone(), serie["points"].clone(), tags),
                    )
                })
                .collect();
        assert_eq!(
            series,
            BTreeMap::from([
                (
                    "queue.depth".to_string(),
                    (
                        json!("gauge"),
                        json!([[TIME_SECONDS, 3.5]]),
                        expected_tags()
                    ),
                ),
                (
                    "requests".to_string(),
                    (
                        json!("count"),
                        json!([[TIME_SECONDS + 60, 15.0]]),
                        expected_tags(),
                    ),
                ),
            ])
        );

        let sketches = ddmetric_proto::SketchPayload::decode(requests[SKETCHES_PATH].clone())
            .unwrap()
            .sketches;
        assert_eq!(sketches.len(), 1);
        let sketch = &sketches[0];
        assert_eq!(sketch.metric, "http.server.duration");
        assert_eq!(
            sketch.tags.iter().cloned().collect::<BTreeSet<_>>(),
            expected_tags()
        );
        assert_eq!(sketch.dogsketches.len(), 1);
        let dogsketch = &sketch.dogsketches[0];
        assert_eq!(
            (
                dogsketch.cnt,
                dogsketch.sum,
                dogsketch.avg,
                dogsketch.min,
                dogsketch.max
            ),
            (4, 20.0, 5.0, 1.0, 9.0)
        );
    }
}
