use std::io;

use crate::sinks::prelude::*;
use crate::sinks::util::encoding::{as_tracked_write, Encoder};

use super::{
    logs::encoding::encode as encode_log, metrics::encoding::encode as encode_metrics,
    models::OpentelemetryModel, sink::OpentelemetrySinkError,
    traces::encoding::encode as encode_traces,
};

#[derive(Clone, Debug)]
pub struct OpentelemetryEncoder;

impl Encoder<Result<OpentelemetryModel, OpentelemetrySinkError>> for OpentelemetryEncoder {
    fn encode_input(
        &self,
        input: Result<OpentelemetryModel, OpentelemetrySinkError>,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)> {
        let output = match input? {
            OpentelemetryModel::Logs(log_model) => encode_log(&log_model)?,
            OpentelemetryModel::Metrics(metric_model) => encode_metrics(&metric_model)?,
            OpentelemetryModel::Traces(trace_model) => encode_traces(trace_model)?,
        };
        let size = as_tracked_write::<_, _, io::Error>(writer, &output, |writer, output| {
            writer.write_all(output)?;
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
