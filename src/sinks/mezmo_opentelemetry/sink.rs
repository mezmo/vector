use std::{fmt::Debug, num::NonZeroUsize};

use super::{
    config::OpentelemetryMetricConfig,
    encoding::OpentelemetryEncoder,
    logs::model::OpentelemetryLogsModel,
    metrics::model::{OpentelemetryMetricsModel, OpentelemetryResourceMetrics},
    models::{OpentelemetryModel, OpentelemetryModelMatch, OpentelemetryModelType},
    service::OpentelemetryApiRequest,
    traces::model::OpentelemetryTracesModel,
};

use crate::{
    event::Event,
    http::get_http_scheme_from_uri,
    internal_events::SinkRequestBuildError,
    sinks::prelude::*,
    sinks::util::{
        Compression, RequestBuilder, SinkBuilderExt, metadata::RequestMetadataBuilder,
        request_builder::EncodeResult,
    },
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use mezmo::{user_log_error, user_trace::MezmoUserLog};
use tower::Service;
use vector_lib::{
    config::log_schema,
    event::Value,
    finalization::{EventFinalizers, Finalizable},
    lookup::PathPrefix,
    partition::Partitioner,
    request_metadata::RequestMetadata,
    sink::StreamSink,
    stream::{BatcherSettings, DriverResponse},
};

#[derive(Debug)]
pub struct OpentelemetrySinkError {
    message: String,
}

impl OpentelemetrySinkError {
    pub fn new(msg: &str) -> Self {
        OpentelemetrySinkError {
            message: String::from(msg),
        }
    }

    pub fn boxed(msg: &str) -> Box<Self> {
        Box::new(OpentelemetrySinkError {
            message: String::from(msg),
        })
    }
}

impl From<std::io::Error> for OpentelemetrySinkError {
    fn from(error: std::io::Error) -> Self {
        Self::new(&error.to_string())
    }
}

impl From<prost::EncodeError> for OpentelemetrySinkError {
    fn from(error: prost::EncodeError) -> Self {
        Self::new(&error.to_string())
    }
}

impl std::fmt::Display for OpentelemetrySinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OpentelemetrySinkError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<OpentelemetrySinkError> for std::io::Error {
    fn from(error: OpentelemetrySinkError) -> Self {
        Self::other(error)
    }
}

impl UserLoggingError for OpentelemetrySinkError {
    fn log_msg(&self) -> Option<Value> {
        Some(self.to_string().into())
    }
}

#[derive(Clone)]
struct OpentelemetryRequestBuilder {
    encoder: OpentelemetryEncoder,
    compression: Compression,
    metric_config: OpentelemetryMetricConfig,
    mezmo_ctx: Option<MezmoContext>,
}

impl RequestBuilder<(OpentelemetryModelType, Vec<Event>)> for OpentelemetryRequestBuilder {
    type Metadata = (OpentelemetryModelType, EventFinalizers);
    type Events = Result<OpentelemetryModel, Self::Error>;
    type Encoder = OpentelemetryEncoder;
    type Payload = Bytes;
    type Request = OpentelemetryApiRequest;
    type Error = OpentelemetrySinkError;

    fn compression(&self) -> Compression {
        self.compression
    }

    fn encoder(&self) -> &Self::Encoder {
        &self.encoder
    }

    fn split_input(
        &self,
        input: (OpentelemetryModelType, Vec<Event>),
    ) -> (Self::Metadata, RequestMetadataBuilder, Self::Events) {
        let (model_type, mut events) = input;

        let mut dropped_events_indeces = vec![];

        let model: Self::Events = match model_type {
            OpentelemetryModelType::Logs => {
                let mut logs: Vec<OpentelemetryLogsModel> = vec![];
                for (i, event) in events.iter().enumerate() {
                    match OpentelemetryLogsModel::try_from(event.clone()) {
                        Ok(m) => logs.push(m),
                        Err(err) => {
                            let mut captured_data = Value::from([]);
                            if let Some(log) = event.maybe_as_log() {
                                captured_data = Value::from(btreemap! {
                                    "message" => log.get_message().unwrap_or(&Value::Null).clone(),
                                    "metadata" =>  log.get((PathPrefix::Event, log_schema().user_metadata_key())).unwrap_or(&Value::Null).clone(),
                                    "timestamp" =>  log.get_timestamp().unwrap_or(&Value::Null).clone(),
                                })
                            }

                            user_log_error!(
                                self.mezmo_ctx,
                                Value::from(format!("{err}")),
                                captured_data: captured_data
                            );

                            dropped_events_indeces.push(i);
                        }
                    }
                }
                Ok(OpentelemetryModel::Logs(logs))
            }
            OpentelemetryModelType::Traces { partitioner_key: _ } => {
                let mut traces: Vec<OpentelemetryTracesModel> = vec![];
                for (i, event) in events.iter().enumerate() {
                    match OpentelemetryTracesModel::try_from(event.clone()) {
                        Ok(m) => traces.push(m),
                        Err(err) => {
                            let mut captured_data = Value::from([]);
                            if let Some(log) = event.maybe_as_log() {
                                captured_data = Value::from(btreemap! {
                                    "message" => log.get_message().unwrap_or(&Value::Null).clone(),
                                    "metadata" =>  log.get((PathPrefix::Event, log_schema().user_metadata_key())).unwrap_or(&Value::Null).clone(),
                                    "timestamp" =>  log.get_timestamp().unwrap_or(&Value::Null).clone(),
                                })
                            }

                            user_log_error!(
                                self.mezmo_ctx,
                                Value::from(format!("{err}")),
                                captured_data: captured_data
                            );

                            dropped_events_indeces.push(i);
                        }
                    }
                }
                Ok(OpentelemetryModel::Traces(traces))
            }
            OpentelemetryModelType::Metrics { partitioner_key: _ } => {
                let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
                for (i, event) in events.iter().enumerate() {
                    match OpentelemetryMetricsModel::try_from((event.clone(), &self.metric_config))
                    {
                        Ok(m) => metrics.push(m),
                        Err(err) => {
                            let mut captured_data = Value::from([]);
                            if let Some(metric) = event.clone().try_into_metric() {
                                captured_data = Value::from(btreemap! {
                                    "name" => Value::from(metric.name()),
                                    "kind" => Value::from(metric.kind()),
                                    "metadata" =>  metric.arbitrary_value().value().get(log_schema().user_metadata_key()).unwrap_or(&Value::Null).clone(),
                                    "timestamp" =>  metric.timestamp(),
                                })
                            }

                            user_log_error!(
                                self.mezmo_ctx,
                                Value::from(format!("{err}")),
                                captured_data: captured_data
                            );

                            dropped_events_indeces.push(i);
                        }
                    }
                }

                match OpentelemetryResourceMetrics::try_from(metrics) {
                    Ok(resource_metrics) => Ok(OpentelemetryModel::Metrics(resource_metrics)),
                    Err(error) => {
                        user_log_error!(self.mezmo_ctx, Value::from(format!("{error}")));
                        Err(error)
                    }
                }
            }
            OpentelemetryModelType::Unknown => {
                let err = OpentelemetrySinkError::new(&format!(
                    "Unsupported events detected: {} events",
                    events.len()
                ));

                for event in events.iter() {
                    let mut captured_data = Value::from([]);

                    if let Some(log) = event.maybe_as_log() {
                        captured_data = Value::from(btreemap! {
                            "message" => log.get_message().unwrap_or(&Value::Null).clone(),
                            "metadata" =>  log.get((PathPrefix::Event, log_schema().user_metadata_key())).unwrap_or(&Value::Null).clone(),
                            "timestamp" =>  log.get_timestamp().unwrap_or(&Value::Null).clone(),
                        })
                    }

                    if let Some(metric) = event.clone().try_into_metric() {
                        captured_data = Value::from(btreemap! {
                            "name" => Value::from(metric.name()),
                            "kind" => Value::from(metric.kind()),
                            "metadata" =>  metric.arbitrary_value().value().get(log_schema().user_metadata_key()).unwrap_or(&Value::Null).clone(),
                            "timestamp" =>  metric.timestamp(),
                        })
                    }

                    user_log_error!(
                        self.mezmo_ctx,
                        Value::from(format!("{err}")),
                        captured_data: captured_data
                    );
                }

                Err(err)
            }
        };

        // Remove events which failed to be converted to a model
        // to be able to get correct finalizers further ahead.
        for i in dropped_events_indeces {
            events.remove(i);
        }

        let finalizers = events.take_finalizers();
        let builder = RequestMetadataBuilder::from_events(&events);

        ((model_type, finalizers), builder, model)
    }

    fn build_request(
        &self,
        batch_metadata: Self::Metadata,
        metadata: RequestMetadata,
        payload: EncodeResult<Self::Payload>,
    ) -> Self::Request {
        let (model_type, finalizers) = batch_metadata;

        OpentelemetryApiRequest {
            compression: self.compression,
            payload: payload.into_payload(),
            metadata,
            finalizers,
            model_type,
        }
    }
}

struct OpentelemetryTypePartitioner;

impl Partitioner for OpentelemetryTypePartitioner {
    type Item = Event;
    type Key = OpentelemetryModelType;

    fn partition(&self, item: &Self::Item) -> Self::Key {
        let model_type = OpentelemetryLogsModel::maybe_match(item)
            .or(OpentelemetryMetricsModel::maybe_match(item))
            .or(OpentelemetryTracesModel::maybe_match(item));

        if let Some(model_type) = model_type {
            return model_type;
        }

        OpentelemetryModelType::Unknown
    }
}

pub struct OpentelemetrySink<S> {
    pub service: S,
    pub encoder: OpentelemetryEncoder,
    pub compression: Compression,
    pub batcher_settings: BatcherSettings,
    pub metric_config: OpentelemetryMetricConfig,
    pub mezmo_ctx: Option<MezmoContext>,
}

impl<S> OpentelemetrySink<S>
where
    S: Service<OpentelemetryApiRequest> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: DriverResponse + Send + 'static,
    S::Error: Debug + Into<crate::Error> + Send,
{
    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let builder_limit = NonZeroUsize::new(64).unwrap();
        let request_builder = OpentelemetryRequestBuilder {
            encoder: self.encoder,
            compression: self.compression,
            metric_config: self.metric_config,
            mezmo_ctx: self.mezmo_ctx,
        };

        use http::Uri;
        // let protocol = get_http_scheme_from_uri(&self.endpoint);
        let protocol = get_http_scheme_from_uri(&Uri::from_static("http://localhost"));

        input
            .batched_partitioned(OpentelemetryTypePartitioner, || {
                self.batcher_settings.as_byte_size_config()
            })
            .request_builder(builder_limit, request_builder)
            .filter_map(
                |request: Result<OpentelemetryApiRequest, OpentelemetrySinkError>| async move {
                    request
                        .map_err(|error| {
                            emit!(SinkRequestBuildError { error });
                        })
                        .ok()
                },
            )
            .into_driver(self.service)
            .protocol(protocol)
            .run()
            .await
    }
}

#[async_trait]
impl<S> StreamSink<Event> for OpentelemetrySink<S>
where
    S: Service<OpentelemetryApiRequest> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: DriverResponse + Send + 'static,
    S::Error: Debug + Into<crate::Error> + Send,
{
    async fn run(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
