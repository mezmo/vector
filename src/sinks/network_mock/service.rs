use std::fmt;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::sinks::{prelude::*, util::http::HttpRequest};

// Response

pub(super) struct NetworkMockResponse {
    events_byte_size: GroupedCountByteSize,
    raw_byte_size: usize,
}

impl DriverResponse for NetworkMockResponse {
    fn event_status(&self) -> EventStatus {
        EventStatus::Delivered
    }

    fn events_sent(&self) -> &GroupedCountByteSize {
        &self.events_byte_size
    }

    fn bytes_sent(&self) -> Option<usize> {
        Some(self.raw_byte_size)
    }
}

// Error

#[derive(Debug)]
pub(super) struct NetworkMockError;

impl fmt::Display for NetworkMockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("network_mock: simulated request failure")
    }
}

impl std::error::Error for NetworkMockError {}

// Service

#[derive(Clone)]
pub(super) struct NetworkMockService {
    pub(super) mean_ms: u64,
    pub(super) jitter_ms: u64,
    pub(super) error_rate: f64,
}

impl Service<HttpRequest<()>> for NetworkMockService {
    type Response = NetworkMockResponse;
    type Error = NetworkMockError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: HttpRequest<()>) -> Self::Future {
        let metadata = std::mem::take(request.metadata_mut());
        let raw_byte_size = metadata.request_encoded_size();
        let events_byte_size = metadata.into_events_estimated_json_encoded_byte_size();

        let mean_ms = self.mean_ms;
        let jitter_ms = self.jitter_ms;
        let error_rate = self.error_rate;

        Box::pin(async move {
            let latency = if jitter_ms > 0 {
                let jitter = rand::random::<u64>() % (jitter_ms * 2 + 1);
                let offset = mean_ms.saturating_sub(jitter_ms) + jitter;
                Duration::from_millis(offset)
            } else {
                Duration::from_millis(mean_ms)
            };

            debug!("Simulated request latency: {:?}", latency);
            tokio::time::sleep(latency).await;

            if error_rate > 0.0 && rand::random::<f64>() < error_rate {
                warn!(message = "Simulated request error.", error_rate);
                return Err(NetworkMockError);
            }

            Ok(NetworkMockResponse {
                events_byte_size,
                raw_byte_size,
            })
        })
    }
}

// Retry logic

#[derive(Debug, Clone)]
pub(super) struct NetworkMockRetryLogic;

impl RetryLogic for NetworkMockRetryLogic {
    type Error = NetworkMockError;
    type Request = HttpRequest<()>;
    type Response = NetworkMockResponse;

    fn is_retriable_error(&self, _error: &Self::Error) -> bool {
        true
    }
}
