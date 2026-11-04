use super::model::OpentelemetryTracesModel;
use crate::sinks::mezmo_opentelemetry::sink::OpentelemetrySinkError;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message;

use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use opentelemetry_sdk::Resource;

/// Returns one `ResourceSpans` per distinct resource, each holding one `ScopeSpans` per
/// distinct scope under it.
pub fn group_spans(models: Vec<OpentelemetryTracesModel>) -> Vec<ResourceSpans> {
    // Resources are matched on the SDK value: its proto attributes come out in hash order.
    let mut groups: Vec<(Resource, ResourceSpans)> = vec![];

    for OpentelemetryTracesModel(span) in models {
        let resource = span.resource.clone().into_owned();
        let mut converted: ResourceSpans = span.into();

        let index = match groups.iter().position(|(known, _)| *known == resource) {
            Some(index) => index,
            None => {
                groups.push((resource, converted));
                continue;
            }
        };
        let group = &mut groups[index].1;

        for scope_spans in converted.scope_spans.drain(..) {
            match group.scope_spans.iter_mut().find(|known| {
                known.scope == scope_spans.scope && known.schema_url == scope_spans.schema_url
            }) {
                Some(known) => known.spans.extend(scope_spans.spans),
                None => group.scope_spans.push(scope_spans),
            }
        }
    }

    groups.into_iter().map(|(_, group)| group).collect()
}

pub fn encode(models: Vec<OpentelemetryTracesModel>) -> Result<Vec<u8>, OpentelemetrySinkError> {
    let mut buf = vec![];

    if models.is_empty() {
        return Ok(buf);
    }

    let req = ExportTraceServiceRequest {
        resource_spans: group_spans(models),
    };

    req.encode(&mut buf).map_err(OpentelemetrySinkError::from)?;

    Ok(buf)
}
