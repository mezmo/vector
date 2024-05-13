use http::Request;

use crate::{
    config::SinkContext,
    http::HttpClient,
    mezmo::user_trace::MezmoUserLog,
    sinks::{
        opentelemetry::{config::OpentelemetryEndpoint, Auth},
        HealthcheckError,
    },
    user_log_error,
};

use vrl::value::Value;

pub(crate) async fn healthcheck(
    endpoint: OpentelemetryEndpoint,
    client: HttpClient,
    auth: Option<Auth>,
    cx: SinkContext,
) -> crate::Result<()> {
    let mut request = Request::post(endpoint.healthcheck()).body(hyper::Body::empty())?;

    if let Some(auth) = auth {
        match auth {
            Auth::Basic(http_auth) => http_auth.apply(&mut request),
        }
    }

    let response = client.send(request).await?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let msg = Value::from(format!(
            "Error returned from destination with status code: {}",
            status
        ));
        user_log_error!(cx.mezmo_ctx, msg);
    }
    match status {
        status if status.is_success() => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}
