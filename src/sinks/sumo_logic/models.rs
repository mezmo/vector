use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vector_core::event::{Event, Value};

use crate::sinks::prometheus::collector::{MetricCollector, StringCollector};

use super::sink::SumoLogicSinkError;

type KeyValData = HashMap<String, Value>;
type DataStore = HashMap<String, Vec<KeyValData>>;

pub enum SumoLogicModel {
    Logs(SumoLogsModel),
    Metrics(SumoMetricsModel),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SumoLogsModel(pub Vec<DataStore>);

impl SumoLogsModel {
    pub fn new(logs_array: Vec<KeyValData>) -> Self {
        let mut logs_store = DataStore::new();
        logs_store.insert("logs".to_owned(), logs_array);
        Self(vec![logs_store])
    }
}

impl TryFrom<Vec<Event>> for SumoLogsModel {
    type Error = SumoLogicSinkError;

    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut logs_array = vec![];
        for buf_event in buf_events {
            if let Event::Log(log) = buf_event {
                let mut log_model = KeyValData::new();
                for (k, v) in log.convert_to_fields() {
                    log_model.insert(k, v.clone());
                }
                if log.get("message").is_none() {
                    log_model.insert(
                        "message".to_owned(),
                        Value::from("log from mezmo".to_owned()),
                    );
                }
                logs_array.push(log_model);
            }
        }

        if !logs_array.is_empty() {
            Ok(Self::new(logs_array))
        } else {
            Err(SumoLogicSinkError::new("No valid logs to process"))
        }
    }
}

/// The SumoMetricsModel is a vector of structs meant
/// to conform to the Prometheus metrics format.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SumoMetricsModel(pub Vec<String>);

impl SumoMetricsModel {
    pub fn new(metrics_array: Vec<String>) -> Self {
        let mut metrics_data_array = vec![];
        for m in metrics_array {
            metrics_data_array.push(m)
        }

        Self(metrics_data_array)
    }
}

impl TryFrom<Vec<Event>> for SumoMetricsModel {
    type Error = SumoLogicSinkError;

    /// Takes in a Vec<Event> and uses the Prometheus StringCollector type to
    /// encode a vector of string to initialize the SumoMetricsModel
    fn try_from(buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        let mut metrics_array = vec![];
        for buf_event in buf_events {
            if let Event::Metric(metric) = buf_event {
                let mut string_metrics = StringCollector::new();

                // TODO: This currently sets bucktes and quantiles to &[] because when the MetricValue is
                // MetricValue::AggrogatedHistoram or MetricValue::AggrogatedSummary those values are provided.
                // In order for this to work for a MetricValue::Distribution we'd need to add a default value for
                // those fields.
                string_metrics.encode_metric(metric.namespace(), &[], &[], &metric);
                metrics_array.push(string_metrics.finish());
            }
        }

        if !metrics_array.is_empty() {
            Ok(SumoMetricsModel::new(metrics_array))
        } else {
            Err(SumoLogicSinkError::new("No valid metrics to process"))
        }
    }
}
