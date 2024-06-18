use indexmap::IndexMap;
use std::task::{Context, Poll};

use crate::{
    http::HttpClient,
    mezmo::user_trace::UserLoggingResponse,
    sinks::{
        opentelemetry::{config::OpentelemetryEndpoint, models::OpentelemetryModelType, Auth},
        util::Compression,
    },
};
use bytes::Bytes;
use futures::future::BoxFuture;
use http::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE},
    HeaderName, HeaderValue, Request,
};
use hyper::Body;
use tower::Service;
use vector_lib::{
    finalization::{EventFinalizers, EventStatus, Finalizable},
    request_metadata::{GroupedCountByteSize, MetaDescriptive, RequestMetadata},
    stream::DriverResponse,
};

use super::sink::OpentelemetrySinkError;

#[derive(Clone, Debug)]
pub struct OpentelemetryApiRequest {
    pub payload: Bytes,
    pub compression: Compression,
    pub metadata: RequestMetadata,
    pub finalizers: EventFinalizers,
    pub model_type: OpentelemetryModelType,
}

impl OpentelemetryApiRequest {
    const fn get_model_type(&self) -> OpentelemetryModelType {
        self.model_type
    }
}

impl Finalizable for OpentelemetryApiRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        std::mem::take(&mut self.finalizers)
    }
}

impl MetaDescriptive for OpentelemetryApiRequest {
    fn get_metadata(&self) -> &RequestMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut RequestMetadata {
        &mut self.metadata
    }
}

#[derive(Debug)]
pub struct OpentelemetryApiResponse {
    event_status: EventStatus,
    metadata: RequestMetadata,
    events_byte_size: GroupedCountByteSize,
}

impl DriverResponse for OpentelemetryApiResponse {
    fn event_status(&self) -> EventStatus {
        self.event_status
    }

    fn events_sent(&self) -> &GroupedCountByteSize {
        &self.events_byte_size
    }

    fn bytes_sent(&self) -> Option<usize> {
        Some(self.metadata.request_encoded_size())
    }
}

impl UserLoggingResponse for OpentelemetryApiResponse {}

#[derive(Clone, Debug)]
pub struct OpentelemetryService {
    pub endpoint: OpentelemetryEndpoint,
    pub client: HttpClient,
    pub auth: Option<Auth>,
    pub headers: IndexMap<HeaderName, HeaderValue>,
}

impl Service<OpentelemetryApiRequest> for OpentelemetryService {
    type Response = OpentelemetryApiResponse;
    type Error = OpentelemetrySinkError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: OpentelemetryApiRequest) -> Self::Future {
        let mut client = self.client.clone();
        let uri: http::Uri;

        match self.endpoint.endpoint(request.get_model_type()) {
            Some(val) => uri = val,
            None => {
                return Box::pin(async move {
                    Err(OpentelemetrySinkError::new(&format!(
                        "Endpoint is not defined for model type: {}",
                        "Unknown"
                    )))
                })
            }
        };

        let metadata = std::mem::take(request.metadata_mut());
        let events_byte_size = metadata
            .clone()
            .into_events_estimated_json_encoded_byte_size();
        let http_request = Request::post(&uri);

        let http_request = if let Some(ca) = request.compression.content_encoding() {
            http_request.header(CONTENT_ENCODING, ca)
        } else {
            http_request
        };

        let mut http_request = http_request
            .header(CONTENT_LENGTH, request.payload.len())
            .header(CONTENT_TYPE, "application/x-protobuf")
            .body(Body::from(request.payload))
            .expect("building HTTP request failed unexpectedly");

        let headers = http_request.headers_mut();

        for (name, value) in self.headers.iter() {
            headers.insert(name, value.clone());
        }

        if let Some(auth) = &self.auth {
            match auth {
                Auth::Basic(http_auth) => http_auth.apply(&mut http_request),
            }
        }

        Box::pin(async move {
            match client.call(http_request).await {
                Ok(_) => Ok(OpentelemetryApiResponse {
                    event_status: EventStatus::Delivered,
                    metadata: metadata.clone(),
                    events_byte_size,
                }),
                Err(error) => Err(OpentelemetrySinkError::new(&format!(
                    "HTTP request error: {}",
                    error
                ))),
            }
        })
    }
}
