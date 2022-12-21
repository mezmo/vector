//! This sink sends data to Google Chronicles unstructured log entries endpoint.
//! See https://cloud.google.com/chronicle/docs/reference/ingestion-api#unstructuredlogentries
//! for more information.
use bytes::{Bytes, BytesMut};
use futures::future;
use futures_util::{future::BoxFuture, task::Poll};
use goauth::scopes::Scope;
use http::{header::HeaderValue, Request, StatusCode, Uri};
use hyper::Body;
use indoc::indoc;
use serde_json::json;
use snafu::Snafu;
use std::io;
use tokio_util::codec::Encoder as _;
use tower::{Service, ServiceBuilder};
use vector_common::request_metadata::{MetaDescriptive, RequestMetadata};
use vector_config::configurable_component;
use vector_core::{
    config::{AcknowledgementsConfig, Input},
    event::{Event, EventFinalizers, Finalizable},
    sink::VectorSink,
};

use crate::{
    codecs::{self, EncodingConfig},
    config::{log_schema, GenerateConfig, SinkConfig, SinkContext},
    gcp::{GcpAuthConfig, GcpAuthenticator},
    http::HttpClient,
    sinks::{
        gcs_common::{
            config::{healthcheck_response, GcsRetryLogic},
            service::GcsResponse,
            sink::GcsSink,
        },
        util::{
            encoding::{as_tracked_write, Encoder},
            metadata::RequestMetadataBuilder,
            partitioner::KeyPartitioner,
            request_builder::EncodeResult,
            BatchConfig, Compression, RequestBuilder, SinkBatchSettings, TowerRequestConfig,
        },
        Healthcheck,
    },
    template::{Template, TemplateParseError},
    tls::{TlsConfig, TlsSettings},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum GcsHealthcheckError {
    #[snafu(display("log_type template parse error: {}", source))]
    LogTypeTemplate { source: TemplateParseError },

    #[snafu(display("Endpoint not found"))]
    NotFound,

    #[snafu(display("The authentication provided is invalid"))]
    InvalidAuth,
}

/// Google Chronicle regions.
#[configurable_component]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    /// EU region.
    Eu,

    /// US region.
    Us,

    /// APAC region.
    Asia,
}

impl Region {
    /// Each region has a its own endpoint.
    const fn endpoint(self) -> &'static str {
        match self {
            Region::Eu => "https://europe-malachiteingestion-pa.googleapis.com",
            Region::Us => "https://malachiteingestion-pa.googleapis.com",
            Region::Asia => "https://asia-southeast1-malachiteingestion-pa.googleapis.com",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ChronicleUnstructuredDefaultBatchSettings;

// Chronicle Ingestion API has a 1MB limit[1] for unstructured log entries. We're also using a
// conservatively low batch timeout to ensure events make it to Chronicle in a timely fashion, but
// high enough that it allows for reasonable batching.
//
// [1]: https://cloud.google.com/chronicle/docs/reference/ingestion-api#unstructuredlogentries
impl SinkBatchSettings for ChronicleUnstructuredDefaultBatchSettings {
    const MAX_EVENTS: Option<usize> = None;
    const MAX_BYTES: Option<usize> = Some(1_000_000);
    const TIMEOUT_SECS: f64 = 15.0;
}

/// Configuration for the `gcp_chronicle_unstructured` sink.
#[configurable_component(sink("gcp_chronicle_unstructured"))]
#[derive(Clone, Debug)]
pub struct ChronicleUnstructuredConfig {
    /// The endpoint to send data to.
    pub endpoint: Option<String>,

    #[configurable(derived)]
    pub region: Option<Region>,

    /// The Unique identifier (UUID) corresponding to the Chronicle instance.
    #[configurable(validation(format = "uuid"))]
    pub customer_id: String,

    #[serde(flatten)]
    pub auth: GcpAuthConfig,

    #[configurable(derived)]
    #[serde(default)]
    pub batch: BatchConfig<ChronicleUnstructuredDefaultBatchSettings>,

    #[configurable(derived)]
    pub encoding: EncodingConfig,

    #[configurable(derived)]
    #[serde(default)]
    pub request: TowerRequestConfig,

    #[configurable(derived)]
    pub tls: Option<TlsConfig>,

    /// The type of log entries in a request.
    ///
    /// This must be one of the [supported log types][unstructured_log_types_doc], otherwise
    /// Chronicle will reject the entry with an error.
    ///
    /// [unstructured_log_types_doc]: https://cloud.google.com/chronicle/docs/ingestion/parser-list/supported-default-parsers
    pub log_type: Template,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    acknowledgements: AcknowledgementsConfig,
}

impl GenerateConfig for ChronicleUnstructuredConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(indoc! {r#"
            credentials_path = "/path/to/credentials.json"
            customer_id = "customer_id"
            log_type = "log_type"
            encoding.codec = "text"
        "#})
        .unwrap()
    }
}

pub fn build_healthcheck(
    client: HttpClient,
    base_url: &str,
    auth: Option<GcpAuthenticator>,
) -> crate::Result<Healthcheck> {
    let uri = base_url.parse::<Uri>()?;

    let healthcheck = async move {
        let mut request = http::Request::get(&uri).body(Body::empty())?;

        match auth {
            Some(auth) => {
                auth.apply(&mut request);

                let response = client.send(request).await?;
                healthcheck_response(response, auth, GcsHealthcheckError::NotFound.into())
            }
            None => Err(GcsHealthcheckError::InvalidAuth.into()),
        }
    };

    Ok(Box::pin(healthcheck))
}

#[derive(Debug, Snafu)]
pub enum ChronicleError {
    #[snafu(display("Region or endpoint not defined"))]
    RegionOrEndpoint,
    #[snafu(display("You can only specify one of region or endpoint"))]
    BothRegionAndEndpoint,
}

#[async_trait::async_trait]
impl SinkConfig for ChronicleUnstructuredConfig {
    async fn build(&self, cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        // Unlike Vector's upstream behavior, if the initial `auth` result is an error
        // we should continue building the topology. Convert the result into an option
        // that can be shared with consumers, while logging the error in-place.
        let auth = self.auth.build(Scope::MalachiteIngestion).await;
        if let Err(err) = &auth {
            warn!("Invalid authentication: {}", err)
        }
        let auth = auth.ok();

        let tls = TlsSettings::from_options(&self.tls)?;
        let client = HttpClient::new(tls, cx.proxy())?;

        let endpoint = self.create_endpoint("v2/unstructuredlogentries:batchCreate")?;

        // For the healthcheck we see if we can fetch the list of available log types.
        let healthcheck_endpoint = self.create_endpoint("v2/logtypes")?;

        let healthcheck = build_healthcheck(client.clone(), &healthcheck_endpoint, auth.clone())?;
        let sink = self.build_sink(client, endpoint, auth)?;

        Ok((sink, healthcheck))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

impl ChronicleUnstructuredConfig {
    fn build_sink(
        &self,
        client: HttpClient,
        base_url: String,
        auth: Option<GcpAuthenticator>,
    ) -> crate::Result<VectorSink> {
        use crate::sinks::util::service::ServiceBuilderExt;

        let request = self.request.unwrap_with(&TowerRequestConfig {
            rate_limit_num: Some(1000),
            ..Default::default()
        });

        let batch_settings = self.batch.into_batcher_settings()?;

        let partitioner = self.key_partitioner()?;

        let svc = ServiceBuilder::new()
            .settings(request, GcsRetryLogic)
            .service(ChronicleService::new(client, base_url, auth));

        let request_settings = RequestSettings::new(self)?;

        let sink = GcsSink::new(svc, request_settings, partitioner, batch_settings);

        Ok(VectorSink::from_event_streamsink(sink))
    }

    fn key_partitioner(&self) -> crate::Result<KeyPartitioner> {
        Ok(KeyPartitioner::new(self.log_type.clone()))
    }

    fn create_endpoint(&self, path: &str) -> Result<String, ChronicleError> {
        Ok(format!(
            "{}/{}",
            match (&self.endpoint, self.region) {
                (Some(endpoint), None) => endpoint.trim_end_matches('/'),
                (None, Some(region)) => region.endpoint(),
                (Some(_), Some(_)) => return Err(ChronicleError::BothRegionAndEndpoint),
                (None, None) => return Err(ChronicleError::RegionOrEndpoint),
            },
            path
        ))
    }
}

#[derive(Clone, Debug)]
pub struct ChronicleRequest {
    pub body: Bytes,
    pub finalizers: EventFinalizers,
    metadata: RequestMetadata,
}

impl Finalizable for ChronicleRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        std::mem::take(&mut self.finalizers)
    }
}

impl MetaDescriptive for ChronicleRequest {
    fn get_metadata(&self) -> RequestMetadata {
        self.metadata
    }
}

#[derive(Clone, Debug)]
struct ChronicleEncoder {
    customer_id: String,
    encoder: codecs::Encoder<()>,
    transformer: codecs::Transformer,
}

impl Encoder<(String, Vec<Event>)> for ChronicleEncoder {
    fn encode_input(
        &self,
        input: (String, Vec<Event>),
        writer: &mut dyn io::Write,
    ) -> io::Result<usize> {
        let (partition_key, events) = input;
        let mut encoder = self.encoder.clone();
        let events = events
            .into_iter()
            .filter_map(|mut event| {
                let timestamp = event
                    .as_log()
                    .get(log_schema().timestamp_key())
                    .and_then(|ts| ts.as_timestamp())
                    .cloned();
                let mut bytes = BytesMut::new();
                self.transformer.transform(&mut event);
                encoder.encode(event, &mut bytes).ok()?;

                let mut value = json!({
                    "log_text": String::from_utf8_lossy(&bytes),
                });

                if let Some(ts) = timestamp {
                    value.as_object_mut().unwrap().insert(
                        "ts_rfc3339".to_string(),
                        ts.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
                            .into(),
                    );
                }

                Some(value)
            })
            .collect::<Vec<_>>();

        let json = json!({
            "customer_id": self.customer_id,
            "log_type": partition_key,
            "entries": events,
        });

        let size = as_tracked_write::<_, _, io::Error>(writer, &json, |writer, json| {
            serde_json::to_writer(writer, json)?;
            Ok(())
        })?;

        Ok(size)
    }
}

// Settings required to produce a request that do not change per
// request. All possible values are pre-computed for direct use in
// producing a request.
#[derive(Clone, Debug)]
struct RequestSettings {
    encoder: ChronicleEncoder,
}

struct ChronicleRequestPayload {
    bytes: Bytes,
}

impl From<Bytes> for ChronicleRequestPayload {
    fn from(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

impl AsRef<[u8]> for ChronicleRequestPayload {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl RequestBuilder<(String, Vec<Event>)> for RequestSettings {
    type Metadata = EventFinalizers;
    type Events = (String, Vec<Event>);
    type Encoder = ChronicleEncoder;
    type Payload = ChronicleRequestPayload;
    type Request = ChronicleRequest;
    type Error = io::Error;

    fn compression(&self) -> Compression {
        Compression::None
    }

    fn encoder(&self) -> &Self::Encoder {
        &self.encoder
    }

    fn split_input(
        &self,
        input: (String, Vec<Event>),
    ) -> (Self::Metadata, RequestMetadataBuilder, Self::Events) {
        let (partition_key, mut events) = input;
        let finalizers = events.take_finalizers();

        let builder = RequestMetadataBuilder::from_events(&events);
        (finalizers, builder, (partition_key, events))
    }

    fn build_request(
        &self,
        finalizers: Self::Metadata,
        metadata: RequestMetadata,
        payload: EncodeResult<Self::Payload>,
    ) -> Self::Request {
        ChronicleRequest {
            body: payload.into_payload().bytes,
            finalizers,
            metadata,
        }
    }
}

impl RequestSettings {
    fn new(config: &ChronicleUnstructuredConfig) -> crate::Result<Self> {
        let transformer = config.encoding.transformer();
        let serializer = config.encoding.config().build()?;
        let encoder = crate::codecs::Encoder::<()>::new(serializer);
        let encoder = ChronicleEncoder {
            customer_id: config.customer_id.clone(),
            encoder,
            transformer,
        };
        Ok(Self { encoder })
    }
}

#[derive(Debug, Clone)]
pub struct ChronicleService {
    client: HttpClient,
    base_url: String,
    auth: Option<GcpAuthenticator>,
}

impl ChronicleService {
    pub const fn new(client: HttpClient, base_url: String, auth: Option<GcpAuthenticator>) -> Self {
        Self {
            client,
            base_url,
            auth,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ChronicleResponseError {
    #[snafu(display("Server responded with an error: {}", code))]
    ServerError { code: StatusCode },
    #[snafu(display("Failed to make HTTP(S) request: {}", error))]
    HttpError { error: crate::http::HttpError },
}

impl Service<ChronicleRequest> for ChronicleService {
    type Response = GcsResponse;
    type Error = ChronicleResponseError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: ChronicleRequest) -> Self::Future {
        let mut builder = Request::post(&self.base_url);
        let headers = builder.headers_mut().unwrap();
        headers.insert(
            "content-type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            "content-length",
            HeaderValue::from_str(&request.body.len().to_string()).unwrap(),
        );

        let metadata = request.get_metadata();

        let mut http_request = builder.body(Body::from(request.body)).unwrap();

        match &self.auth {
            Some(auth) => {
                auth.apply(&mut http_request);
            }
            None => {
                return Box::pin(future::ok(GcsResponse {
                    inner: None,
                    protocol: "http",
                    metadata: request.metadata,
                }));
            }
        }

        let mut client = self.client.clone();
        Box::pin(async move {
            match client.call(http_request).await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        Ok(GcsResponse {
                            inner: Some(response),
                            protocol: "http",
                            metadata,
                        })
                    } else {
                        Err(ChronicleResponseError::ServerError { code: status })
                    }
                }
                Err(error) => Err(ChronicleResponseError::HttpError { error }),
            }
        })
    }
}

#[cfg(all(test, feature = "chronicle-integration-tests"))]
mod integration_tests {
    use reqwest::{Client, Method, Response};
    use serde::{Deserialize, Serialize};
    use vector_core::event::{BatchNotifier, BatchStatus};

    use super::*;
    use crate::test_util::{
        components::{
            run_and_assert_sink_compliance, run_and_assert_sink_error, COMPONENT_ERROR_TAGS,
            SINK_TAGS,
        },
        random_events_with_stream, random_string, trace_init,
    };

    const ADDRESS_ENV_VAR: &str = "CHRONICLE_ADDRESS";

    fn config(log_type: &str, auth_path: &str) -> ChronicleUnstructuredConfig {
        let address = std::env::var(ADDRESS_ENV_VAR).unwrap();
        let config = format!(
            indoc! { r#"
             endpoint = "{}"
             customer_id = "customer id"
             credentials_path = "{}"
             log_type = "{}"
             encoding.codec = "text"
        "# },
            address, auth_path, log_type
        );

        let config: ChronicleUnstructuredConfig = toml::from_str(&config).unwrap();
        config
    }

    async fn config_build(
        log_type: &str,
        auth_path: &str,
    ) -> crate::Result<(VectorSink, crate::sinks::Healthcheck)> {
        let cx = SinkContext::new_test();
        config(log_type, auth_path).build(cx).await
    }

    #[tokio::test]
    async fn publish_events() {
        trace_init();

        let log_type = random_string(10);
        let (sink, healthcheck) = config_build(&log_type, "/chronicleauth.json")
            .await
            .expect("Building sink failed");

        healthcheck.await.expect("Health check failed");

        let (batch, mut receiver) = BatchNotifier::new_with_receiver();
        let (input, events) = random_events_with_stream(100, 100, Some(batch));
        run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;
        assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

        let response = pull_messages(&log_type).await;
        let messages = response
            .into_iter()
            .map(|message| message.log_text)
            .collect::<Vec<_>>();
        assert_eq!(input.len(), messages.len());
        for i in 0..input.len() {
            let data = serde_json::to_value(&messages[i]).unwrap();
            let expected = serde_json::to_value(input[i].as_log().get("message").unwrap()).unwrap();
            assert_eq!(data, expected);
        }
    }

    #[tokio::test]
    async fn invalid_credentials() {
        trace_init();

        let log_type = random_string(10);
        // Test with an auth file that doesnt match the public key sent to the dummy chronicle server.
        let sink = config_build(&log_type, "/invalidchronicleauth.json").await;

        assert!(sink.is_err())
    }

    #[tokio::test]
    async fn publish_invalid_events() {
        trace_init();

        // The chronicle-emulator we are testing against is setup so a `log_type` of "INVALID"
        // will return a `400 BAD_REQUEST`.
        let log_type = "INVALID";
        let (sink, healthcheck) = config_build(log_type, "/chronicleauth.json")
            .await
            .expect("Building sink failed");

        healthcheck.await.expect("Health check failed");

        let (batch, mut receiver) = BatchNotifier::new_with_receiver();
        let (_input, events) = random_events_with_stream(100, 100, Some(batch));
        run_and_assert_sink_error(sink, events, &COMPONENT_ERROR_TAGS).await;
        assert_eq!(receiver.try_recv(), Ok(BatchStatus::Rejected));
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Log {
        customer_id: String,
        log_type: String,
        log_text: String,
        ts_rfc3339: String,
    }

    async fn request(method: Method, path: &str, log_type: &str) -> Response {
        let address = std::env::var(ADDRESS_ENV_VAR).unwrap();
        let url = format!("{}/{}", address, path);
        Client::new()
            .request(method.clone(), &url)
            .query(&[("log_type", log_type)])
            .send()
            .await
            .unwrap_or_else(|_| panic!("Sending {} request to {} failed", method, url))
    }

    async fn pull_messages(log_type: &str) -> Vec<Log> {
        request(Method::GET, "logs", log_type)
            .await
            .json::<Vec<Log>>()
            .await
            .expect("Extracting pull data failed")
    }
}