use super::callsite::CallsiteIdentity;
use super::MezmoContext;
use bytes::Bytes;
use futures_util::future::BoxFuture;
use futures_util::{future::ready, Stream, StreamExt};
use serde::ser::StdError;
use std::future::Future;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Poll};
use std::time::Instant;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tower::Service;
use tracing::log::Level;
use vector_lib::event::metric::mezmo::TransformError;
use vector_lib::{event::LogEvent, ByteSizeOf};
use vrl::value::Value;

use crate::http::HttpError;
use crate::sinks::util::http::HttpBatchService;
use crate::user_log_error;

static USER_LOG: OnceLock<UserLog> = OnceLock::new();

const DEFAULT_RATE_LIMIT_UNINITIALIZED: u64 = 10; // 10 seconds
const LOG_CACHE_RATE_LIMIT_MAX_CAPACITY: u64 = 5_000;

pub fn init(rate_limit: u64) {
    USER_LOG
        .set(UserLog::new(rate_limit))
        .expect("user log was already initialized");
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct LogIdentifier {
    component_id: String,
    identity: CallsiteIdentity,
}

#[derive(Debug)]
struct State {
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

fn get_user_log_sender() -> &'static broadcast::Sender<LogEvent> {
    &USER_LOG
        .get_or_init(|| UserLog::new(DEFAULT_RATE_LIMIT_UNINITIALIZED))
        .sender
}

fn try_send_user_log(log: LogEvent, rate_limit: Option<u64>, id: LogIdentifier) {
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
                        debug!("user log is [{:?}] is being rate limited", state.log);
                    }
                    _ => {}
                }
            } else {
                // If we saw this event 3 or more times total, emit an event that indicates the total number of times we
                // rate limited the event in the limit period.
                if previous_count > 1 {
                    debug!(
                        "user log [{:?}] has been rate limited {} times.",
                        state.log,
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
            // `captured_data` should always have a value for the sink to insert
            event.insert(
                "meta.mezmo.captured_data",
                captured_data.unwrap_or(Value::Null),
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

// Invalid responses are converted to Errors in most underlying services.
// For others, the message or status code must be extracted from the response.
pub trait UserLoggingResponse {
    fn log_msg(&self) -> Option<Value> {
        None
    }

    fn log_captured_data(&self) -> Option<Value> {
        None
    }
}

pub trait UserLoggingError {
    fn log_msg(&self) -> Option<Value>;
}

/// A wrapping service that tries to log any error results from the inner service to
/// the Mezmo user logs, if defined.
#[derive(Clone)]
pub struct MezmoLoggingService<S> {
    inner: S,
    ctx: Option<MezmoContext>,
}

impl<S> MezmoLoggingService<S> {
    pub(crate) const fn new(inner: S, ctx: Option<MezmoContext>) -> Self {
        Self { inner, ctx }
    }
}

impl<S, Req> Service<Req> for MezmoLoggingService<S>
where
    S: Service<Req> + Send,
    S::Future: 'static + Send,
    S::Response: UserLoggingResponse + Send,
    S::Error: UserLoggingError + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let ctx = self.ctx.clone();
        let res = self.inner.call(req);
        Box::pin(async move {
            let res = res.await;
            match &res {
                Ok(response) => {
                    if let Some(msg) = response.log_msg() {
                        match response.log_captured_data() {
                            Some(captured_data) => {
                                user_log_error!(ctx, msg, captured_data: Value::from(btreemap! {
                                    "captured_data" => captured_data
                                }));
                            }
                            None => {
                                user_log_error!(ctx, msg);
                            }
                        }
                    }
                }
                Err(err) => {
                    if let Some(msg) = err.log_msg() {
                        user_log_error!(ctx, msg);
                    }
                }
            }
            res
        })
    }
}

/// A wrapper around HttpBatchService that logs errors and client-side/server-side
/// response status codes.
pub struct MezmoHttpBatchLoggingService<F, B = Bytes> {
    inner: HttpBatchService<F, B>,
    ctx: Option<MezmoContext>,
}

impl<F, B> MezmoHttpBatchLoggingService<F, B> {
    pub const fn new(inner: HttpBatchService<F, B>, ctx: Option<MezmoContext>) -> Self {
        MezmoHttpBatchLoggingService { inner, ctx }
    }
}

impl<F, B> Service<B> for MezmoHttpBatchLoggingService<F, B>
where
    F: Future<Output = crate::Result<hyper::Request<Bytes>>> + Send + 'static,
    B: ByteSizeOf + Send + 'static,
{
    type Response = <HttpBatchService<F, B> as Service<B>>::Response;
    type Error = <HttpBatchService<F, B> as Service<B>>::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, body: B) -> Self::Future {
        let ctx = self.ctx.clone();
        let res = self.inner.call(body);
        Box::pin(async move {
            let res = res.await;
            match &res {
                Ok(response) => {
                    if response.status().is_client_error() || response.status().is_server_error() {
                        let msg = Value::from(format!(
                            "Error returned from destination with status code: {}",
                            response.status()
                        ));
                        user_log_error!(ctx, msg);
                    }
                }
                Err(err) => match err.deref().downcast_ref::<HttpError>() {
                    Some(err) => {
                        user_log_error!(ctx, Value::from(format!("{err}")));
                    }
                    None => {
                        warn!(message = "Unable to format service error for user logs", %err);
                        user_log_error!(ctx, Value::from("Request failed".to_string()));
                    }
                },
            }
            res
        })
    }
}

impl<F, B> Clone for MezmoHttpBatchLoggingService<F, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

pub fn handle_transform_error(ctx: &Option<MezmoContext>, err: TransformError) {
    match err {
        TransformError::FieldNull { field } => {
            user_log_error!(ctx, format!("Required field '{}' is null", field));
        }
        TransformError::FieldNotFound { field } => {
            user_log_error!(
                ctx,
                format!("Required field '{}' not found in the log event", field)
            );
        }
        TransformError::FieldInvalidType { field } => {
            user_log_error!(ctx, format!("Field '{}' type is not valid", field));
        }
        TransformError::InvalidMetricType { type_name } => {
            user_log_error!(ctx, format!("Metric type '{}' is not supported", type_name));
        }
        TransformError::ParseIntOverflow { field } => {
            user_log_error!(
                ctx,
                format!(
                    "Field '{}' could not be parsed as an unsigned integer",
                    field
                )
            );
        }
        TransformError::NumberTruncation { field } => {
            user_log_error!(
                ctx,
                format!("Field '{}' was truncated during parsing", field)
            );
        }
    };
}

pub fn handle_deserializer_error(ctx: &Option<MezmoContext>, err: Box<dyn StdError>) {
    user_log_error!(ctx, format!("Protobuf validation failed: {}", err));
}

#[cfg(test)]
mod tests {
    use crate::{user_log, user_log_debug, user_log_info, user_log_warn};

    use super::*;
    use serial_test::serial;
    use snafu::Snafu;
    use tokio::{
        select,
        time::{sleep, Duration},
    };
    use tokio_test::{assert_pending, assert_ready};
    use tower_test::{
        assert_request_eq,
        mock::{self as service_mock, Mock},
    };

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
            res[0]
                .get(".meta.mezmo.captured_data")
                .unwrap()
                .as_str()
                .unwrap(),
            "some string captured_data",
            "captured_data can be a string"
        );
        assert_eq!(
            res[1].get(".meta.mezmo.captured_data").unwrap(),
            &Value::from(vec![1, 2, 3]),
            "captured_data can be an array"
        );
        assert_eq!(
            res[2].get(".meta.mezmo.captured_data").unwrap(),
            &Value::from(btreemap! {
                "message" => "some event message",
                "metadata" => btreemap! {
                    "status" => 2,
                    "method" => "POST",
                }
            }),
            "captured_data can be an object"
        );
        assert_eq!(
            res[3]
                .get(".meta.mezmo.captured_data")
                .unwrap()
                .as_str()
                .unwrap(),
            "a debug captured_data",
            "explicit debug call has captured_data"
        );
        assert_eq!(
            res[3].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "DEBUG",
            "explicit debug call has level"
        );
        assert_eq!(
            res[4]
                .get(".meta.mezmo.captured_data")
                .unwrap()
                .as_str()
                .unwrap(),
            "an info captured_data",
            "explicit info call has captured_data"
        );
        assert_eq!(
            res[4].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "INFO",
            "explicit info call has level"
        );
        assert_eq!(
            res[5]
                .get(".meta.mezmo.captured_data")
                .unwrap()
                .as_str()
                .unwrap(),
            "a warn captured_data",
            "explicit warn call has captured_data"
        );
        assert_eq!(
            res[5].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "WARN",
            "explicit warn call has level"
        );
        assert_eq!(
            res[6]
                .get(".meta.mezmo.captured_data")
                .unwrap()
                .as_str()
                .unwrap(),
            "an error captured_data",
            "explicit error call has captured_data"
        );
        assert_eq!(
            res[6].get(".meta.mezmo.level").unwrap().as_str().unwrap(),
            "ERROR",
            "explicit error call has level"
        );
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("Unit test error"))]
    struct ServiceTestError {}
    impl UserLoggingError for ServiceTestError {
        fn log_msg(&self) -> Option<Value> {
            Some("log_msg(): error".into())
        }
    }

    struct ServiceTestResponse {}
    impl UserLoggingResponse for ServiceTestResponse {
        fn log_msg(&self) -> Option<Value> {
            Some("log_msg(): response".into())
        }

        fn log_captured_data(&self) -> Option<Value> {
            Some(Value::Object(btreemap! {
                "response" => r#"{"error": "badness"}"#
            }))
        }
    }

    struct MockWrapperService {
        inner: Mock<&'static str, &'static str>,
    }

    impl Service<&'static str> for MockWrapperService {
        type Response = ServiceTestResponse;
        type Error = ServiceTestError;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx).map_err(|_| ServiceTestError {})
        }

        fn call(&mut self, req: &'static str) -> Self::Future {
            let res = self.inner.call(req);
            Box::pin(async move {
                match res.await {
                    Ok(_) => Ok(ServiceTestResponse {}),
                    Err(_) => Err(ServiceTestError {}),
                }
            })
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_mezmo_logging_service_error() {
        let mut log_stream = UserLogSubscription::subscribe().into_stream();
        let (mut mock, mut handle) =
            service_mock::spawn_with(|mock: service_mock::Mock<&str, &str>| {
                let id =
                    "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
                let ctx = MezmoContext::try_from(id).ok();
                MezmoLoggingService::new(MockWrapperService { inner: mock }, ctx)
            });

        assert_pending!(handle.poll_request());
        assert_ready!(mock.poll_ready()).unwrap();

        let res = mock.call("input");
        assert_request_eq!(handle, "input").send_error(ServiceTestError {});
        assert!(res.await.is_err());

        let log_res = select! {
            event = log_stream.next() => event.expect("next should produce a value"),
            _ = sleep(Duration::from_secs(1)) => panic!("fetching log stream event should not time out"),
        };

        let msg = log_res.get(".message").unwrap().as_str().unwrap();
        assert_eq!(msg, "log_msg(): error");
    }

    #[tokio::test]
    #[serial]
    async fn test_mezmo_logging_service_response() {
        let mut log_stream = UserLogSubscription::subscribe().into_stream();
        let (mut mock, mut handle) =
            service_mock::spawn_with(|mock: service_mock::Mock<&str, &str>| {
                let id =
                    "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
                let ctx = MezmoContext::try_from(id).ok();
                MezmoLoggingService::new(MockWrapperService { inner: mock }, ctx)
            });

        assert_pending!(handle.poll_request());
        assert_ready!(mock.poll_ready()).unwrap();

        let res = mock.call("input");
        assert_request_eq!(handle, "input").send_response("testing");
        assert!(res.await.is_ok());

        let log_res = select! {
            event = log_stream.next() => event.expect("next should produce a value"),
            _ = sleep(Duration::from_secs(1)) => panic!("fetching log stream event should not time out"),
        };

        let msg = log_res.get(".message").unwrap().as_str().unwrap();
        assert_eq!(msg, "log_msg(): response");

        let captured_data = log_res
            .get(".meta.mezmo.captured_data")
            .expect("captured data should exist");

        assert_eq!(
            captured_data,
            &Value::Object(btreemap! {
                "captured_data" => btreemap!{
                    "response" => r#"{"error": "badness"}"#
                }
            })
        );
    }
}
