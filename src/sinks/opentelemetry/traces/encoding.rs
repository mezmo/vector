use super::model::OpentelemetryTracesModel;
use crate::sinks::opentelemetry::sink::OpentelemetrySinkError;

pub fn encode(_model: &OpentelemetryTracesModel) -> Result<Vec<u8>, OpentelemetrySinkError> {
    // Trace model to Protobuf encoding
    // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-otlp/src/exporter/http/trace.rs#L72-L83
    Err(OpentelemetrySinkError::new(
        "Opentelemetry traces encoding is not implemented yet",
    ))
}
