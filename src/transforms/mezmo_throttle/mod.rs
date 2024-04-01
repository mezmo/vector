use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use serde_with::serde_as;
use snafu::Snafu;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use std::{num::NonZeroU32, pin::Pin};
use vector_lib::config::{clone_input_definitions, LogNamespace, OutputId, TransformOutput};
use vector_lib::configurable::configurable_component;

use crate::{
    conditions::{AnyCondition, Condition},
    config::{DataType, Input, TransformConfig, TransformContext},
    event::Event,
    internal_events::{TemplateRenderingError, ThrottleEventDiscarded},
    schema,
    template::Template,
    transforms::{TaskTransform, Transform},
};

#[cfg(test)]
use std::sync::atomic::{AtomicI64, Ordering};

mod config;

pub trait Clock: Clone {
    fn now(&self) -> i64;
}

#[derive(Clone)]
struct ThrottleClock {}
impl Clock for ThrottleClock {
    #[inline(always)]
    fn now(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

pub struct ThrottleBucket {
    window_ms: i64,
    threshold: NonZeroU32,
    deque: VecDeque<i64>,
}

impl ThrottleBucket {
    pub fn new(window_ms: i64, threshold: NonZeroU32) -> ThrottleBucket {
        ThrottleBucket {
            window_ms,
            threshold,
            deque: Default::default(),
        }
    }

    fn retain_recent(&mut self, now: i64) {
        while let Some(first) = self.deque.front() {
            if now - *first >= self.window_ms {
                self.deque.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn accept(&mut self, now: i64) -> Result<(), ()> {
        self.retain_recent(now);
        if self.deque.len() as u32 >= self.threshold.get() {
            return Err(());
        }

        self.deque.push_back(now);
        Ok(())
    }
}

pub struct Throttle<C: Clock> {
    keys: HashMap<Option<String>, ThrottleBucket>,
    window_ms: i64,
    threshold: NonZeroU32,
    key_field: Option<Template>,
    exclude: Option<Condition>,
    clock: C,
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

        Ok(Self {
            keys: Default::default(),
            window_ms: config.window_secs.as_millis() as i64,
            key_field: config.key_field.clone(),
            threshold,
            exclude,
            clock,
        })
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
        Box::pin(stream! {
          loop {
            let done = tokio::select! {
                biased;

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
                                });

                                match self.keys
                                .entry(key.clone())
                                .or_insert_with(|| ThrottleBucket::new(self.window_ms, self.threshold))
                                .accept(self.clock.now()) {
                                    Ok(()) => {
                                        Some(event)
                                    }
                                    _ => {
                                        if let Some(key) = key {
                                            emit!(ThrottleEventDiscarded{key})
                                        } else {
                                            emit!(ThrottleEventDiscarded{key: "None".to_string()})
                                        }
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
            if done { break }
          }
        })
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("`threshold`, and `window_secs` must be non-zero"))]
    NonZero,
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, task::Poll};

    use futures::SinkExt;

    use super::*;
    use crate::{
        event::LogEvent, test_util::components::assert_transform_compliance,
        transforms::test::create_topology,
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
    window_secs = 0.005
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
window_secs = 0.005
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
    window_secs = 0.005
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

    #[tokio::test]
    async fn emits_internal_events() {
        assert_transform_compliance(async move {
            let config = MezmoThrottleConfig {
                threshold: 1,
                window_secs: Duration::from_secs_f64(1.0),
                key_field: None,
                exclude: None,
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
