use super::model::OpentelemetryTracesModel;
use crate::sinks::opentelemetry::sink::OpentelemetrySinkError;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message;

use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use opentelemetry_proto::tonic::trace::v1::ScopeSpans;

pub fn group_spans(models: Vec<OpentelemetryTracesModel>) -> Vec<ResourceSpans> {
    let resource_spans: Vec<ResourceSpans> =
        models.into_iter().map(|model| model.0.into()).collect();

    let mut spans: Vec<_> = vec![];
    let resource = resource_spans[0].resource.clone();
    let scope_span = resource_spans[0].scope_spans[0].clone();
    let schema_url = resource_spans[0].schema_url.clone();

    for resource_span in resource_spans.into_iter() {
        for scope_span in resource_span.scope_spans.into_iter() {
            for span in scope_span.spans.into_iter() {
                spans.push(span);
            }
        }
    }

    let scope_spans = vec![ScopeSpans {
        scope: scope_span.scope,
        spans,
        schema_url: scope_span.schema_url,
    }];

    vec![ResourceSpans {
        resource,
        scope_spans,
        schema_url,
    }]
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
