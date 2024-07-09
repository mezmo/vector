use crate::mezmo::user_trace::MezmoLoggingService;
use crate::sinks::util::retries::RetryLogic;
use crate::sinks::util::ServiceBuilderExt;
use crate::{
    config::{AcknowledgementsConfig, DataType, GenerateConfig, Input, SinkConfig, SinkContext},
    http::HttpClient,
    sinks::{
        opentelemetry::{Auth, OpentelemetrySinkAuth},
        util::{http::RequestConfig, BatchConfig, Compression, SinkBatchSettings},
        Healthcheck, VectorSink,
    },
};

use async_trait::async_trait;
use http::{header::AUTHORIZATION, uri::InvalidUri, HeaderName, HeaderValue, Uri};
use indexmap::IndexMap;
use tower::ServiceBuilder;
use vector_lib::configurable::configurable_component;
use vector_lib::tls::{TlsConfig, TlsSettings};

use super::models::OpentelemetryModelType;
use super::service::OpentelemetryApiResponse;
use super::sink::OpentelemetrySinkError;
use super::{
    encoding::OpentelemetryEncoder, service::OpentelemetryService, sink::OpentelemetrySink,
};

const DEFAULT_MAX_EVENTS: usize = 100;
const DEFAULT_MAX_BYTES: usize = 1_000_000;

#[derive(Clone, Debug, Default)]
pub struct OpentelemetryRetry;

impl RetryLogic for OpentelemetryRetry {
    type Error = OpentelemetrySinkError;
    type Response = OpentelemetryApiResponse;

    fn is_retriable_error(&self, _error: &Self::Error) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OpentelemetryDefaultBatchSettings;

// URL reference for Opentelemetry settings batch settings
impl SinkBatchSettings for OpentelemetryDefaultBatchSettings {
    const MAX_EVENTS: Option<usize> = Some(DEFAULT_MAX_EVENTS);
    const MAX_BYTES: Option<usize> = Some(DEFAULT_MAX_BYTES);
    const TIMEOUT_SECS: f64 = 1.0;
}

#[derive(Debug)]
pub struct OpentelemetrySinkEndpointError {
    message: String,
}

impl OpentelemetrySinkEndpointError {
    pub fn new(msg: &str) -> Self {
        OpentelemetrySinkEndpointError {
            message: String::from(msg),
        }
    }
}

impl From<InvalidUri> for OpentelemetrySinkEndpointError {
    fn from(error: InvalidUri) -> Self {
        Self::new(&error.to_string())
    }
}

impl From<&str> for OpentelemetrySinkEndpointError {
    fn from(error: &str) -> Self {
        Self::new(error)
    }
}

impl From<http::Error> for OpentelemetrySinkEndpointError {
    fn from(error: http::Error) -> Self {
        Self::new(&error.to_string())
    }
}

impl std::fmt::Display for OpentelemetrySinkEndpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OpentelemetrySinkEndpointError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Default)]
pub struct OpentelemetryEndpoint {
    logs_uri: Uri,
    metrics_uri: Uri,
    traces_uri: Uri,
}

impl OpentelemetryEndpoint {
    pub fn endpoint(&self, model_type: OpentelemetryModelType) -> Option<Uri> {
        match model_type {
            OpentelemetryModelType::Logs => Some(self.logs_uri.clone()),
            OpentelemetryModelType::Metrics { .. } => Some(self.metrics_uri.clone()),
            OpentelemetryModelType::Traces { .. } => Some(self.traces_uri.clone()),
            OpentelemetryModelType::Unknown => None,
        }
    }
}

impl TryFrom<&OpentelemetrySinkConfig> for OpentelemetryEndpoint {
    type Error = OpentelemetrySinkEndpointError;

    fn try_from(config: &OpentelemetrySinkConfig) -> Result<Self, Self::Error> {
        let uri = config
            .endpoint
            .parse::<Uri>()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let scheme = uri.scheme_str().ok_or("Endpoint scheme is invalid")?;
        let authority = uri
            .authority()
            .map(|a| a.as_str())
            .ok_or("Endpoint authority is invalid")?;

        let mut path = uri.path();
        if path.ends_with('/') {
            let mut path_chars = path.chars();
            path_chars.next_back();
            path = path_chars.as_str();
        }
        let query = uri.query().unwrap_or("");

        let logs_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query(path.to_owned() + "/v1/logs?" + query)
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let metrics_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query(path.to_owned() + "/v1/metrics?" + query)
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let traces_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query(path.to_owned() + "/v1/traces?" + query)
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        Ok(Self {
            logs_uri,
            metrics_uri,
            traces_uri,
        })
    }
}

/// Configuration for the `opentelemetry_logs` sink.
#[configurable_component(sink("opentelemetry"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct OpentelemetrySinkConfig {
    /// The endpoint to send data to.
    ///
    /// The endpoint should include the scheme and the port to send to.
    #[configurable(metadata(docs::examples = "https://localhost:8087"))]
    pub endpoint: String,

    #[configurable(derived)]
    pub auth: Option<OpentelemetrySinkAuth>,

    #[configurable(derived)]
    #[serde(default)]
    pub batch: BatchConfig<OpentelemetryDefaultBatchSettings>,

    #[configurable(derived)]
    #[serde(default)]
    pub compression: Compression,

    #[configurable(derived)]
    #[serde(default)]
    pub request: RequestConfig,

    #[configurable(derived)]
    pub tls: Option<TlsConfig>,

    /// Default buckets to use for aggregating [distribution][dist_metric_docs] metrics into histograms.
    ///
    /// [dist_metric_docs]: https://vector.dev/docs/about/under-the-hood/architecture/data-model/metric/#distribution
    #[serde(default = "super::default_histogram_buckets")]
    #[configurable(metadata(docs::advanced))]
    pub buckets: Vec<f64>,

    /// Acknowlegements option
    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    acknowledgements: AcknowledgementsConfig,
}

impl OpentelemetrySinkConfig {
    pub(super) fn build_client(&self, cx: SinkContext) -> crate::Result<HttpClient> {
        let tls = TlsSettings::from_options(&self.tls)?;
        let client = HttpClient::new(tls, cx.proxy())?;
        Ok(client)
    }
}

impl GenerateConfig for OpentelemetrySinkConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(
            r#"endpoint = "http://localhost:3100"
            compression = "none""#,
        )
        .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct OpentelemetryMetricConfig {
    pub buckets: Vec<f64>,
}

#[async_trait]
#[typetag::serde(name = "opentelemetry")]
impl SinkConfig for OpentelemetrySinkConfig {
    async fn build(&self, ctx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let endpoint = OpentelemetryEndpoint::try_from(self)?;

        let auth = match &self.auth {
            Some(OpentelemetrySinkAuth::Basic { user, password }) => {
                Some(Auth::Basic(crate::http::Auth::Basic {
                    user: user.clone(),
                    password: password.clone().into(),
                }))
            }
            Some(OpentelemetrySinkAuth::Bearer { token }) => {
                Some(Auth::Basic(crate::http::Auth::Bearer {
                    token: token.clone(),
                }))
            }
            None => None,
        };

        let batcher_settings = self
            .batch
            .validate()?
            .limit_max_events(self.batch.max_events.unwrap_or(DEFAULT_MAX_EVENTS))?
            .into_batcher_settings()?;

        let request_settings = self.request.tower.into_settings();

        let client = self.build_client(ctx.clone())?;

        let healthcheck = healthcheck();

        let headers = validate_headers(&self.request.headers, self.auth.is_some())?;

        let service = ServiceBuilder::new()
            .settings(request_settings, OpentelemetryRetry)
            .service(MezmoLoggingService::new(
                OpentelemetryService {
                    endpoint: endpoint.clone(),
                    client,
                    auth,
                    headers,
                },
                ctx.mezmo_ctx.clone(),
            ));

        let metric_config = OpentelemetryMetricConfig {
            buckets: self.buckets.clone(),
        };

        let compression = self.compression;
        let sink = OpentelemetrySink {
            service,
            encoder: OpentelemetryEncoder,
            compression,
            batcher_settings,
            metric_config,
            mezmo_ctx: ctx.mezmo_ctx,
        };
        Ok((
            VectorSink::from_event_streamsink(sink),
            Box::pin(healthcheck),
        ))
    }

    fn input(&self) -> Input {
        Input::new(DataType::Metric | DataType::Log)
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

pub(crate) async fn healthcheck() -> crate::Result<()> {
    Ok(())
}

fn validate_headers(
    headers: &IndexMap<String, String>,
    configures_auth: bool,
) -> crate::Result<IndexMap<HeaderName, HeaderValue>> {
    let headers = crate::sinks::util::http::validate_headers(headers)?;

    for name in headers.keys() {
        if configures_auth && name == AUTHORIZATION {
            return Err("Authorization header can not be used with defined auth options".into());
        }
    }

    Ok(headers)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sinks::{opentelemetry::config::OpentelemetrySinkConfig, util::test::load_sink};
    use indoc::indoc;

    #[test]
    fn test_otlp_sink_endpoint_logs() {
        // Case: Endpoint URI has no path or query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Logs)
                .expect("Get log endpoint error"),
            "https://localhost:8087/v1/logs"
        );

        // Case: Endpoint URI has path but no query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Logs)
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/logs"
        );

        // Case: Endpoint URI has path and query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path?query=val"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Logs)
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/logs?query=val"
        );
    }

    #[test]
    fn test_otlp_sink_endpoint_metrics() {
        // Case: Endpoint URI has no path or query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Metrics {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0],
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/v1/metrics"
        );

        // Case: Endpoint URI has path but no query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Metrics {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0],
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/metrics"
        );

        // Case: Endpoint URI has path and query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path?query=val"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Metrics {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0],
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/metrics?query=val"
        );
    }

    #[test]
    fn test_otlp_sink_endpoint_traces() {
        // Case: Endpoint URI has no path or query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Traces {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0]
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/v1/traces"
        );

        // Case: Endpoint URI has path but no query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Traces {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0]
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/traces"
        );

        // Case: Endpoint URI has path and query
        let config = indoc! {r#"
            endpoint = "https://localhost:8087/some_intermediate_path?query=val"
        "#};
        let (config, _) =
            load_sink::<OpentelemetrySinkConfig>(config).expect("Config parsing error");
        let endpoint = OpentelemetryEndpoint::try_from(&config).expect("Endpoint parsing error");

        assert_eq!(
            endpoint
                .endpoint(OpentelemetryModelType::Traces {
                    partitioner_key: [0, 0, 0, 0, 0, 0, 0, 0]
                })
                .expect("Get log endpoint error"),
            "https://localhost:8087/some_intermediate_path/v1/traces?query=val"
        );
    }
}
