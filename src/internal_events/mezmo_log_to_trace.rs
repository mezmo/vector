use metrics::counter;
use vector_lib::internal_event::InternalEvent;

#[derive(Debug)]
pub struct MezmoLogToTraceEventDropped<'a> {
    pub reason: &'a str,
}

impl InternalEvent for MezmoLogToTraceEventDropped<'_> {
    fn emit(self) {
        warn!(
            message = "Event dropped.",
            reason = %self.reason,
            internal_log_rate_limit = true,
        );
        counter!(
            "mezmo_log_to_trace_events_dropped_total",
            "reason" => self.reason.to_string(),
        )
        .increment(1);
    }
}
