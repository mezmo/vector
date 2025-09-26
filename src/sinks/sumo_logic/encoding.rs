use std::io;

use crate::sinks::prelude::*;
use crate::sinks::util::encoding::{as_tracked_write, Encoder};
use serde::Serialize;

use super::models::{SumoLogicModel, SumoMetricsModel};
use super::sink::SumoLogicSinkError;

#[derive(Clone, Debug)]
pub struct SumoLogicEncoder;

impl Encoder<Result<SumoLogicModel, SumoLogicSinkError>> for SumoLogicEncoder {
    fn encode_input(
        &self,
        input: Result<SumoLogicModel, SumoLogicSinkError>,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)> {
        let json = match input? {
            SumoLogicModel::Logs(log_model) => to_json(&log_model)?,
            SumoLogicModel::Metrics(metric_model) => metrics_to_utf8(&metric_model)?,
        };
        let size = as_tracked_write::<_, _, io::Error>(writer, &json, |writer, json| {
            writer.write_all(json)?;
            Ok(())
        })?;
        io::Result::Ok((
            size,
            GroupedCountByteSize::Untagged {
                size: CountByteSize(size, JsonSize::new(size)),
            },
        ))
    }
}

fn to_json<T: Serialize>(model: &T) -> Result<Vec<u8>, SumoLogicSinkError> {
    match serde_json::to_vec(model) {
        Ok(mut json) => {
            json.push(b'\n');
            Ok(json)
        }
        Err(error) => Err(SumoLogicSinkError::new(&format!(
            "Failed generating JSON: {error}",
        ))),
    }
}

/// Takes a SumoMetricsModel and transforms it into a Prometheus metrics payload.
fn metrics_to_utf8(model: &SumoMetricsModel) -> Result<Vec<u8>, SumoLogicSinkError> {
    let mut metric_bytes: Vec<u8> = Vec::new();
    for m in model.0.iter() {
        metric_bytes.extend(m.clone().into_bytes())
    }

    Ok(metric_bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_metrics_to_utf8() {
        let mut test_metrics = vec![];
        let expected_string = String::from("test_vector_metric{test=\"value\"} 0 123456789");
        test_metrics.push("test_vector_metric{{test=\"value\"}} 0 123456789".to_string());

        let test_utf8 = metrics_to_utf8(&SumoMetricsModel::new(test_metrics));

        assert_eq!(expected_string.into_bytes()[0], test_utf8.unwrap()[0]);
    }
}
