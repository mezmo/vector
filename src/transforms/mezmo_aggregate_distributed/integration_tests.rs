#![allow(unused_imports)]
use std::task::Poll;

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
    Event::Log(metric::mezmo::from_metric(&Metric::new(name, kind, value)))
}

fn make_config(toml: &str) -> config::MezmoAggregateDistributedConfig {
    toml::from_str::<config::MezmoAggregateDistributedConfig>(toml)
        .unwrap()
        .into()
}

fn make_component_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("v1:aggregate:transform:{uuid}:{uuid}:{uuid}")
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

        assert_eq!(
            result
                .unwrap()
                .as_log()
                .get(".message.value")
                .unwrap()
                .as_float()
                .unwrap(),
            99.0
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
        strategy = "avg"
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
        metric::MetricValue::Counter { value: 2.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 3.0 },
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

        assert_eq!(
            result
                .unwrap()
                .as_log()
                .get(".message.value")
                .unwrap()
                .as_float()
                .unwrap(),
            2.0
        );

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_aggregate_distributed_multiple() {
    let config = make_config(
        r#"
        window_duration_ms = 2000
        flush_tick_ms = 1000
        strategy = "sum"
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
        metric::MetricValue::Counter { value: 1.0 },
    );
    let event_3 = make_metric(
        "counter_a",
        metric::MetricKind::Incremental,
        metric::MetricValue::Counter { value: 1.0 },
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
        assert_eq!(
            result
                .unwrap()
                .as_log()
                .get(".message.value")
                .unwrap()
                .as_float()
                .unwrap(),
            6.0
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
