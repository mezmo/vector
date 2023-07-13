use std::{fmt::Debug, num::NonZeroUsize, sync::Arc};

use super::{
    config::{SumoLogicCredentials, SumoLogicModelType},
    encoding::SumoLogicEncoder,
    models::{SumoLogicModel, SumoLogsModel, SumoMetricsModel},
    service::SumoLogicApiRequest,
};
use crate::{
    codecs::Transformer,
    event::Event,
    http::get_http_scheme_from_uri,
    internal_events::SinkRequestBuildError,
    mezmo::user_trace::UserLoggingError,
    sinks::util::{
        metadata::RequestMetadataBuilder, request_builder::EncodeResult, Compression,
        RequestBuilder, SinkBuilderExt,
    },
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use tower::Service;
use vector_common::{
    finalization::{EventFinalizers, Finalizable},
    request_metadata::RequestMetadata,
};
use vector_core::{
    event::Value,
    sink::StreamSink,
    stream::{BatcherSettings, DriverResponse},
};

#[derive(Debug)]
pub struct SumoLogicSinkError {
    message: String,
}

impl SumoLogicSinkError {
    pub fn new(msg: &str) -> Self {
        SumoLogicSinkError {
            message: String::from(msg),
        }
    }

    pub fn boxed(msg: &str) -> Box<Self> {
        Box::new(SumoLogicSinkError {
            message: String::from(msg),
        })
    }
}

impl From<std::io::Error> for SumoLogicSinkError {
    fn from(error: std::io::Error) -> Self {
        Self::new(&error.to_string())
    }
}

impl std::fmt::Display for SumoLogicSinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SumoLogicSinkError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<SumoLogicSinkError> for std::io::Error {
    fn from(error: SumoLogicSinkError) -> Self {
        Self::new(std::io::ErrorKind::Other, error)
    }
}

impl UserLoggingError for SumoLogicSinkError {
    fn log_msg(&self) -> Option<Value> {
        Some(self.to_string().into())
    }
}

#[derive(Clone)]
struct SumoLogicRequestBuilder {
    transformer: Transformer,
    encoder: SumoLogicEncoder,
    compression: Compression,
    credentials: Arc<SumoLogicCredentials>,
    category: String,
    model: SumoLogicModelType,
}

impl RequestBuilder<Vec<Event>> for SumoLogicRequestBuilder {
    type Metadata = EventFinalizers;
    type Events = Result<SumoLogicModel, Self::Error>;
    type Encoder = SumoLogicEncoder;
    type Payload = Bytes;
    type Request = SumoLogicApiRequest;
    type Error = SumoLogicSinkError;

    fn compression(&self) -> Compression {
        self.compression
    }

    fn encoder(&self) -> &Self::Encoder {
        &self.encoder
    }

    fn split_input(
        &self,
        mut input: Vec<Event>,
    ) -> (Self::Metadata, RequestMetadataBuilder, Self::Events) {
        for event in input.iter_mut() {
            self.transformer.transform(event);
        }

        let builder = RequestMetadataBuilder::from_events(&input);

        let finalizers = input.take_finalizers();
        let model: Result<SumoLogicModel, SumoLogicSinkError> = match self.model {
            SumoLogicModelType::Logs => Ok(SumoLogicModel::Logs(
                SumoLogsModel::try_from(input).expect("error with log events input"),
            )),
            SumoLogicModelType::Metrics => Ok(SumoLogicModel::Metrics(
                SumoMetricsModel::try_from(input).expect("error with metrics input"),
            )),
        };

        (finalizers, builder, model)
    }

    fn build_request(
        &self,
        finalizers: Self::Metadata,
        metadata: RequestMetadata,
        payload: EncodeResult<Self::Payload>,
    ) -> Self::Request {
        SumoLogicApiRequest {
            credentials: Arc::clone(&self.credentials),
            compression: self.compression,
            category: self.category.clone(),
            payload: payload.into_payload(),
            metadata,
            finalizers,
        }
    }
}

pub struct SumoLogicSink<S> {
    pub service: S,
    pub transformer: Transformer,
    pub encoder: SumoLogicEncoder,
    pub credentials: Arc<SumoLogicCredentials>,
    pub compression: Compression,
    pub category: String,
    pub model: SumoLogicModelType,
    pub batcher_settings: BatcherSettings,
}

impl<S> SumoLogicSink<S>
where
    S: Service<SumoLogicApiRequest> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: DriverResponse + Send + 'static,
    S::Error: Debug + Into<crate::Error> + Send,
{
    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let builder_limit = NonZeroUsize::new(64);
        let request_builder = SumoLogicRequestBuilder {
            transformer: self.transformer,
            encoder: self.encoder,
            compression: self.compression,
            category: self.category,
            credentials: Arc::clone(&self.credentials),
            model: self.model,
        };
        let protocol = get_http_scheme_from_uri(
            &self
                .credentials
                .build_uri()
                .expect("error building sumo logic endpoint"),
        );

        input
            .batched(self.batcher_settings.into_byte_size_config())
            .request_builder(builder_limit, request_builder)
            .filter_map(
                |request: Result<SumoLogicApiRequest, SumoLogicSinkError>| async move {
                    match request {
                        Err(error) => {
                            emit!(SinkRequestBuildError { error });
                            None
                        }
                        Ok(req) => Some(req),
                    }
                },
            )
            .into_driver(self.service)
            .protocol(protocol)
            .run()
            .await
    }
}

#[async_trait]
impl<S> StreamSink<Event> for SumoLogicSink<S>
where
    S: Service<SumoLogicApiRequest> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: DriverResponse + Send + 'static,
    S::Error: Debug + Into<crate::Error> + Send,
{
    async fn run(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
