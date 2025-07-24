use crate::{callsite::CallsiteIdentity, context::MezmoContext};
use futures_util::{future::ready, Stream, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tracing::debug;
use vector_core::event::LogEvent;
use vrl::value::Value;

static PIPELINE_STATE_VARIABLE_CHANGE_ACTION_LOGGER: OnceLock<
    PipelineStateVariableChangeActionLogger,
> = OnceLock::new();

const DEFAULT_RATE_LIMIT_UNINITIALIZED: u64 = 5; // 5 second
const PIPELINE_STATE_VARIABLE_CHANGE_ACTION_CACHE_RATE_LIMIT_MAX_CAPACITY: u64 = 5_000;

pub fn init(rate_limit: u64) {
    PIPELINE_STATE_VARIABLE_CHANGE_ACTION_LOGGER
        .set(PipelineStateVariableChangeActionLogger::new(rate_limit))
        .expect("PIPELINE_STATE_VARIABLE_CHANGE_ACTION_LOGGER was already initialized");
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PipelineStateVariableChangeIdentifier {
    component_id: String,
    identity: CallsiteIdentity,
}

#[derive(Debug)]
pub struct State {
    start: Instant,
    count: u64,
    limit: u64,
    log: LogEvent,
}

impl State {
    fn new(log: LogEvent, limit: u64) -> Self {
        Self {
            start: Instant::now(),
            count: 0,
            limit,
            log,
        }
    }

    fn reset(&mut self) {
        self.start = Instant::now();
        self.count = 1;
    }

    fn increment_count(&mut self) -> u64 {
        let prev = self.count;
        self.count += 1;
        prev
    }

    fn should_limit(&self) -> bool {
        self.start.elapsed().as_secs() < self.limit
    }
}

#[derive(Debug)]
struct PipelineStateVariableChangeActionLogger {
    sender: Sender<LogEvent>,
    action_rate_limit_cache:
        moka::sync::Cache<PipelineStateVariableChangeIdentifier, Arc<Mutex<State>>>,
    rate_limit: u64,
}

impl PipelineStateVariableChangeActionLogger {
    fn new(rate_limit: u64) -> Self {
        PipelineStateVariableChangeActionLogger {
            sender: broadcast::channel(1000).0,
            action_rate_limit_cache: moka::sync::Cache::new(
                PIPELINE_STATE_VARIABLE_CHANGE_ACTION_CACHE_RATE_LIMIT_MAX_CAPACITY,
            ),
            rate_limit,
        }
    }
}

pub fn get_pipeline_state_variable_change_action_sender() -> &'static broadcast::Sender<LogEvent> {
    &PIPELINE_STATE_VARIABLE_CHANGE_ACTION_LOGGER
        .get_or_init(|| {
            PipelineStateVariableChangeActionLogger::new(DEFAULT_RATE_LIMIT_UNINITIALIZED)
        })
        .sender
}

pub fn try_send_pipeline_state_variable_change_action(
    log: LogEvent,
    id: PipelineStateVariableChangeIdentifier,
) {
    if let Some(pipeline_state_variable_change_action_logger) =
        PIPELINE_STATE_VARIABLE_CHANGE_ACTION_LOGGER.get()
    {
        let entry = pipeline_state_variable_change_action_logger
            .action_rate_limit_cache
            .entry(id)
            .or_insert_with(|| {
                Arc::new(Mutex::new(State::new(
                    log.clone(),
                    pipeline_state_variable_change_action_logger.rate_limit,
                )))
            });

        if let Ok(state) = Arc::<Mutex<State>>::clone(entry.value()).lock().as_mut() {
            let previous_count = state.increment_count();
            if state.should_limit() {
                match previous_count {
                    0 => match pipeline_state_variable_change_action_logger
                        .sender
                        .send(log)
                    {
                        Ok(_) => {}
                        Err(_) => {
                            debug!("failed to send pipeline_state_variable_change_action; likely no source consuming data")
                        }
                    },
                    1 => {
                        debug!(
                            "pipeline_state_variable_change_action is [{:?}] is being rate limited",
                            state.log.get_message().unwrap_or(&("".into()))
                        );
                    }
                    _ => {}
                }
            } else {
                // If we saw this event 2 or more times total, emit an event that indicates the total number of times we
                // rate limited the event in the limit period.
                if previous_count > 1 {
                    debug!(
                        "pipeline_state_variable_change_action [{:?}] has been rate limited {} times.",
                        state.log.get_message().unwrap_or(&("".into())),
                        previous_count - 1
                    );
                }

                // We're not rate limiting anymore, so we also emit the current event as normal.. but we update our rate
                // limiting state since this is effectively equivalent to seeing the event again for the first time.
                match pipeline_state_variable_change_action_logger
                    .sender
                    .send(log)
                {
                    Ok(_) => {}
                    Err(_) => {
                        debug!("failed to send pipeline state variable change action; likely no source consuming data")
                    }
                }

                state.reset();
            }
        }
    }
}

/// This is the struct used to obtain access to consume the user log information in other parts of the code base.
pub struct PipelineStateVariableChangeActionSubscription {
    rx: Receiver<LogEvent>,
}

impl PipelineStateVariableChangeActionSubscription {
    /// Create a new PipelineEventActionSubscription object that can be injected into other structs or functions.
    pub fn subscribe() -> Self {
        let rx = get_pipeline_state_variable_change_action_sender().subscribe();
        Self { rx }
    }

    /// Consumes the current PipelineEventActionSubscription object to produce a Stream of log events.
    pub fn into_stream(self) -> impl Stream<Item = LogEvent> + Unpin {
        BroadcastStream::new(self.rx).filter_map(|e| ready(e.ok()))
    }
}

/// Defines a set of methods intended to enforce the setting of pipeline state variables
pub trait PipelineStateVariableChangeActionLog {
    fn set_pipeline_state_variable(&self, name: String, value: Value, identity: CallsiteIdentity);
}

impl PipelineStateVariableChangeActionLog for Option<MezmoContext> {
    fn set_pipeline_state_variable(&self, name: String, value: Value, identity: CallsiteIdentity) {
        if self.is_none() {
            debug!("PipelineStateVariableChangeActionLog::set_pipeline_state_variable called with no context");
            return;
        }

        let ctx = self.clone().unwrap();

        let mut event = LogEvent::default();
        event.insert(
            "meta.mezmo.pipeline_event_action",
            "set_pipeline_state_variable",
        );
        event.insert("meta.mezmo.account_id", Value::from(&ctx.account_id));
        if ctx.pipeline_id.is_some() {
            event.insert(
                "meta.mezmo.pipeline_id",
                Value::from(ctx.pipeline_id.as_ref().unwrap()),
            );
        }
        event.insert(
            "meta.mezmo.component_id",
            Value::from(ctx.component_id.as_str()),
        );
        event.insert(
            "meta.mezmo.component_kind",
            Value::from(&ctx.component_kind),
        );
        event.insert("meta.mezmo.internal", Value::from(ctx.internal));
        event.insert("message.name", name.clone());
        event.insert("message.value", value.clone());

        let mut state_map = HashMap::new();
        state_map.insert(name, value);

        let serialized = serde_json::to_string(&state_map).unwrap();
        event.insert("message.state_variable", serialized);

        try_send_pipeline_state_variable_change_action(
            event,
            PipelineStateVariableChangeIdentifier {
                component_id: ctx.id.to_owned(),
                identity,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::functions::set_pipeline_state_variable::internal_set_pipeline_state_variable;
    use crate::set_pipeline_state_variable;

    use super::*;
    use serial_test::serial;
    use tokio::{
        select,
        time::{sleep, Duration},
    };

    const TEST_VRL_POSITION: Option<usize> = Some(10);

    #[tokio::test]
    #[serial]
    async fn no_mezmo_context() {
        let mut log_stream =
            PipelineStateVariableChangeActionSubscription::subscribe().into_stream();

        set_pipeline_state_variable!(
            &None,
            None,
            "var1".to_string(),
            Value::from("value1".to_string())
        );

        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_millis(100)) => true,
        };

        assert!(timeout, "expected a timeout since nothing was sent");
    }

    #[tokio::test]
    #[serial]
    async fn test_pipeline_event_set_pipeline_state_variable() {
        let id = "v1:js-script:internal_source:component_abc:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).unwrap();
        let log_stream = PipelineStateVariableChangeActionSubscription::subscribe().into_stream();

        let key = "test_variable1".to_string();
        let val = Value::from("test_value1".to_string());
        _ = internal_set_pipeline_state_variable(&ctx, TEST_VRL_POSITION, key.clone(), val.clone());

        // the expected state map setup
        let mut state_map = HashMap::new();
        state_map.insert(key.clone(), val.clone());
        let expected_serialized_state_variable = serde_json::to_string(&state_map).unwrap();

        // Note: the log channel/stream is global state and not reset between test
        // cases or runs. To avoid hanging awaits, tests should always take the
        // number of elements they expect to find.
        let res: Vec<LogEvent> = log_stream.take(1).collect().await;
        let actual = res
            .into_iter()
            .next()
            .expect("should have received a log event");

        let event_type = actual
            .get(".meta.mezmo.pipeline_event_action")
            .expect("should contain a pipeline_event_action type")
            .as_str()
            .expect("pipeline_event_action should be a string");
        assert_eq!(event_type, "set_pipeline_state_variable".to_string());

        let account_id = actual
            .get(".meta.mezmo.account_id")
            .expect("should contain account_id")
            .as_str()
            .expect("account_id should be a string");
        assert_eq!(account_id, "account_123");

        let pipeline_id = actual
            .get(".meta.mezmo.pipeline_id")
            .expect("should contain pipeline_id")
            .as_str()
            .expect("pipeline_id should be a string");
        assert_eq!(pipeline_id, "pipeline_123");

        let component_id = actual
            .get(".meta.mezmo.component_id")
            .expect("should contain component_id")
            .as_str()
            .expect("component_id should be a string");
        assert_eq!(component_id, "component_abc");

        let component_kind = actual
            .get(".meta.mezmo.component_kind")
            .expect("should contain component_kind")
            .as_str()
            .expect("component_kind should be a string");
        assert_eq!(component_kind, "source");

        let internal = actual
            .get(".meta.mezmo.internal")
            .expect("should contain internal")
            .as_boolean()
            .expect("internal should be a boolean");
        assert!(internal);

        let name = actual
            .get(".message.name")
            .expect("should contain message name")
            .as_str()
            .expect("message.name should be a string");
        assert_eq!(name, "test_variable1");

        let value = actual
            .get(".message.value")
            .expect("should contain a message value")
            .as_str()
            .expect("message.value should be a string");
        assert_eq!(value, "test_value1");

        let actual_state_variable = actual
            .get(".message.state_variable")
            .expect("should contain a message state_variable")
            .as_str()
            .expect("message.state_variable should be a string");
        assert_eq!(actual_state_variable, expected_serialized_state_variable);
    }

    #[tokio::test]
    #[serial]
    async fn pipeline_state_variable_change_test_rate_limiting() {
        let id = "v1:js-script:internal_source:component_def:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        let mut log_stream =
            PipelineStateVariableChangeActionSubscription::subscribe().into_stream();

        for _ in 0..10 {
            set_pipeline_state_variable!(
                ctx,
                None,
                "var1".to_string(),
                Value::from("value1".to_string())
            );
        }

        assert!(log_stream.next().await.is_some(), "expected a log event");

        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_millis(100)) => true,
        };

        assert!(timeout, "expected a timeout because of rate limiting");
    }
}
