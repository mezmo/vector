use std::time::Instant;

use metrics::gauge;
use vector_core::internal_event::InternalEvent;

#[derive(Debug)]
pub struct Heartbeat {
    pub since: Instant,
}

impl InternalEvent for Heartbeat {
    fn emit(self) {
        trace!(target: "vector", message = "Beep.");
        gauge!("uptime_seconds", self.since.elapsed().as_secs() as f64);
    }
}
