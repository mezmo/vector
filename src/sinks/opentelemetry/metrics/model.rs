use crate::sinks::opentelemetry::{
    models::{OpentelemetryModelMatch, OpentelemetryModelType},
    sink::OpentelemetrySinkError,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use vector_lib::event::Event;

#[derive(Debug)]
pub struct OpentelemetryMetricsModel(pub ResourceMetrics);

impl OpentelemetryModelMatch for OpentelemetryMetricsModel {
    fn maybe_match(_event: &Event) -> Option<OpentelemetryModelType> {
        // TODO Metrics are not supported yet.
        // It'll be implemented within LOG-19372 ticket
        let _ = OpentelemetryModelType::Metrics;
        None
    }
}

impl OpentelemetryMetricsModel {
    // pub fn new(metrics: ResourceMetrics) -> Self {
    //     Self(metrics)
    // }
}

impl TryFrom<Event> for OpentelemetryMetricsModel {
    type Error = OpentelemetrySinkError;

    fn try_from(_buf_event: Event) -> Result<Self, Self::Error> {
        // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-sdk/src/metrics/data/mod.rs#L15

        Err(OpentelemetrySinkError::new(
            "Opentelemetry metric model is not implemented yet",
        ))
    }
}
