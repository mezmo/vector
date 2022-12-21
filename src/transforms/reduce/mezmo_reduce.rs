// This module mimics the `reduce` vector transform, but it operates against the .message
// property of the log event instead of the root-level properies (Vector's implementation).
// This implentation also (de)serializes date fields that are specified by the user, making sure
// to return date fields in the same format as originally received. For example, an epoch field
// can be an integer or a string, and it will match the output type based on the incoming data.

use async_stream::stream;
use chrono::{TimeZone, Utc};
use futures::{stream, Stream, StreamExt};
use indexmap::IndexMap;
use std::collections::BTreeMap;
use std::{
    collections::{hash_map, HashMap},
    pin::Pin,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use vector_config::configurable_component;

use super::merge_strategy::*;
use crate::{
    conditions::{AnyCondition, Condition},
    config::{DataType, Input, Output, TransformConfig, TransformContext, TransformDescription},
    event::{discriminant::Discriminant, Event, EventMetadata, LogEvent, Value},
    internal_events::ReduceStaleEventFlushed,
    schema,
    transforms::{TaskTransform, Transform},
};

/// Configuration for the `mezmo-reduce` transform.
#[configurable_component(transform)]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields, default)]
pub struct MezmoReduceConfig {
    /// The maximum period of time to wait after the last event is received, in milliseconds, before a combined event should be considered complete.
    pub expire_after_ms: Option<u64>,

    /// The interval to check for and flush any expired events, in milliseconds.
    pub flush_period_ms: Option<u64>,

    /// An ordered list of fields by which to group events.
    ///
    /// Each group with matching values for the specified keys is reduced independently, allowing you to keep
    /// independent event streams separate. When no fields are specified, all events will be combined in a single group.
    ///
    /// For example, if `group_by = ["host", "region"]`, then all incoming events that have the same host and region
    /// will be grouped together before being reduced.
    #[serde(default)]
    pub group_by: Vec<String>,

    /// A map of field names to custom merge strategies.
    ///
    /// For each field specified, the given strategy will be used for combining events rather than the default behavior.
    ///
    /// The default behavior is as follows:
    ///
    /// - The first value of a string field is kept, subsequent values are discarded.
    /// - For timestamp fields the first is kept and a new field `[field-name]_end` is added with the last received timestamp value.
    /// - Numeric values are summed.
    #[serde(default)]
    pub merge_strategies: IndexMap<String, MergeStrategy>,

    /// A condition used to distinguish the final event of a transaction.
    ///
    /// If this condition resolves to `true` for an event, the current transaction is immediately flushed with this event.
    pub ends_when: Option<AnyCondition>,

    /// A condition used to distinguish the first event of a transaction.
    ///
    /// If this condition resolves to `true` for an event, the previous transaction is flushed (without this event) and a new transaction is started.
    pub starts_when: Option<AnyCondition>,

    /// Mezmo-specific. Since dates can be serialized, users can specify which properties should be dates, and what format can
    /// be used to parse them. This eventually will translate Value::Bytes into a Value::Timestamp
    #[serde(default)]
    pub date_formats: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct MezmoMetadata {
    date_formats: Arc<HashMap<String, String>>,

    /// Mezmo-specific. This will track the Kind of Value that reduce should send back when the reduce is complete. For example,
    /// an epoch time may come in as an integer, and thus should go out as an integer (and not a Timestamp).
    /// This structure is keyed by the Property location and the value is the kind type (either string or integer in our case).
    date_kinds: Arc<RwLock<HashMap<String, String>>>,
}

impl MezmoMetadata {
    fn new(date_formats: HashMap<String, String>) -> Self {
        Self {
            date_formats: Arc::new(date_formats),
            date_kinds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_date_kind(&self, date_prop: &str) -> String {
        let map = self.date_kinds.read().unwrap();
        map.get(date_prop)
            .expect("date_kinds map should contain the requested date_prop")
            .clone()
    }

    fn save_date_kind(&self, date_prop: &str, kind: &str) {
        {
            let map = self.date_kinds.read().unwrap();
            if map.get(date_prop).is_some() {
                return; // no need to do anything else
            }
        } // Drops read lock
        let mut map = self
            .date_kinds
            .write()
            .expect("Cannot get mutable reference RwLock for date_kinds");

        map.insert(date_prop.to_owned(), kind.to_owned());
    }
}

inventory::submit! {
    TransformDescription::new::<MezmoReduceConfig>("mezmo-reduce")
}

impl_generate_config_from_default!(MezmoReduceConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo-reduce")]
impl TransformConfig for MezmoReduceConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        MezmoReduce::new(self, &context.enrichment_tables).map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(&self, _: &schema::Definition) -> Vec<Output> {
        vec![Output::default(DataType::Log)]
    }

    fn transform_type(&self) -> &'static str {
        "mezmo-reduce"
    }
}

#[derive(Debug)]
struct ReduceState {
    fields: HashMap<String, Box<dyn ReduceValueMerger>>,
    stale_since: Instant,
    metadata: EventMetadata,
    mezmo_metadata: MezmoMetadata,
}

impl ReduceState {
    fn new(
        e: LogEvent,
        strategies: &IndexMap<String, MergeStrategy>,
        mezmo_metadata: MezmoMetadata,
    ) -> Self {
        let (value, metadata) = e.into_parts();

        let fields = if let Value::Object(fields) = value {
            // For Mezmo: set `fields` equal to the contents of `.message`
            let mut fields: BTreeMap<String, Value> = fields.into();
            if let Some(Value::Object(message_object)) = fields.get("message") {
                fields = message_object.clone();
            }
            fields
                .into_iter()
                .filter_map(|(k, v)| {
                    if let Some(strat) = strategies.get(&k) {
                        match get_value_merger(v, strat) {
                            Ok(m) => Some((k, m)),
                            Err(error) => {
                                warn!(message = "Failed to create merger.", field = ?k, %error);
                                None
                            }
                        }
                    } else {
                        Some((k, v.into()))
                    }
                })
                .collect()
        } else {
            HashMap::new()
        };

        Self {
            stale_since: Instant::now(),
            fields,
            metadata,
            mezmo_metadata,
        }
    }

    fn add_event(&mut self, e: LogEvent, strategies: &IndexMap<String, MergeStrategy>) {
        let (value, metadata) = e.into_parts();
        self.metadata.merge(metadata);

        let fields = if let Value::Object(fields) = value {
            // For Mezmo: set `fields` equal to the contents of `.message`
            let mut fields: BTreeMap<String, Value> = fields.into();
            if let Some(Value::Object(message_object)) = fields.get("message") {
                fields = message_object.clone();
            }
            fields
        } else {
            BTreeMap::new()
        };

        for (k, v) in fields.into_iter() {
            let strategy = strategies.get(&k);
            match self.fields.entry(k) {
                hash_map::Entry::Vacant(entry) => {
                    if let Some(strat) = strategy {
                        match get_value_merger(v, strat) {
                            Ok(m) => {
                                entry.insert(m);
                            }
                            Err(error) => {
                                warn!(message = "Failed to merge value.", %error);
                            }
                        }
                    } else {
                        entry.insert(v.clone().into());
                    }
                }
                hash_map::Entry::Occupied(mut entry) => {
                    if let Err(error) = entry.get_mut().add(v.clone()) {
                        warn!(message = "Failed to merge value.", %error);
                    }
                }
            }
        }
        self.stale_since = Instant::now();
    }

    // Mezmo-specific method. Take the timestamp fields (and their _end counterparts) and
    // create a Value() that matches the incoming data type for the field, e.g. a String.
    fn coerce_from_timestamp_if_needed(&self, log_event: &mut LogEvent) {
        let date_formats = &self.mezmo_metadata.date_formats;
        if date_formats.len() == 0 {
            debug!(message = "There are no custom date formats to coerce");
            return ();
        }

        let message_obj = log_event.get_mut("message").unwrap();

        for (date_prop, format) in date_formats.iter() {
            let end_prop = format!("{}_end", date_prop);
            let start_str = date_prop.as_str();
            let end_str = end_prop.as_str();

            if let Some(Value::Timestamp(start_date)) = message_obj.get(start_str) {
                if let Some(Value::Timestamp(end_date)) = message_obj.get(end_str) {
                    let start_date_string = start_date.format(format).to_string();
                    let end_date_string = end_date.format(format).to_string();

                    let date_kind = self.mezmo_metadata.get_date_kind(start_str);

                    let (coerced_start_value, coerced_end_value) = match date_kind.as_str() {
                        "string" => {
                            debug!(
                                message = "Coercing date back into string",
                                start_date_string, end_date_string
                            );
                            (Value::from(start_date_string), Value::from(end_date_string))
                        }
                        "integer" => {
                            debug!(
                                message = "Coercing date back to integer",
                                start_date_string, end_date_string
                            );
                            let start_val = start_date_string
                            .parse::<i64>()
                            .and_then(|date_int| Ok(Value::from(date_int)))
                            .unwrap_or_else(|error| {
                                warn!(message = "Could not coerce start date back into an integer Value", date_prop, %error);
                                Value::from(start_date_string)
                            });
                            let end_val = end_date_string
                            .parse::<i64>()
                            .and_then(|date_int| Ok(Value::from(date_int)))
                            .unwrap_or_else(|error| {
                                warn!(message = "Could not coerce end date back into an integer Value", end_prop, %error);
                                Value::from(end_date_string)
                            });

                            (start_val, end_val)
                        }
                        _ => {
                            warn!(
                                message = "mezmo_meta did not contain prop kind for date property",
                                date_prop
                            );
                            continue;
                        }
                    };
                    message_obj.insert(start_str, coerced_start_value);
                    message_obj.insert(end_str, coerced_end_value);
                }
            }
        }
        ()
    }

    fn flush(mut self) -> LogEvent {
        let mut event = LogEvent::new_with_metadata(self.metadata.clone());

        for (k, v) in self.fields.drain() {
            // When the resulting event is created from the mezmo-reduce accumulator,
            // we need to inject its results into the `.message` property
            let key = format!("message.{}", k);
            if let Err(error) = v.insert_into(key, &mut event) {
                warn!(message = "Failed to merge values for field.", %error);
            }
        }
        self.coerce_from_timestamp_if_needed(&mut event);
        event
    }
}

pub struct MezmoReduce {
    expire_after: Duration,
    flush_period: Duration,
    group_by: Vec<String>,
    merge_strategies: IndexMap<String, MergeStrategy>,
    reduce_merge_states: HashMap<Discriminant, ReduceState>,
    ends_when: Option<Condition>,
    starts_when: Option<Condition>,
    mezmo_metadata: MezmoMetadata,
}

impl MezmoReduce {
    pub fn new(
        config: &MezmoReduceConfig,
        enrichment_tables: &enrichment::TableRegistry,
    ) -> crate::Result<Self> {
        if config.ends_when.is_some() && config.starts_when.is_some() {
            return Err("only one of `ends_when` and `starts_when` can be provided".into());
        }

        let ends_when = config
            .ends_when
            .as_ref()
            .map(|c| c.build(enrichment_tables))
            .transpose()?;
        let starts_when = config
            .starts_when
            .as_ref()
            .map(|c| c.build(enrichment_tables))
            .transpose()?;
        let group_by = config.group_by.clone().into_iter().collect();

        Ok(MezmoReduce {
            expire_after: Duration::from_millis(config.expire_after_ms.unwrap_or(30000)),
            flush_period: Duration::from_millis(config.flush_period_ms.unwrap_or(1000)),
            group_by,
            merge_strategies: config.merge_strategies.clone(),
            reduce_merge_states: HashMap::new(),
            ends_when,
            starts_when,
            mezmo_metadata: MezmoMetadata::new(config.date_formats.clone()),
        })
    }

    fn flush_into(&mut self, output: &mut Vec<Event>) {
        let mut flush_discriminants = Vec::new();
        for (k, t) in &self.reduce_merge_states {
            if t.stale_since.elapsed() >= self.expire_after {
                flush_discriminants.push(k.clone());
            }
        }
        for k in &flush_discriminants {
            if let Some(t) = self.reduce_merge_states.remove(k) {
                emit!(ReduceStaleEventFlushed);
                output.push(Event::from(t.flush()));
            }
        }
    }

    fn flush_all_into(&mut self, output: &mut Vec<Event>) {
        self.reduce_merge_states
            .drain()
            .for_each(|(_, s)| output.push(Event::from(s.flush())));
    }

    fn push_or_new_reduce_state(&mut self, event: LogEvent, discriminant: Discriminant) {
        match self.reduce_merge_states.entry(discriminant) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(ReduceState::new(
                    event,
                    &self.merge_strategies,
                    self.mezmo_metadata.clone(),
                ));
            }
            hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().add_event(event, &self.merge_strategies);
            }
        }
    }

    // Mezmo-specific method. Fields that are specified with `date_formats` and a corresponding
    // `format` should be parsed from their string versions and sent through the reduce process
    // as a Value::Timestamp.
    fn coerce_into_timestamp_if_needed(&mut self, log_event: &mut LogEvent) {
        if self.mezmo_metadata.date_formats.len() == 0 {
            return ();
        }
        for (prop, format) in self.mezmo_metadata.date_formats.iter() {
            let prop_str = prop.as_str();
            if let Some(value) = log_event.get(prop_str) {
                let parse_result = Utc.datetime_from_str(value.to_string_lossy().as_str(), format);
                match parse_result {
                    Ok(date) => {
                        let value_kind = value.kind_str();
                        debug!(
                            message = "Coercing value into a Timestamp and saving metadata",
                            prop, value_kind
                        );
                        self.mezmo_metadata.save_date_kind(prop_str, value_kind);
                        log_event.insert(prop_str, Value::from(date));
                    }
                    Err(error) => {
                        warn!(message = "Failed to parse date field", field = prop, format = format, %error);
                    }
                };
            }
        }
        ()
    }

    // Mezmo-specific method. Incoming events from Mezmo will have all customer fields inside
    // the `.message` property. Create a new Event with all those properties at the root level
    // before sending through reduce.
    fn flatten_log_event_message(&mut self, event: &Event) -> Option<Event> {
        match event {
            Event::Log(log_event) => {
                if let Some(Value::Object(message_object)) = log_event.get("message") {
                    let mut flattened_log_event =
                        LogEvent::from_map(message_object.clone(), log_event.metadata().clone());

                    self.coerce_into_timestamp_if_needed(&mut flattened_log_event);

                    return Some(Event::from(flattened_log_event));
                }
                return None;
            }
            _ => None,
        }
    }

    fn transform_one(&mut self, output: &mut Vec<Event>, event: Event) {
        // Mezmo functionality here creates a new Event with the `.message` properties moved
        // to the root of the new event. This way, we can reuse all the complex functionality
        // of Condition and whether or not the reduce accumulator should stop, and how group_by works.

        let event = self.flatten_log_event_message(&event).unwrap_or(event);

        let (starts_here, event) = match &self.starts_when {
            Some(condition) => condition.check(event),
            None => (false, event),
        };

        let (ends_here, event) = match &self.ends_when {
            Some(condition) => condition.check(event),
            None => (false, event),
        };

        let event = event.into_log();
        let discriminant = Discriminant::from_log_event(&event, &self.group_by);

        if starts_here {
            if let Some(state) = self.reduce_merge_states.remove(&discriminant) {
                output.push(state.flush().into());
            }

            self.push_or_new_reduce_state(event, discriminant)
        } else if ends_here {
            output.push(match self.reduce_merge_states.remove(&discriminant) {
                Some(mut state) => {
                    state.add_event(event, &self.merge_strategies);
                    state.flush().into()
                }
                None => {
                    ReduceState::new(event, &self.merge_strategies, self.mezmo_metadata.clone())
                        .flush()
                        .into()
                }
            })
        } else {
            self.push_or_new_reduce_state(event, discriminant)
        }

        self.flush_into(output);
    }
}

impl TaskTransform<Event> for MezmoReduce {
    fn transform(
        self: Box<Self>,
        mut input_rx: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>>
    where
        Self: 'static,
    {
        let mut me = self;

        let poll_period = me.flush_period;

        let mut flush_stream = tokio::time::interval(poll_period);

        Box::pin(
            stream! {
              loop {
                let mut output = Vec::new();
                let done = tokio::select! {
                    _ = flush_stream.tick() => {
                      me.flush_into(&mut output);
                      false
                    }
                    maybe_event = input_rx.next() => {
                      match maybe_event {
                        None => {
                          me.flush_all_into(&mut output);
                          true
                        }
                        Some(event) => {
                          me.transform_one(&mut output, event);
                          false
                        }
                      }
                    }
                };
                yield stream::iter(output.into_iter());
                if done { break }
              }
            }
            .flatten(),
        )
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;
    use crate::{
        config::TransformConfig,
        event::{LogEvent, Value},
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<MezmoReduceConfig>();
    }

    #[tokio::test]
    async fn mezmo_reduce_default_behavior() {
        let reduce = toml::from_str::<MezmoReduceConfig>(r#""#)
            .unwrap()
            .build(&TransformContext::default())
            .await
            .unwrap();
        let reduce = reduce.into_task();
        let start_date = Utc::now();
        let end_date = Utc.timestamp(1_500_000_000, 0);

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("first string")),
                ("my_date".to_owned(), Value::from(start_date)),
            ]),
        );
        let metadata_1 = e_1.metadata().clone();

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("second string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Added in the second event"),
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("third string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Ignored, cause it's added in the THIRD event"),
                ),
                ("my_date".to_owned(), Value::from(end_date)),
            ]),
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.my_num"], 30.into());
        assert_eq!(output_1["message.my_string"], "first string".into());
        assert_eq!(
            output_1["message.e2_string"],
            "Added in the second event".into()
        );
        assert_eq!(output_1["message.my_date"], start_date.into());
        assert_eq!(output_1["message.my_date_end"], end_date.into());
        assert_eq!(output_1.metadata(), &metadata_1);
    }

    #[tokio::test]
    async fn mezmo_reduce_from_end_condition() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
[ends_when]
  type = "check_fields"
  ".test_end.exists" = true
"#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("second string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Added in the second event"),
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("third string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Ignored, cause it's added in the THIRD event"),
                ),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("fourth string")),
                ("test_end".to_owned(), Value::from("first end")),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("fifth string")),
                ("test_end".to_owned(), Value::from("second end")),
            ]),
        );
        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.my_num"], 40.into());
        assert_eq!(output_1["message.my_string"], "first string".into());
        assert_eq!(
            output_1["message.e2_string"],
            "Added in the second event".into()
        );
        assert_eq!(output_1["message.test_end"], "first end".into());

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_2["message.my_num"], 10.into());
        assert_eq!(output_2["message.my_string"], "fifth string".into());
        assert_eq!(output_2["message.test_end"], "second end".into());
    }

    #[tokio::test]
    async fn mezmo_reduce_from_start_condition() {
        // For clarity, the difference between `ends_when` and `starts_when` is whether or
        // not the *current* event is included in the accumulation. `ends_when` will accumulate
        // the current event then start a new reduce on the *next* event. `starts_when` will
        // immediately flush the previous reduce and start a new reduce using the current event.

        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
[starts_when]
  type = "vrl"
  source = ".start_new_here == true"
"#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("second string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Added in the second event"),
                ),
                (
                    "start_new_here".to_owned(),
                    Value::from(false), // Should NOT start a new one because it's false
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("third string")),
                (
                    "e2_string".to_owned(),
                    Value::from("Ignored, cause it's added in the THIRD event"),
                ),
                ("start_new_here".to_owned(), Value::from(true)),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("fourth string")),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("fifth string")),
            ]),
        );
        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.my_num"], 20.into());
        assert_eq!(output_1["message.my_string"], "first string".into());
        assert_eq!(
            output_1["message.e2_string"],
            "Added in the second event".into()
        );

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_2["message.my_num"], 30.into());
        assert_eq!(output_2["message.my_string"], "third string".into());
        assert_eq!(output_2["message.start_new_here"], true.into());
    }

    #[tokio::test]
    async fn mezmo_reduce_with_group_by() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
group_by = [ "request_id" ]

[ends_when]
  type = "vrl"
  source = ".stop_here == true"
"#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("my_num".to_owned(), Value::from(10)),
                ("my_string".to_owned(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("2")),
                ("my_num".to_owned(), Value::from(11)),
                ("my_string".to_owned(), Value::from("second string")),
                (
                    "other_string".to_owned(),
                    Value::from("Added in the second event"),
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("my_num".to_owned(), Value::from(12)),
                ("my_string".to_owned(), Value::from("third string")),
                (
                    "other_string".to_owned(),
                    Value::from("Added in the third event"),
                ),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("2")),
                ("my_num".to_owned(), Value::from(13)),
                ("my_string".to_owned(), Value::from("Ignore this string")),
                (
                    "other_string".to_owned(),
                    Value::from("Ignore this string also"),
                ),
                ("stop_here".to_owned(), Value::from(true)),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("my_num".to_owned(), Value::from(14)),
                ("my_string".to_owned(), Value::from("fifth string")),
                ("stop_here".to_owned(), Value::from(true)),
            ]),
        );
        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        // Since request_id=2 was "ended" first, we should expect it to be first here. Using a `ends_when`
        // helps cut down on flakey results as the order could change if we don't specify it.
        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.request_id"], "2".into());
        assert_eq!(output_1["message.my_num"], 24.into());
        assert_eq!(output_1["message.my_string"], "second string".into());
        assert_eq!(
            output_1["message.other_string"],
            "Added in the second event".into()
        );

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_2["message.request_id"], "1".into());
        assert_eq!(output_2["message.my_num"], 36.into());
        assert_eq!(output_2["message.my_string"], "first string".into());
        assert_eq!(
            output_2["message.other_string"],
            "Added in the third event".into()
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_merge_strategies() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
    group_by = [ "request_id" ]

    merge_strategies.foo = "concat"
    merge_strategies.bar = "array"
    merge_strategies.baz = "max"

    [ends_when]
      type = "check_fields"
      "test_end.exists" = true
    "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("foo".to_owned(), Value::from("first foo")),
                ("bar".to_owned(), Value::from("first bar")),
                ("baz".to_owned(), Value::from(2)),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("foo".to_owned(), Value::from("second foo")),
                ("bar".to_owned(), Value::from(2)),
                ("baz".to_owned(), Value::from("not number")),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), Value::from("1")),
                ("foo".to_owned(), Value::from(10)),
                ("bar".to_owned(), Value::from("third bar")),
                ("baz".to_owned(), Value::from(3)),
                ("test_end".to_owned(), Value::from("yep")),
            ]),
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.request_id"], "1".into());
        assert_eq!(output_1["message.foo"], "first foo second foo".into());
        assert_eq!(
            output_1["message.bar"],
            Value::Array(vec!["first bar".into(), 2.into(), "third bar".into()]),
        );
        assert_eq!(output_1["message.baz"], 3.into());
    }

    #[tokio::test]
    async fn mezmo_reduce_missing_group_by() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
    group_by = [ "request_id" ]

    [ends_when]
      type = "check_fields"
      "test_end.exists" = true
    "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("counter".to_owned(), 1.into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([("counter".to_owned(), 2.into())]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("counter".to_owned(), 3.into()),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("counter".to_owned(), 4.into()),
                ("test_end".to_owned(), "yep".into()),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("counter".to_owned(), 5.into()),
                ("extra_field".to_owned(), "value1".into()),
                ("test_end".to_owned(), "yep".into()),
            ]),
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.counter"], 8.into());

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_2["message.extra_field"], "value1".into());
        assert_eq!(output_2["message.counter"], 7.into());
    }

    #[tokio::test]
    async fn mezmo_reduce_arrays_in_payload() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
    group_by = [ "request_id" ]

    merge_strategies.foo = "array"
    merge_strategies.bar = "concat"

    [ends_when]
      type = "check_fields"
      "test_end.exists" = true
    "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("foo".to_owned(), json!([1, 3]).into()),
                ("bar".to_owned(), json!([1, 3]).into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "2".into()),
                ("foo".to_owned(), json!([2, 4]).into()),
                ("bar".to_owned(), json!([2, 4]).into()),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("foo".to_owned(), json!([5, 7]).into()),
                ("bar".to_owned(), json!([5, 7]).into()),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "1".into()),
                ("foo".to_owned(), json!("done").into()),
                ("bar".to_owned(), json!("done").into()),
                ("test_end".to_owned(), "yep".into()),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "2".into()),
                ("foo".to_owned(), json!([6, 8]).into()),
                ("bar".to_owned(), json!([6, 8]).into()),
            ]),
        );

        let mut e_6 = LogEvent::default();
        e_6.insert(
            "message",
            BTreeMap::from([
                ("request_id".to_owned(), "2".into()),
                ("foo".to_owned(), json!("done").into()),
                ("bar".to_owned(), json!("done").into()),
                ("test_end".to_owned(), "yep".into()),
            ]),
        );

        let inputs = vec![
            e_1.into(),
            e_2.into(),
            e_3.into(),
            e_4.into(),
            e_5.into(),
            e_6.into(),
        ];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.request_id"], "1".into());
        assert_eq!(
            output_1["message.foo"],
            json!([[1, 3], [5, 7], "done"]).into()
        );
        assert_eq!(output_1["message.bar"], json!([1, 3, 5, 7, "done"]).into());

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_2["message.request_id"], "2".into());
        assert_eq!(
            output_2["message.foo"],
            json!([[2, 4], [6, 8], "done"]).into()
        );
        assert_eq!(output_2["message.bar"], json!([2, 4, 6, 8, "done"]).into());
    }

    #[tokio::test]
    async fn mezmo_reduce_timestamps_with_path_notation() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
    [date_formats]
      ".ts" = "%Y-%m-%d %H:%M:%S"
      ".epoch" = "%s"
      ".epoch_str" = "%s"

    [ends_when]
      type = "check_fields"
      "test_end.exists" = true
    "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();

        let mut e_1 = LogEvent::default();
        e_1.insert(
            "message",
            BTreeMap::from([
                ("ts".to_owned(), "2014-11-28 12:00:09".into()),
                ("epoch".to_owned(), 1671134262.into()),
                ("epoch_str".to_owned(), "1671134262".into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("ts".to_owned(), "2014-11-28 13:00:09".into()),
                ("epoch".to_owned(), 1671134263.into()),
                ("epoch_str".to_owned(), "1671134263".into()),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("ts".to_owned(), "2014-11-28 14:00:09".into()),
                ("epoch".to_owned(), 1671134264.into()),
                ("epoch_str".to_owned(), "1671134264".into()),
                ("test_end".to_owned(), "yup".into()),
            ]),
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.test_end"], "yup".into());
        assert_eq!(output_1["message.ts"], "2014-11-28 12:00:09".into());
        assert_eq!(output_1["message.ts_end"], "2014-11-28 14:00:09".into());
        assert_eq!(output_1["message.epoch"], 1671134262.into());
        assert_eq!(output_1["message.epoch_end"], 1671134264.into());
        assert_eq!(output_1["message.epoch_str"], "1671134262".into());
        assert_eq!(output_1["message.epoch_str_end"], "1671134264".into());
    }
}