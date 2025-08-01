use async_stream::stream;
use bytes::Buf;
use futures::Stream;
use hyper::Body;
use indexmap::IndexMap;
use serde_json;
use tokio::time;
use url::Url;
use vector_lib::configurable::configurable_component;

use crate::{
    config::{self, provider::ProviderConfig, ProxyConfig},
    http::HttpClient,
    signal,
    tls::{TlsConfig, TlsSettings},
};

use super::BuildResult;

/// Request settings.
#[configurable_component]
#[derive(Clone, Debug)]
pub struct RequestConfig {
    /// HTTP headers to add to the request.
    #[serde(default)]
    pub headers: IndexMap<String, String>,

    /// Payload sent in the request. If present, request is POSTed with uptime_sec included.
    #[serde(default)]
    pub payload: Option<String>,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            headers: IndexMap::new(),
            payload: None,
        }
    }
}

/// Configuration for the `http` provider.
#[configurable_component(provider("http"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct HttpConfig {
    /// URL for the HTTP provider.
    url: Option<Url>,

    #[configurable(derived)]
    request: RequestConfig,

    /// How often to poll the provider, in seconds.
    poll_interval_secs: u64,

    #[serde(flatten)]
    tls_options: Option<TlsConfig>,

    #[configurable(derived)]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    proxy: ProxyConfig,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: None,
            request: RequestConfig::default(),
            poll_interval_secs: 30,
            tls_options: None,
            proxy: Default::default(),
        }
    }
}

/// Makes an HTTP request to the provided endpoint, returning the String body.
async fn http_request(
    url: &Url,
    tls_options: &Option<TlsConfig>,
    headers: &IndexMap<String, String>,
    payload: &Option<String>,
    proxy: &ProxyConfig,
) -> Result<bytes::Bytes, String> {
    let tls_settings = TlsSettings::from_options(tls_options).map_err(|_| "Invalid TLS options")?;
    let http_client =
        HttpClient::<Body>::new(tls_settings, proxy).map_err(|_| "Invalid TLS settings")?;

    // Build HTTP request.
    let mut builder = http::request::Builder::new().uri(url.to_string());

    // Augment with headers. These may be required e.g. for authentication to
    // private endpoints.
    for (header, value) in headers.iter() {
        builder = builder.header(header.as_str(), value.as_str());
    }

    // If a payload is provided, we're POSTing that to the provider endpoint instead of a basic GET
    let res = match payload {
        Some(p) => builder
            .method("POST")
            .header("Content-Type", "application/json")
            .body(p.clone().into()),
        None => builder.body(Body::empty()),
    };

    let request = res.map_err(|_| "Couldn't create HTTP request".to_string())?;

    debug!(
        message = "Attempting to retrieve configuration.",
        url = ?url.as_str()
    );

    let response = http_client.send(request).await.map_err(|err| {
        let message = "HTTP error";
        error!(
            message = ?message,
            error = ?err,
            url = ?url.as_str());

        format!("{message}. Error: {err:?}")
    })?;

    info!(
        message = "Config response received.",
        url = ?url.as_str(),
        status_code = ?response.status()
    );

    hyper::body::to_bytes(response.into_body())
        .await
        .map_err(|err| {
            let message = "Error interpreting response.";
            let cause = err.into_cause();
            error!(
                    message = ?message,
                    error = ?cause);

            format!("{message} Error: {cause:?}")
        })
}

/// Calls `http_request`, serializing the result to a `ConfigBuilder`.
async fn http_request_to_config_builder(
    url: &Url,
    tls_options: &Option<TlsConfig>,
    headers: &IndexMap<String, String>,
    payload: &Option<String>,
    proxy: &ProxyConfig,
) -> BuildResult {
    let config_str = http_request(url, tls_options, headers, payload, proxy)
        .await
        .map_err(|e| vec![e])?;

    config::load(config_str.chunk(), crate::config::format::Format::Toml)?
}

/// Polls the HTTP endpoint after/every `poll_interval_secs`, returning a stream of `ConfigBuilder`.
fn poll_http(
    poll_interval_secs: u64,
    url: Url,
    tls_options: Option<TlsConfig>,
    headers: IndexMap<String, String>,
    mut heartbeat: Option<serde_json::Value>,
    proxy: ProxyConfig,
    mut loaded_config_hash: String,
) -> impl Stream<Item = signal::SignalTo> {
    let start_time = time::Instant::now();
    let duration = time::Duration::from_secs(poll_interval_secs);
    let mut interval = time::interval_at(start_time + duration, duration);

    stream! {
        loop {
            interval.tick().await;
            let uptime_sec = start_time.elapsed().as_secs();
            match http_request_to_config_builder(&url, &tls_options, &headers, &get_current_heartbeat_payload(&mut heartbeat, uptime_sec), &proxy).await {
                Ok(config_builder) => {
                    let current_hash = config_builder.sha256_hash();
                    // Make sure we only send the reload signal when the config changed
                    if current_hash != loaded_config_hash {
                        info!("Sending reload config signal");
                        loaded_config_hash = current_hash;

                        yield signal::SignalTo::ReloadFromConfigBuilder(config_builder)
                    }
                },
                Err(e) => {
                    error!("Error loading configuration from HTTP: {e:?}");
                },
            };

            debug!(
                message = "HTTP provider is waiting.",
                poll_interval_secs = ?poll_interval_secs,
                url = ?url.as_str());
        }
    }
}

impl ProviderConfig for HttpConfig {
    async fn build(&mut self, signal_handler: &mut signal::SignalHandler) -> BuildResult {
        let url = self
            .url
            .take()
            .ok_or_else(|| vec!["URL is required for the `http` provider.".to_owned()])?;

        let tls_options = self.tls_options.take();
        let poll_interval_secs = self.poll_interval_secs;
        let request = self.request.clone();

        let mut heartbeat = match request.payload {
            Some(p) => match serde_json::from_str::<serde_json::Value>(p.as_str()) {
                Ok(v) => {
                    if v.is_object() {
                        Some(v)
                    } else {
                        return Err(vec![format!(
                            "HTTTP Provider request payload \"{p}\" is not a json object"
                        )]);
                    }
                }
                Err(e) => {
                    return Err(vec![format!(
                        "Error parsing HTTP Provider request payload \"{p}\" as json: {e:?}"
                    )]);
                }
            },
            None => None,
        };

        let proxy = ProxyConfig::from_env().merge(&self.proxy);
        let config_builder = http_request_to_config_builder(
            &url,
            &tls_options,
            &request.headers,
            &get_current_heartbeat_payload(&mut heartbeat, 0),
            &proxy,
        )
        .await?;

        // Poll for changes to remote configuration.
        signal_handler.add(poll_http(
            poll_interval_secs,
            url,
            tls_options,
            request.headers.clone(),
            heartbeat,
            proxy.clone(),
            config_builder.sha256_hash(),
        ));

        Ok(config_builder)
    }
}

fn get_current_heartbeat_payload(
    heartbeat: &mut Option<serde_json::Value>,
    uptime_sec: u64,
) -> Option<String> {
    match heartbeat {
        Some(v) => {
            v.as_object_mut()
                .unwrap()
                .insert("uptime_sec".to_string(), uptime_sec.into());
            Some(v.to_string())
        }
        None => None,
    }
}

impl_generate_config_from_default!(HttpConfig);
