use crate::{callsite::CallsiteIdentity, context::MezmoContext, user_log_error};
use futures_util::{future::ready, Stream, StreamExt};
use serde::ser::StdError;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tracing::log::Level;
use tracing::{debug, warn};
use vector_core::event::metric::mezmo::TransformError;
use vector_core::event::LogEvent;
use vrl::btreemap;
use vrl::value::Value;

static USER_LOG: OnceLock<UserLog> = OnceLock::new();

const DEFAULT_RATE_LIMIT_UNINITIALIZED: u64 = 10; // 10 seconds
const LOG_CACHE_RATE_LIMIT_MAX_CAPACITY: u64 = 5_000;

pub fn init(rate_limit: u64) {
    USER_LOG
        .set(UserLog::new(rate_limit))
        .expect("user log was already initialized");
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct LogIdentifier {
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
struct UserLog {
    sender: Sender<LogEvent>,
    log_rate_limit_cache: moka::sync::Cache<LogIdentifier, Arc<Mutex<State>>>,
    rate_limit: u64,
}

impl UserLog {
    fn new(rate_limit: u64) -> Self {
        UserLog {
            sender: broadcast::channel(1000).0,
            log_rate_limit_cache: moka::sync::Cache::new(LOG_CACHE_RATE_LIMIT_MAX_CAPACITY),
            rate_limit,
        }
    }
}

pub fn get_user_log_sender() -> &'static broadcast::Sender<LogEvent> {
    &USER_LOG
        .get_or_init(|| UserLog::new(DEFAULT_RATE_LIMIT_UNINITIALIZED))
        .sender
}

pub fn try_send_user_log(log: LogEvent, rate_limit: Option<u64>, id: LogIdentifier) {
    if let Some(user_log) = USER_LOG.get() {
        let entry = user_log.log_rate_limit_cache.entry(id).or_insert_with(|| {
            Arc::new(Mutex::new(State::new(
                log.clone(),
                rate_limit.unwrap_or(user_log.rate_limit),
            )))
        });

        if let Ok(state) = Arc::<Mutex<State>>::clone(entry.value()).lock().as_mut() {
            let previous_count = state.increment_count();
            if state.should_limit() {
                match previous_count {
                    0 => match user_log.sender.send(log) {
                        Ok(_) => {}
                        Err(_) => {
                            debug!("failed to send user log; likely no source consuming data")
                        }
                    },
                    1 => {
                        debug!(
                            "user log is [{:?}] is being rate limited",
                            state.log.get_message().unwrap_or(&("".into()))
                        );
                    }
                    _ => {}
                }
            } else {
                // If we saw this event 3 or more times total, emit an event that indicates the total number of times we
                // rate limited the event in the limit period.
                if previous_count > 1 {
                    debug!(
                        "user log [{:?}] has been rate limited {} times.",
                        state.log.get_message().unwrap_or(&("".into())),
                        previous_count - 1
                    );
                }

                // We're not rate limiting anymore, so we also emit the current event as normal.. but we update our rate
                // limiting state since this is effectively equivalent to seeing the event again for the first time.
                match user_log.sender.send(log) {
                    Ok(_) => {}
                    Err(_) => {
                        debug!("failed to send user log; likely no source consuming data")
                    }
                }

                state.reset();
            }
        }
    }
}

/// This is the struct used to obtain access to consume the user log information in other
/// parts of the code base.
pub struct UserLogSubscription {
    rx: Receiver<LogEvent>,
}

impl UserLogSubscription {
    /// Create a new UserLogSubscription object that can be injected into other structs or
    /// functions.
    pub fn subscribe() -> Self {
        let rx = get_user_log_sender().subscribe();
        Self { rx }
    }

    /// Consumes the current UserLogSubscription object to produce a Stream of log events.
    pub fn into_stream(self) -> impl Stream<Item = LogEvent> + Unpin {
        BroadcastStream::new(self.rx).filter_map(|e| ready(e.ok()))
    }
}

/// Defines a set of methods that can be used to generate log messages that are intended for
/// the end user of the pipeline to see.
pub trait MezmoUserLog {
    fn log(
        &self,
        level: Level,
        msg: Value,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    );

    fn debug(
        &self,
        msg: impl Into<Value>,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    ) {
        self.log(
            Level::Debug,
            msg.into(),
            rate_limit,
            captured_data,
            identity,
        );
    }

    fn info(
        &self,
        msg: impl Into<Value>,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    ) {
        self.log(Level::Info, msg.into(), rate_limit, captured_data, identity);
    }

    fn warn(
        &self,
        msg: impl Into<Value>,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    ) {
        self.log(Level::Warn, msg.into(), rate_limit, captured_data, identity);
    }

    fn error(
        &self,
        msg: impl Into<Value>,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    ) {
        self.log(
            Level::Error,
            msg.into(),
            rate_limit,
            captured_data,
            identity,
        );
    }
}

impl MezmoUserLog for Option<MezmoContext> {
    fn log(
        &self,
        level: Level,
        msg: Value,
        rate_limit: Option<u64>,
        captured_data: Option<Value>,
        identity: CallsiteIdentity,
    ) {
        if let Some(ctx) = self {
            let mut event = LogEvent::default();
            event.insert("meta.mezmo.level", Value::from(level.to_string()));
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

            // To preserve data types of the `captured data`, AND to ensure data type consistency
            // with the DB that stores this data, wrap the real `captured_data` value within
            // a btreemap! keyed by "captured_data". This way, the JSONB column in the DB
            // can store captured data of any data type.
            event.insert(
                "meta.mezmo.captured_data",
                // `captured_data` should always have a value for the postgres sink to insert.
                captured_data.map_or_else(
                    || Value::Null,
                    |v| Value::from(btreemap! { "captured_data" => v }),
                ),
            );
            event.insert("message", msg);
            try_send_user_log(
                event,
                rate_limit,
                LogIdentifier {
                    component_id: ctx.id.to_owned(),
                    identity,
                },
            );
        } else if cfg!(debug_assertions) {
            // Components that lack a valid component ID (and thus, a valid MezmoContext) will
            // not emit any user_logs, which is unexpected during testing and difficult to
            // track down without reading this code. Warn the developer, save their sanity.
            warn!("A user_log was emitted without a valid component_id or `MezmoContext`")
        }
    }
}

/// Emits a user_log for the given error. Note that the Display implementation must be
/// appropriate as a user-facing error.
pub fn handle_transform_error(ctx: &Option<MezmoContext>, err: TransformError) {
    user_log_error!(ctx, err.to_string());
}

pub fn handle_deserializer_error(ctx: &Option<MezmoContext>, err: Box<dyn StdError>) {
    user_log_error!(ctx, format!("Protobuf validation failed: {}", err));
}

#[cfg(test)]
mod tests {
    use crate::{user_log, user_log_debug, user_log_info, user_log_warn};

    use super::*;
    use serial_test::serial;
    use tokio::{
        select,
        time::{sleep, Duration},
    };

    // The exitence of the captured data wrapper is a hidden implementation detail.
    // This function will create the wrapper so the values can be tested more directly
    // against what the value of the `.meta.mezmo.captured_data` field will be.
    fn captured_data_wrapper(captured_data: Value) -> Value {
        Value::from(btreemap! {
            "captured_data" => captured_data
        })
    }

    #[tokio::test]
    #[serial]
    async fn test_logging_from_standard_component() {
        let id = "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        let log_stream = UserLogSubscription::subscribe().into_stream();

        user_log_debug!(ctx, "debug msg");
        user_log_info!(ctx, "info msg");
        user_log_warn!(ctx, "warn msg");
        user_log_error!(ctx, "error msg");

        // Note: the log channel/stream is global state and not reset between test
        // cases or runs. To avoid hanging awaits, tests should always take the
        // number of elements they expect to find.
        let res: Vec<LogEvent> = log_stream.take(4).collect().await;

        let expectd_values = vec![
            ("DEBUG", "debug msg"),
            ("INFO", "info msg"),
            ("WARN", "warn msg"),
            ("ERROR", "error msg"),
        ];

        for ((exp_level, exp_msg), actual) in expectd_values.into_iter().zip(res.iter()) {
            let level = actual
                .get(".meta.mezmo.level")
                .expect("should contain a level value")
                .as_str()
                .expect("level should be a string");
            assert_eq!(level, exp_level);

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

            let captured_data = actual
                .get(".meta.mezmo.captured_data")
                .expect("should contain captured_data")
                .as_str();
            assert_eq!(captured_data, None);

            let msg = actual
                .get(".message")
                .expect("should contain a message")
                .as_str()
                .expect("message should be a string");
            assert_eq!(msg, exp_msg);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_logging_from_nonstandard_component() {
        let id = "random_component_name".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        assert!(
            ctx.is_none(),
            "test expected a None context as a precondition"
        );

        let mut log_stream = UserLogSubscription::subscribe().into_stream();

        user_log_debug!(ctx, "debug msg");
        user_log_info!(ctx, "info msg");
        user_log_warn!(ctx, "warn msg");
        user_log_error!(ctx, "error msg");

        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_secs(1)) => true,
        };

        assert!(
            timeout,
            "expected empty log_stream for nonstandard context values"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_rate_limiting() {
        let id = "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        let mut log_stream = UserLogSubscription::subscribe().into_stream();

        for _ in 0..10 {
            user_log_error!(ctx, "error msg", rate_limit_secs: 2);
        }

        assert!(log_stream.next().await.is_some(), "expected an error log");

        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_millis(100)) => true,
        };

        assert!(timeout, "expected a timeout because of rate limiting");
    }

    #[tokio::test]
    #[serial]
    async fn test_input() {
        let id = "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        let log_stream = UserLogSubscription::subscribe().into_stream();

        // Using the helper macros
        user_log_error!(ctx, "error msg", captured_data: Value::from("some string captured_data"));
        user_log_warn!(ctx, "warning msg", captured_data: Value::from(vec![1, 2, 3]));
        user_log_error!(ctx, "error msg with object", captured_data: Value::from(btreemap! {
            "message" => "some event message",
            "metadata" => btreemap! {
                "status" => 2,
                "method" => "POST",
            }
        }));

        // `captured_data` can be included in other calls as long as all parameters are specified
        user_log!(
            "debug",
            ctx,
            "debug msg",
            None,
            Some(Value::from("a debug captured_data")),
            None
        );
        user_log!(
            "info",
            ctx,
            "info msg",
            None,
            Some(Value::from("an info captured_data")),
            None
        );
        user_log!(
            "warn",
            ctx,
            "warn msg",
            None,
            Some(Value::from("a warn captured_data")),
            None
        );
        user_log!(
            "error",
            ctx,
            "error msg",
            None,
            Some(Value::from("an error captured_data")),
            None
        );

        let res: Vec<LogEvent> = log_stream.take(7).collect().await;

        assert_eq!(res.len(), 7, "number of log events is correct");
        assert_eq!(
            res[0].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from("some string captured_data")),
            "captured_data can be a string"
        );
        assert_eq!(
            res[1].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from(vec![1, 2, 3])),
            "captured_data can be an array"
        );
        assert_eq!(
            res[2].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from(btreemap! {
                "message" => "some event message",
                "metadata" => btreemap! {
                    "status" => 2,
                    "method" => "POST",
                }
            })),
            "captured_data can be an object"
        );
        assert_eq!(
            res[3].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from("a debug captured_data")),
            "explicit debug call has captured_data"
        );
        assert_eq!(
            res[3].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "DEBUG",
            "explicit debug call has level"
        );
        assert_eq!(
            res[4].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from("an info captured_data")),
            "explicit info call has captured_data"
        );
        assert_eq!(
            res[4].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "INFO",
            "explicit info call has level"
        );
        assert_eq!(
            res[5].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from("a warn captured_data")),
            "explicit warn call has captured_data"
        );
        assert_eq!(
            res[5].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "WARN",
            "explicit warn call has level"
        );
        assert_eq!(
            res[6].get(".meta.mezmo.captured_data").unwrap(),
            &captured_data_wrapper(Value::from("an error captured_data")),
            "explicit error call has captured_data"
        );
        assert_eq!(
            res[6].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "ERROR",
            "explicit error call has level"
        );
    }
}
