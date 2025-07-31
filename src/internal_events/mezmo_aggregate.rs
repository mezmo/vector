use metrics::counter;
use vector_lib::internal_event::InternalEvent;

#[derive(Debug)]
pub struct MezmoAggregateEventRecorded;

impl InternalEvent for MezmoAggregateEventRecorded {
    fn emit(self) {
        counter!("mezmo_aggregate_events_recorded_total").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoAggregateFlushed;

impl InternalEvent for MezmoAggregateFlushed {
    fn emit(self) {
        counter!("mezmo_aggregate_flushes_total").increment(1);
    }
}

#[derive(Debug)]
pub struct MezmoAggregateUpdateFailed;

impl InternalEvent for MezmoAggregateUpdateFailed {
    fn emit(self) {
        counter!("mezmo_aggregate_failed_updates").increment(1);
    }
}
