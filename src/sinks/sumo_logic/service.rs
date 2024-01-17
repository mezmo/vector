use std::{
    sync::Arc,
    task::{Context, Poll},
};

use crate::{http::HttpClient, mezmo::user_trace::UserLoggingResponse, sinks::util::Compression};
use bytes::Bytes;
use futures::future::BoxFuture;
use http::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE},
    Request,
};
use hyper::Body;
use tower::Service;
use vector_lib::{
    finalization::{EventFinalizers, EventStatus, Finalizable},
    request_metadata::{GroupedCountByteSize, MetaDescriptive, RequestMetadata},
    stream::DriverResponse,
};

use super::{
    config::{SumoLogicCredentials, SumoLogicModelType},
    sink::SumoLogicSinkError,
};

#[derive(Clone, Debug)]
pub struct SumoLogicApiRequest {
    pub payload: Bytes,
    pub category: String,
    pub credentials: Arc<SumoLogicCredentials>,
    pub compression: Compression,
    pub model: SumoLogicModelType,
    pub metadata: RequestMetadata,
    pub finalizers: EventFinalizers,
}

impl Finalizable for SumoLogicApiRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        std::mem::take(&mut self.finalizers)
    }
}

impl MetaDescriptive for SumoLogicApiRequest {
    fn get_metadata(&self) -> &RequestMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut RequestMetadata {
        &mut self.metadata
    }
}

#[derive(Debug)]
pub struct SumoLogicApiResponse {
    event_status: EventStatus,
    metadata: RequestMetadata,
    events_byte_size: GroupedCountByteSize,
}

impl DriverResponse for SumoLogicApiResponse {
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

impl UserLoggingResponse for SumoLogicApiResponse {}

#[derive(Clone, Debug)]
pub struct SumoLogicService {
    pub client: HttpClient,
}

impl Service<SumoLogicApiRequest> for SumoLogicService {
    type Response = SumoLogicApiResponse;
    type Error = SumoLogicSinkError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: SumoLogicApiRequest) -> Self::Future {
        let mut client = self.client.clone();
        let uri = request
            .credentials
            .build_uri()
            .expect("error building sumo logic endpoint");

        let metadata = std::mem::take(request.metadata_mut());
        let events_byte_size = metadata
            .clone()
            .into_events_estimated_json_encoded_byte_size();
        let http_request = Request::post(&uri);

        let http_request = match request.model {
            SumoLogicModelType::Metrics => {
                // The Sumo Logic API only excepts a few different types of metrics
                // formats. For now, we are only sending Prometheus metrics.
                http_request.header(CONTENT_TYPE, "application/vnd.sumologic.prometheus")
            }
            _ => http_request.header(CONTENT_TYPE, "application/json"),
        };

        let http_request = if let Some(ca) = request.compression.content_encoding() {
            http_request.header(CONTENT_ENCODING, ca)
        } else {
            http_request
        };

        let http_request = http_request
            .header(CONTENT_LENGTH, request.payload.len())
            .header("X-Sumo-Category", request.category)
            .body(Body::from(request.payload))
            .expect("building HTTP request failed unexpectedly");

        Box::pin(async move {
            match client.call(http_request).await {
                Ok(_) => Ok(SumoLogicApiResponse {
                    event_status: EventStatus::Delivered,
                    metadata: metadata.clone(),
                    events_byte_size,
                }),
                Err(error) => Err(SumoLogicSinkError::new(&format!(
                    "HTTP request error: {}",
                    error
                ))),
            }
        })
    }
}
