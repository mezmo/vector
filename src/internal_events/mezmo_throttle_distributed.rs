use metrics::counter;
use vector_lib::internal_event::{ComponentEventsDropped, InternalEvent, INTENTIONAL};

#[derive(Debug)]
pub(crate) struct MezmoThrottleDistributedEventThrottled;

impl InternalEvent for MezmoThrottleDistributedEventThrottled {
    fn emit(self) {
        counter!("mezmo_throttle_events_throttled_total").increment(1);
        counter!("mezmo_throttle_events_throttled_total", "component_id" => "global").increment(1);
        emit!(ComponentEventsDropped::<INTENTIONAL> {
            count: 1,
            reason: "Event throttled.",
        })
    }
}

#[derive(Debug)]
pub struct MezmoThrottleDistributedEventChecked;

impl InternalEvent for MezmoThrottleDistributedEventChecked {
    fn emit(self) {
        counter!("mezmo_throttle_events_checked_total").increment(1);
        counter!("mezmo_throttle_events_checked_total", "component_id" => "global").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoThrottleDistributedCheckFailed {
    pub err: String,
}

impl InternalEvent for MezmoThrottleDistributedCheckFailed {
    fn emit(self) {
        error!(
            error = %self.err,
            internal_log_rate_limit = true,
            "Unable to check rate-limit for event",
        );
        counter!("mezmo_throttle_check_failed_total").increment(1);
        counter!("mezmo_throttle_check_failed_total", "component_id" => "global").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoThrottleDistributedCheckRetried {
    pub attempt: usize,
    pub delay_ms: u128,
}

impl InternalEvent for MezmoThrottleDistributedCheckRetried {
    fn emit(self) {
        debug!(
            attempt = self.attempt,
            delay_ms = self.delay_ms,
            "Retrying rate-limit check for event..."
        );
        counter!("mezmo_throttle_check_retried_total").increment(1);
        counter!("mezmo_throttle_check_retried_total", "component_id" => "global").increment(1);
    }
}
