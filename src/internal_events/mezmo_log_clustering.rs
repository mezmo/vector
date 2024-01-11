use metrics::histogram;
use std::time::Duration;
use vector_lib::internal_event::InternalEvent;

pub struct MezmoLogClusteringStore {
    pub elapsed: Duration,
    pub total_usage_records: usize,
}

impl InternalEvent for MezmoLogClusteringStore {
    fn emit(self) {
        histogram!("mezmo_log_clustering_store_seconds", self.elapsed);
        histogram!(
            "mezmo_log_clustering_store_records",
            self.total_usage_records as f64
        );
    }
}
