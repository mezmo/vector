use metrics::gauge;

use super::InternalEvent;

#[derive(Debug)]
pub struct AggregatedProfileChanged {
    pub count: usize,
}

impl InternalEvent for AggregatedProfileChanged {
    #[allow(clippy::cast_precision_loss)]
    fn emit(self) {
        gauge!("usage_metrics_aggregated_profiles_size", self.count as f64);
    }

    fn name(&self) -> Option<&'static str> {
        Some("UsageMetricsAggregatedProfilesSize")
    }
}
