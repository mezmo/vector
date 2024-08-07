use bytes::Bytes;
use futures_util::future::BoxFuture;
use futures_util::{future::ready, Stream, StreamExt};
use std::future::Future;
use std::ops::Deref;
use std::task::{Context, Poll};
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::BroadcastStream;
use tower::Service;
use vector_lib::{event::LogEvent, ByteSizeOf};
use vrl::value::Value;

use mezmo::{
    user_log_error,
    user_trace::{get_user_log_sender, MezmoUserLog},
    MezmoContext,
};

use crate::http::HttpError;
use crate::sinks::util::http::HttpBatchService;

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
                                user_log_error!(ctx, msg, captured_data: captured_data);
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
#[cfg(test)]
mod tests {
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

    // The exitence of the captured data wrapper is a hidden implementation detail.
    // This function will create the wrapper so the values can be tested more directly
    // against what the value of the `.meta.mezmo.captured_data` field will be.
    fn captured_data_wrapper(captured_data: Value) -> Value {
        Value::from(btreemap! {
            "captured_data" => captured_data
        })
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
            &captured_data_wrapper(Value::Object(btreemap! {
                "response" => r#"{"error": "badness"}"#
            }))
        );
    }
}
