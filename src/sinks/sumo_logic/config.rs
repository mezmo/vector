use std::sync::Arc;

use crate::mezmo::user_trace::MezmoLoggingService;
use crate::sinks::sumo_logic::healthcheck::healthcheck;
use crate::sinks::util::retries::RetryLogic;
use crate::sinks::util::{ServiceBuilderExt, TowerRequestConfig};
use crate::{
    codecs::Transformer,
    config::{AcknowledgementsConfig, DataType, GenerateConfig, Input, SinkConfig, SinkContext},
    http::HttpClient,
    sinks::{
        util::{BatchConfig, Compression, SinkBatchSettings},
        Healthcheck, VectorSink,
    },
};
use async_trait::async_trait;
use futures_util::FutureExt;
use http::uri::InvalidUri;
use http::Uri;
use tower::ServiceBuilder;
use vector_common::sensitive_string::SensitiveString;
use vector_config::configurable_component;
use vector_core::tls::{TlsConfig, TlsSettings};

use super::service::SumoLogicApiResponse;
use super::sink::SumoLogicSinkError;
use super::{encoding::SumoLogicEncoder, service::SumoLogicService, sink::SumoLogicSink};

const DEFAULT_MAX_EVENTS: usize = 100;
const DEFAULT_MAX_BYTES: usize = 1_000_000;

#[derive(Clone, Debug, Default)]
pub struct SumoLogicRetry;

impl RetryLogic for SumoLogicRetry {
    type Error = SumoLogicSinkError;
    type Response = SumoLogicApiResponse;

    fn is_retriable_error(&self, _error: &Self::Error) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SumoLogicDefaultBatchSettings;

// URL reference for Sumo Logic settings batch settings:
// https://help.sumologic.com/docs/send-data/hosted-collectors/http-source/logs-metrics/
impl SinkBatchSettings for SumoLogicDefaultBatchSettings {
    const MAX_EVENTS: Option<usize> = Some(DEFAULT_MAX_EVENTS);
    const MAX_BYTES: Option<usize> = Some(DEFAULT_MAX_BYTES);
    const TIMEOUT_SECS: f64 = 1.0;
}

#[configurable_component]
#[derive(Clone, Debug, Derivative)]
#[serde(rename_all = "snake_case")]
#[derivative(Default)]
/// The model type to send to Sumo Logic
pub enum SumoLogicModelType {
    /// Send logs type
    #[derivative(Default)]
    Logs,
    /// Send metrics type
    Metrics,
}

#[configurable_component]
#[derive(Clone, Debug)]
/// Authentication struct holds collection endpoint
pub struct SumoLogicCredentials {
    /// Sumo Logic endpoint
    pub(crate) endpoint: SensitiveString,
}

impl SumoLogicCredentials {
    pub fn build_uri(&self) -> Result<Uri, InvalidUri> {
        let url_as_string = self.endpoint.inner().to_string();
        match url_as_string.parse::<Uri>() {
            Ok(uri) => Ok(uri),
            Err(error) => {
                error!(
                    "Error building URI, confirm the endpoint provided is correct: {}",
                    error
                );
                Err(error)
            }
        }
    }
}

impl From<&SumoLogicSinkConfig> for SumoLogicCredentials {
    fn from(config: &SumoLogicSinkConfig) -> Self {
        Self {
            endpoint: config.endpoint.clone(),
        }
    }
}

/// Configuration for the `sumo_logic_logs` sink.
#[configurable_component(sink("sumo_logic"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct SumoLogicSinkConfig {
    #[configurable(derived)]
    #[serde(default)]
    pub batch: BatchConfig<SumoLogicDefaultBatchSettings>,

    #[configurable(derived)]
    #[serde(default = "mezmo_default_category")]
    pub category: String,

    #[configurable(derived)]
    #[serde(default)]
    pub model: SumoLogicModelType,

    #[configurable(derived)]
    #[serde(default)]
    pub compression: Compression,

    // For Sumo Logic POST data is securly encoded via enpoint:
    // https://help.sumologic.com/docs/send-data/hosted-collectors/http-source/
    //
    // This URL should be kept secret as it is used to send data.
    //
    /// Sumo Logic URL used to send data.
    #[configurable(derived)]
    pub endpoint: SensitiveString,

    #[configurable(derived)]
    #[serde(
        default,
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    pub encoding: Transformer,

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

impl SumoLogicSinkConfig {
    pub(super) fn build_client(&self, cx: SinkContext) -> crate::Result<HttpClient> {
        let tls = TlsSettings::from_options(&self.tls)?;
        let client = HttpClient::new(tls, cx.proxy())?;
        Ok(client)
    }
}

impl GenerateConfig for SumoLogicSinkConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(
            r#"endpoint = "http://localhost:3100"
            compression = "none"
            category = "mezmo-pipeline"
            model = "logs""#,
        )
        .unwrap()
    }
}

#[async_trait]
#[typetag::serde(name = "sumo_logic")]
impl SinkConfig for SumoLogicSinkConfig {
    async fn build(&self, ctx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let batcher_settings = self
            .batch
            .validate()?
            .limit_max_events(self.batch.max_events.unwrap_or(DEFAULT_MAX_EVENTS))?
            .into_batcher_settings()?;

        let request_limits = self.request.unwrap_with(&Default::default());
        let client = self.build_client(ctx.clone())?;
        let healthcheck =
            healthcheck(client.clone(), SumoLogicCredentials::from(self).into()).boxed();
        let service = ServiceBuilder::new()
            .settings(request_limits, SumoLogicRetry)
            .service(MezmoLoggingService::new(
                SumoLogicService { client },
                ctx.mezmo_ctx,
            ));
        let credentials = Arc::from(SumoLogicCredentials::from(self));
        let compression = self.compression;
        let model = self.model.clone();
        let sink = SumoLogicSink {
            service,
            transformer: self.encoding.clone(),
            encoder: SumoLogicEncoder,
            credentials,
            category: self.category.clone(),
            model,
            compression,
            batcher_settings,
        };
        Ok((VectorSink::from_event_streamsink(sink), healthcheck))
    }

    fn input(&self) -> Input {
        match &self.model {
            SumoLogicModelType::Metrics => Input::new(DataType::Metric),
            SumoLogicModelType::Logs => Input::new(DataType::Log),
        }
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

pub fn mezmo_default_category() -> String {
    String::from("mezmo-pipeline")
}
