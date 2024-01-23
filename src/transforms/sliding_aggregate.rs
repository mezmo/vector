use crate::{
    conditions::{AnyCondition, Condition},
    config::{TransformConfig, TransformContext},
    mezmo::user_trace::MezmoUserLog,
    mezmo::MezmoContext,
    transforms::remap::RemapConfig,
    user_log_error,
};
use async_stream::stream;
use chrono::Utc;
use futures::{Stream, StreamExt};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::pin::Pin;
use std::time::Duration;
use tokio::select;
use vector_lib::configurable::configurable_component;
use vector_lib::enrichment::TableRegistry;
use vector_lib::{
    config::{clone_input_definitions, DataType, Input, LogNamespace, OutputId, TransformOutput},
    event::{Event, LogEvent, VrlTarget},
    schema::Definition,
    transform::{TaskTransform, Transform},
};
use vrl::{
    btreemap,
    compiler::{
        runtime::{Runtime, Terminate},
        Program,
    },
    path::{parse_target_path, OwnedTargetPath},
    prelude::*,
    value::Value,
};

#[cfg(test)]
use std::sync::atomic::{AtomicI64, Ordering};

/// Configuration for the `sliding_aggregate` transform.
#[configurable_component(transform("sliding_aggregate", "Mezmo sliding aggregate"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct SlidingAggregateConfig {
    /// The sliding window duration in milliseconds to use when
    /// determining the aggregate values.
    #[serde(default = "default_window_duration_ms")]
    window_duration_ms: u32,

    /// Set how often the transform will check for events that are expired or have triggered
    /// a flush condition.
    #[serde(default = "default_flush_tick_ms")]
    flush_tick_ms: u64,

    /// Maximum number of keys to maintain in the transform's map
    #[serde(default = "default_mem_cardinality_limit")]
    mem_cardinality_limit: u32,

    /// The maximum number of sliding window structs to allow per metric series
    #[serde(default = "default_mem_window_limit")]
    mem_window_limit: u32,

    /// The minimum window over which to aggregate data
    #[serde(default = "default_min_window_size_ms")]
    min_window_size_ms: u32,

    // LOG-18567: Two additional properties that were considered as part of the spike but
    // pushed out of the spike effort. They include:
    //   * mem_cardinality_limit - the maximum number of keys to maintain in the transform's BTreeMap.
    //   * mem_window_limit - the maximum number of sliding window structs to allow per metric series.
    /// Define a VRL condition that when it evaluates to true on the aggregated event will
    /// also cause the aggregated event to be flushed to the output stream. This may produce
    /// values outside of their sliding window because of a true condition.
    flush_condition: Option<AnyCondition>,

    /// The different field paths that form the event identity. Events with matching values
    /// will be aggregated together.
    event_id_fields: Vec<String>,

    /// An optional path to the timestamp field on the event. If a field path isn't supplied
    /// or if the event is missing the timestamp, the current system time will be used.
    event_timestamp_field: Option<String>,

    /// The VRL program that produces a new accumulated aggregate event from the prior accumulated
    /// event and the new event.
    source: String,
}

const fn default_window_duration_ms() -> u32 {
    2 * 1000
}

const fn default_flush_tick_ms() -> u64 {
    5
}

const fn default_mem_cardinality_limit() -> u32 {
    2000
}

const fn default_mem_window_limit() -> u32 {
    200
}

const fn default_min_window_size_ms() -> u32 {
    5000
}

impl_generate_config_from_default!(SlidingAggregateConfig);

impl SlidingAggregateConfig {
    /// This method does all of the work of turning a SlidingAggregateConfig instance into a
    /// SlidingAggregate instance. It's separate from the build() method so tests get the
    /// actual SlidingAggregate type and can then reach into the type to target test cases.
    async fn internal_build(&self, ctx: &TransformContext) -> crate::Result<SlidingAggregate> {
        // Leverage the remap transform to build the VRL program from the string source code. This
        // could be moved into a shared function between the two but this works.
        let remap_config = RemapConfig {
            source: Some(self.source.clone()),
            ..Default::default()
        };
        let (program, _, _, _) = remap_config.compile_vrl_program(
            ctx.enrichment_tables.clone(),
            ctx.merged_schema_definition.clone(),
            ctx.mezmo_ctx.clone(),
        )?;

        let mut event_key_fields = Vec::new();
        for id_path in &self.event_id_fields {
            event_key_fields.push(parse_target_path(id_path)?);
        }

        let flush_condition = self
            .flush_condition
            .as_ref()
            .map(|cond| cond.build(&ctx.enrichment_tables, ctx.mezmo_ctx.clone()))
            .transpose()?;

        let window_size_ms = self.window_duration_ms as i64; // ok cast since u32::MAX < i64::MAX

        let event_timestamp_field = self
            .event_timestamp_field
            .as_ref()
            .map(|s| parse_target_path(s.as_str()))
            .transpose()?;

        Ok(SlidingAggregate::new(
            self.flush_tick_ms,
            event_key_fields,
            program,
            event_timestamp_field,
            flush_condition,
            ctx.mezmo_ctx.clone(),
            AggregatorLimits::new(
                self.mem_window_limit,
                self.mem_cardinality_limit,
                self.min_window_size_ms,
                window_size_ms,
            ),
        ))
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "sliding_aggregate")]
impl TransformConfig for SlidingAggregateConfig {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        self.internal_build(ctx).await.map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _enrichment_tables: TableRegistry,
        input_definitions: &[(OutputId, Definition)],
        _global_log_namespace: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(
            DataType::Log,
            clone_input_definitions(input_definitions),
        )]
    }
}

#[derive(Debug)]
struct AggregateWindow {
    size_ms: Range<i64>,
    event: Event,
}

impl AggregateWindow {
    const fn new(event: Event, window_start: i64, window_size: i64) -> Self {
        let max_window = window_start..window_start + window_size;
        Self {
            size_ms: max_window,
            event,
        }
    }

    const fn from_parts(size_ms: Range<i64>, event: Event) -> Self {
        Self { size_ms, event }
    }

    fn is_expired(&self, current_time: i64) -> bool {
        !self.contains_timestamp(current_time)
    }

    fn contains_timestamp(&self, value: i64) -> bool {
        self.size_ms.contains(&value)
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
pub struct SlidingAggregate {
    data: HashMap<u64, VecDeque<AggregateWindow>>,
    flush_tick_ms: u64,
    flush_condition: Option<Condition>,
    event_key_fields: Vec<OwnedTargetPath>,
    event_timestamp_field: Option<OwnedTargetPath>,
    event_merge_program: Program,
    vrl_runtime: Runtime,
    clock: AggregateClock,
    mezmo_ctx: Option<MezmoContext>,
    aggregator_limits: AggregatorLimits,
}

impl SlidingAggregate {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        flush_tick_ms: u64,
        event_key_fields: Vec<OwnedTargetPath>,
        event_merge_program: Program,
        event_timestamp_field: Option<OwnedTargetPath>,
        flush_condition: Option<Condition>,
        mezmo_ctx: Option<MezmoContext>,
        aggregator_limits: AggregatorLimits,
    ) -> Self {
        Self {
            data: HashMap::new(),
            flush_tick_ms,
            flush_condition,
            event_key_fields,
            event_timestamp_field,
            event_merge_program,
            vrl_runtime: Runtime::default(),
            clock: AggregateClock::SystemCall,
            mezmo_ctx,
            aggregator_limits,
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

    /// Executes the aggregation VRL program againt the current accumulated event and the new event.
    fn run_merge_vrl(&mut self, accum_event: Event, new_event: Event) -> Result<Event, Terminate> {
        let (accum_value, mut accum_meta) = accum_event.into_log().into_parts();
        let (new_value, new_meta) = new_event.into_log().into_parts();
        accum_meta.merge(new_meta);

        let mut vrl_target = VrlTarget::LogEvent(
            Value::from(btreemap! {
                "accum" => accum_value,
                "event" => new_value,
            }),
            accum_meta.clone(),
        );

        let timezone = TimeZone::parse("UTC").unwrap();
        let value =
            self.vrl_runtime
                .resolve(&mut vrl_target, &self.event_merge_program, &timezone)?;
        self.vrl_runtime.clear();
        Ok(Event::from(LogEvent::from_parts(value, accum_meta)))
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
        let mut last_event_timestamp: Option<i64> = None;
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
                // Get the timestamp of the most recent window before potential mutations
                // occur due to calls to run_merge_vrl
                if let Some(last_window) = aggregations.back() {
                    last_event_timestamp = Some(self.get_event_timestamp(&last_window.event));
                }
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
                    }
                }

                let should_allocate_new_window = match last_event_timestamp {
                    Some(last_timestamp) => {
                        event_timestamp
                            > last_timestamp + (self.aggregator_limits.min_window_size_ms as i64)
                    }
                    None => true,
                };

                // Every new event starts a new window that further points can be rolled up into, aka
                // the windows slide to the next event.
                if should_allocate_new_window {
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
            for (i, window) in windows.into_iter().enumerate() {
                let mut should_flush = false;
                let is_expired = window.is_expired(current_time);

                let AggregateWindow {
                    size_ms, mut event, ..
                } = window;
                if let Some(flush_condition) = &self.flush_condition {
                    let res = flush_condition.check(event);
                    should_flush = res.0;
                    event = res.1;
                }

                if is_expired || should_flush {
                    flush_end = i + 1;
                }

                new_window_list.push_back(AggregateWindow::from_parts(size_ms, event));
            }

            // Flush additional items if the number of windows exceeds memory limit
            let flush_excess_windows = new_window_list.len() - flush_end
                > self.aggregator_limits.mem_window_limit as usize;
            if flush_excess_windows {
                flush_end = new_window_list.len() - self.aggregator_limits.mem_window_limit as usize
            }

            // With upper bound of the flush range known, drain the first elements and push them
            // into the output Vec.
            for datum in new_window_list.drain(0..flush_end) {
                output.push(datum.event);
            }

            // Not everything from the metric series might have been drained so any non-empty windows
            // need to be inserted back into the transform's struct.
            if !new_window_list.is_empty() {
                self.data.insert(series, new_window_list);
            }
        }
    }
}

impl TaskTransform<Event> for SlidingAggregate {
    fn transform(
        mut self: Box<Self>,
        mut input_events: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
        Box::pin(stream! {
            let mut flush_interval = tokio::time::interval(Duration::from_millis(self.flush_tick_ms));
            let mut output:Vec<Event> = Vec::new();
            let mut done = false;
            while !done {
                select! {
                    _ = flush_interval.tick() => {
                        self.flush_finalized(&mut output);
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
            }
        })
    }
}

#[derive(Debug)]
pub struct AggregatorLimits {
    /// The maximum number of sliding window structs to allow per metric series
    pub mem_window_limit: u32,
    /// Maximum number of keys to maintain in the transform's map
    pub mem_cardinality_limit: u32,
    /// A new window is allocated only if an event's time surpasses the last
    /// saved window's time by a minimum window size
    pub min_window_size_ms: u32,
    /// The size of each sliding window in milliseconds. Arriving events
    /// aggregate into a window if their timestamp falls within the window
    pub window_duration_ms: i64,
}

impl AggregatorLimits {
    const fn new(
        mem_window_limit: u32,
        mem_cardinality_limit: u32,
        min_window_size_ms: u32,
        window_duration_ms: i64,
    ) -> Self {
        Self {
            mem_window_limit,
            mem_cardinality_limit,
            min_window_size_ms,
            window_duration_ms,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    use vector_lib::event::LogEvent;
    use vrl::btreemap;

    async fn new_aggregator(
        flush_condition: Option<&str>,
        memory_limits: AggregatorLimits,
    ) -> SlidingAggregate {
        let flush_condition = match flush_condition {
            None => "".to_string(),
            Some(stmt) => format!(
                r#"
                flush_condition = """
                {stmt}
                """
                "#
            ),
        };

        let AggregatorLimits {
            mem_window_limit,
            mem_cardinality_limit,
            min_window_size_ms,
            window_duration_ms,
        } = memory_limits;

        let config = format!(
            r#"
            window_duration_ms = {window_duration_ms}
            mem_cardinality_limit = {mem_cardinality_limit}
            mem_window_limit = {mem_window_limit}
            min_window_size_ms = {min_window_size_ms}
            flush_tick_ms = 1
            event_id_fields = [".message.name", ".message.tags"]
            event_timestamp_field = ".metadata.timestamp"
            source = """
                new_acc = {{ }}

                new_acc.metadata = object!(.accum.metadata)
                new_acc.metadata = merge(object!(.event.metadata), new_acc.metadata, deep: true)

                new_acc.message = object!(.accum.message)
                new_acc.message = merge(object!(.event.message), new_acc.message, deep: true)
                new_acc.message.value.value, err = .accum.message.value.value + .event.message.value.value
                . = new_acc
            """
            {flush_condition}
            "#
        );

        let config: SlidingAggregateConfig = toml::from_str(config.as_str()).unwrap();
        let ctx = TransformContext::default();
        let mut aggregator = match config.internal_build(&ctx).await {
            Err(e) => panic!("{e}"),
            Ok(aggregator) => aggregator,
        };
        aggregator.clock = AggregateClock::Counter(AtomicI64::new(1));
        aggregator
    }

    const fn default_aggregator_limits() -> AggregatorLimits {
        AggregatorLimits::new(200, 2000, 5000, 5)
    }

    const fn aggregator_limits_custom_window_size(min_window_size: u32) -> AggregatorLimits {
        AggregatorLimits::new(200, 2000, min_window_size, 5)
    }

    fn counter_event(
        name: impl Into<String>,
        tags: Option<BTreeMap<String, String>>,
        value: f64,
    ) -> Event {
        let name = name.into();
        let tags = match tags {
            None => "{}".to_string(),
            Some(tags) => serde_json::to_string(&tags).unwrap(),
        };
        let counter = format!(
            r#"{{
                "metadata": {{ }},
                "message": {{
                    "name": "{name}",
                    "kind": "absolute",
                    "namespace": "ns",
                    "tags": {tags},
                    "value": {{
                        "type": "count",
                        "value": {value}
                    }}
                }}
            }}"#,
        );
        let log_event: LogEvent = serde_json::from_str::<BTreeMap<String, Value>>(counter.as_str())
            .unwrap()
            .into();
        Event::from(log_event)
    }

    fn counter_event_custom_timestamp(
        name: impl Into<String>,
        tags: Option<BTreeMap<String, String>>,
        value: f64,
        timestamp: u32,
    ) -> Event {
        let mut event = counter_event(name, tags, value);
        let metadata = json!({
            "timestamp": timestamp
        });
        event.as_mut_log().insert("metadata", metadata);
        event
    }

    fn metric_event_key(name: impl Into<String>, tags: Option<BTreeMap<String, String>>) -> u64 {
        let mut hasher = DefaultHasher::new();

        let name = Value::from(name.into());
        name.hash(&mut hasher);

        let mut res_tags = BTreeMap::new();
        if let Some(tags) = tags {
            for (k, v) in tags {
                res_tags.insert(k, Value::from(v));
            }
        }
        let tags = Value::from(res_tags);
        tags.hash(&mut hasher);

        hasher.finish()
    }

    // Since events are held in a HashMap inside the aggregate component, the order they are
    // drained can vary between test runs. This function sorts the events by name so that test
    // assertions can ignore the hashing randomness.
    fn fix_event_ordering(events: &mut [Event]) {
        events.sort_by(|left, right| {
            let left_name = left.as_log().get(".message.name").unwrap();
            let right_name = right.as_log().get(".message.name").unwrap();
            left_name.as_str().cmp(&right_name.as_str())
        });
    }

    fn assert_windows_eq(expected_events: Vec<Event>, windows: &VecDeque<AggregateWindow>) {
        assert_events_eq(
            expected_events,
            windows
                .iter()
                .map(|w| w.event.clone())
                .collect::<Vec<Event>>(),
        );
        assert!(windows
            .iter()
            .all(|AggregateWindow { size_ms, .. }| size_ms.end - size_ms.start == 5))
    }

    fn assert_events_eq(expected_events: Vec<Event>, actual_events: Vec<Event>) {
        assert_eq!(
            expected_events.len(),
            actual_events.len(),
            "number of actual_events does not match expected_events"
        );
        for (actual_event, expected_event) in actual_events.iter().zip(expected_events) {
            assert_eq!(*actual_event, expected_event);
        }
    }

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<SlidingAggregateConfig>();
    }

    #[tokio::test]
    async fn record_single_metric() {
        let mut target = new_aggregator(None, default_aggregator_limits()).await;
        target.record(counter_event("a", None, 10.0));
        assert_eq!(target.data.len(), 1);

        let key = metric_event_key("a", None);
        let val = target.data.get(&key).unwrap();
        assert!(val.capacity() >= 2);

        let AggregateWindow {
            size_ms: actual_size_ms,
            event: actual_event,
            ..
        } = val.get(0).unwrap();
        assert_eq!(5, actual_size_ms.end - actual_size_ms.start);
        assert_eq!(*actual_event, counter_event("a", None, 10.0));
    }

    #[tokio::test]
    async fn record_overlapping_windows() {
        let mut target = new_aggregator(None, aggregator_limits_custom_window_size(0)).await;
        target.record(counter_event_custom_timestamp("a", None, 3.0, 1));
        target.record(counter_event_custom_timestamp("a", None, 4.0, 3));
        assert_eq!(
            target.data.len(),
            1,
            "number of hashmap records didn't match"
        );

        let key = metric_event_key("a", None);
        let actual = target.data.get(&key).unwrap();
        assert_eq!(actual.len(), 2, "number of sliding windows didn't match");
        assert_windows_eq(
            vec![
                counter_event_custom_timestamp("a", None, 7.0, 1),
                counter_event_custom_timestamp("a", None, 4.0, 3),
            ],
            actual,
        );
    }

    #[tokio::test]
    async fn record_nonoverlapping_windows() {
        let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10)).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event_custom_timestamp("a", None, 4.0, 15));
        assert_eq!(
            target.data.len(),
            1,
            "number of hashmap records didn't match"
        );

        let key = metric_event_key("a", None);
        let actual = target.data.get(&key).unwrap();
        assert_eq!(actual.len(), 2, "number of sliding windows didn't match");
        assert_windows_eq(
            vec![
                counter_event("a", None, 3.0),
                counter_event_custom_timestamp("a", None, 4.0, 15),
            ],
            actual,
        );
    }

    #[tokio::test]
    async fn record_group_by_tags() {
        let mut target = new_aggregator(None, aggregator_limits_custom_window_size(0)).await;
        target.record(counter_event(
            "a",
            Some(btreemap! { "host" => "host-1"}),
            3.0,
        ));
        target.record(counter_event(
            "a",
            Some(btreemap! { "host" => "host-2"}),
            2.0,
        ));
        target.record(counter_event_custom_timestamp(
            "a",
            Some(btreemap! { "host" => "host-1"}),
            4.0,
            3,
        ));
        assert_eq!(
            target.data.len(),
            2,
            "number of hashmap records didn't match"
        );

        // Check metrics tagged {host: host-1}
        let host_1_key = metric_event_key("a", Some(btreemap! { "host" => "host-1"}));
        let actual_events = target.data.get(&host_1_key).unwrap();
        assert_windows_eq(
            vec![
                // timestamp of event 1 is updated after merge with event 2
                counter_event_custom_timestamp("a", Some(btreemap! { "host" => "host-1"}), 7.0, 3),
                counter_event_custom_timestamp("a", Some(btreemap! { "host" => "host-1"}), 4.0, 3),
            ],
            actual_events,
        );

        // Check metrics tagged {host: host-2}
        let host_1_key = metric_event_key("a", Some(btreemap! { "host" => "host-2"}));
        let actual_events = target.data.get(&host_1_key).unwrap();
        assert_windows_eq(
            vec![counter_event(
                "a",
                Some(btreemap! { "host" => "host-2"}),
                2.0,
            )],
            actual_events,
        );
    }

    #[tokio::test]
    async fn record_drops_events_when_cardinality_is_exceeded() {
        let mut target = new_aggregator(None, AggregatorLimits::new(200, 2, 5000, 5)).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 5.0));
        target.record(counter_event("c", None, 6.0));
        assert_eq!(
            target.data.len(),
            2,
            "number of hashmap records didn't match"
        );

        let key = metric_event_key("c", None);
        let actual = target.data.get(&key);
        assert!(
            actual.is_none(),
            "keys were added after exceeding cardinality limit"
        );
    }

    #[tokio::test]
    async fn record_skips_creating_window() {
        let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10)).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 7.0));
        // event overlaps existing window but does not allocate new window
        target.record(counter_event_custom_timestamp("a", None, 6.0, 4));
        assert_eq!(
            target.data.len(),
            2,
            "number of hashmap records didn't match"
        );

        let key = metric_event_key("a", None);
        let actual = target.data.get(&key).unwrap();
        assert_eq!(actual.len(), 1, "number of sliding windows didn't match");
        // timestamp is updated after merge of event metadata
        assert_windows_eq(
            vec![counter_event_custom_timestamp("a", None, 9.0, 4)],
            actual,
        );
    }

    #[tokio::test]
    async fn record_creates_new_windows_when_event_exceeds_min_window() {
        let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10)).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 7.0));
        // use explicit timestamp instead of shared atomic value
        target.record(counter_event_custom_timestamp("a", None, 6.0, 15));
        assert_eq!(
            target.data.len(),
            2,
            "number of hashmap records didn't match"
        );

        let key = metric_event_key("a", None);
        let actual = target.data.get(&key).unwrap();
        assert_eq!(actual.len(), 2, "number of sliding windows didn't match");
        assert_windows_eq(
            vec![
                counter_event("a", None, 3.0),
                counter_event_custom_timestamp("a", None, 6.0, 15),
            ],
            actual,
        );
    }

    #[tokio::test]
    async fn flush_when_empty() {
        let mut target = new_aggregator(None, default_aggregator_limits()).await;
        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn flush_no_expired() {
        let mut target = new_aggregator(None, default_aggregator_limits()).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 3.0));

        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn flush_only_expired() {
        let mut target = new_aggregator(None, default_aggregator_limits()).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 3.0));
        target.clock.increment_by(10);
        target.record(counter_event("a", None, 3.0));

        let mut actual_events = vec![];
        target.flush_finalized(&mut actual_events);
        fix_event_ordering(&mut actual_events);
        assert_events_eq(
            vec![counter_event("a", None, 3.0), counter_event("b", None, 3.0)],
            actual_events,
        );
    }

    #[tokio::test]
    async fn flush_on_conditional_value() {
        let mut target = new_aggregator(
            Some("to_string!(.message.value.value) > \"5\""),
            default_aggregator_limits(),
        )
        .await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 3.0));
        target.record(counter_event("a", None, 3.0));

        let mut actual_events = vec![];
        target.flush_finalized(&mut actual_events);
        fix_event_ordering(&mut actual_events);
        assert_events_eq(vec![counter_event("a", None, 6.0)], actual_events);
    }

    #[tokio::test]
    async fn flush_on_conditional_tag() {
        let mut target = new_aggregator(
            Some(".message.tags.region == \"foo\""),
            default_aggregator_limits(),
        )
        .await;
        target.record(counter_event(
            "a",
            Some(btreemap! { "region" => "foo"}),
            2.0,
        ));
        target.record(counter_event(
            "a",
            Some(btreemap! { "region" => "bar"}),
            4.0,
        ));
        target.record(counter_event(
            "b",
            Some(btreemap! { "region" => "foo"}),
            6.0,
        ));
        target.record(counter_event("b", None, 8.0));
        assert_eq!(target.data.len(), 4);

        let mut actual_events = vec![];
        target.flush_finalized(&mut actual_events);
        fix_event_ordering(&mut actual_events);
        assert_events_eq(
            vec![
                counter_event("a", Some(btreemap! { "region" => "foo"}), 2.0),
                counter_event("b", Some(btreemap! { "region" => "foo"}), 6.0),
            ],
            actual_events,
        );
    }

    #[tokio::test]
    async fn flushes_excess_windows_to_stay_within_window_limits() {
        let mut target = new_aggregator(None, AggregatorLimits::new(2, 5000, 0, 5)).await;
        target.record(counter_event("a", None, 3.0));
        target.record(counter_event("b", None, 3.0));
        // use explicit timestamps to force new window allocations
        target.record(counter_event_custom_timestamp("a", None, 4.0, 12));
        target.record(counter_event_custom_timestamp("a", None, 5.0, 13));
        target.record(counter_event_custom_timestamp("a", None, 6.0, 14));
        target.record(counter_event_custom_timestamp("a", None, 7.0, 15));
        // Increment clock such that explicit timestamp counters are not expired
        target.clock.increment_by(14);

        let mut actual_events = vec![];
        target.flush_finalized(&mut actual_events);
        fix_event_ordering(&mut actual_events);
        assert_events_eq(
            vec![
                counter_event("a", None, 3.0),
                counter_event_custom_timestamp("a", None, 22.0, 12),
                counter_event_custom_timestamp("a", None, 18.0, 13),
                counter_event("b", None, 3.0),
            ],
            actual_events,
        );
    }
}
