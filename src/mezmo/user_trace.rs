use super::MezmoContext;
use bytes::Bytes;
use futures_util::future::BoxFuture;
use futures_util::{future::ready, Stream, StreamExt};
use once_cell::sync::OnceCell;
use std::future::Future;
use std::task::{Context, Poll};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tower::Service;
use tracing::log::Level;
use value::Value;
use vector_core::{event::LogEvent, ByteSizeOf};

use crate::sinks::util::http::HttpBatchService;

static USER_LOG_SENDER: OnceCell<Sender<LogEvent>> = OnceCell::new();

fn get_user_log_sender() -> &'static broadcast::Sender<LogEvent> {
    USER_LOG_SENDER.get_or_init(|| broadcast::channel(1000).0)
}

fn try_send_user_log(log: LogEvent) {
    if let Some(sender) = USER_LOG_SENDER.get() {
        match sender.send(log) {
            Ok(_) => {}
            Err(_) => debug!("failed to send user log; likely no source consuming data"),
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
    fn log(&self, level: Level, msg: Value);

    fn debug(&self, msg: impl Into<Value>) {
        self.log(Level::Debug, msg.into());
    }

    fn info(&self, msg: impl Into<Value>) {
        self.log(Level::Info, msg.into());
    }

    fn warn(&self, msg: impl Into<Value>) {
        self.log(Level::Warn, msg.into());
    }

    fn error(&self, msg: impl Into<Value>) {
        self.log(Level::Error, msg.into());
    }
}

impl MezmoUserLog for Option<MezmoContext> {
    fn log(&self, level: Level, msg: Value) {
        if let Some(ctx) = self {
            let mut event = LogEvent::default();
            event.insert("meta.mezmo.level", Value::from(level.to_string()));
            event.insert("meta.mezmo.account_id", Value::from(&ctx.account_id));
            event.insert("meta.mezmo.pipeline_id", Value::from(&ctx.pipeline_id));
            event.insert(
                "meta.mezmo.component_id",
                Value::from(ctx.component_id.as_str()),
            );
            event.insert(
                "meta.mezmo.component_kind",
                Value::from(&ctx.component_kind),
            );
            event.insert("meta.mezmo.internal", Value::from(ctx.internal));
            event.insert("message", msg);
            try_send_user_log(event);
        }
    }
}

// Invalid responses are converted to Errors in most underlying services.
// For others, the message or status code must be extracted from the response.
pub trait UserLoggingResponse {
    fn log_msg(&self) -> Option<Value> {
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
                        ctx.error(msg);
                    }
                }
                Err(err) => {
                    if let Some(msg) = err.log_msg() {
                        ctx.error(msg);
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
    pub fn new(inner: HttpBatchService<F, B>, ctx: Option<MezmoContext>) -> Self {
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
                        ctx.error(msg);
                    }
                }
                Err(err) => {
                    let msg = Value::from(format!("{err:?}"));
                    ctx.error(msg);
                }
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

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_logging_from_standard_component() {
        let id = "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        let log_stream = UserLogSubscription::subscribe().into_stream();

        ctx.debug("debug msg");
        ctx.info("info msg");
        ctx.warn("warn msg");
        ctx.error("error msg");

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

            let msg = actual
                .get(".message")
                .expect("should contain a message")
                .as_str()
                .expect("message should be a string");
            assert_eq!(msg, exp_msg);
        }
    }

    #[tokio::test]
    async fn test_logging_from_nonstandard_component() {
        let id = "random_component_name".to_owned();
        let ctx = MezmoContext::try_from(id).ok();
        assert!(
            ctx.is_none(),
            "test expected a None context as a precondition"
        );

        let mut log_stream = UserLogSubscription::subscribe().into_stream();

        ctx.debug("debug msg");
        ctx.info("info msg");
        ctx.warn("warn msg");
        ctx.error("error msg");

        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_secs(1)) => true,
        };

        assert!(
            timeout,
            "expected empty log_stream for nonstandard context values"
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
            let res = self.inner.call(req.clone());
            Box::pin(async move {
                match res.await {
                    Ok(_) => Ok(ServiceTestResponse {}),
                    Err(_) => Err(ServiceTestError {}),
                }
            })
        }
    }

    #[tokio::test]
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
        assert!(res.await.is_err());

        let log_res = select! {
            event = log_stream.next() => event.expect("next should produce a value"),
            _ = sleep(Duration::from_secs(1)) => panic!("fetching log stream event should not time out"),
        };

        let msg = log_res.get(".message").unwrap().as_str().unwrap();
        assert_eq!(msg, "log_msg(): response");
    }
}
