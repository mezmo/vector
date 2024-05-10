use crate::sinks::opentelemetry::{
    logs::model::OpentelemetryLogsModel, sink::OpentelemetrySinkError,
};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use prost::Message;

pub fn encode(models: Vec<OpentelemetryLogsModel>) -> Result<Vec<u8>, OpentelemetrySinkError> {
    let req = ExportLogsServiceRequest {
        resource_logs: models.into_iter().map(|model| model.0.into()).collect(),
    };

    let mut buf = vec![];
    req.encode(&mut buf).map_err(OpentelemetrySinkError::from)?;

    Ok(buf)
}
