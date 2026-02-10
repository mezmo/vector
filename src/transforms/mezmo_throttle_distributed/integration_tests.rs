#![allow(unused_imports)]
use std::task::Poll;

use tokio::sync::mpsc;
use vrl::core::Value;

use super::*;
use vector_lib::{configurable::component, lookup::event_path};

use crate::{
    event::{Event, LogEvent, metric},
    test_util::components::assert_transform_compliance,
    topology::RunningTopology,
    transforms::test::create_topology_with_name,
};
use tokio_stream::wrappers::ReceiverStream;

fn make_event(msg: Option<&str>) -> Event {
    match msg {
        Some(msg) => LogEvent::from_str_legacy(msg).into(),
        None => LogEvent::default().into(),
    }
}

fn make_config(toml: &str) -> MezmoThrottleDistributedConfig {
    toml::from_str::<config::MezmoThrottleDistributedConfig>(toml)
        .unwrap()
        .into()
}

fn make_component_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("v1:throttle:transform:{uuid}:{uuid}:{uuid}")
}

async fn make_instance(
    config: MezmoThrottleDistributedConfig,
    component_id: &str,
) -> (RunningTopology, mpsc::Sender<Event>, ReceiverStream<Event>) {
    let (tx, rx) = mpsc::channel(10);
    let (topology, out) =
        create_topology_with_name(ReceiverStream::new(rx), config, component_id).await;

    (topology, tx, ReceiverStream::new(out))
}

#[tokio::test]
async fn test_mezmo_throttle_distributed_global() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            threshold = 2
        "#,
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        tx.send(make_event(None)).await.unwrap();
        tx.send(make_event(None)).await.unwrap();

        let mut output_events = vec![];
        while output_events.len() < 2 {
            let event = out.next().await.expect("received None in output stream");
            output_events.push(event);
        }

        assert_eq!(output_events.len(), 2);
        tx.send(make_event(None)).await.unwrap();

        // Still pending, event 3 throttled
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Window elapsed, event 4 is not throttled
        tx.send(make_event(None)).await.unwrap();
        out.next().await.expect("received None in output stream");

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_throttle_distributed_bucketed() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            threshold = 1
            key_field = "{{.message}}"
        "#,
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        tx.send(make_event(None)).await.unwrap();
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("B"))).await.unwrap();

        let mut output_events = vec![];
        while output_events.len() < 3 {
            let event = out.next().await.expect("received None in output stream");
            output_events.push(event);
        }

        assert_eq!(output_events.len(), 3);

        // Threshold reached for each bucket, future remains pending
        tx.send(make_event(None)).await.unwrap();
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("B"))).await.unwrap();
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        // New buckets unthrottled
        tx.send(make_event(Some("C"))).await.unwrap();
        out.next().await.expect("received None in output stream");

        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Window elapsed, all buckets refreshed
        tx.send(make_event(None)).await.unwrap();
        out.next().await.expect("received None in output stream");

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_throttle_distributed_with_exclusion() {
    let config = make_config(
        r#"
            window_duration_ms = 1000
            threshold = 1
            exclude = '''
              .message == "A"
            '''
        "#,
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("B"))).await.unwrap();

        let mut output_events = vec![];
        while output_events.len() < 4 {
            let event = out.next().await.expect("received None in output stream");
            output_events.push(event);
        }

        assert_eq!(output_events.len(), 4);

        // Threshold reached for bucket B, future remains pending
        tx.send(make_event(Some("B"))).await.unwrap();
        assert_eq!(Poll::Pending, futures::poll!(out.next()));

        // But bucket A still unthrottled
        tx.send(make_event(Some("A"))).await.unwrap();
        out.next().await.expect("received None in output stream");

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}

#[tokio::test]
async fn test_mezmo_throttle_distributed_with_cardinality_exceeded() {
    let config = make_config(
        r#"
            window_duration_ms = 100000
            window_cardinality_limit = 2
            threshold = 1
            key_field = "{{.message}}"
        "#,
    );

    assert_transform_compliance(async {
        let component_id = make_component_id();
        let (topology, tx, mut out) = make_instance(config.clone(), &component_id).await;

        // Bucket A + B throttled, C exceeds cardinality limit and is unthrottled
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("A"))).await.unwrap();
        tx.send(make_event(Some("B"))).await.unwrap();
        tx.send(make_event(Some("B"))).await.unwrap();
        tx.send(make_event(Some("C"))).await.unwrap();
        tx.send(make_event(Some("C"))).await.unwrap();

        let mut output_events = vec![];
        while output_events.len() < 4 {
            let event = out.next().await.expect("received None in output stream");
            output_events.push(event);
        }

        assert_eq!(output_events.len(), 4);

        drop(tx);
        topology.stop().await;
        assert_eq!(out.next().await, None);
    })
    .await;
}
