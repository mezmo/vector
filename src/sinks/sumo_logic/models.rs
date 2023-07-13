use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vector_core::event::{Event, Value};

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
            Err(SumoLogicSinkError::new("No valid logs to generate"))
        }
    }
}

// The metrics model is not yet implemented but is
// stubbed out for future development.
#[derive(Serialize, Deserialize, Debug)]
pub struct SumoMetricsModel(pub Vec<DataStore>);

#[allow(dead_code)]
impl SumoMetricsModel {
    pub fn new(_metrics_array: Vec<KeyValData>) -> Self {
        unimplemented!()
    }
}

impl TryFrom<Vec<Event>> for SumoMetricsModel {
    type Error = SumoLogicSinkError;

    fn try_from(_buf_events: Vec<Event>) -> Result<Self, Self::Error> {
        unimplemented!() // will panic if called
    }
}
