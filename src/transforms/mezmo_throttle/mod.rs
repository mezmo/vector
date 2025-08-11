use crate::{
    conditions::{AnyCondition, Condition},
    config::TransformContext,
    event::Event,
    internal_events::{TemplateRenderingError, ThrottleEventDiscarded},
    mezmo::persistence::{PersistenceConnection, RocksDBPersistenceConnection},
    template::Template,
    transforms::TaskTransform,
};
use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use mezmo::user_trace::MezmoUserLog;
use mezmo::{user_log_error, MezmoContext};
use rand::Rng;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use std::{num::NonZeroU32, pin::Pin};
use vrl::value::Value;

mod config;
#[cfg(test)]
mod tests;

// The key for the state persistence db.
const STATE_PERSISTENCE_KEY: &str = "state";

pub trait Clock: Clone + Sync {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn still_active(&mut self, now: i64) -> bool {
        self.retain_recent(now);
        !self.deque.is_empty()
    }
}

fn event_key_value(event: &Event, key_field: &Option<Template>) -> String {
    let res = key_field.as_ref().and_then(|template| {
        template
            .render_string(event)
            .map_err(|error| {
                emit!(TemplateRenderingError {
                    error,
                    field: Some("key_field"),
                    drop_event: false,
                })
            })
            .ok()
    });
    res.unwrap_or("".to_string())
}

pub struct Throttle<C: Clock> {
    mezmo_ctx: Option<MezmoContext>,
    keys: HashMap<String, ThrottleBucket>,
    window_ms: u64,
    threshold: NonZeroU32,
    key_field: Option<Template>,
    exclude: Option<Condition>,
    clock: C,
    state_persistence: Option<Arc<dyn PersistenceConnection>>,
    state_persistence_tick_ms: u64,
    state_persistence_max_jitter_ms: u64,
    max_keys_allowed: usize,
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

        let mezmo_ctx = context.mezmo_ctx.clone();
        let state_persistence_tick_ms = config.state_persistence_tick_ms;
        let state_persistence_max_jitter_ms = config.state_persistence_max_jitter_ms;
        let state_persistence: Option<Arc<dyn PersistenceConnection>> =
            match (&config.state_persistence_base_path, &mezmo_ctx) {
                (Some(base_path), Some(mezmo_ctx)) => Some(Arc::new(
                    RocksDBPersistenceConnection::new(base_path, mezmo_ctx)?,
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
            mezmo_ctx,
            max_keys_allowed: config.max_keys_allowed,
        })
    }

    /// Saves the current `data` to persistent storage. This is intended to be called from the
    /// polling loop on an interval defined by the `state_persistence_tick_ms` field.
    async fn persist_state(&self) {
        if let Some(state_persistence) = &self.state_persistence {
            let data = self.keys.clone();
            let state_persistence = Arc::clone(state_persistence);

            let handle = tokio::task::spawn_blocking(move || {
                let value = serde_json::to_string(&data)?;
                state_persistence.set(STATE_PERSISTENCE_KEY, &value)
            })
            .await;

            match handle {
                Ok(result) => match result {
                    Ok(_) => {
                        debug!("MezmoThrottle: state persisted");
                    }
                    Err(err) => {
                        error!("MezmoThrottle: failed to persist state: {}", err);
                    }
                },
                Err(err) => error!("MezmoThrottle: failed to execute persistence task: {}", err),
            }
        }
    }

    fn should_throttle(&mut self, event: Event) -> Option<Event> {
        // LOG-20577: Only add an entry to the throttle HashMap if there is room to protect
        // the SaaS resources (memory and persistence storage).
        let key = event_key_value(&event, &self.key_field);
        if !self.keys.contains_key(&key) && self.keys.len() >= self.max_keys_allowed {
            user_log_error!(
                self.mezmo_ctx,
                "Reached the limit of unique event key values to throttle. Throttle is disabled for this key value.",
                captured_data: Value::from(key.clone())
            );
            return Some(event);
        }

        self.keys
            .entry(key.clone())
            .or_insert_with(|| ThrottleBucket::new(self.window_ms, self.threshold))
            .accept(self.clock.now())
            .map(|_| event)
            .or_else(|| {
                emit!(ThrottleEventDiscarded {
                    key,
                    // Set to true to maintain previous behaviour
                    emit_events_discarded_per_key: true
                });
                None
            })
    }
}

// Handles loading initial state from persistent storage, returning an appropriate
// default value if the state is not found or cannot be deserialized.
#[allow(clippy::borrowed_box)]
fn load_initial_state(
    state_persistence: &Arc<dyn PersistenceConnection>,
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
                        let jitter = rand::rng().random_range(0..=self.state_persistence_max_jitter_ms);
                        tokio::time::sleep(Duration::from_millis(jitter)).await;
                        self.persist_state().await;
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
                                    self.should_throttle(event)
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

                // It's important with the limit on the number of entries allowed to remove any of the
                // entries that are now empty.
                self.keys.retain(|_, bucket| bucket.still_active(self.clock.now()));

                if done {
                    self.persist_state().await;
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
