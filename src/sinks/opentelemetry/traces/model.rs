use crate::sinks::opentelemetry::{
    models::{OpentelemetryModelMatch, OpentelemetryModelType},
    sink::OpentelemetrySinkError,
};
use opentelemetry_sdk::export::trace::SpanData;
use std::collections::HashMap;
use vector_lib::{
    config::log_schema,
    event::{Event, Value},
    lookup::PathPrefix,
};

type DataStore = HashMap<String, Vec<SpanData>>;

#[derive(Debug)]
pub struct OpentelemetryTracesModel(pub Vec<DataStore>);

impl OpentelemetryModelMatch for OpentelemetryTracesModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        let log = event.clone().into_log();
        let message = log.get_message();
        let metadata = log.get((PathPrefix::Event, log_schema().user_metadata_key()));

        if metadata.and(message).is_some() {
            let scope = metadata.unwrap().get("scope");

            if scope.is_some() {
                if let Some(Value::Bytes(ref level)) = metadata.unwrap().get("level") {
                    let trace_id = message.unwrap().get("trace.id");
                    let span_id = message.unwrap().get("span.id");

                    if String::from_utf8_lossy(level).into_owned() == "trace"
                        && trace_id.and(span_id).is_some()
                    {
                        return Some(OpentelemetryModelType::Traces);
                    }
                }
            }
        }

        None
    }
}

impl OpentelemetryTracesModel {
    pub fn new(traces_data_array: Vec<SpanData>) -> Self {
        let mut traces_store = DataStore::new();
        traces_store.insert("traces".to_owned(), traces_data_array);
        Self(vec![traces_store])
    }
}

impl TryFrom<Vec<Event>> for OpentelemetryTracesModel {
    type Error = OpentelemetrySinkError;

    fn try_from(_buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-sdk/src/export/trace.rs#L71
        let traces_array = vec![];
        let _ = Self::new(traces_array);

        Err(OpentelemetrySinkError::new(
            "Opentelemetry trace model is not implemented yet",
        ))
    }
}
