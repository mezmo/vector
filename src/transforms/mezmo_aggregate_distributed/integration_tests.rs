#![allow(unused_imports)]
use std::task::Poll;

use chrono::DateTime;
use futures::FutureExt;
use tokio::sync::mpsc;

use super::*;
use vector_lib::{configurable::component, lookup::event_path};

use crate::{
    event::{metric, Event, LogEvent},
    test_util::components::assert_transform_compliance,
    topology::RunningTopology,
    transforms::test::create_topology_with_name,
};
use tokio_stream::wrappers::ReceiverStream;

fn make_metric(name: &str, kind: metric::MetricKind, value: metric::MetricValue) -> Event {
    Event::Log(metric::mezmo::from_metric(
        &Metric::new(name, kind, value).with_namespace(Some("test-namespace".to_owned())),
    ))
}

fn make_config(toml: &str) -> config::MezmoAggregateDistributedConfig {
    toml::from_str::<config::MezmoAggregateDistributedConfig>(toml)
        .unwrap()
        .into()
}

fn make_component_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("v1:aggregate-metrics:transform:{uuid}:{uuid}:{uuid}")
}

async fn make_instance(
    config: MezmoAggregateDistributedConfig,
    component_id: &str,
) -> (RunningTopology, mpsc::Sender<Event>, ReceiverStream<Event>) {
    let (tx, rx) = mpsc::channel(10);
    let (topology, out) =
        create_topology_with_name(ReceiverStream::new(rx), config, component_id).await;

    (topology, tx, ReceiverStream::new(out))
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_sum() {
    let config = make_config(
        r#"
        window_duration_ms = 2000
        flush_tick_ms = 1000
        flush_grace_period_ms = 1000
        strategy = "sum"
    "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 33.0 },
    );
    let event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 33.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 33.0 },
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        tx.send(event_3).await.unwrap();

        // nothing ready yet, awaiting the flush tick...
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut result: Option<Event> = None;
        while result.is_none() {
            if let Some(event) = out.next().await {
                result = Some(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        // back to pending
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let log = result.unwrap().as_log().to_owned();
        assert!(log.get(".message.window_end").is_some());
        assert!(log.get(".message.window_start").is_some());
        assert_eq!(
            *log.get(".timestamp").unwrap(),
            *log.get(".message.window_end").unwrap(),
        );

        assert_eq!(*log.get(".message.count").unwrap(), Value::from(3));
        assert_eq!(
            *log.get(".message.kind").unwrap(),
            Value::from("incremental")
        );
        assert_eq!(
            *log.get(".message.namespace").unwrap(),
            Value::from("test-namespace")
        );
        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("sum"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("counter"),
                KeyString::from("value") => Value::from(99.0),
            }),
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_avg() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            flush_tick_ms = 1000
            flush_grace_period_ms = 1000
            strategy = "avg"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 1.0 },
    );
    let event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 2.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 3.0 },
    );

    assert_transform_compliance(async {
        let (tx, rx) = mpsc::channel(10);
        let (topology, out) = create_topology_with_name(
            ReceiverStream::new(rx),
            config,
            make_component_id().as_str(),
        )
        .await;
        let mut out = ReceiverStream::new(out);

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        tx.send(event_3).await.unwrap();

        // nothing ready yet, awaiting the flush tick...
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut result: Option<Event> = None;
        while result.is_none() {
            if let Some(event) = out.next().await {
                result = Some(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        // back to pending
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let log = result.unwrap().as_log().to_owned();

        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("avg"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("gauge"),
                KeyString::from("value") => Value::from(2.0),
            }),
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_min() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            flush_tick_ms = 1000
            flush_grace_period_ms = 1000
            strategy = "min"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 1.0 },
    );
    let event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 20.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 300.0 },
    );

    assert_transform_compliance(async {
        let (tx, rx) = mpsc::channel(10);
        let (topology, out) = create_topology_with_name(
            ReceiverStream::new(rx),
            config,
            make_component_id().as_str(),
        )
        .await;
        let mut out = ReceiverStream::new(out);

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        tx.send(event_3).await.unwrap();

        // nothing ready yet, awaiting the flush tick...
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut result: Option<Event> = None;
        while result.is_none() {
            if let Some(event) = out.next().await {
                result = Some(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        // back to pending
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let log = result.unwrap().as_log().to_owned();

        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("min"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("counter"),
                KeyString::from("value") => Value::from(1.0),
            }),
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_max() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            flush_tick_ms = 1000
            flush_grace_period_ms = 1000
            strategy = "max"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 1.0 },
    );
    let event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 20.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 300.0 },
    );

    assert_transform_compliance(async {
        let (tx, rx) = mpsc::channel(10);
        let (topology, out) = create_topology_with_name(
            ReceiverStream::new(rx),
            config,
            make_component_id().as_str(),
        )
        .await;
        let mut out = ReceiverStream::new(out);

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        tx.send(event_3).await.unwrap();

        // nothing ready yet, awaiting the flush tick...
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut result: Option<Event> = None;
        while result.is_none() {
            if let Some(event) = out.next().await {
                result = Some(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        // back to pending
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let log = result.unwrap().as_log().to_owned();

        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("max"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("counter"),
                KeyString::from("value") => Value::from(300.0),
            }),
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_multiple_instances() {
    let config = make_config(
        r#"
            window_duration_ms = 2000
            flush_tick_ms = 1000
            flush_grace_period_ms = 1000
            strategy = "sum"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 1.0 },
    );
    let event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 1.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Gauge { value: 1.0 },
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (top_1, tx_1, mut rx_1) = make_instance(config.clone(), &component_id).await;
        let (top_2, tx_2, mut rx_2) = make_instance(config.clone(), &component_id).await;
        tx_1.send(event_1.clone()).await.unwrap();
        tx_2.send(event_1.clone()).await.unwrap();
        tx_1.send(event_2.clone()).await.unwrap();
        tx_2.send(event_2.clone()).await.unwrap();
        tx_1.send(event_3.clone()).await.unwrap();
        tx_2.send(event_3.clone()).await.unwrap();

        let result = tokio::select! {
            res = rx_1.next() => res,
            res = rx_2.next() => res,
        };

        assert!(result.is_some(), "expected result from one instance");

        let log = result.unwrap().as_log().to_owned();
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("gauge"),
                KeyString::from("value") => Value::from(6.0),
            }),
        );

        drop(tx_1);
        drop(tx_2);
        top_1.stop().await;
        top_2.stop().await;
        assert_eq!(rx_1.next().await, None);
        assert_eq!(rx_2.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_with_cardinality_exceeded() {
    let config = make_config(
        r#"
            window_duration_ms = 2000
            window_cardinality_limit = 2
            flush_tick_ms = 1000
            flush_grace_period_ms = 1000
            flush_batch_size = 2
            strategy = "sum"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 1.0 },
    );
    let event_2 = make_metric(
        "counter_b",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 2.0 },
    );
    let event_3 = make_metric(
        "counter_c",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 3.0 },
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        tx.send(event_3).await.unwrap();

        // nothing ready yet, awaiting the flush tick...
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut outputs = vec![];
        while outputs.len() < 2 {
            if let Some(event) = out.next().await {
                outputs.push(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        // back to pending, event_3 is never recorded, no more output events
        // will be produced.
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        // outputs for `counter_a` and `counter_b`
        assert!(outputs.iter().any(|e| {
            e.as_log()
                .get(".message.name")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("counter_a")
        }));

        assert!(outputs.iter().any(|e| {
            e.as_log()
                .get(".message.name")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("counter_b")
        }));

        // none of the outputs contain `counter_c` which exceeded the cardinality
        assert!(!outputs.iter().any(|e| {
            e.as_log()
                .get(".message.name")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("counter_c")
        }));

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

/// Tests that the initial window flush timestamp and flush grace period is respected.
/// Events are sent with old timestamps (vs wall-clock) over a duration that spans the
/// `window_duration_ms`. All values are aggregated into the same window, and only a single
/// output event is generated.
#[tokio::test]
async fn test_mezmo_aggregate_distributed_delayed() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            flush_tick_ms = 1000
            flush_grace_period_ms = 3000
            strategy = "sum"
        "#,
    );

    let old_timestamp = Value::Timestamp(
        DateTime::parse_from_rfc3339("2025-01-01T00:00:01.000+05:00")
            .unwrap()
            .into(),
    );

    let mut event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 10.0 },
    );
    event_1
        .as_mut_log()
        .insert(".timestamp", old_timestamp.clone());

    let mut event_2 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 10.0 },
    );
    event_2
        .as_mut_log()
        .insert(".timestamp", old_timestamp.clone());

    let mut event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 10.0 },
    );
    event_3
        .as_mut_log()
        .insert(".timestamp", old_timestamp.clone());

    assert_transform_compliance(async {
        let (tx, rx) = mpsc::channel(10);
        let (topology, out) = create_topology_with_name(
            ReceiverStream::new(rx),
            config,
            make_component_id().as_str(),
        )
        .await;
        let mut out = ReceiverStream::new(out);

        // With the window duration of 1s and a grace period of 3s, send all events over
        // a period of 3s. The output event should not be flushed until 3s + 1s = 4s.
        tx.send(event_1).await.unwrap();
        sleep(Duration::from_millis(1000)).await;
        tx.send(event_2).await.unwrap();
        sleep(Duration::from_millis(1000)).await;
        tx.send(event_3).await.unwrap();
        sleep(Duration::from_millis(1000)).await;

        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let mut result: Option<Event> = None;
        while result.is_none() {
            if let Some(event) = out.next().await {
                result = Some(event);
            } else {
                panic!("Unexpectedly received None in output stream");
            }
        }

        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        let log = result.unwrap().as_log().to_owned();

        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("sum"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("counter"),
                KeyString::from("value") => Value::from(30.0),
            }),
        );

        // Ensure start/end timestamps reflect the observed timestamp for the window
        assert_eq!(
            *log.get(".message.window_start").unwrap(),
            Value::Integer(1735671601000)
        );
        assert_eq!(
            *log.get(".message.window_end").unwrap(),
            Value::Integer(1735671602000)
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

// Tests a forced flush of all windows on shutdown of the component, even when not
// "finalized" per the `window_end_ts` + flush_grace_period_ms`.
#[tokio::test]
async fn test_mezmo_aggregate_distributed_forced_flush_all_on_shutdown() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            # don't flush during this test
            flush_tick_ms = 9999999
            # set a long grace period to simulate a far-future finalization of the window
            flush_grace_period_ms = 9999999
            flush_all_on_shutdown = true
            strategy = "sum"
        "#,
    );

    let event_1 = make_metric(
        "counter_a",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 10.0 },
    );
    let event_2 = make_metric(
        "counter_b",
        metric::MetricKind::Absolute,
        metric::MetricValue::Counter { value: 10.0 },
    );

    assert_transform_compliance(async {
        let (tx, rx) = mpsc::channel(10);
        let (topology, out_rx) = create_topology_with_name(
            ReceiverStream::new(rx),
            config,
            make_component_id().as_str(),
        )
        .await;

        let mut out = ReceiverStream::new(out_rx);
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        tx.send(event_1).await.unwrap();
        tx.send(event_2).await.unwrap();
        drop(tx);

        let mut out = Box::pin(out);
        let mut stop_fut = topology.stop().fuse();
        let mut flushed = vec![];

        loop {
            select! {
                _ = &mut stop_fut => break,
                maybe_event = out.next().fuse() => {
                    match maybe_event {
                        Some(event) => flushed.push(event), // event yielded from shutdown call
                        None => break, // shutdown
                    }
                }
            }
        }

        assert_eq!(flushed.len(), 2, "Expected two output events on shutdown");

        let log = flushed[0].as_log().to_owned();
        assert_eq!(*log.get(".message.strategy").unwrap(), Value::from("sum"));
        assert_eq!(
            *log.get(".message.value").unwrap(),
            Value::Object(btreemap! {
                KeyString::from("type") => Value::from("counter"),
                KeyString::from("value") => Value::from(10.0),
            }),
        );

        assert_eq!(out.next().await, None);
    })
    .await;
}
