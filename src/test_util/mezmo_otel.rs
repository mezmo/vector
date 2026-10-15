//! OpenTelemetry metric envelopes as ingestion-service writes them to the `open-telemetry`
//! source, for tests of what metric destinations receive.

use std::collections::BTreeMap;

use serde_json::json;
use vector_lib::event::{KeyString, Value};

use crate::event::{Event, LogEvent, Metric, metric::mezmo::to_metric};

/// 2026-10-15T12:00:00Z
pub const TIME_SECONDS: i64 = 1_792_065_600;

pub fn nanos(seconds: i64) -> String {
    format!("{seconds}000000000")
}

/// The resource and scope both carry an `http.route` attribute so tests can check that the data
/// point attribute wins.
pub fn envelope(name: &str, mut data_point: serde_json::Value) -> Event {
    data_point["attributes"] = json!({"http.route": "/cart"});
    let message = json!({
        "resource": {"attributes": {"service.name": "checkout", "k8s.pod.uid": 7}},
        "scope": {
            "name": "io.opentelemetry.http",
            "version": "1.2.0",
            "attributes": {"scope.attr": "s", "http.route": "scope-loses"},
        },
        "type": "metric",
        "record": {
            "metric": {"name": name, "description": "A description", "unit": "ms"},
            "dataPoint": data_point,
        },
    });

    let mut log: LogEvent =
        serde_json::from_value::<BTreeMap<KeyString, Value>>(json!({"message": message}))
            .unwrap()
            .into();
    log.metadata_mut()
        .value_mut()
        .insert("mezmo_native_otel", Value::Boolean(true));
    Event::Log(log)
}

pub fn envelope_metric(name: &str, data_point: serde_json::Value) -> Metric {
    to_metric(&envelope(name, data_point).into_log())
        .unwrap()
        .unwrap()
}
