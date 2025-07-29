use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use prost::Message;

use super::model::OpentelemetryResourceMetrics;
use crate::sinks::mezmo_opentelemetry::sink::OpentelemetrySinkError;

pub fn encode(model: OpentelemetryResourceMetrics) -> Result<Vec<u8>, OpentelemetrySinkError> {
    // Metric model to Protobuf encoding
    // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-otlp/src/exporter/http/metrics.rs#L52-L61

    let req: ExportMetricsServiceRequest = (&model.0).into();

    let mut buf = vec![];
    req.encode(&mut buf).map_err(OpentelemetrySinkError::from)?;

    Ok(buf)
}
