use crate::{
    conditions::Condition, mezmo::persistence::PersistenceConnection,
    mezmo::user_trace::MezmoUserLog, mezmo::MezmoContext, user_log_error,
};
use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::pin::Pin;
use std::time::Duration;
use tokio::select;
use vector_lib::{
    event::{Event, LogEvent, VrlTarget},
    transform::TaskTransform,
};
use vrl::{
    btreemap,
    compiler::{
        runtime::{Runtime, Terminate},
        Program,
    },
    path::OwnedTargetPath,
    prelude::*,
    value::Value,
};

#[cfg(test)]
use std::sync::atomic::{AtomicI64, Ordering};

mod config;
#[cfg(test)]
mod tests;

// The key for the state persistence db. This transform only stores a single value
// representing the entire "state" of aggregation.
const STATE_PERSISTENCE_KEY: &str = "state";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AggregateWindow {
    size_ms: Range<i64>,
    event: Event,

    // Set to true if the window has been retained in the aggregate list, so it can
    // be used for flush condition comparisons over prior aggregations. The serde
    // tags prevent including the field in the JSON persisted form if false, which
    // is expected to be most of the objects in the list.
    //
    // Previously flushed windows should never be flushed from the aggregate component
    // in the future.
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    flushed: bool,
}

impl AggregateWindow {
    fn new(event: Event, window_start: i64, window_size: i64) -> Self {
        let size_ms = window_start..window_start + window_size;

        let mut event = event;
        let log_event = event.as_mut_log();
        log_event.insert(".metadata.aggregate.event_count", Value::from(1));
        log_event.insert(
            ".metadata.aggregate.start_timestamp",
            Value::from(size_ms.start),
        );
        log_event.insert(
            ".metadata.aggregate.end_timestamp",
            Value::from(size_ms.end),
        );

        Self {
            size_ms,
            event,
            flushed: false,
        }
    }

    fn contains_timestamp(&self, value: i64) -> bool {
        self.size_ms.contains(&value)
    }

    fn should_flush(
        self,
        current_time: i64,
        flush_condition: &Option<Condition>,
        prev_event: Option<Event>,
    ) -> (bool, Self) {
        // If already flushed, there is no need to flush this again. Windows can be
        // allocated and already flushed if they are being retained solely to check
        // a flush condition against a prior window value.
        if self.flushed {
            return (false, self);
        }

        // If the window is expired, don't bother executing the VRL flush condition.
        if !self.contains_timestamp(current_time) {
            return (true, self);
        }

        match flush_condition {
            None => (false, self),
            Some(flush_condition) => {
                let Self {
                    size_ms,
                    mut event,
                    flushed,
                    ..
                } = self;
                if let Some(Event::Log(prev_event)) = prev_event {
                    let prev_event = prev_event.value().clone();
                    event.as_mut_log().insert("%previous", prev_event);
                }
                let (should_flush, mut event) = flush_condition.check(event);
                event.as_mut_log().remove("%previous");

                let event = Self {
                    size_ms,
                    event,
                    flushed,
                };
                (should_flush, event)
            }
        }
    }

    fn increment_event_count(&mut self) {
        match self
            .event
            .as_mut_log()
            .get_mut(".metadata.aggregate.event_count")
        {
            Some(Value::Integer(count)) => *count += 1,
            _ => panic!(
                "missing event_count metadata: size_ms={:?},flushed={},event={:?}",
                self.size_ms,
                self.flushed,
                serde_json::to_string(&self.event)
            ),
        }
    }

    fn set_flushed(&mut self, ts: i64) {
        self.event
            .as_mut_log()
            .insert(".metadata.aggregate.flush_timestamp", Value::from(ts));
        self.flushed = true;
    }
}

// Create a simple struct that either returns the time from the system clock,
// used in prod code paths, or a simple counter, used in automated test paths
// to avoid depending on CI wonk.
#[derive(Debug)]
enum AggregateClock {
    SystemCall,

    #[cfg(test)]
    Counter(AtomicI64),
}

#[cfg(not(test))]
impl AggregateClock {
    #[inline(always)]
    fn now(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

#[cfg(test)]
impl AggregateClock {
    fn now(&self) -> i64 {
        match self {
            Self::SystemCall => Utc::now().timestamp_millis(),
            Self::Counter(val) => val.load(Ordering::Relaxed),
        }
    }

    fn increment_by(&self, i: i64) {
        match self {
            Self::SystemCall => panic!("cannot increment a system call clock impl"),
            Self::Counter(val) => val.fetch_add(i, Ordering::Relaxed),
        };
    }
}

#[derive(Debug)]
pub struct MezmoAggregateV2 {
    data: HashMap<u64, VecDeque<AggregateWindow>>,
    flush_tick_ms: u64,
    flush_condition: Option<Condition>,
    event_key_fields: Vec<OwnedTargetPath>,
    event_timestamp_field: Option<OwnedTargetPath>,
    event_merge_program: Program,
    vrl_runtime: Runtime,
    clock: AggregateClock,
    mezmo_ctx: Option<MezmoContext>,
    aggregator_limits: config::AggregatorLimits,
    state_persistence: Option<Box<dyn PersistenceConnection>>,
    state_persistence_tick_ms: u64,
    state_persistence_max_jitter_ms: u64,
}

impl MezmoAggregateV2 {
    #[allow(clippy::missing_const_for_fn, clippy::too_many_arguments)]
    pub(crate) fn new(
        flush_tick_ms: u64,
        event_key_fields: Vec<OwnedTargetPath>,
        event_merge_program: Program,
        event_timestamp_field: Option<OwnedTargetPath>,
        flush_condition: Option<Condition>,
        mezmo_ctx: Option<MezmoContext>,
        aggregator_limits: config::AggregatorLimits,
        state_persistence: Option<Box<dyn PersistenceConnection>>,
        state_persistence_tick_ms: u64,
        state_persistence_max_jitter_ms: u64,
    ) -> Self {
        let initial_data = match &state_persistence {
            Some(state_persistence) => load_initial_state(state_persistence),
            None => HashMap::new(),
        };

        Self {
            data: initial_data,
            flush_tick_ms,
            flush_condition,
            event_key_fields,
            event_timestamp_field,
            event_merge_program,
            vrl_runtime: Runtime::default(),
            clock: AggregateClock::SystemCall,
            mezmo_ctx,
            aggregator_limits,
            state_persistence,
            state_persistence_tick_ms,
            state_persistence_max_jitter_ms,
        }
    }

    /// Generates a hashed code based on the configured event key fields. This results in a smaller
    /// key value to store in memory and avoids complications with interior mutability.
    fn get_event_key(&self, event: &Event) -> u64 {
        let mut hasher = DefaultHasher::new();
        let event = event.as_log();
        for path in &self.event_key_fields {
            let val = event.get(path).unwrap_or(&Value::Null).to_owned();
            val.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Executes the aggregation VRL program against the current accumulated event and the new event.
    fn run_merge_vrl(&mut self, accum_event: Event, new_event: Event) -> Result<Event, Terminate> {
        let (accum_value, mut accum_meta) = accum_event.into_log().into_parts();
        let (new_value, new_meta) = new_event.into_log().into_parts();
        accum_meta.merge(new_meta);

        let aggregate_meta = accum_value
            .get(".metadata.aggregate")
            .map(ToOwned::to_owned)
            .expect("accumulated event should always contain aggregate metadata");

        let mut vrl_target = VrlTarget::LogEvent(
            Value::from(btreemap! {
                "accum" => accum_value,
                "event" => new_value,
            }),
            accum_meta.clone(),
        );

        let timezone = TimeZone::parse("UTC").unwrap();
        let mut value =
            self.vrl_runtime
                .resolve(&mut vrl_target, &self.event_merge_program, &timezone)?;
        self.vrl_runtime.clear();
        value.insert(".metadata.aggregate", aggregate_meta);
        Ok(Event::from(LogEvent::from_parts(value, accum_meta)))
    }

    fn should_alloc_new_window(
        &self,
        aggregations: &VecDeque<AggregateWindow>,
        event_timestamp: i64,
    ) -> bool {
        match aggregations.back() {
            Some(last_window) if !last_window.flushed => {
                let alloc_at =
                    last_window.size_ms.start + (self.aggregator_limits.min_window_size_ms as i64);
                event_timestamp >= alloc_at
            }
            _ => true,
        }
    }

    /// Executes the aggregation program with the new event, collecting the aggregate into sliding windows. Aggregate
    /// events that are ready to be released/flushed will not be further mutated but remain in memory until the flush
    /// method is called, typically on a polling cycle.
    fn record(&mut self, event: Event) {
        if self.data.len() >= (self.aggregator_limits.mem_cardinality_limit as usize) {
            user_log_error!(
                self.mezmo_ctx,
                Value::from("Aggregate event dropped; cardinality limit exceeded".to_string())
            );
            return;
        }

        let event_key = self.get_event_key(&event);
        let event_timestamp = self.get_event_timestamp(&event);

        // Invoking the VRL runtime requires a mutable borrow and since we can't have two mutable
        // borrows against self at the same time, this code needs to remove the entry from the aggregation
        // cache to update the results.
        let mut event_aggregations = self.data.remove(&event_key);
        // Stores the original timestamp of an existing event before rolling in
        // new events
        match &mut event_aggregations {
            None => {
                let mut windows = VecDeque::new();
                windows.push_back(AggregateWindow::new(
                    event.to_owned(),
                    event_timestamp,
                    self.aggregator_limits.window_duration_ms,
                ));
                event_aggregations = Some(windows);
            }
            Some(aggregations) => {
                // Start looking from the back for windows where this event should be rolled
                // up into. Since the windows are stored in order, the loop can stop at the first
                // window that would not be rolled up into.
                for i in (0..aggregations.len()).rev() {
                    if let Some(window) = aggregations.get_mut(i) {
                        if !window.contains_timestamp(event_timestamp) {
                            break;
                        }
                        match self.run_merge_vrl(window.event.clone(), event.to_owned()) {
                            Err(e) => error!("dropping event; failed to execute VRL program on event to aggregate: {e}"),
                            Ok(new_acc) => window.event = new_acc
                        };
                        window.increment_event_count();
                    }
                }

                // Every new event starts a new window that further points can be rolled up into, aka
                // the windows slide to the next event.
                if self.should_alloc_new_window(aggregations, event_timestamp) {
                    let new_window = AggregateWindow::new(
                        event,
                        event_timestamp,
                        self.aggregator_limits.window_duration_ms,
                    );
                    aggregations.push_back(new_window);
                }
            }
        }

        // Put the entry that was removed after the update back into the cache
        self.data.insert(event_key, event_aggregations.unwrap());
    }

    fn get_event_timestamp(&self, event: &Event) -> i64 {
        let event_timestamp = self
            .event_timestamp_field
            .as_ref()
            .and_then(|p| event.as_log().get(p));
        match event_timestamp {
            Some(Value::Timestamp(dt)) => dt.timestamp_millis(),
            Some(Value::Integer(ts_int)) => *ts_int,
            _ => self.clock.now(),
        }
    }

    /// Check windows starting from the front (oldest) for expired entries. All expired
    /// entries will be drained from the active collection and then produced as values
    /// to emit as output.
    fn flush_finalized(&mut self, output: &mut Vec<Event>) {
        let current_time = self.clock.now();

        // To comply with rust's borrow checker, this method needs to take ownership of the currently
        // allocated BTreeMap and replace it with a new, empty allocated BTreeMap.
        let data = std::mem::take(&mut self.data);
        for (series, windows) in data.into_iter() {
            let mut flush_end = 0;

            // The VRL runtime API forces us to transfer ownership of the event into the VRL runtime
            // and then returns an event, the same one in the case of conditions. Since ownership
            // needs to thread this way, a new VecDeque is allocated to collect the result in the same
            // order as the input while we find the upper range bound of things to flush.
            let mut new_window_list = VecDeque::with_capacity(windows.capacity());
            let mut prev_window = None;
            for (i, window) in windows.into_iter().enumerate() {
                let (should_flush, event) =
                    window.should_flush(current_time, &self.flush_condition, prev_window);
                if should_flush {
                    flush_end = i + 1;
                }
                prev_window = Some(event.event.clone());
                new_window_list.push_back(event);
            }

            // Flush additional items if the number of windows exceeds memory limit
            let flush_excess_windows = new_window_list.len() - flush_end
                > self.aggregator_limits.mem_window_limit as usize;
            if flush_excess_windows {
                flush_end = new_window_list.len() - self.aggregator_limits.mem_window_limit as usize
            }

            // With upper bound of the flush range known, drain windows from the front of the window
            // list. A copy of the last drained element needs to be added back to the head of the window
            // list with the drained flag set. If it's not retained, there is no previous window for the
            // flush_condition check to use when checking the oldest aggregation window.
            let mut to_flush = new_window_list.drain(0..flush_end);
            let retained = to_flush.next_back();
            for mut datum in to_flush {
                if !datum.flushed {
                    datum.set_flushed(current_time);
                    output.push(datum.event);
                }
            }

            if let Some(mut retain) = retained {
                if !retain.flushed {
                    retain.set_flushed(current_time);
                    output.push(retain.event.clone());
                    new_window_list.push_front(retain);
                }
            }

            // Not everything from the metric series might have been drained so skip adding back
            // a series if there are no windows stored.
            if !new_window_list.is_empty() {
                self.data.insert(series, new_window_list);
            }
        }
    }

    /// Saves the current `data` to persistent storage. This is intended to be called from the
    /// polling loop on an interval defined by the `state_persistence_tick_ms` field.
    fn persist_state(&self) {
        if let Some(state_persistence) = &self.state_persistence {
            let value = serde_json::to_string(&self.data);
            if let Err(err) = value {
                error!("MezmoAggregateV2: failed to serialize state: {}", err);
                return;
            }

            match state_persistence.set(STATE_PERSISTENCE_KEY, &value.unwrap()) {
                Ok(_) => debug!("MezmoAggregateV2: state persisted"),
                Err(err) => error!("MezmoAggregateV2: failed to persist state: {}", err),
            }
        }
    }
}

// Handles loading initial state from persistent storage, returning an appropriate
// default value if the state is not found or cannot be deserialized.
#[allow(clippy::borrowed_box)]
fn load_initial_state(
    state_persistence: &Box<dyn PersistenceConnection>,
) -> HashMap<u64, VecDeque<AggregateWindow>> {
    match state_persistence.get("state") {
        Ok(state) => match state {
            Some(state) => match serde_json::from_str(&state) {
                Ok(state) => state,
                Err(err) => {
                    error!(
                        "Failed to deserialize state from persistence: {}, component_id",
                        err
                    );
                    HashMap::new()
                }
            },
            None => HashMap::new(),
        },
        Err(err) => {
            error!(
                "Failed to load state from persistence: {}, component_id",
                err
            );
            HashMap::new()
        }
    }
}

impl TaskTransform<Event> for MezmoAggregateV2 {
    fn transform(
        mut self: Box<Self>,
        mut input_events: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
        Box::pin(stream! {
            let mut flush_interval = tokio::time::interval(Duration::from_millis(self.flush_tick_ms));
            let mut state_persistence_interval = tokio::time::interval(Duration::from_millis(self.state_persistence_tick_ms));
            let mut output:Vec<Event> = Vec::new();
            let mut done = false;

            match &self.state_persistence {
                Some(_) => debug!("MezmoAggregateV2: state persistence enabled"),
                None => debug!("MezmoAggregateV2: state persistence not enabled"),
            }
            while !done {
                select! {
                    _ = flush_interval.tick() => {
                        self.flush_finalized(&mut output);
                    },
                    _ = state_persistence_interval.tick() => {
                        let jitter = rand::thread_rng().gen_range(0..=self.state_persistence_max_jitter_ms);
                        tokio::time::sleep(Duration::from_millis(jitter)).await;
                        self.persist_state();
                    },
                    maybe_event = input_events.next() => {
                        match maybe_event {
                            None => {
                                for (_, windows) in self.data.iter_mut() {
                                    for datum in windows.drain(0..) {
                                        output.push(datum.event);
                                    }
                                }
                                done = true;
                            },
                            Some(event) => self.record(event),
                        }
                    }
                }
                for event in output.drain(..) {
                    yield event;
                }

                if done {
                    self.persist_state()
                }
            }
        })
    }
}
