use assay::assay;
use std::sync::{
    Arc,
    atomic::{AtomicI64, Ordering},
};
use std::task::Poll;
use tempfile::tempdir;

use futures::SinkExt;

use super::*;
use crate::{
    event::LogEvent,
    test_util::components::assert_transform_compliance,
    transforms::{Transform, test::create_topology},
};
use config::MezmoThrottleConfig;
use mezmo::MezmoContext;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use vrl::value::Value;

#[derive(Clone, Default)]
struct MockThrottleClock {
    now: Arc<AtomicI64>,
}
impl MockThrottleClock {
    fn increment_by(&self, i: i64) {
        self.now.fetch_add(i, Ordering::Relaxed);
    }
}
impl Clock for MockThrottleClock {
    #[inline(always)]
    fn now(&self) -> i64 {
        self.now.load(Ordering::Relaxed)
    }
}

#[test]
fn generate_config() {
    crate::test_util::test_generate_config::<MezmoThrottleConfig>();
}

#[tokio::test]
async fn throttle_events() {
    let clock = MockThrottleClock::default();
    let config = toml::from_str::<MezmoThrottleConfig>(
        r#"
    threshold = 2
    window_ms = 5
    "#,
    )
    .unwrap();

    let throttle = Throttle::new(&config, &TransformContext::default(), clock.clone())
        .map(Transform::event_task)
        .unwrap();

    let throttle = throttle.into_task();

    let (mut tx, rx) = futures::channel::mpsc::channel(10);
    let mut out_stream = throttle.transform_events(Box::pin(rx));

    // tokio interval is always immediately ready, so we poll once to make sure
    // we trip it/set the interval in the future
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.send(LogEvent::default().into()).await.unwrap();
    tx.send(LogEvent::default().into()).await.unwrap();

    let mut count = 0_u8;
    while count < 2 {
        if let Some(_event) = out_stream.next().await {
            count += 1;
        } else {
            panic!("Unexpectedly received None in output stream");
        }
    }
    assert_eq!(2, count);

    clock.increment_by(2);

    tx.send(LogEvent::default().into()).await.unwrap();

    // We should be back to pending, having the second event dropped
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    clock.increment_by(3);

    tx.send(LogEvent::default().into()).await.unwrap();

    // The rate limiter should now be refreshed and allow an additional event through
    if let Some(_event) = out_stream.next().await {
    } else {
        panic!("Unexpectedly received None in output stream");
    }

    // We should be back to pending, having nothing waiting for us
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.disconnect();

    // And still nothing there
    assert_eq!(Poll::Ready(None), futures::poll!(out_stream.next()));
}

#[tokio::test]
async fn throttle_exclude() {
    let clock = MockThrottleClock::default();
    let config = toml::from_str::<MezmoThrottleConfig>(
        r#"
threshold = 2
window_ms = 5
exclude = """
exists(.special)
"""
"#,
    )
    .unwrap();

    let throttle = Throttle::new(&config, &TransformContext::default(), clock.clone())
        .map(Transform::event_task)
        .unwrap();

    let throttle = throttle.into_task();

    let (mut tx, rx) = futures::channel::mpsc::channel(10);
    let mut out_stream = throttle.transform_events(Box::pin(rx));

    // tokio interval is always immediately ready, so we poll once to make sure
    // we trip it/set the interval in the future
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.send(LogEvent::default().into()).await.unwrap();
    tx.send(LogEvent::default().into()).await.unwrap();

    let mut count = 0_u8;
    while count < 2 {
        if let Some(_event) = out_stream.next().await {
            count += 1;
        } else {
            panic!("Unexpectedly received None in output stream");
        }
    }
    assert_eq!(2, count);

    clock.increment_by(2);

    tx.send(LogEvent::default().into()).await.unwrap();

    // We should be back to pending, having the second event dropped
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    let mut special_log = LogEvent::default();
    special_log.insert("special", "true");
    tx.send(special_log.into()).await.unwrap();
    // The rate limiter should allow this log through regardless of current limit
    if let Some(_event) = out_stream.next().await {
    } else {
        panic!("Unexpectedly received None in output stream");
    }

    clock.increment_by(3);

    tx.send(LogEvent::default().into()).await.unwrap();

    // The rate limiter should now be refreshed and allow an additional event through
    if let Some(_event) = out_stream.next().await {
    } else {
        panic!("Unexpectedly received None in output stream");
    }

    // We should be back to pending, having nothing waiting for us
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.disconnect();

    // And still nothing there
    assert_eq!(Poll::Ready(None), futures::poll!(out_stream.next()));
}

#[tokio::test]
async fn throttle_buckets() {
    let clock = MockThrottleClock::default();
    let config = toml::from_str::<MezmoThrottleConfig>(
        r#"
    threshold = 1
    window_ms = 5
    key_field = "{{ bucket }}"
    "#,
    )
    .unwrap();

    let throttle = Throttle::new(&config, &TransformContext::default(), clock)
        .map(Transform::event_task)
        .unwrap();

    let throttle = throttle.into_task();

    let (mut tx, rx) = futures::channel::mpsc::channel(10);
    let mut out_stream = throttle.transform_events(Box::pin(rx));

    // tokio interval is always immediately ready, so we poll once to make sure
    // we trip it/set the interval in the future
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    let mut log_a = LogEvent::default();
    log_a.insert("bucket", "a");
    let mut log_b = LogEvent::default();
    log_b.insert("bucket", "b");
    tx.send(log_a.into()).await.unwrap();
    tx.send(log_b.into()).await.unwrap();

    let mut count = 0_u8;
    while count < 2 {
        if let Some(_event) = out_stream.next().await {
            count += 1;
        } else {
            panic!("Unexpectedly received None in output stream");
        }
    }
    assert_eq!(2, count);

    // We should be back to pending, having nothing waiting for us
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.disconnect();

    // And still nothing there
    assert_eq!(Poll::Ready(None), futures::poll!(out_stream.next()));
}

#[assay(env = [("POD_NAME", "vector-test0-0")])]
async fn with_initial_state() {
    #[allow(deprecated)]
    let tmp_path = tempdir().expect("Could not create temp dir").into_path();
    let state_persistence_base_path = tmp_path.to_str().unwrap();

    let clock = MockThrottleClock::default();
    let config = toml::from_str::<MezmoThrottleConfig>(
        format!(
            r#"
    threshold = 4
    window_ms = 5
    state_persistence_base_path = "{state_persistence_base_path}"
    "#
        )
        .as_str(),
    )
    .unwrap();

    let mezmo_ctx = MezmoContext::try_from(
        "v1:throttle:transform:component_id:pipeline_id:cea71e55-a1ec-4e5f-a5c0-c0e10b1a571c"
            .to_string(),
    )
    .ok();
    let context = TransformContext {
        mezmo_ctx,
        ..Default::default()
    };

    let mut throttle = Throttle::new(&config, &context, clock.clone()).unwrap();

    // This config allows 4 events over the window.
    // Initialize the state with 2 events, then send 2 more below to hit the threshold.
    // Subsequent events do not pass through until the clock is advanced past the window.
    let mut initial_deque = VecDeque::new();
    initial_deque.push_back(0);
    initial_deque.push_back(0);
    let initial_keys = HashMap::from([(
        "".to_string(),
        ThrottleBucket {
            window_ms: 5,
            threshold: NonZeroU32::new(4).unwrap(),
            deque: initial_deque,
        },
    )]);
    throttle.keys = initial_keys;
    throttle.persist_state().await;

    let throttle = Transform::event_task(throttle);
    let throttle = throttle.into_task();

    let (mut tx, rx) = futures::channel::mpsc::channel(10);
    let mut out_stream = throttle.transform_events(Box::pin(rx));

    // tokio interval is always immediately ready, so we poll once to make sure
    // we trip it/set the interval in the future
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    // Send 2 events to hit the threshold
    tx.send(LogEvent::default().into()).await.unwrap();
    tx.send(LogEvent::default().into()).await.unwrap();

    let mut count = 0_u8;
    while count < 2 {
        if let Some(_event) = out_stream.next().await {
            count += 1;
        } else {
            panic!("Unexpectedly received None in output stream");
        }
    }
    assert_eq!(2, count);

    clock.increment_by(2);

    tx.send(LogEvent::default().into()).await.unwrap();

    // Pending after the 3rd event (represents the 5th event from this instance)
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    clock.increment_by(3);

    tx.send(LogEvent::default().into()).await.unwrap();

    // The rate limiter should now be refreshed and allow an additional event through
    if let Some(_event) = out_stream.next().await {
    } else {
        panic!("Unexpectedly received None in output stream");
    }

    // We should be back to pending, having nothing waiting for us
    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    tx.disconnect();

    // And still nothing there
    assert_eq!(None, out_stream.next().await);
}

#[tokio::test]
async fn emits_internal_events() {
    assert_transform_compliance(async move {
        let config = MezmoThrottleConfig {
            threshold: 1,
            window_ms: 1000,
            key_field: None,
            exclude: None,
            state_persistence_base_path: None,
            state_persistence_tick_ms: 1,
            state_persistence_max_jitter_ms: 1,
            max_keys_allowed: 10,
        };
        let (tx, rx) = mpsc::channel(1);
        let (topology, mut out) = create_topology(ReceiverStream::new(rx), config).await;

        let log = LogEvent::from("hello world");
        tx.send(log.into()).await.unwrap();

        _ = out.recv().await;

        drop(tx);
        topology.stop().await;
        assert_eq!(out.recv().await, None);
    })
    .await
}

#[tokio::test]
async fn limit_throttle_keys() {
    let clock = MockThrottleClock::default();
    let config = toml::from_str::<MezmoThrottleConfig>(
        r#"
        threshold = 1
        window_ms = 6
        key_field = "{{ test_key  }}"
        max_keys_allowed = 1
        "#,
    )
    .unwrap();

    let throttle = Throttle::new(&config, &TransformContext::default(), clock.clone())
        .map(Transform::event_task)
        .unwrap()
        .into_task();
    let (mut tx, rx) = futures::channel::mpsc::channel(10);
    let mut out_stream = throttle.transform_events(Box::pin(rx));

    assert_eq!(Poll::Pending, futures::poll!(out_stream.next()));

    let test_events = vec![
        btreemap! {
            "test_key" => "abc",
            "value" => 1,
        },
        btreemap! {
            "test_key" => "def",
            "value" => 2,
        },
        btreemap! {
            "test_key" => "def",
            "value" => 3,
        },
        btreemap! {
            "test_key" => "abc",
            "value" => 4,
        },
    ];
    for event in test_events.clone() {
        let event = LogEvent::from(Value::from(event));
        tx.send(Event::from(event)).await.unwrap();
        clock.increment_by(1);
    }
    tx.disconnect();

    let mut out_events = vec![];
    while let Some(event) = out_stream.next().await {
        let (event_body, _) = event.into_log().into_parts();
        out_events.push(event_body);
    }

    assert_eq!(
        out_events,
        [
            Value::from(test_events[0].clone()),
            Value::from(test_events[1].clone()),
            Value::from(test_events[2].clone()),
        ]
    );
}
