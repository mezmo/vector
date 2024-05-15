use crate::mezmo::user_trace::MezmoLoggingService;
use crate::sinks::opentelemetry::healthcheck::healthcheck;
use crate::sinks::util::retries::RetryLogic;
use crate::sinks::util::{ServiceBuilderExt, TowerRequestConfig};
use crate::{
    config::{AcknowledgementsConfig, DataType, GenerateConfig, Input, SinkConfig, SinkContext},
    http::HttpClient,
    sinks::{
        opentelemetry::{Auth, OpentelemetrySinkAuth},
        util::{BatchConfig, Compression, SinkBatchSettings},
        Healthcheck, VectorSink,
    },
};

use async_trait::async_trait;
use futures_util::FutureExt;
use http::{uri::InvalidUri, Uri};
use tower::ServiceBuilder;
use vector_lib::configurable::configurable_component;
use vector_lib::tls::{TlsConfig, TlsSettings};

use super::models::OpentelemetryModelType;
use super::service::OpentelemetryApiResponse;
use super::sink::OpentelemetrySinkError;
use super::{
    encoding::OpentelemetryEncoder, service::OpentelemetryService, sink::OpentelemetrySink,
};

const OPENTELEMETRY_HEALTHCHECK_PORT: &str = "13133";

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
    healthcheck_uri: Uri,
    logs_uri: Uri,
    metrics_uri: Uri,
    traces_uri: Uri,
}

impl OpentelemetryEndpoint {
    pub fn healthcheck(&self) -> Uri {
        self.healthcheck_uri.clone()
    }

    pub fn endpoint(&self, model_type: OpentelemetryModelType) -> Option<Uri> {
        match model_type {
            OpentelemetryModelType::Logs => Some(self.logs_uri.clone()),
            OpentelemetryModelType::Metrics => Some(self.metrics_uri.clone()),
            OpentelemetryModelType::Traces => Some(self.traces_uri.clone()),
            OpentelemetryModelType::Unknown => None,
        }
    }
}

impl TryFrom<String> for OpentelemetryEndpoint {
    type Error = OpentelemetrySinkEndpointError;

    fn try_from(endpoint: String) -> Result<Self, Self::Error> {
        let uri = endpoint
            .parse::<Uri>()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let scheme = uri.scheme_str().ok_or("Endpoint scheme is invalid")?;
        let authority = uri
            .authority()
            .map(|a| a.as_str())
            .ok_or("Endpoint authority is invalid")?;
        let host = uri.host().ok_or("Endpoint host is invalid")?;

        let healthcheck_uri = Uri::builder()
            .scheme(scheme)
            .authority(host.to_owned() + ":" + OPENTELEMETRY_HEALTHCHECK_PORT)
            .path_and_query("/")
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let logs_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query("/v1/logs")
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let metrics_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query("/v1/metrics")
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        let traces_uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query("/v1/traces")
            .build()
            .map_err(OpentelemetrySinkEndpointError::from)?;

        Ok(Self {
            healthcheck_uri,
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
    pub request: TowerRequestConfig,

    #[configurable(derived)]
    pub tls: Option<TlsConfig>,

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

#[async_trait]
#[typetag::serde(name = "opentelemetry")]
impl SinkConfig for OpentelemetrySinkConfig {
    async fn build(&self, ctx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let endpoint = OpentelemetryEndpoint::try_from(self.endpoint.clone())?;

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

        let request_limits = self.request.unwrap_with(&Default::default());

        let client = self.build_client(ctx.clone())?;

        let healthcheck =
            healthcheck(endpoint.clone(), client.clone(), auth.clone(), ctx.clone()).boxed();

        let service = ServiceBuilder::new()
            .settings(request_limits, OpentelemetryRetry)
            .service(MezmoLoggingService::new(
                OpentelemetryService {
                    endpoint: endpoint.clone(),
                    client,
                    auth,
                },
                ctx.mezmo_ctx.clone(),
            ));

        let compression = self.compression;
        let sink = OpentelemetrySink {
            service,
            encoder: OpentelemetryEncoder,
            compression,
            batcher_settings,
            mezmo_ctx: ctx.mezmo_ctx,
        };
        Ok((VectorSink::from_event_streamsink(sink), healthcheck))
    }

    fn input(&self) -> Input {
        Input::new(DataType::Metric | DataType::Log)
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}
