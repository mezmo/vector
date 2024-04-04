use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use std::{num::NonZeroU32, pin::Pin};

use crate::{
    conditions::{AnyCondition, Condition},
    config::TransformContext,
    event::Event,
    internal_events::{TemplateRenderingError, ThrottleEventDiscarded},
    mezmo::persistence::{PersistenceConnection, RocksDBPersistenceConnection},
    template::Template,
    transforms::TaskTransform,
};

mod config;

// The key for the state persistence db.
const STATE_PERSISTENCE_KEY: &str = "state";

pub trait Clock: Clone {
    fn now(&self) -> i64;
}

#[derive(Clone)]
struct ThrottleClock {}
impl ThrottleClock {
    pub const fn new() -> ThrottleClock {
        ThrottleClock {}
    }
}
impl Clock for ThrottleClock {
    #[inline(always)]
    fn now(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThrottleBucket {
    window_ms: u64,
    threshold: NonZeroU32,
    deque: VecDeque<i64>,
}

impl ThrottleBucket {
    pub fn new(window_ms: u64, threshold: NonZeroU32) -> ThrottleBucket {
        ThrottleBucket {
            window_ms,
            threshold,
            deque: Default::default(),
        }
    }

    fn retain_recent(&mut self, now: i64) {
        while let Some(first) = self.deque.front() {
            let elapsed: u64 = (now - *first).try_into().expect("Clock moved backwards");
            if elapsed >= self.window_ms {
                self.deque.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn accept(&mut self, now: i64) -> Option<()> {
        self.retain_recent(now);
        if self.deque.len() as u32 >= self.threshold.get() {
            return None;
        }

        self.deque.push_back(now);
        Some(())
    }
}

pub struct Throttle<C: Clock> {
    keys: HashMap<String, ThrottleBucket>,
    window_ms: u64,
    threshold: NonZeroU32,
    key_field: Option<Template>,
    exclude: Option<Condition>,
    clock: C,
    state_persistence: Option<Box<dyn PersistenceConnection>>,
    state_persistence_tick_ms: u64,
    state_persistence_max_jitter_ms: u64,
}

impl<C> Throttle<C>
where
    C: Clock,
{
    pub fn new(
        config: &config::MezmoThrottleConfig,
        context: &TransformContext,
        clock: C,
    ) -> crate::Result<Self> {
        let threshold = match NonZeroU32::new(config.threshold) {
            Some(threshold) => threshold,
            None => return Err(Box::new(ConfigError::NonZero)),
        };

        let exclude = config
            .exclude
            .as_ref()
            .map(|condition| condition.build(&context.enrichment_tables, context.mezmo_ctx.clone()))
            .transpose()?;

        let state_persistence_tick_ms = config.state_persistence_tick_ms;
        let state_persistence_max_jitter_ms = config.state_persistence_max_jitter_ms;
        let state_persistence: Option<Box<dyn PersistenceConnection>> = match (
            &config.state_persistence_base_path,
            context.mezmo_ctx.clone(),
        ) {
            (Some(base_path), Some(mezmo_ctx)) => Some(Box::new(
                RocksDBPersistenceConnection::new(base_path, &mezmo_ctx)?,
            )),
            (_, Some(mezmo_ctx)) => {
                debug!(
                    "MezmoThrottle: state persistence not enabled for component {}",
                    mezmo_ctx.id()
                );
                None
            }
            (_, _) => None,
        };

        let initial_data: HashMap<String, ThrottleBucket> = match &state_persistence {
            Some(state_persistence) => load_initial_state(state_persistence),
            None => HashMap::new(),
        };

        Ok(Self {
            keys: initial_data,
            window_ms: config.window_ms,
            key_field: config.key_field.clone(),
            threshold,
            exclude,
            clock,
            state_persistence,
            state_persistence_tick_ms,
            state_persistence_max_jitter_ms,
        })
    }

    /// Saves the current `data` to persistent storage. This is intended to be called from the
    /// polling loop on an interval defined by the `state_persistence_tick_ms` field.
    fn persist_state(&self) {
        if let Some(state_persistence) = &self.state_persistence {
            let value = serde_json::to_string(&self.keys);
            if let Err(err) = value {
                error!("MezmoThrottle: failed to serialize state: {}", err);
                return;
            }

            match state_persistence.set(STATE_PERSISTENCE_KEY, &value.unwrap()) {
                Ok(_) => debug!("MezmoThrottle: state persisted"),
                Err(err) => error!("MezmoThrottle: failed to persist state: {}", err),
            }
        }
    }
}

// Handles loading initial state from persistent storage, returning an appropriate
// default value if the state is not found or cannot be deserialized.
#[allow(clippy::borrowed_box)]
fn load_initial_state(
    state_persistence: &Box<dyn PersistenceConnection>,
) -> HashMap<String, ThrottleBucket> {
    match state_persistence.get(STATE_PERSISTENCE_KEY) {
        Ok(state) => match state {
            Some(state) => match serde_json::from_str(&state) {
                Ok(state) => state,
                Err(err) => {
                    error!(
                        "Failed to deserialize state from persistence: {}, component_id",
                        err
                    );
                    HashMap::new()
                }
            },
            None => HashMap::new(),
        },
        Err(err) => {
            error!(
                "Failed to load state from persistence: {}, component_id",
                err
            );
            HashMap::new()
        }
    }
}

impl<C> TaskTransform<Event> for Throttle<C>
where
    C: Clock + Send + 'static,
{
    fn transform(
        mut self: Box<Self>,
        mut input_rx: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>>
    where
        Self: 'static,
    {
        let mut state_persistence_interval =
            tokio::time::interval(Duration::from_millis(self.state_persistence_tick_ms));

        Box::pin(stream! {
            loop {
                let done = tokio::select! {
                    _ = state_persistence_interval.tick() => {
                        let jitter = rand::thread_rng().gen_range(0..=self.state_persistence_max_jitter_ms);
                        tokio::time::sleep(Duration::from_millis(jitter)).await;
                        self.persist_state();
                        false
                  },
                  maybe_event = input_rx.next() => {
                        match maybe_event {
                            None => true,
                            Some(event) => {
                                let (throttle, event) = match self.exclude.as_ref() {
                                    Some(condition) => {
                                        let (result, event) = condition.check(event);
                                        (!result, event)
                                    },
                                    _ => (true, event)
                                };
                                let output = if throttle {

                                    let key = self.key_field.as_ref().and_then(|t| {
                                        t.render_string(&event)
                                            .map_err(|error| {
                                                emit!(TemplateRenderingError {
                                                    error,
                                                    field: Some("key_field"),
                                                    drop_event: false,
                                                })
                                            })
                                            .ok()
                                    }).unwrap_or("".to_string());

                                    match self.keys
                                    .entry(key.clone())
                                    .or_insert_with(|| ThrottleBucket::new(self.window_ms, self.threshold))
                                    .accept(self.clock.now()) {
                                        Some(_) => {
                                            Some(event)
                                        }
                                        None => {
                                            emit!(ThrottleEventDiscarded{key});
                                            None
                                        }
                                    }
                                } else {
                                    Some(event)
                                };
                                if let Some(event) = output {
                                    yield event;
                                }
                                false
                            }
                        }
                    }
                };
                if done {
                    self.persist_state();
                    break;
                }
            }
        })
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("`threshold`, and `window_ms` must be non-zero"))]
    NonZero,
}

#[cfg(test)]
mod tests {
    use assay::assay;
    use std::sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    };
    use std::task::Poll;
    use tempfile::tempdir;

    use futures::SinkExt;

    use super::*;
    use crate::{
        event::LogEvent,
        mezmo::MezmoContext,
        test_util::components::assert_transform_compliance,
        transforms::{test::create_topology, Transform},
    };
    use config::MezmoThrottleConfig;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

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
        throttle.persist_state();

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
        assert_eq!(Poll::Ready(None), futures::poll!(out_stream.next()));
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
}
