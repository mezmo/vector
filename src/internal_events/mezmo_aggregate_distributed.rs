use metrics::counter;
use vector_lib::internal_event::InternalEvent;
use vector_lib::internal_event::{ComponentEventsDropped, UNINTENTIONAL};

#[derive(Debug)]
pub struct MezmoAggregateDistributedEventRecorded;

impl InternalEvent for MezmoAggregateDistributedEventRecorded {
    fn emit(self) {
        counter!("mezmo_aggregate_events_recorded_total").increment(1);
        counter!("mezmo_aggregate_events_recorded_total", "component_id" => "global").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoAggregateDistributedFlushed {
    pub event_count: u64,
}

impl InternalEvent for MezmoAggregateDistributedFlushed {
    fn emit(self) {
        counter!("mezmo_aggregate_flush_total").increment(1);
        counter!("mezmo_aggregate_flush_total", "component_id" => "global").increment(1);
        counter!("mezmo_aggregate_flush_events_total").increment(self.event_count);
        counter!("mezmo_aggregate_flush_events_total", "component_id" => "global")
            .increment(self.event_count);
    }
}

#[derive(Debug)]
pub struct MezmoAggregateDistributedFlushFailed {
    pub err: String,
}

impl InternalEvent for MezmoAggregateDistributedFlushFailed {
    fn emit(self) {
        error!("Failed to flush expired windows: {}", self.err);
        counter!("mezmo_aggregate_flush_failed_total").increment(1);
        counter!("mezmo_aggregate_flush_failed_total", "component_id" => "global").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoAggregateDistributedRecordFailed {
    pub drop_reason: &'static str,
    pub err: String,
}

impl InternalEvent for MezmoAggregateDistributedRecordFailed {
    fn emit(self) {
        error!("Failed to record metric: {}", self.err);
        counter!("mezmo_aggregate_record_failed_total").increment(1);
        counter!("mezmo_aggregate_record_failed_total", "component_id" => "global").increment(1);
        emit!(ComponentEventsDropped::<UNINTENTIONAL> {
            count: 1,
            reason: self.drop_reason,
        });
    }
}

#[derive(Debug)]
pub struct MezmoAggregateDistributedRecordRetried {
    pub attempt: usize,
    pub delay_ms: u128,
}

impl InternalEvent for MezmoAggregateDistributedRecordRetried {
    fn emit(self) {
        debug!(
            attempt = self.attempt,
            delay_ms = self.delay_ms,
            "Failed to record metric, retrying..."
        );
        counter!("mezmo_aggregate_record_retried_total").increment(1);
        counter!("mezmo_aggregate_record_retried_total", "component_id" => "global").increment(1);
    }
}
