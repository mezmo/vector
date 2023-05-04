use http::Request;
use std::sync::Arc;

use crate::{http::HttpClient, sinks::HealthcheckError};

use super::config::SumoLogicCredentials;

pub(crate) async fn healthcheck(
    client: HttpClient,
    credentials: Arc<SumoLogicCredentials>,
) -> crate::Result<()> {
    let uri = credentials.build_uri()?;

    let request = Request::post(uri).body(hyper::Body::empty())?;

    let response = client.send(request).await?;

    match response.status() {
        status if status.is_success() => Ok(()),
        other => Err(HealthcheckError::UnexpectedStatus { status: other }.into()),
    }
}
