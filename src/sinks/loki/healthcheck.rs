use super::config::LokiConfig;
use crate::{config::SinkContext, http::HttpClient};
use mezmo::{user_log_error, user_trace::MezmoUserLog};
use vrl::value::Value;

async fn fetch_status(
    endpoint: &str,
    config: &LokiConfig,
    client: &HttpClient,
) -> crate::Result<http::StatusCode> {
    let endpoint = config.endpoint.append_path(endpoint)?;

    let mut req = http::Request::get(endpoint.uri)
        .body(hyper::Body::empty())
        .expect("Building request never fails.");

    if let Some(auth) = &config.auth {
        auth.apply(&mut req);
    }

    Ok(client.send(req).await?.status())
}

pub async fn healthcheck(
    config: LokiConfig,
    client: HttpClient,
    cx: SinkContext,
) -> crate::Result<()> {
    let status = match fetch_status("ready", &config, &client).await? {
        // Issue https://github.com/vectordotdev/vector/issues/6463
        http::StatusCode::NOT_FOUND => {
            debug!("Endpoint `/ready` not found. Retrying healthcheck with top level query.");
            fetch_status("", &config, &client).await?
        }
        status => status,
    };

    if status.is_client_error() || status.is_server_error() {
        let msg = Value::from(format!(
            "Error returned from destination with status code: {}",
            status
        ));
        user_log_error!(cx.mezmo_ctx, msg);
    }

    match status {
        http::StatusCode::OK => Ok(()),
        _ => Err(format!("A non-successful status returned: {}", status).into()),
    }
}
