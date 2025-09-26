use http::Request;
use std::sync::Arc;

use crate::{config::SinkContext, http::HttpClient, sinks::HealthcheckError};
use mezmo::{user_log_error, user_trace::MezmoUserLog};
use vrl::value::Value;

use super::config::SumoLogicCredentials;

pub(crate) async fn healthcheck(
    client: HttpClient,
    credentials: Arc<SumoLogicCredentials>,
    cx: SinkContext,
) -> crate::Result<()> {
    let uri = credentials.build_uri()?;

    let request = Request::post(uri).body(hyper::Body::empty())?;

    let response = client.send(request).await?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let msg = Value::from(format!(
            "Error returned from destination with status code: {status}",
        ));
        user_log_error!(cx.mezmo_ctx, msg);
    }
    match status {
        status if status.is_success() => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}
