use std::io;

use serde::Serialize;

use crate::sinks::util::encoding::{as_tracked_write, Encoder};

use super::models::SumoLogicModel;
use super::sink::SumoLogicSinkError;

#[derive(Clone, Debug)]
pub struct SumoLogicEncoder;

impl Encoder<Result<SumoLogicModel, SumoLogicSinkError>> for SumoLogicEncoder {
    fn encode_input(
        &self,
        input: Result<SumoLogicModel, SumoLogicSinkError>,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize> {
        let json = match input? {
            SumoLogicModel::Logs(log_model) => to_json(&log_model)?,
            SumoLogicModel::Metrics(metric_model) => to_json(&metric_model)?,
        };
        let size = as_tracked_write::<_, _, io::Error>(writer, &json, |writer, json| {
            writer.write_all(json)?;
            Ok(())
        })?;
        io::Result::Ok(size)
    }
}

pub fn to_json<T: Serialize>(model: &T) -> Result<Vec<u8>, SumoLogicSinkError> {
    match serde_json::to_vec(model) {
        Ok(mut json) => {
            json.push(b'\n');
            Ok(json)
        }
        Err(error) => Err(SumoLogicSinkError::new(&format!(
            "Failed generating JSON: {}",
            error
        ))),
    }
}
