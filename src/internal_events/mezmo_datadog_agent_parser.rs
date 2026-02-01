use metrics::counter;
use vector_lib::internal_event::{ComponentEventsDropped, InternalEvent, UNINTENTIONAL};

#[derive(Debug)]
pub struct MezmoDatadogAgentParserError<'a> {
    pub error: &'a str,
    pub event_type: Option<&'a str>,
}

#[derive(Debug)]
pub struct MezmoDatadogAgentParserInvalidItem<'a> {
    pub error: &'a str,
    pub item_type: &'a str,
    pub event_type: Option<&'a str>,
}

#[derive(Debug)]
pub struct MezmoDatadogAgentParserDroppedSpan<'a> {
    pub missing_fields: &'a str,
}

impl InternalEvent for MezmoDatadogAgentParserError<'_> {
    fn emit(self) {
        let event_type = self.event_type.unwrap_or("unknown");
        error!(
            message = "Failed to parse Datadog agent payload.",
            error = %self.error,
            event_type = %event_type,
            internal_log_rate_limit = true,
        );
        counter!(
            "mezmo_datadog_agent_parser_errors_total",
            "event_type" => event_type.to_string(),
        )
        .increment(1);
    }
}

impl InternalEvent for MezmoDatadogAgentParserInvalidItem<'_> {
    fn emit(self) {
        let event_type = self.event_type.unwrap_or("unknown");
        error!(
            message = "Invalid item error.",
            error = %self.error,
            item_type = %self.item_type,
            event_type = %event_type,
            internal_log_rate_limit = true,
        );
        counter!(
            "mezmo_datadog_agent_parser_invalid_items_total",
            "item_type" => self.item_type.to_string(),
            "event_type" => event_type.to_string(),
        )
        .increment(1);
    }
}

impl InternalEvent for MezmoDatadogAgentParserDroppedSpan<'_> {
    fn emit(self) {
        let reason = format!("Missing {} for span", self.missing_fields);
        warn!(
            message = %reason,
            internal_log_rate_limit = true,
        );
        emit!(ComponentEventsDropped::<UNINTENTIONAL> {
            count: 1,
            reason: &reason,
        });
    }
}
