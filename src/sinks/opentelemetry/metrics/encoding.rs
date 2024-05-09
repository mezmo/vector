use super::model::OpentelemetryMetricsModel;
use crate::sinks::opentelemetry::sink::OpentelemetrySinkError;

pub fn encode(_model: Vec<OpentelemetryMetricsModel>) -> Result<Vec<u8>, OpentelemetrySinkError> {
    // Metric model to Protobuf encoding
    // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-otlp/src/exporter/http/metrics.rs#L52-L61
    Err(OpentelemetrySinkError::new(
        "Opentelemetry metrics encoding is not implemented yet",
    ))
}
