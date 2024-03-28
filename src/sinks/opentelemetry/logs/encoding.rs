use crate::sinks::opentelemetry::{
    logs::model::OpentelemetryLogsModel, sink::OpentelemetrySinkError,
};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use prost::Message;

pub fn encode(model: &OpentelemetryLogsModel) -> Result<Vec<u8>, OpentelemetrySinkError> {
    let logs = model.0[0].get("logs").unwrap().clone();

    let req = ExportLogsServiceRequest {
        resource_logs: logs.into_iter().map(Into::into).collect(),
    };

    let mut buf = vec![];
    req.encode(&mut buf).map_err(OpentelemetrySinkError::from)?;

    Ok(buf)
}
