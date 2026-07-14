use std::sync::Arc;

use http::Request;

use super::{NewRelicApi, NewRelicCredentials};
use crate::{http::HttpClient, sinks::HealthcheckError};

pub(crate) async fn healthcheck(
    client: HttpClient,
    credentials: Arc<NewRelicCredentials>,
) -> crate::Result<()> {
    // The Events/Metrics/Logs ingest APIs accept an empty POST body for a connectivity/auth check.
    // The Trace API rejects an empty body, so send a minimal valid (empty) trace payload with the
    // required format headers instead.
    let request = match credentials.api {
        NewRelicApi::Traces => Request::post(credentials.get_uri())
            .header("Api-Key", credentials.license_key.clone())
            .header("Content-Type", "application/json")
            .header("Data-Format", "newrelic")
            .header("Data-Format-Version", "1")
            .body(hyper::Body::from(r#"[{"spans":[]}]"#))
            .unwrap(),
        _ => Request::post(credentials.get_uri())
            .header("Api-Key", credentials.license_key.clone())
            .body(hyper::Body::empty())
            .unwrap(),
    };

    let response = client.send(request).await?;

    match response.status() {
        status if status.is_success() => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}
