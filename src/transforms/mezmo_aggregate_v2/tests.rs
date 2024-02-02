use crate::config::TransformContext;
use crate::mezmo::MezmoContext;
use crate::transforms::mezmo_aggregate_v2::config::{AggregatorLimits, MezmoAggregateV2Config};
use crate::transforms::mezmo_aggregate_v2::*;
use assay::assay;
use serde_json::json;
use std::collections::BTreeMap;
use tempfile::tempdir;
use vector_lib::event::LogEvent;
use vrl::btreemap;

const ACCOUNT_ID: &str = "bea71e55-a1ec-4e5f-a5c0-c0e10b1a571c";

fn test_mezmo_context() -> MezmoContext {
    MezmoContext::try_from(format!(
        "v1:aggregate-v2:transform:component_id:pipeline_id:{}",
        ACCOUNT_ID
    ))
    .unwrap()
}

async fn new_aggregator(
    flush_condition: Option<&str>,
    memory_limits: AggregatorLimits,
    state_persistence_base_path: Option<&str>,
) -> MezmoAggregateV2 {
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

    let state_persistence_base_path = match state_persistence_base_path {
        None => "".to_string(),
        Some(base_path) => format!("state_persistence_base_path = \"{base_path}\""),
    };

    let config = format!(
        r#"
            window_duration_ms = {window_duration_ms}
            mem_cardinality_limit = {mem_cardinality_limit}
            mem_window_limit = {mem_window_limit}
            min_window_size_ms = {min_window_size_ms}
            flush_tick_ms = 1
            event_id_fields = [".message.name", ".message.tags"]
            event_timestamp_field = ".metadata.timestamp"
            {state_persistence_base_path}
            state_persistence_tick_ms = 1
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

    let config: MezmoAggregateV2Config = toml::from_str(config.as_str()).unwrap();
    let mezmo_ctx = test_mezmo_context();

    let ctx = TransformContext {
        mezmo_ctx: Some(mezmo_ctx),
        ..Default::default()
    };
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
    crate::test_util::test_generate_config::<MezmoAggregateV2Config>();
}

#[tokio::test]
async fn record_single_metric() {
    let mut target = new_aggregator(None, default_aggregator_limits(), None).await;
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
    let mut target = new_aggregator(None, aggregator_limits_custom_window_size(0), None).await;
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
    let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10), None).await;
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
    let mut target = new_aggregator(None, aggregator_limits_custom_window_size(0), None).await;
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
    let mut target = new_aggregator(None, AggregatorLimits::new(200, 2, 5000, 5), None).await;
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
    let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10), None).await;
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
    let mut target = new_aggregator(None, aggregator_limits_custom_window_size(10), None).await;
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
    let mut target = new_aggregator(None, default_aggregator_limits(), None).await;
    let mut res = vec![];
    target.flush_finalized(&mut res);
    assert!(res.is_empty());
}

#[tokio::test]
async fn flush_no_expired() {
    let mut target = new_aggregator(None, default_aggregator_limits(), None).await;
    target.record(counter_event("a", None, 3.0));
    target.record(counter_event("b", None, 3.0));

    let mut res = vec![];
    target.flush_finalized(&mut res);
    assert!(res.is_empty());
}

#[tokio::test]
async fn flush_only_expired() {
    let mut target = new_aggregator(None, default_aggregator_limits(), None).await;
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
        None,
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
        None,
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
async fn flush_using_prev_value() {
    let key = metric_event_key("a", None);
    let mut target = new_aggregator(
        Some(
            r#"
            res, err = (.message.value.value / %previous.message.value.value) >= 1.5
            if err != null { false } else { res }
        "#,
        ),
        default_aggregator_limits(),
        None,
    )
    .await;
    target.record(counter_event("a", None, 1.0));
    target.clock.increment_by(1);
    target.record(counter_event("a", None, 1.0));

    // Assert that the internal state of the aggregate windows match what we expect for
    // two events that have not exceeded the trigger condition nor has the window elapsed.
    let data_windows = target
        .data
        .get(&key)
        .expect("metric key should have an allocated vecdeque");
    assert_eq!(1, data_windows.len());
    assert!(!data_windows.front().unwrap().flushed);

    // Neither of these data points should trigger the flush condition nor is the window
    // expired therefore nothing should be flushed. (clock = 2)
    let mut actual_events = vec![];
    target.flush_finalized(&mut actual_events);
    assert_eq!(actual_events.len(), 0);
    let data_windows = target
        .data
        .get(&key)
        .expect("metric key should have an allocated vecdeque");
    assert_eq!(1, data_windows.len());
    assert!(!data_windows.front().unwrap().flushed);

    // Now increment the clock past the window expiration. This should then flush the
    // current window. (clock = 6)
    target.clock.increment_by(4);
    target.flush_finalized(&mut actual_events);
    assert_eq!(actual_events.len(), 1);
    assert_eq!(
        *actual_events
            .get(0)
            .unwrap()
            .as_log()
            .get(".message.value.value")
            .unwrap(),
        Value::from(2)
    );
    let data_windows = target
        .data
        .get(&key)
        .expect("metric key should have an allocated vecdeque");
    assert_eq!(1, data_windows.len());
    assert!(data_windows.front().unwrap().flushed);

    actual_events.clear();

    // Now record a large event that should trigger the flush condition. This should
    // flush even without the window expiring. (clock = 6)
    target.record(counter_event("a", None, 100.0));
    target.flush_finalized(&mut actual_events);
    assert_eq!(actual_events.len(), 1);
    let event = actual_events
        .get(0)
        .unwrap()
        .as_log()
        .get(".message.value.value")
        .unwrap();
    assert_eq!(*event, Value::from(100));

    let data_windows = target
        .data
        .get(&key)
        .expect("metric key should have an allocated vecdeque");
    assert_eq!(1, data_windows.len());
    assert!(data_windows.front().unwrap().flushed);
}

#[tokio::test]
async fn flushes_excess_windows_to_stay_within_window_limits() {
    let mut target = new_aggregator(None, AggregatorLimits::new(2, 5000, 0, 5), None).await;
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

#[tokio::test]
async fn window_alloc_limit_over_time() {
    let mut target = new_aggregator(None, AggregatorLimits::new(10, 10, 3, 5), None).await;
    for _ in 0..6 {
        target.record(counter_event("a", None, 1.0));
        target.clock.increment_by(2);
    }

    assert_eq!(target.data.len(), 1);
    let mut actual = vec![];
    let key = metric_event_key("a", None);
    for window in target
        .data
        .get(&key)
        .expect("event key should exist in map")
    {
        actual.push((window.size_ms.clone(), window.event.clone()));
    }

    /*
       Based on the settings, every window should be 5 clock ticks long with a new window allocated 3 ticks after the start
       of the current (last) window. Since the test simulates events arriving every 2 ticks, the events should:

       1     2     3     4     5     6     7     8     9     10    11    12    13    14
       +-----+-----+-----+-----+-----+
       | 1.0 |     | 1.0 |     | 1.0 |
       +-----+-----+-----+-----+-----+-----+-----+-----+-----+
                               | 1.0 |     | 1.0 |     | 1.0 |
                               +-----+-----+-----+-----+-----+-----+-----+-----+-----+
                                                       | 1.0 |     | 1.0 |     | --- |
                                                       +-----+-----+-----+-----+-----+
    */
    assert_eq!(
        vec![
            (1..6, counter_event("a", None, 3.0)),
            (5..10, counter_event("a", None, 3.0)),
            (9..14, counter_event("a", None, 2.0)),
        ],
        actual
    );
}

#[assay(env = [("POD_NAME", "vector-test0-0")])]
async fn with_initial_state() {
    let tmp_path = tempdir().expect("Could not create temp dir").into_path();
    let state_persistence_base_path = tmp_path.to_str();
    let limits = AggregatorLimits::new(1, 5000, 0, 5);

    let mut target = new_aggregator(None, limits.clone(), state_persistence_base_path).await;
    target.record(counter_event("a", None, 3.0));
    target.record(counter_event("b", None, 3.0));

    let mut res = vec![];
    let initial_data = target.data.clone();
    target.flush_finalized(&mut res); // no-op, window has not elapsed
    target.persist_state();
    assert!(res.is_empty());

    let mut new_target = new_aggregator(None, limits, state_persistence_base_path).await;
    assert_eq!(
        new_target.data.len(),
        initial_data.len(),
        "initial data state does not match"
    );

    let mut new_res = vec![];
    new_target.record(counter_event("a", None, 3.0));
    new_target.record(counter_event("b", None, 3.0));
    new_target.clock.increment_by(10);
    new_target.flush_finalized(&mut new_res);
    assert!(!new_res.is_empty());

    fix_event_ordering(&mut new_res);
    assert_events_eq(
        vec![counter_event("a", None, 6.0), counter_event("b", None, 6.0)],
        new_res,
    );
}
