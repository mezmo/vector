use metrics::{counter, gauge};

use super::{InternalEvent, NamedInternalEvent};

#[derive(Debug)]
pub struct AggregatedProfileChanged {
    pub count: usize,
}

impl NamedInternalEvent for AggregatedProfileChanged {
    fn name(&self) -> &'static str {
        "UsageMetricsAggregatedProfilesSize"
    }
}

impl InternalEvent for AggregatedProfileChanged {
    #[allow(clippy::cast_precision_loss)]
    fn emit(self) {
        gauge!("usage_metrics_aggregated_profiles_size").set(self.count as f64);
    }
}

pub struct InsertFailed {
    pub error: String,
}

impl NamedInternalEvent for InsertFailed {
    fn name(&self) -> &'static str {
        "UsageMetricsInsertFailed"
    }
}

impl InternalEvent for InsertFailed {
    fn emit(self) {
        counter!("usage_metrics_insert_failed").increment(1);

        error!(message = "Usage metrics insert failed", error = self.error);
    }
}
