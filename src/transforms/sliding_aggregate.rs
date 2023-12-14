use crate::{
    conditions::{AnyCondition, Condition},
    config::{TransformConfig, TransformContext},
};
use async_stream::stream;
use chrono::Utc;
use enrichment::TableRegistry;
use futures::{Stream, StreamExt};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::ops::Range;
use std::pin::Pin;
use std::time::Duration;
use tokio::select;
use vector_config_macros::configurable_component;
use vector_core::{
    config::{DataType, Input, LogNamespace, OutputId, TransformOutput},
    event::{metric::MetricSeries, Event},
    schema::Definition,
    transform::{TaskTransform, Transform},
};

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

    // LOG-18567: Two additional properties that were considered as part of the spike but
    // pushed out of the spike effort. They include:
    //   * mem_cardinality_limit - the maximum number of keys to maintain in the transform's BTreeMap.
    //   * mem_window_limit - the maxmium number of sliding window structs to allow per metric series.
    /// Define a VRL condition that when it evaluates to true on the aggregated event will
    /// also cause the aggregated event to be flushed to the output stream. This may produce
    /// values outside of their sliding window because of a true condition.
    flush_condition: Option<AnyCondition>,
}

const fn default_window_duration_ms() -> u32 {
    2 * 1000
}

const fn default_flush_tick_ms() -> u64 {
    5
}

impl_generate_config_from_default!(SlidingAggregateConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "sliding_aggregate")]
impl TransformConfig for SlidingAggregateConfig {
    async fn build(&self, ctx: &TransformContext) -> crate::Result<Transform> {
        let flush_condition = self
            .flush_condition
            .as_ref()
            .map(|cond| cond.build(&ctx.enrichment_tables, ctx.mezmo_ctx.clone()))
            .transpose()?;
        let window_size_ms = self.window_duration_ms as i64; // ok cast since u32::MAX < i64::MAX
        Ok(Transform::event_task(SlidingAggregate::new(
            window_size_ms,
            self.flush_tick_ms,
            flush_condition,
        )))
    }

    fn input(&self) -> Input {
        Input::metric()
    }

    fn outputs(
        &self,
        _enrichment_tables: TableRegistry,
        _input_definitions: &[(OutputId, Definition)],
        _global_log_namespace: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Metric, HashMap::new())]
    }
}

#[derive(Debug)]
struct AggregateWindow {
    size_ms: Range<i64>,
    event: Event,
}

impl AggregateWindow {
    fn new(event: Event, window_size: i64) -> Self {
        let window_start = match event.as_metric().data().timestamp() {
            Some(timestamp) => timestamp.timestamp_millis(),
            None => Utc::now().timestamp_millis(),
        };
        let max_window = window_start..window_start + window_size;
        Self {
            size_ms: max_window,
            event,
        }
    }

    const fn from_parts(size_ms: Range<i64>, event: Event) -> Self {
        Self { size_ms, event }
    }

    fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp_millis();
        !self.contains_timestamp(now)
    }

    fn contains_timestamp(&self, value: i64) -> bool {
        self.size_ms.contains(&value)
    }

    fn update_value(&mut self, other: &Event) {
        let metric = self.event.as_mut_metric();
        if metric.data_mut().update(other.as_metric().data()) {
            metric
                .metadata_mut()
                .merge(other.as_metric().metadata().clone());
        }
    }
}

#[derive(Debug)]
pub struct SlidingAggregate {
    data: BTreeMap<MetricSeries, VecDeque<AggregateWindow>>,
    window_size_ms: i64,
    flush_tick_ms: u64,
    flush_condition: Option<Condition>,
}

impl SlidingAggregate {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        window_size_ms: i64,
        flush_tick_ms: u64,
        flush_condition: Option<Condition>,
    ) -> Self {
        let data = BTreeMap::new();
        Self {
            data,
            window_size_ms,
            flush_tick_ms,
            flush_condition,
        }
    }

    fn record(&mut self, event: Event) {
        match self.data.get_mut(event.as_metric().series()) {
            None => {
                let key = event.as_metric().series().clone();
                let mut windows = VecDeque::new();
                windows.push_back(AggregateWindow::new(event, self.window_size_ms));
                self.data.insert(key, windows); // LOG-18567: should enforce max number of keys in BTreeMap
            }
            Some(entry) => {
                let metric = event.as_metric();
                // If the event has a timestamp, use that value. If not, use the current
                // UTC timestamp when fitting it into windows.
                let event_timestamp = match metric.data().time.timestamp {
                    Some(timestamp) => timestamp.timestamp_millis(),
                    None => Utc::now().timestamp_millis(),
                };

                // Start looking from the back for windows where this event should be rolled
                // up into. Since the windows are stored in order, the loop can stop at the first
                // window that would not be rolled up into.
                for i in (0..entry.len()).rev() {
                    let window = entry.get_mut(i).unwrap();
                    if !window.contains_timestamp(event_timestamp) {
                        break;
                    }
                    window.update_value(&event);
                }

                // Every new event starts a new window that further points can be rolled up into, aka
                // the windows slide to the next event.
                let new_window = AggregateWindow::new(event, self.window_size_ms);
                entry.push_back(new_window); // LOG-18567: should enforce max number of windows in VecDeque
            }
        }
    }

    /// Check windows starting from the front (oldest) for expired entries. All expired
    /// entries will be drained from the active collection and then produced as values
    /// to emit as output.
    fn flush_finalized(&mut self, output: &mut Vec<Event>) {
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
                let is_expired = window.is_expired();

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

#[cfg(test)]
mod test {
    use super::*;
    use vector_core::{
        event::{
            metric::{MetricName, MetricTags},
            Metric, MetricKind, MetricValue,
        },
        metric_tags,
    };

    fn metric(
        name: impl Into<String>,
        kind: MetricKind,
        tags: Option<MetricTags>,
        value: f64,
    ) -> Event {
        let value = match kind {
            MetricKind::Absolute => MetricValue::Gauge { value },
            MetricKind::Incremental => MetricValue::Counter { value },
        };
        let metric = Metric::new(name, kind, value).with_tags(tags);
        Event::from(metric)
    }

    fn metric_series(name: impl Into<String>, tags: Option<MetricTags>) -> MetricSeries {
        MetricSeries {
            name: MetricName {
                name: name.into(),
                namespace: None,
            },
            tags,
        }
    }

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<SlidingAggregateConfig>();
    }

    #[test]
    fn record_single_metric() {
        let mut target = SlidingAggregate::new(5, 1, None);
        target.record(metric("a", MetricKind::Incremental, None, 10.0));
        assert_eq!(target.data.len(), 1);

        let key = metric_series("a", None);
        let val = target.data.get(&key).unwrap();
        assert!(val.capacity() >= 2);

        let AggregateWindow { size_ms, event, .. } = val.get(0).unwrap();
        assert_eq!(5, size_ms.end - size_ms.start);
        assert_eq!(event.as_metric().data().kind, MetricKind::Incremental);
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 10.0)
        );
    }

    #[tokio::test]
    async fn record_overlapping_windows() {
        let mut target = SlidingAggregate::new(5, 1, None);
        target.record(metric("a", MetricKind::Incremental, None, 3.0));
        tokio::time::sleep(Duration::from_millis(2)).await;
        target.record(metric("a", MetricKind::Incremental, None, 4.0));
        assert_eq!(
            target.data.len(),
            1,
            "number of hashmap records didn't match"
        );

        let key = metric_series("a", None);
        let val = target.data.get(&key).unwrap();
        assert_eq!(val.len(), 2, "number of sliding windows didn't match");

        let AggregateWindow { size_ms, event, .. } = val.get(0).unwrap();
        assert_eq!(size_ms.end - size_ms.start, 5);
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 7.0 )
        );

        let AggregateWindow { size_ms, event, .. } = val.get(1).unwrap();
        assert_eq!(size_ms.end - size_ms.start, 5);
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 4.0 )
        );
    }

    #[tokio::test]
    async fn record_nonoverlapping_windows() {
        let mut target = SlidingAggregate::new(5, 1, None);
        target.record(metric("a", MetricKind::Incremental, None, 3.0));
        tokio::time::sleep(Duration::from_millis(15)).await;
        target.record(metric("a", MetricKind::Incremental, None, 4.0));
        assert_eq!(
            target.data.len(),
            1,
            "number of hashmap records didn't match"
        );

        let key = metric_series("a", None);
        let val = target.data.get(&key).unwrap();
        assert_eq!(val.len(), 2, "number of sliding windows didn't match");

        let AggregateWindow { size_ms, event, .. } = val.get(0).unwrap();
        assert_eq!(size_ms.end - size_ms.start, 5);
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 3.0 )
        );

        let AggregateWindow { size_ms, event, .. } = val.get(1).unwrap();
        assert_eq!(size_ms.end - size_ms.start, 5);
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 4.0 )
        );
    }

    #[test]
    fn record_group_by_tags() {
        let mut target = SlidingAggregate::new(5, 1, None);
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("host" => "host-1")),
            3.0,
        ));
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("host" => "host-2")),
            2.0,
        ));
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("host" => "host-1")),
            4.0,
        ));
        assert_eq!(
            target.data.len(),
            2,
            "number of hashmap records didn't match"
        );

        // Check metrics tagged {host: host-1}
        let key = metric_series("a", Some(metric_tags!("host" => "host-1")));
        let val = target.data.get(&key).unwrap();
        assert_eq!(val.len(), 2, "number of sliding windows didn't match");
        let AggregateWindow { event, .. } = val.get(0).unwrap();
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 7.0 )
        );
        let AggregateWindow { event, .. } = val.get(1).unwrap();
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 4.0 )
        );

        // Check metrics tagged {host: host-2}
        let key = metric_series("a", Some(metric_tags!("host" => "host-2")));
        let val = target.data.get(&key).unwrap();
        assert_eq!(val.len(), 1, "number of sliding windows didn't match");
        let AggregateWindow { event, .. } = val.get(0).unwrap();
        assert!(
            matches!(event.as_metric().data().value, MetricValue::Counter { value } if value == 2.0 )
        );
    }

    #[test]
    fn flush_when_empty() {
        let mut target = SlidingAggregate::new(5, 1, None);
        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn flush_no_expired() {
        // LOG-18845: Use a very large aggregation window in the test so that even when executing on a busy
        // Jenkins node, nothing should ever become expired.
        let mut target = SlidingAggregate::new(60_000, 1, None);
        target.record(metric("a", MetricKind::Incremental, None, 3.0));
        target.record(metric("b", MetricKind::Incremental, None, 3.0));

        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn flush_only_expired() {
        let mut target = SlidingAggregate::new(5, 1, None);
        target.record(metric("a", MetricKind::Incremental, None, 3.0));
        target.record(metric("b", MetricKind::Absolute, None, 3.0));
        tokio::time::sleep(Duration::from_millis(10)).await;
        target.record(metric("a", MetricKind::Incremental, None, 3.0));

        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert_eq!(res.len(), 2);

        let data = res.get(0).unwrap().as_metric().data();
        assert_eq!(data.kind, MetricKind::Incremental);
        assert!(matches!(data.value, MetricValue::Counter { value } if value == 3.0));

        assert_eq!(target.data.len(), 1);
        let key = metric_series("a", None);
        assert_eq!(target.data.get(&key).unwrap().len(), 1);
    }

    /*
     * LOG-18567: This test case sets up an aggregator to flush when the counter value exceeds 5.0.
     * Due to limitations with VRL access to fields in `Metric` instances, the test fails because
     * the flush condition never evaluates to true. The fix for this test case was punted from the
     * spike because there isn't a quick, clear fix.
     *
     * Related: https://github.com/vectordotdev/vector/issues/5521
     */
    #[ignore]
    #[test]
    fn flush_on_conditional_value() {
        let more_than_five = AnyCondition::String("to_string!(.counter.value) > \"5\"".to_string())
            .build(&Default::default(), Default::default())
            .map_err(|err| panic!("{err}"))
            .ok();

        let mut target = SlidingAggregate::new(5, 1, more_than_five);
        target.record(metric("a", MetricKind::Incremental, None, 3.0));
        target.record(metric("b", MetricKind::Incremental, None, 3.0));
        target.record(metric("a", MetricKind::Incremental, None, 3.0));

        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert_eq!(res.len(), 1);

        let data = res.get(0).unwrap().as_metric().data();
        assert_eq!(data.kind, MetricKind::Incremental);
        assert!(matches!(data.value, MetricValue::Counter { value } if value == 6.0));
    }

    #[test]
    fn flush_on_conditional_tag() {
        let table_registry = TableRegistry::default();
        let tag_condition = AnyCondition::String(".tags.region == \"foo\"".to_string())
            .build(&table_registry, None)
            .ok();

        let mut target = SlidingAggregate::new(5, 1, tag_condition);
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("region" => "foo")),
            2.0,
        ));
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("region" => "bar")),
            4.0,
        ));
        target.record(metric(
            "a",
            MetricKind::Incremental,
            Some(metric_tags!("region" => "foo")),
            3.0,
        ));
        target.record(metric(
            "b",
            MetricKind::Incremental,
            Some(metric_tags!("region" => "foo")),
            6.0,
        ));
        target.record(metric("b", MetricKind::Incremental, None, 8.0));
        assert_eq!(target.data.len(), 4);

        let mut res = vec![];
        target.flush_finalized(&mut res);
        assert_eq!(res.len(), 3);
    }
}
