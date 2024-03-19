use std::task::Poll;

use crate::{http::get_http_scheme_from_uri, mezmo::user_trace::UserLoggingResponse};
use bytes::Bytes;
use futures::future;
use futures::future::BoxFuture;
use http::{
    header::{HeaderName, HeaderValue},
    Request, Uri,
};
use hyper::Body;
use tower::Service;
use vector_lib::request_metadata::{GroupedCountByteSize, MetaDescriptive, RequestMetadata};
use vector_lib::stream::DriverResponse;
use vrl::value::Value;

use crate::{
    event::{EventFinalizers, EventStatus, Finalizable},
    gcp::GcpAuthenticator,
    http::{HttpClient, HttpError},
};

#[derive(Debug, Clone)]
pub struct GcsService {
    client: HttpClient,
    base_url: String,
    auth: Option<GcpAuthenticator>,
}

impl GcsService {
    pub const fn new(
        client: HttpClient,
        base_url: String,
        auth: Option<GcpAuthenticator>,
    ) -> GcsService {
        GcsService {
            client,
            base_url,
            auth,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GcsRequest {
    pub key: String,
    pub body: Bytes,
    pub settings: GcsRequestSettings,
    pub finalizers: EventFinalizers,
    pub metadata: RequestMetadata,
}

impl Finalizable for GcsRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        std::mem::take(&mut self.finalizers)
    }
}

impl MetaDescriptive for GcsRequest {
    fn get_metadata(&self) -> &RequestMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut RequestMetadata {
        &mut self.metadata
    }
}

// Settings required to produce a request that do not change per
// request. All possible values are pre-computed for direct use in
// producing a request.
#[derive(Clone, Debug)]
pub struct GcsRequestSettings {
    pub acl: Option<HeaderValue>,
    pub content_type: HeaderValue,
    pub content_encoding: Option<HeaderValue>,
    pub storage_class: HeaderValue,
    pub headers: Vec<(HeaderName, HeaderValue)>,
}

#[derive(Debug)]
pub struct GcsResponse {
    pub inner: Option<http::Response<Body>>,
    pub protocol: &'static str,
    pub metadata: RequestMetadata,
}

impl DriverResponse for GcsResponse {
    fn event_status(&self) -> EventStatus {
        match &self.inner {
            Some(inner) => {
                if inner.status().is_success() {
                    EventStatus::Delivered
                } else if inner.status().is_server_error() {
                    EventStatus::Errored
                } else {
                    EventStatus::Rejected
                }
            }
            None => EventStatus::Rejected,
        }
    }

    fn events_sent(&self) -> &GroupedCountByteSize {
        self.metadata.events_estimated_json_encoded_byte_size()
    }

    fn bytes_sent(&self) -> Option<usize> {
        Some(self.metadata.request_encoded_size())
    }
}

impl UserLoggingResponse for GcsResponse {
    fn log_msg(&self) -> Option<Value> {
        match &self.inner {
            Some(response) => {
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    Some(
                        format!(
                            "Error returned from destination with status code: {}",
                            status
                        )
                        .into(),
                    )
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl Service<GcsRequest> for GcsService {
    type Response = GcsResponse;
    type Error = HttpError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Emission of an internal event in case of errors is handled upstream by the caller.
    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // Emission of internal events for errors and dropped events is handled upstream by the caller.
    fn call(&mut self, request: GcsRequest) -> Self::Future {
        let settings = request.settings;
        let metadata = request.metadata;

        let uri = format!("{}{}", self.base_url, request.key)
            .parse::<Uri>()
            .unwrap();
        let protocol = get_http_scheme_from_uri(&uri);

        let mut builder = Request::put(uri);
        let headers = builder.headers_mut().unwrap();
        headers.insert("content-type", settings.content_type);
        headers.insert(
            "content-length",
            HeaderValue::from_str(&request.body.len().to_string()).unwrap(),
        );
        settings
            .content_encoding
            .map(|ce| headers.insert("content-encoding", ce));
        settings.acl.map(|acl| headers.insert("x-goog-acl", acl));
        headers.insert("x-goog-storage-class", settings.storage_class);
        for (p, v) in settings.headers {
            headers.insert(p, v);
        }

        let mut http_request = builder.body(Body::from(request.body)).unwrap();

        match &self.auth {
            Some(auth) => {
                auth.apply(&mut http_request);
            }
            None => {
                return Box::pin(future::ok(GcsResponse {
                    inner: None,
                    protocol,
                    metadata,
                }));
            }
        }

        let mut client = self.client.clone();
        Box::pin(async move {
            let result = client.call(http_request).await;
            result.map(|inner| GcsResponse {
                inner: Some(inner),
                protocol,
                metadata,
            })
        })
    }
}
