use super::model::OpentelemetryTracesModel;
use crate::sinks::opentelemetry::sink::OpentelemetrySinkError;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message;

pub fn encode(traces: OpentelemetryTracesModel) -> Result<Vec<u8>, OpentelemetrySinkError> {
    // Trace model to Protobuf encoding
    // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-otlp/src/exporter/http/trace.rs#L72-L83

    let req = ExportTraceServiceRequest {
        resource_spans: traces.0.into_iter().map(Into::into).collect(),
    };

    let mut buf = vec![];
    req.encode(&mut buf).map_err(OpentelemetrySinkError::from)?;

    Ok(buf)
}
