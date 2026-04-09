use futures::FutureExt;
use vector_lib::configurable::configurable_component;

use super::{
    encoder::NetworkMockEncoder,
    request_builder::NetworkMockRequestBuilder,
    service::{NetworkMockRetryLogic, NetworkMockService},
    sink::NetworkMockSink,
};
use crate::sinks::{prelude::*, util::BatchConfig};

/// Latency simulation settings.
#[configurable_component]
#[derive(Clone, Debug)]
pub struct LatencyConfig {
    /// Mean latency in milliseconds per simulated request.
    #[serde(default = "default_mean_ms")]
    mean_ms: u64,

    /// Maximum jitter in milliseconds, applied uniformly around the mean.
    /// Actual latency per request is `mean_ms + uniform(-jitter_ms, +jitter_ms)`.
    #[serde(default = "default_jitter_ms")]
    jitter_ms: u64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            mean_ms: default_mean_ms(),
            jitter_ms: default_jitter_ms(),
        }
    }
}

const fn default_mean_ms() -> u64 {
    50
}

const fn default_jitter_ms() -> u64 {
    10
}

#[derive(Clone, Copy, Debug, Default)]
struct NetworkMockDefaultBatchSettings;

impl SinkBatchSettings for NetworkMockDefaultBatchSettings {
    const MAX_EVENTS: Option<usize> = Some(1000);
    const MAX_BYTES: Option<usize> = Some(1_048_576);
    const TIMEOUT_SECS: f64 = 1.0;
}

/// Configuration for the `network_mock` sink.
#[configurable_component(sink(
    "network_mock",
    "Simulate network request behavior for throughput testing."
))]
#[derive(Clone, Debug)]
pub struct NetworkMockConfig {
    /// Latency simulation settings.
    #[configurable(derived)]
    #[serde(default)]
    latency: LatencyConfig,

    #[configurable(derived)]
    #[serde(default)]
    batch: BatchConfig<NetworkMockDefaultBatchSettings>,

    #[configurable(derived)]
    #[serde(default)]
    request: TowerRequestConfig,

    /// Fraction of requests that return a retriable error (0.0 to 1.0).
    #[serde(default)]
    error_rate: f64,

    #[configurable(derived)]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    encoding: Transformer,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::is_default"
    )]
    acknowledgements: AcknowledgementsConfig,
}

impl GenerateConfig for NetworkMockConfig {
    fn generate_config() -> toml::Value {
        toml::from_str("").unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "network_mock")]
impl SinkConfig for NetworkMockConfig {
    async fn build(&self, _cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        if !(0.0..=1.0).contains(&self.error_rate) {
            return Err("error_rate must be between 0.0 and 1.0".into());
        }

        let batch_settings = self.batch.validate()?.into_batcher_settings()?;

        let request_builder = NetworkMockRequestBuilder {
            encoder: NetworkMockEncoder {
                transformer: self.encoding.clone(),
            },
            compression: Compression::None,
        };

        let service = NetworkMockService {
            mean_ms: self.latency.mean_ms,
            jitter_ms: self.latency.jitter_ms,
            error_rate: self.error_rate,
        };

        let request_limits = self.request.into_settings();
        let service = ServiceBuilder::new()
            .settings(request_limits, NetworkMockRetryLogic)
            .service(service);

        let sink = NetworkMockSink::new(service, batch_settings, request_builder);
        let healthcheck = futures::future::ok(()).boxed();

        Ok((VectorSink::from_event_streamsink(sink), healthcheck))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<NetworkMockConfig>();
    }
}
