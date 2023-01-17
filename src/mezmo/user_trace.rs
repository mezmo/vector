use super::MezmoContext;
use futures_util::{future::ready, Stream, StreamExt};
use once_cell::sync::OnceCell;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tower::Service;
use tracing::log::Level;
use value::Value;
use vector_core::event::LogEvent;

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
    fn log(&self, level: Level, msg: &str);

    fn debug(&self, msg: &str) {
        self.log(Level::Debug, msg);
    }

    fn info(&self, msg: &str) {
        self.log(Level::Info, msg);
    }

    fn warn(&self, msg: &str) {
        self.log(Level::Warn, msg);
    }

    fn error(&self, msg: &str) {
        self.log(Level::Error, msg);
    }
}

impl MezmoUserLog for Option<MezmoContext> {
    fn log(&self, level: Level, msg: &str) {
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
            event.insert("message", Value::from(msg));
            try_send_user_log(event);
        }
    }
}

/// A wrapping service that tries to log any error results from the inner service to
/// the Mezmo user logs, if defined.
struct MezmoLoggingService<S> {
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
    S: Service<Req>,
    S::Future: 'static,
    S::Error: Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn core::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let ctx = self.ctx.clone();
        let res = self.inner.call(req);
        Box::pin(async move {
            let res = res.await;
            if let Err(value) = &res {
                // Expand this as we learn more about specific use cases for the logs
                let log_msg = format!("{value:?}");
                ctx.error(&log_msg);
            }
            res
        })
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
    use tower_test::{assert_request_eq, mock as service_mock};

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
    #[snafu(display("Unit test error: {note}"))]
    struct ServiceTestError {
        note: &'static str,
    }

    #[tokio::test]
    async fn test_mezmo_logging_service() {
        let mut log_stream = UserLogSubscription::subscribe().into_stream();
        let (mut mock, mut handle) =
            service_mock::spawn_with(|mock: service_mock::Mock<&str, &str>| {
                let id =
                    "v1:kafka:internal_source:component_abc:pipeline_123:account_123".to_owned();
                let ctx = MezmoContext::try_from(id).ok();
                MezmoLoggingService::new(mock, ctx)
            });

        assert_pending!(handle.poll_request());
        assert_ready!(mock.poll_ready()).unwrap();

        let res = mock.call("input");
        assert_request_eq!(handle, "input").send_error(ServiceTestError { note: "test error" });
        assert!(res.await.is_err());

        let log_res = select! {
            event = log_stream.next() => event.expect("next should produce a value"),
            _ = sleep(Duration::from_secs(1)) => panic!("fetching log stream event should not time out"),
        };

        let msg = log_res.get(".message").unwrap().as_str().unwrap();
        assert_eq!(msg, "ServiceTestError { note: \"test error\" }");
    }
}
