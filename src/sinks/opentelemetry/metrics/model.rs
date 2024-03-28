use crate::sinks::opentelemetry::{
    models::{OpentelemetryModelMatch, OpentelemetryModelType},
    sink::OpentelemetrySinkError,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use std::collections::HashMap;
use std::str::FromStr;
use vector_lib::{
    config::log_schema,
    event::{Event, Value},
    lookup::PathPrefix,
};

type DataStore = HashMap<String, Vec<ResourceMetrics>>;

enum OpentelemetryMetricsType {
    Gauge,
    Sum,
    Histogram,
    ExponentialHistogram,
    Summary,
}

impl FromStr for OpentelemetryMetricsType {
    type Err = ();
    fn from_str(input: &str) -> Result<OpentelemetryMetricsType, Self::Err> {
        match input {
            "gauge" => Ok(OpentelemetryMetricsType::Gauge),
            "sum" => Ok(OpentelemetryMetricsType::Sum),
            "histogram" => Ok(OpentelemetryMetricsType::Histogram),
            "exponential_histogram" => Ok(OpentelemetryMetricsType::ExponentialHistogram),
            "summary" => Ok(OpentelemetryMetricsType::Summary),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct OpentelemetryMetricsModel(pub Vec<DataStore>);

impl OpentelemetryModelMatch for OpentelemetryMetricsModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        let log = event.clone().into_log();
        let message = log.get_message();
        let metadata = log.get((PathPrefix::Event, log_schema().user_metadata_key()));

        if metadata.and(message).is_some() {
            let scope = metadata.unwrap().get("scope");

            if scope.is_some() {
                if let Some(value) = message.unwrap().get("value") {
                    if let Some(Value::Bytes(ref metric_type)) = value.get("type") {
                        if OpentelemetryMetricsType::from_str(
                            String::from_utf8_lossy(metric_type).into_owned().as_str(),
                        )
                        .is_ok()
                        {
                            return Some(OpentelemetryModelType::Metrics);
                        }
                    }
                }
            }
        }

        None
    }
}

impl OpentelemetryMetricsModel {
    pub fn new(metrics_data_array: Vec<ResourceMetrics>) -> Self {
        let mut metrics_store = DataStore::new();
        metrics_store.insert("metrics".to_owned(), metrics_data_array);
        Self(vec![metrics_store])
    }
}

impl TryFrom<Vec<Event>> for OpentelemetryMetricsModel {
    type Error = OpentelemetrySinkError;

    fn try_from(_buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-sdk/src/metrics/data/mod.rs#L15
        let metrics_array = vec![];
        let _ = Self::new(metrics_array);

        Err(OpentelemetrySinkError::new(
            "Opentelemetry metric model is not implemented yet",
        ))
    }
}
