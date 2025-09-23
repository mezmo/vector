// This module mimics the `reduce` vector transform, but it operates against the .message
// property of the log event instead of the root-level properties (Vector's implementation).
// This implementation also (de)serializes date fields that are specified by the user, making sure
// to return date fields in the same format as originally received. For example, an epoch field
// can be an integer or a string, and it will match the output type based on the incoming data.

use rand::Rng;
use std::collections::BTreeMap;
use std::{
    collections::{hash_map, HashMap},
    mem,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

pub use super::merge_strategy::*;
use crate::{
    conditions::{AnyCondition, Condition},
    config::{DataType, Input, TransformConfig, TransformContext},
    event::{discriminant::Discriminant, Event, EventMetadata, LogEvent},
    internal_events::ReduceStaleEventFlushed,
    mezmo::persistence::{PersistenceConnection, RocksDBPersistenceConnection},
    transforms::{TaskTransform, Transform},
};
use async_stream::stream;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use futures::{stream, Stream, StreamExt};
use indexmap::IndexMap;
use serde;
use serde_with::serde_as;
use vector_lib::config::{log_schema, OutputId, TransformOutput};
use vector_lib::configurable::configurable_component;
use vector_lib::lookup::lookup_v2::{parse_target_path, OwnedSegment};
use vector_lib::lookup::owned_value_path;
use vector_lib::schema::Definition;

use crate::event::{KeyString, Value};
use vector_lib::config::LogNamespace;

// The key for the state persistence db.
const STATE_PERSISTENCE_KEY: &str = "state";

/// Combined state structure for serialization to RocksDB
/// Uses a JSON-compatible format by converting Discriminant to Vec<Value>
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct PersistedState {
    /// Store discriminant map as Vec of (discriminant_values, reduce_state) pairs
    merge_state: Vec<(Vec<Option<Value>>, ReduceState)>,
    date_kinds_state: HashMap<String, String>,
}

impl PersistedState {
    /// Convert from the runtime HashMap format to the serializable format
    #[allow(clippy::mutable_key_type)]
    fn from_runtime_state(
        merge_state: &HashMap<Discriminant, ReduceState>,
        date_kinds_state: &HashMap<String, String>,
    ) -> Self {
        let merge_state: Vec<(Vec<Option<Value>>, ReduceState)> = merge_state
            .iter()
            .map(|(discriminant, reduce_state)| {
                (discriminant.values().clone(), reduce_state.clone())
            })
            .collect();

        Self {
            merge_state,
            date_kinds_state: date_kinds_state.clone(),
        }
    }

    /// Convert to the runtime HashMap format
    fn to_runtime_state(&self) -> (HashMap<Discriminant, ReduceState>, HashMap<String, String>) {
        #[allow(clippy::mutable_key_type)]
        let merge_state: HashMap<Discriminant, ReduceState> = self
            .merge_state
            .iter()
            .map(|(values, reduce_state)| {
                let discriminant = Discriminant::from_values(values.clone());
                (discriminant, reduce_state.clone())
            })
            .collect();

        (merge_state, self.date_kinds_state.clone())
    }
}

/// Configuration for the `mezmo_reduce` transform.
#[serde_as]
#[configurable_component(transform("mezmo_reduce"))]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoReduceConfig {
    /// The maximum period of time to wait after the last event is received, in milliseconds, before
    /// a combined event should be considered complete.
    #[serde(default = "default_expire_after_ms")]
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    #[derivative(Default(value = "default_expire_after_ms()"))]
    pub expire_after_ms: Duration,

    /// The interval to check for and flush any expired events, in milliseconds.
    #[serde(default = "default_flush_period_ms")]
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    #[derivative(Default(value = "default_flush_period_ms()"))]
    pub flush_period_ms: Duration,

    /// An ordered list of fields by which to group events.
    ///
    /// Each group with matching values for the specified keys is reduced independently, allowing
    /// you to keep independent event streams separate. When no fields are specified, all events
    /// will be combined in a single group.
    ///
    /// For example, if `group_by = ["host", "region"]`, then all incoming events that have the same
    /// host and region will be grouped together before being reduced.
    #[serde(default)]
    #[configurable(metadata(
        docs::examples = "request_id",
        docs::examples = "user_id",
        docs::examples = "transaction_id",
    ))]
    pub group_by: Vec<String>,

    /// A map of field names to custom merge strategies.
    ///
    /// For each field specified, the given strategy will be used for combining events rather than
    /// the default behavior.
    ///
    /// The default behavior is as follows:
    ///
    /// - The first value of a string field is kept, subsequent values are discarded.
    /// - For timestamp fields the first is kept and a new field `[field-name]_end` is added with
    ///   the last received timestamp value.
    /// - Numeric values are summed.
    #[serde(default)]
    pub merge_strategies: IndexMap<KeyString, MergeStrategy>,

    /// The maximum number of events to group together.
    pub max_events: Option<NonZeroUsize>,

    /// A condition used to distinguish the final event of a transaction.
    ///
    /// If this condition resolves to `true` for an event, the current transaction is immediately
    /// flushed with this event.
    pub ends_when: Option<AnyCondition>,

    /// A condition used to distinguish the first event of a transaction.
    ///
    /// If this condition resolves to `true` for an event, the previous transaction is flushed
    /// (without this event) and a new transaction is started.
    pub starts_when: Option<AnyCondition>,

    /// Mezmo-specific. Since dates can be serialized, users can specify which properties should be dates, and what format can
    /// be used to parse them. This eventually will translate Value::Bytes into a Value::Timestamp
    #[serde(default)]
    pub date_formats: HashMap<String, String>,

    /// Sets the base path for the persistence connection.
    /// NOTE: Leaving this value empty will disable state persistence.
    #[serde(default = "default_state_persistence_base_path")]
    pub(super) state_persistence_base_path: Option<String>,

    /// Set how often the state of this transform will be persisted to the [PersistenceConnection]
    /// storage backend.
    #[serde(default = "default_state_persistence_tick_ms")]
    pub(super) state_persistence_tick_ms: u64,

    /// The maximum amount of jitter (ms) to add to the `state_persistence_tick_ms`
    /// flush interval.
    #[serde(default = "default_state_persistence_max_jitter_ms")]
    pub(super) state_persistence_max_jitter_ms: u64,
}

const fn default_expire_after_ms() -> Duration {
    Duration::from_millis(30000)
}

const fn default_flush_period_ms() -> Duration {
    Duration::from_millis(1000)
}

const fn default_state_persistence_base_path() -> Option<String> {
    None
}

const fn default_state_persistence_tick_ms() -> u64 {
    30000
}

const fn default_state_persistence_max_jitter_ms() -> u64 {
    750
}

#[derive(Debug, Clone)]
struct MezmoMetadata {
    date_formats: HashMap<String, String>,

    /// Mezmo-specific. This will track the Kind of Value that reduce should send back when the reduce is complete. For example,
    /// an epoch time may come in as an integer, and thus should go out as an integer (and not a Timestamp).
    /// This structure is keyed by the Property location and the value is the kind type (either string or integer in our case).
    date_kinds: Arc<RwLock<HashMap<String, String>>>,
}

impl serde::Serialize for MezmoMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MezmoMetadata", 2)?;
        state.serialize_field("date_formats", &self.date_formats)?;

        // Extract the HashMap from Arc<RwLock<>>
        let date_kinds = self.date_kinds.read().unwrap();
        state.serialize_field("date_kinds", &*date_kinds)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for MezmoMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct MezmoMetadataHelper {
            date_formats: HashMap<String, String>,
            date_kinds: HashMap<String, String>,
        }

        let helper = MezmoMetadataHelper::deserialize(deserializer)?;
        Ok(MezmoMetadata {
            date_formats: helper.date_formats,
            date_kinds: Arc::new(RwLock::new(helper.date_kinds)),
        })
    }
}

impl MezmoMetadata {
    fn new(date_formats: HashMap<String, String>, date_kinds: HashMap<String, String>) -> Self {
        let date_formats = date_formats
            .into_iter()
            .map(|(k, v)| {
                match k.strip_prefix(log_schema().message_key().unwrap().to_string().as_str()) {
                    Some(stripped) => (stripped.to_string(), v),
                    None => (k, v),
                }
            })
            .collect();

        Self {
            date_formats,
            date_kinds: Arc::new(RwLock::new(date_kinds)),
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

const REDUCE_BYTE_THRESHOLD_PER_STATE_DEFAULT: usize = 100 * 1024; // 100K
const REDUCE_BYTE_THRESHOLD_ALL_STATES_DEFAULT: usize = 1024 * 1024; // 1MB

impl_generate_config_from_default!(MezmoReduceConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_reduce")]
impl TransformConfig for MezmoReduceConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        MezmoReduce::new(self, context).map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        _: &[(OutputId, Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Log, HashMap::new())]
    }
}

#[derive(Debug)]
struct ReduceState {
    fields: HashMap<KeyString, Box<dyn ReduceValueMerger>>,
    message_fields: HashMap<KeyString, Box<dyn ReduceValueMerger>>, // Mezmo-specific. Fields under "message".
    started_at: Instant,
    metadata: EventMetadata,
    mezmo_metadata: MezmoMetadata,
    size_estimate: usize,
    events: usize,
}

impl serde::Serialize for ReduceState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use super::merge_strategy::SerializableReduceValueMerger;
        use serde::ser::SerializeStruct;
        use std::time::SystemTime;

        let mut state = serializer.serialize_struct("ReduceState", 7)?;

        // Convert HashMap<KeyString, Box<dyn ReduceValueMerger>> to serializable form
        let serializable_fields: Result<
            HashMap<KeyString, SerializableReduceValueMerger>,
            S::Error,
        > = self
            .fields
            .iter()
            .map(|(k, v)| {
                let any_ref = v.as_any();
                if let Some(serializable) = SerializableReduceValueMerger::from_any_ref(any_ref) {
                    Ok((k.clone(), serializable))
                } else {
                    Err(serde::ser::Error::custom(
                        "Unable to serialize ReduceValueMerger",
                    ))
                }
            })
            .collect();
        state.serialize_field("fields", &serializable_fields?)?;

        let serializable_message_fields: Result<
            HashMap<KeyString, SerializableReduceValueMerger>,
            S::Error,
        > = self
            .message_fields
            .iter()
            .map(|(k, v)| {
                let any_ref = v.as_any();
                if let Some(serializable) = SerializableReduceValueMerger::from_any_ref(any_ref) {
                    Ok((k.clone(), serializable))
                } else {
                    Err(serde::ser::Error::custom(
                        "Unable to serialize ReduceValueMerger",
                    ))
                }
            })
            .collect();
        state.serialize_field("message_fields", &serializable_message_fields?)?;

        // Convert Instant to duration from UNIX_EPOCH for serialization
        let duration_from_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| serde::ser::Error::custom(format!("SystemTime error: {}", e)))?
            .saturating_sub(self.started_at.elapsed());
        state.serialize_field("started_at_epoch_secs", &duration_from_epoch.as_secs())?;
        state.serialize_field(
            "started_at_epoch_nanos",
            &duration_from_epoch.subsec_nanos(),
        )?;

        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("mezmo_metadata", &self.mezmo_metadata)?;
        state.serialize_field("size_estimate", &self.size_estimate)?;
        state.serialize_field("events", &self.events)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ReduceState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use super::merge_strategy::SerializableReduceValueMerger;
        use std::time::SystemTime;

        #[derive(serde::Deserialize)]
        struct ReduceStateHelper {
            fields: HashMap<KeyString, SerializableReduceValueMerger>,
            message_fields: HashMap<KeyString, SerializableReduceValueMerger>,
            started_at_epoch_secs: u64,
            started_at_epoch_nanos: u32,
            metadata: EventMetadata,
            mezmo_metadata: MezmoMetadata,
            size_estimate: usize,
            events: usize,
        }

        let helper = ReduceStateHelper::deserialize(deserializer)?;

        // Convert serializable form back to HashMap<KeyString, Box<dyn ReduceValueMerger>>
        let fields: HashMap<KeyString, Box<dyn ReduceValueMerger>> = helper
            .fields
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_merger()))
            .collect();

        let message_fields: HashMap<KeyString, Box<dyn ReduceValueMerger>> = helper
            .message_fields
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_merger()))
            .collect();

        // Reconstruct Instant from epoch time
        let target_time = SystemTime::UNIX_EPOCH
            + std::time::Duration::new(helper.started_at_epoch_secs, helper.started_at_epoch_nanos);
        let elapsed_since_target = SystemTime::now()
            .duration_since(target_time)
            .unwrap_or_default();
        let started_at = Instant::now() - elapsed_since_target;

        Ok(ReduceState {
            fields,
            message_fields,
            started_at,
            metadata: helper.metadata,
            mezmo_metadata: helper.mezmo_metadata,
            size_estimate: helper.size_estimate,
            events: helper.events,
        })
    }
}

impl Clone for ReduceState {
    fn clone(&self) -> Self {
        use super::merge_strategy::SerializableReduceValueMerger;

        // Clone the trait objects by converting through serializable form
        let fields: HashMap<KeyString, Box<dyn ReduceValueMerger>> = self
            .fields
            .iter()
            .filter_map(|(k, v)| {
                let any_ref = v.as_any();
                SerializableReduceValueMerger::from_any_ref(any_ref)
                    .map(|serializable| (k.clone(), serializable.into_boxed_merger()))
            })
            .collect();

        let message_fields: HashMap<KeyString, Box<dyn ReduceValueMerger>> = self
            .message_fields
            .iter()
            .filter_map(|(k, v)| {
                let any_ref = v.as_any();
                SerializableReduceValueMerger::from_any_ref(any_ref)
                    .map(|serializable| (k.clone(), serializable.into_boxed_merger()))
            })
            .collect();

        ReduceState {
            fields,
            message_fields,
            started_at: self.started_at,
            metadata: self.metadata.clone(),
            mezmo_metadata: self.mezmo_metadata.clone(),
            size_estimate: self.size_estimate,
            events: self.events,
        }
    }
}

impl ReduceState {
    fn new(
        event: LogEvent,
        message_event: LogEvent,
        strategies: &IndexMap<KeyString, MergeStrategy>,
        mezmo_metadata: MezmoMetadata,
        group_by: &[KeyString],
    ) -> Self {
        let (value, metadata) = event.into_parts();

        // Use default merge strategies for root fields
        let fields = if let Value::Object(fields) = value {
            fields.into_iter().map(|(k, v)| (k, v.into())).collect()
        } else {
            HashMap::new()
        };

        let (value, _) = message_event.into_parts();

        // Create a list of root property names after their field path notations are parsed (."my-thing" === "my-thing")
        // This is used only to disallow using merge_strategies on group_by fields below.
        let mut group_by_lookups: Vec<KeyString> = vec![];
        let name = "group_by_lookup";
        for key in group_by {
            if let Some(root_key) = get_root_property_name_from_path(key, name, false) {
                group_by_lookups.push(root_key.into());
            }
        }

        let mut size_estimate: usize = 0;

        let message_fields = if let Value::Object(fields) = value {
            fields
                .into_iter()
                .filter_map(|(k, v)| {
                    // Do not allow merge strategies on `group_by` fields. Keep the first value, but discard the others.
                    if group_by_lookups.contains(&k) {
                        let m = get_value_merger(v, &MergeStrategy::Discard).unwrap();
                        return Some((k, m));
                    }

                    if let Some(strat) = strategies.get(&k) {
                        match get_value_merger(v, strat) {
                            Ok(m) => {
                                size_estimate += m.size_estimate();
                                Some((k, m))
                            }
                            Err(error) => {
                                warn!(message = "Failed to create merger.", field = ?k, %error);
                                None
                            }
                        }
                    } else {
                        let m: Box<dyn ReduceValueMerger> = v.into();
                        size_estimate += m.size_estimate();
                        Some((k, m))
                    }
                })
                .collect()
        } else {
            HashMap::new()
        };

        Self {
            started_at: Instant::now(),
            fields,
            message_fields,
            metadata, // Contains finalizers from `event`, not `message_event`
            mezmo_metadata,
            size_estimate,
            events: 1,
        }
    }

    fn add_event(
        &mut self,
        event: LogEvent,
        message_event: LogEvent,
        strategies: &IndexMap<KeyString, MergeStrategy>,
    ) {
        let (value, metadata) = event.into_parts();
        self.metadata.merge(metadata);

        let fields = if let Value::Object(fields) = value {
            fields
        } else {
            BTreeMap::new()
        };

        // Use default merge strategies for root fields
        for (k, v) in fields.into_iter() {
            match self.fields.entry(k) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(v.clone().into());
                }
                hash_map::Entry::Occupied(mut entry) => {
                    if let Err(error) = entry.get_mut().add(v.clone()) {
                        warn!(message = "Failed to merge value.", %error);
                    }
                }
            }
        }

        let (value, _) = message_event.into_parts();

        let fields = if let Value::Object(fields) = value {
            fields
        } else {
            BTreeMap::new()
        };

        for (k, v) in fields.into_iter() {
            let strategy = strategies.get(&k);
            match self.message_fields.entry(k) {
                hash_map::Entry::Vacant(entry) => {
                    if let Some(strat) = strategy {
                        match get_value_merger(v, strat) {
                            Ok(m) => {
                                self.size_estimate += m.size_estimate();
                                entry.insert(m);
                            }
                            Err(error) => {
                                warn!(message = "Failed to merge value.", %error);
                            }
                        }
                    } else {
                        let m: Box<dyn ReduceValueMerger> = v.clone().into();
                        self.size_estimate += m.size_estimate();
                        entry.insert(m);
                    }
                }
                hash_map::Entry::Occupied(mut entry) => {
                    // Mezmo-specific: here we are *updating* the size of the value merger. Subtract the original value before
                    // adding whatever the new value is (for example, when array lengths change)
                    let original_size = entry.get().size_estimate();
                    entry.get_mut().add(v.clone()).map_or_else(
                        |error| warn!(message = "Failed to merge value.", %error),
                        |_| {
                            let new_size = entry.get().size_estimate();
                            if new_size < original_size {
                                let delta = original_size - new_size;
                                if delta > self.size_estimate {
                                    self.size_estimate = 0
                                } else {
                                    self.size_estimate -= delta;
                                }
                            } else {
                                self.size_estimate += new_size - original_size;
                            }
                        },
                    );
                }
            }
        }
        self.events += 1;
    }

    // Mezmo-specific method. Take the timestamp fields (and their _end counterparts) and
    // create a Value() that matches the incoming data type for the field, e.g. a String.
    fn coerce_from_timestamp_if_needed(&self, log_event: &mut LogEvent) {
        let date_formats = &self.mezmo_metadata.date_formats;
        if date_formats.is_empty() {
            debug!(message = "There are no custom date formats to coerce");
            return;
        }

        let message_obj = log_event.get_mut("message").unwrap();

        for (date_prop, format) in date_formats.iter() {
            let end_prop = format!("{date_prop}_end");
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
                            .parse::<i64>().map(Value::from)
                            .unwrap_or_else(|error| {
                                warn!(message = "Could not coerce start date back into an integer Value", date_prop, %error);
                                Value::from(start_date_string)
                            });
                            let end_val = end_date_string
                            .parse::<i64>().map(Value::from)
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
    }

    /// Assembles a new event containing the results of this state, including the
    /// accumulated metadata (ie finalizers). The resulting event will end up in `output` via `flush_into()`
    fn flush(mut self) -> LogEvent {
        let metadata_and_finalizers = mem::take(&mut self.metadata);
        let mut event = LogEvent::new_with_metadata(metadata_and_finalizers);

        for (k, v) in self.fields.drain() {
            if let Err(error) = v.insert_into(k.into(), &mut event) {
                warn!(message = "Failed to merge values for field.", %error);
            }
        }

        for (k, v) in self.message_fields.drain() {
            // When the resulting event is created from the mezmo-reduce accumulator,
            // we need to inject its results into the `.message` property, but make it an
            // actual "path" so that special characters are handled.
            let path = owned_value_path!("message", k.as_str()).to_string();
            // BEWARE: Upstream has changed inserts to use `event_path!`, but that stores
            // flattened keys like `message.thing` which breaks us. We need nested objects,
            // so do not accept upstream changes to `merge_strategy.rs`.
            if let Err(error) = v.insert_into(path.into(), &mut event) {
                warn!(message = "Failed to merge values for message field.", %error);
            }
        }

        self.coerce_from_timestamp_if_needed(&mut event);
        self.events = 0;
        event
    }
}

pub struct MezmoReduce {
    expire_after: Duration,
    flush_period: Duration,
    group_by: Vec<KeyString>,
    merge_strategies: IndexMap<KeyString, MergeStrategy>,
    reduce_merge_states: HashMap<Discriminant, ReduceState>,
    ends_when: Option<Condition>,
    starts_when: Option<Condition>,
    mezmo_metadata: MezmoMetadata,
    byte_threshold_per_state: usize,
    byte_threshold_all_states: usize,
    max_events: Option<usize>,
    state_persistence: Option<Arc<dyn PersistenceConnection>>,
    state_persistence_tick_ms: u64,
    state_persistence_max_jitter_ms: u64,
}

impl MezmoReduce {
    pub fn new(config: &MezmoReduceConfig, cx: &TransformContext) -> crate::Result<Self> {
        if config.ends_when.is_some() && config.starts_when.is_some() {
            return Err("only one of `ends_when` and `starts_when` can be provided".into());
        }

        let ends_when = config
            .ends_when
            .as_ref()
            .map(|c| c.build(&cx.enrichment_tables, cx.mezmo_ctx.clone()))
            .transpose()?;
        let starts_when = config
            .starts_when
            .as_ref()
            .map(|c| c.build(&cx.enrichment_tables, cx.mezmo_ctx.clone()))
            .transpose()?;
        let group_by = config
            .group_by
            .clone()
            .into_iter()
            .map(|path| {
                match path.strip_prefix(log_schema().message_key().unwrap().to_string().as_str()) {
                    Some(stripped) => stripped.into(),
                    None => path.into(),
                }
            })
            .collect();
        let max_events = config.max_events.map(|max| max.into());
        let byte_threshold_per_state = match std::env::var("REDUCE_BYTE_THRESHOLD_PER_STATE") {
            Ok(env_var) => env_var
                .parse()
                .unwrap_or(REDUCE_BYTE_THRESHOLD_PER_STATE_DEFAULT),
            _ => REDUCE_BYTE_THRESHOLD_PER_STATE_DEFAULT,
        };
        let byte_threshold_all_states = match std::env::var("REDUCE_BYTE_THRESHOLD_ALL_STATES") {
            Ok(env_var) => env_var
                .parse()
                .unwrap_or(REDUCE_BYTE_THRESHOLD_ALL_STATES_DEFAULT),
            _ => REDUCE_BYTE_THRESHOLD_ALL_STATES_DEFAULT,
        };

        // Strip path notation from merge_strategy fields
        let mut merge_strategies: IndexMap<KeyString, MergeStrategy> = IndexMap::new();
        let name = "merge_strategy";
        for (merge_strategy_key, strategy) in config.merge_strategies.iter() {
            if let Some(root_key) = get_root_property_name_from_path(merge_strategy_key, name, true)
            {
                merge_strategies.insert(root_key.into(), strategy.clone());
            }
        }

        // Strip path notation from date_formats
        let mut date_formats: HashMap<String, String> = HashMap::new();
        let name = "date_format";
        for (date_key, format) in config.date_formats.clone().into_iter() {
            if let Some(root_key) = get_root_property_name_from_path(&date_key.into(), name, true) {
                date_formats.insert(root_key, format.clone());
            }
        }

        let state_persistence_tick_ms = config.state_persistence_tick_ms;
        let state_persistence_max_jitter_ms = config.state_persistence_max_jitter_ms;
        let state_persistence: Option<Arc<dyn PersistenceConnection>> =
            match (&config.state_persistence_base_path, &cx.mezmo_ctx) {
                (Some(base_path), Some(mezmo_ctx)) => Some(Arc::new(
                    RocksDBPersistenceConnection::new(base_path, mezmo_ctx)?,
                )),
                (_, Some(mezmo_ctx)) => {
                    debug!(
                        "MezmoReduce: state persistence not enabled for component {}",
                        mezmo_ctx.id()
                    );
                    None
                }
                (_, _) => None,
            };

        let (reduce_merge_states, date_kinds) = match &state_persistence {
            Some(state_persistence) => load_initial_state(state_persistence),
            None => (HashMap::new(), HashMap::new()),
        };

        Ok(MezmoReduce {
            expire_after: config.expire_after_ms,
            flush_period: config.flush_period_ms,
            group_by,
            merge_strategies,
            reduce_merge_states,
            ends_when,
            starts_when,
            mezmo_metadata: MezmoMetadata::new(date_formats, date_kinds),
            byte_threshold_per_state,
            byte_threshold_all_states,
            max_events,
            state_persistence,
            state_persistence_tick_ms,
            state_persistence_max_jitter_ms,
        })
    }

    /// Add any expired or completed reductions to the output array. Called mostly via an Interval timer.
    fn flush_into(&mut self, output: &mut Vec<Event>) {
        let mut total_states_size_estimate = 0;
        let mut flush_discriminants: BTreeMap<Instant, Discriminant> = BTreeMap::new();

        debug!(
            message = "Flush called",
            number_of_states = &self.reduce_merge_states.len()
        );

        for (discriminant, state) in &self.reduce_merge_states {
            if state.started_at.elapsed() >= self.expire_after {
                debug!(message = "Flushing based on started_at exceeding expire_after_ms");
                flush_discriminants.insert(state.started_at, discriminant.clone());
            } else if state.size_estimate > self.byte_threshold_per_state {
                warn!("Flushing because the state size of {} has exceeded the per-state threshold of {}",
                    state.size_estimate, self.byte_threshold_per_state
                );
                flush_discriminants.insert(state.started_at, discriminant.clone());
            } else {
                // Only add to the total state size if we HAVE NOT flushed the state yet
                total_states_size_estimate += state.size_estimate;
            }
        }

        // Flush any stale states, sorted by started_at.
        // This also emits a "stale event flushed" event, whereas flush_all_into does not (because they're not "stale")
        for (_, discriminant) in flush_discriminants {
            if let Some(state) = self.reduce_merge_states.remove(&discriminant) {
                emit!(ReduceStaleEventFlushed);
                output.push(Event::from(state.flush()));
            }
        }

        debug!("Total size of all states: {}", total_states_size_estimate);
        if total_states_size_estimate > self.byte_threshold_all_states {
            warn!(
                "Flushing all states because the byte total {} exceeds the threshold of {}",
                total_states_size_estimate, self.byte_threshold_all_states
            );
            self.flush_all_into(output);
        }
    }

    /// Adds all accumulated states to the output, regardless of expiry times or start/end conditions.
    fn flush_all_into(&mut self, output: &mut Vec<Event>) {
        // Make sure to sort by `started_at` so that line order is preserved as closely as possible
        let mut sorted_states: Vec<(Discriminant, ReduceState)> =
            self.reduce_merge_states.drain().collect();

        sorted_states.sort_by(|(_, a), (_, b)| a.started_at.cmp(&b.started_at));

        for (_, state) in sorted_states {
            output.push(Event::from(state.flush()))
        }
    }

    fn push_or_new_reduce_state(
        &mut self,
        event: LogEvent,
        message_event: LogEvent,
        discriminant: Discriminant,
    ) {
        match self.reduce_merge_states.entry(discriminant) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(ReduceState::new(
                    event,
                    message_event,
                    &self.merge_strategies,
                    self.mezmo_metadata.clone(),
                    &self.group_by,
                ));
            }
            hash_map::Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .add_event(event, message_event, &self.merge_strategies);
            }
        }
    }

    // Mezmo-specific method. Fields that are specified with `date_formats` and a corresponding
    // `format` should be parsed from their string versions and sent through the reduce process
    // as a Value::Timestamp.
    fn coerce_into_timestamp_if_needed(&mut self, log_event: &mut LogEvent) {
        if self.mezmo_metadata.date_formats.is_empty() {
            return;
        }
        for (prop, format) in self.mezmo_metadata.date_formats.iter() {
            let prop_str = prop.as_str();
            if let Some(value) = log_event.get(prop_str) {
                let parse_result = NaiveDateTime::parse_from_str(&value.to_string_lossy(), format);
                match parse_result {
                    Ok(date) => {
                        // Value::from for dates requires a DateTime<Utc>
                        let date: DateTime<Utc> = Utc.from_utc_datetime(&date);
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
    }

    // Mezmo-specific method. Incoming events from Mezmo will have all customer fields inside
    // the `.message` property. Create a new Event with all those properties at the root level
    // before sending through reduce. The metadata and finalizers will remain in `event` as `message_event`
    // is only used for value analysis.
    fn extract_message_event(&mut self, event: &mut LogEvent) -> Event {
        Event::from(
            if let Some(Value::Object(message_object)) = event.remove("message") {
                let mut message_event = LogEvent::from_map(message_object, Default::default());

                self.coerce_into_timestamp_if_needed(&mut message_event);

                message_event
            } else {
                LogEvent::from_map(Default::default(), Default::default())
            },
        )
    }

    fn transform_one(&mut self, output: &mut Vec<Event>, event: Event) {
        let mut event = event.into_log();

        // Mezmo functionality here creates a new Event with the `.message` properties moved
        // to the root of the new event. This way, we can reuse all the complex functionality
        // of Condition and whether or not the reduce accumulator should stop, and how group_by works.
        let message_event = self.extract_message_event(&mut event);

        let (starts_here, message_event) = match &self.starts_when {
            Some(condition) => condition.check(message_event),
            None => (false, message_event),
        };

        let (mut ends_here, message_event) = match &self.ends_when {
            Some(condition) => condition.check(message_event),
            None => (false, message_event),
        };

        let message_event = message_event.into_log();
        let discriminant = Discriminant::from_log_event(&message_event, &self.group_by);

        if let Some(max_events) = self.max_events {
            if max_events == 1 {
                ends_here = true;
            } else if let Some(entry) = self.reduce_merge_states.get(&discriminant) {
                // The current event will finish this set
                if entry.events + 1 == max_events {
                    ends_here = true;
                }
            }
        }

        if starts_here {
            if let Some(state) = self.reduce_merge_states.remove(&discriminant) {
                output.push(state.flush().into());
            }

            self.push_or_new_reduce_state(event, message_event, discriminant)
        } else if ends_here {
            output.push(match self.reduce_merge_states.remove(&discriminant) {
                Some(mut state) => {
                    state.add_event(event, message_event, &self.merge_strategies);
                    state.flush().into()
                }
                None => ReduceState::new(
                    event,
                    message_event,
                    &self.merge_strategies,
                    self.mezmo_metadata.clone(),
                    &self.group_by,
                )
                .flush()
                .into(),
            })
        } else {
            self.push_or_new_reduce_state(event, message_event, discriminant)
        }
    }

    /// Saves the current `data` to persistent storage. This is intended to be called from the
    /// polling loop on an interval defined by the `state_persistence_tick_ms` field.
    async fn persist_state(&mut self) {
        if let Some(state_persistence) = &self.state_persistence {
            #[allow(clippy::mutable_key_type)]
            let merge_state = self.reduce_merge_states.clone();
            let date_kinds_state = self.mezmo_metadata.date_kinds.read().unwrap().clone();
            let state_persistence = Arc::clone(state_persistence);
            let handle = tokio::task::spawn_blocking(move || {
                let state = PersistedState::from_runtime_state(&merge_state, &date_kinds_state);
                let serialized_state = serde_json::to_string(&state)?;
                state_persistence.set(STATE_PERSISTENCE_KEY, &serialized_state)
            })
            .await;

            match handle {
                Ok(result) => match result {
                    Ok(_) => {
                        // Once persisted, update the status on the finalizers as delivered thus acking the
                        // original events. RocksDB will provide the durability of the aggregate events.
                        for reduce in self.reduce_merge_states.values_mut() {
                            reduce
                                .metadata
                                .take_finalizers()
                                .update_status(vector_lib::event::EventStatus::Delivered);
                        }

                        debug!("MezmoReduce: state persisted");
                    }
                    Err(err) => {
                        error!("MezmoReduce: failed to persist state: {}", err);
                    }
                },
                Err(err) => {
                    error!("MezmoReduce: failed to execute persistence task: {}", err)
                }
            }
        }
    }
}

impl TaskTransform<Event> for MezmoReduce {
    fn transform(
        mut self: Box<Self>,
        mut input_rx: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>>
    where
        Self: 'static,
    {
        let poll_period = self.flush_period;
        let mut flush_stream = tokio::time::interval(poll_period);
        let mut state_persistence_interval =
            tokio::time::interval(Duration::from_millis(self.state_persistence_tick_ms));

        let flush_on_shutdown = match &self.state_persistence {
            Some(_) => {
                debug!("MezmoReduce: state persistence enabled, state will not flush on shutdown");
                false
            }
            None => {
                debug!("MezmoReduce: state persistence not enabled, state will flush on shutdown");
                true
            }
        };

        Box::pin(
            stream! {
                loop {
                    let mut output = Vec::new();
                    let done = tokio::select! {
                        _ = state_persistence_interval.tick() => {
                            let jitter = rand::rng().random_range(0..=self.state_persistence_max_jitter_ms);
                            tokio::time::sleep(Duration::from_millis(jitter)).await;
                            self.persist_state().await;
                            false
                        },
                        _ = flush_stream.tick() => {
                            self.flush_into(&mut output);
                            false
                        }
                        maybe_event = input_rx.next() => {
                            match maybe_event {
                                None => {
                                    if flush_on_shutdown {
                                        self.flush_all_into(&mut output);
                                    }
                                    true
                                }
                                Some(event) => {
                                    self.transform_one(&mut output, event);
                                    false
                                }
                            }
                        }
                    };

                    yield stream::iter(output.into_iter());

                    if done {
                        self.persist_state().await;
                        break
                    }
                }
            }
            .flatten(),
        )
    }
}

// Handles loading initial state from persistent storage, returning an appropriate
// default value if the state is not found or cannot be deserialized.
#[allow(clippy::borrowed_box)]
fn load_initial_state(
    state_persistence: &Arc<dyn PersistenceConnection>,
) -> (HashMap<Discriminant, ReduceState>, HashMap<String, String>) {
    match state_persistence.get(STATE_PERSISTENCE_KEY) {
        Ok(state) => match state {
            Some(state) => match serde_json::from_str::<PersistedState>(&state) {
                Ok(persisted_state) => {
                    debug!("MezmoReduce: existing state found");
                    persisted_state.to_runtime_state()
                }
                Err(err) => {
                    error!(
                        "Failed to deserialize state from persistence: {}, component_id",
                        err
                    );
                    (HashMap::new(), HashMap::new())
                }
            },
            None => {
                debug!("MezmoReduce: no existing state found");
                (HashMap::new(), HashMap::new())
            }
        },
        Err(err) => {
            error!(
                "Failed to load state from persistence: {}, component_id",
                err
            );
            (HashMap::new(), HashMap::new())
        }
    }
}

pub fn get_root_property_name_from_path(
    path_key: &KeyString,
    name: &str,
    error_when_nested: bool,
) -> Option<String> {
    parse_target_path(path_key).map_or_else(
        |e| {
            warn!(
                "Could not extract root property from {} path {}: {}",
                name, path_key, e
            );
            if path_key.is_empty() {
                None
            } else {
                Some(path_key.to_string())
            }
        },
        |target_path| {
            let mut field_count = target_path.path.segments.len();
            if field_count == 0 {
                None
            } else {
                let mut segments = target_path.path.segments;
                // Ignore schema prefixes, which are valid VRL but not relevant to reduce
                if let Some(OwnedSegment::Field(first_element)) = segments.first() {
                    if first_element.as_str() == log_schema().message_key().unwrap().to_string().as_str() {
                        segments.remove(0);
                        field_count = segments.len();
                    }
                }
                match segments.first() {
                    Some(OwnedSegment::Field(root_field)) => {
                        if field_count == 1 {
                            // Normal result - only a root-level path lookup was provided
                            Some(root_field.to_string())
                        } else if error_when_nested {
                            // Told to reject nested path properties
                            error!("Nested path provided for {} path {} when only root-level paths are accepted", name, path_key);
                            None
                        } else {
                            // Nesting found, but told not to error, so return just the root-level field
                            Some(root_field.to_string())
                        }
                    },
                    Some(not_supported) => {
                        warn!("OwnedSegment type not supported {:?} for {}", not_supported, name);
                        None
                    },
                    None => {
                        // This should only happen if the array index for `get` is out of bounds.
                        // This can happen iff the key is "message", leaving no other array elements
                        warn!("Cannot get the zeroith target path element. Out of bounds? {:?}", segments);
                        None
                    },
                }
            }
        }
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::{LogEvent, Value};
    use crate::mezmo::persistence::RocksDBPersistenceConnection;
    use crate::test_util::components::assert_transform_compliance;
    use crate::transforms::test::create_topology;
    use assay::assay;
    use chrono::{Duration, Utc};
    use futures_util::FutureExt;
    use mezmo::MezmoContext;
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::mpsc;
    use tokio::time::sleep;
    use tokio_stream::wrappers::ReceiverStream;
    use vector_lib::btreemap;
    use vector_lib::config::log_schema;
    use vector_lib::finalization::{BatchNotifier, BatchStatus, EventFinalizer, EventFinalizers};

    fn test_mezmo_context() -> MezmoContext {
        MezmoContext::try_from(format!(
            "v1:mezmo-reduce:transform:component_id:pipeline_id:{}",
            uuid::Uuid::new_v4()
        ))
        .unwrap()
    }

    async fn create_test_reduce_with_persistence() -> (MezmoReduce, tempfile::TempDir) {
        let temp_dir = tempdir().expect("Could not create temp dir");
        let state_persistence_base_path = temp_dir.path().to_str().unwrap();

        let config_str = format!(
            r#"
                expire_after_ms = 30000
                flush_period_ms = 10000
                group_by = ["request_id"]
                state_persistence_base_path = "{}"
                state_persistence_tick_ms = 100

                [merge_strategies]
                counter = "sum"
                message = "concat"
                "#,
            state_persistence_base_path
        );

        let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
        let mezmo_ctx = test_mezmo_context();

        let ctx = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };
        let reduce = MezmoReduce::new(&reduce_config, &ctx).unwrap();

        (reduce, temp_dir)
    }

    fn create_test_event(request_id: &str, counter: i64, message: &str) -> Event {
        let mut log_event = LogEvent::default();
        log_event.insert("request_id", request_id);
        log_event.insert("message.counter", counter);
        log_event.insert("message.message", message);
        log_event.insert("message.timestamp", Utc::now());
        Event::Log(log_event)
    }

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<MezmoReduceConfig>();
    }

    #[tokio::test]
    async fn mezmo_reduce_default_behavior_uses_expire_after() {
        // This test is different than the others since it uses `tx.send()` to send each payload, and thus
        // it will follow guidelines for ending the test based on `expire_after_ms`.  The other tests call
        // `transform_events` manually which ends the test after they're consumed.

        let reduce_config = toml::from_str::<MezmoReduceConfig>("expire_after_ms = 3000").unwrap();

        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);

            // _topology isn't used but need to be bound to a name so it's not dropped before the
            // rest of the test can run.
            let (_topology, mut out) =
                create_topology(ReceiverStream::new(rx), reduce_config).await;

            let start_date = Utc::now();
            let end_date = start_date + Duration::seconds(60);

            let mut e_1 = LogEvent::default();
            e_1.insert(
                "message",
                BTreeMap::from([
                    ("my_num".into(), Value::from(10)),
                    ("my_string".into(), Value::from("first string")),
                    ("my_date".into(), Value::from(start_date)),
                ]),
            );
            e_1.insert("timestamp", Value::from(start_date));

            let mut e_2 = LogEvent::default();
            e_2.insert(
                "message",
                BTreeMap::from([
                    ("my_num".into(), Value::from(10)),
                    ("my_string".into(), Value::from("second string")),
                    ("e2_string".into(), Value::from("Added in the second event")),
                ]),
            );
            e_2.insert("timestamp", Value::from(Utc::now()));

            let mut e_3 = LogEvent::default();
            e_3.insert(
                "message",
                BTreeMap::from([
                    ("my_num".into(), Value::from(10)),
                    ("my_string".into(), Value::from("third string")),
                    (
                        "e2_string".into(),
                        Value::from("Ignored, cause it's added in the THIRD event"),
                    ),
                    ("my_date".into(), Value::from(end_date)),
                ]),
            );
            e_3.insert("timestamp", Value::from(end_date));

            for event in vec![e_1.into(), e_2.into(), e_3.into()] {
                tx.send(event).await.unwrap();
            }

            let output_1 = out.recv().await.unwrap().into_log();
            assert_eq!(output_1["message.my_num"], 30.into());
            assert_eq!(output_1["message.my_string"], "first string".into());
            assert_eq!(
                output_1["message.e2_string"],
                "Added in the second event".into()
            );
            assert_eq!(output_1["message.my_date"], start_date.into());
            assert_eq!(output_1["message.my_date_end"], end_date.into());

            // The top-level timestamp field should use the default strategy
            assert_eq!(output_1["timestamp"], start_date.into());
            assert_eq!(output_1["timestamp_end"], end_date.into());
        })
        .await;
    }

    #[tokio::test]
    async fn mezmo_reduce_from_end_condition() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
    [ends_when]
      type = "vrl"
      source = "exists(.test_end)"
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
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("second string")),
                ("e2_string".into(), Value::from("Added in the second event")),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("third string")),
                (
                    "e2_string".into(),
                    Value::from("Ignored, cause it's added in the THIRD event"),
                ),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("fourth string")),
                ("test_end".into(), Value::from("first end")),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("fifth string")),
                ("test_end".into(), Value::from("second end")),
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
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("second string")),
                ("e2_string".into(), Value::from("Added in the second event")),
                (
                    "start_new_here".into(),
                    Value::from(false), // Should NOT start a new one because it's false
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("third string")),
                (
                    "e2_string".into(),
                    Value::from("Ignored, cause it's added in the THIRD event"),
                ),
                ("start_new_here".into(), Value::from(true)),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("fourth string")),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("fifth string")),
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
                ("request_id".into(), Value::from("1")),
                ("my_num".into(), Value::from(10)),
                ("my_string".into(), Value::from("first string")),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("2")),
                ("my_num".into(), Value::from(11)),
                ("my_string".into(), Value::from("second string")),
                (
                    "other_string".into(),
                    Value::from("Added in the second event"),
                ),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("1")),
                ("my_num".into(), Value::from(12)),
                ("my_string".into(), Value::from("third string")),
                (
                    "other_string".into(),
                    Value::from("Added in the third event"),
                ),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("2")),
                ("my_num".into(), Value::from(13)),
                ("my_string".into(), Value::from("Ignore this string")),
                (
                    "other_string".into(),
                    Value::from("Ignore this string also"),
                ),
                ("stop_here".into(), Value::from(true)),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("1")),
                ("my_num".into(), Value::from(14)),
                ("my_string".into(), Value::from("fifth string")),
                ("stop_here".into(), Value::from(true)),
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
          type = "vrl"
          source = "exists(.test_end)"
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
                ("request_id".into(), Value::from("1")),
                ("foo".into(), Value::from("first foo")),
                ("bar".into(), Value::from("first bar")),
                ("baz".into(), Value::from(2)),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("1")),
                ("foo".into(), Value::from("second foo")),
                ("bar".into(), Value::from(2)),
                ("baz".into(), Value::from("not number")),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), Value::from("1")),
                ("foo".into(), Value::from(10)),
                ("bar".into(), Value::from("third bar")),
                ("baz".into(), Value::from(3)),
                ("test_end".into(), Value::from("yep")),
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
          type = "vrl"
          source = "exists(.test_end)"
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
                ("request_id".into(), "1".into()),
                ("counter".into(), 1.into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert("message", BTreeMap::from([("counter".into(), 2.into())]));

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "1".into()),
                ("counter".into(), 3.into()),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "1".into()),
                ("counter".into(), 4.into()),
                ("test_end".into(), "yep".into()),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("counter".into(), 5.into()),
                ("extra_field".into(), "value1".into()),
                ("test_end".into(), "yep".into()),
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
    async fn max_events_0() {
        let reduce_config = toml::from_str::<MezmoReduceConfig>(
            r#"
group_by = [ "id" ]
merge_strategies.id = "retain"
merge_strategies.message = "array"
max_events = 0
            "#,
        );

        match reduce_config {
            Ok(_conf) => unreachable!("max_events=0 should be rejected."),
            Err(err) => assert!(err
                .to_string()
                .contains("invalid value: integer `0`, expected a nonzero usize")),
        }
    }

    #[tokio::test]
    async fn max_events_1() {
        let reduce_config = toml::from_str::<MezmoReduceConfig>(
            r#"
group_by = [ "id" ]
merge_strategies.id = "retain"
merge_strategies.message = "array"
max_events = 1
            "#,
        )
        .unwrap();
        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let (topology, mut out) = create_topology(ReceiverStream::new(rx), reduce_config).await;

            let mut e_1 = LogEvent::default();
            e_1.insert("message", BTreeMap::from([("id".into(), Value::from("1"))]));

            let mut e_2 = LogEvent::default();
            e_2.insert("message", BTreeMap::from([("id".into(), Value::from("1"))]));

            let mut e_3 = LogEvent::default();
            e_3.insert("message", BTreeMap::from([("id".into(), Value::from("1"))]));

            for event in vec![e_1.into(), e_2.into(), e_3.into()] {
                tx.send(event).await.unwrap();
            }

            let output_1 = out.recv().await.unwrap().into_log();
            assert_eq!(output_1["message.id"], "1".into());

            let output_2 = out.recv().await.unwrap().into_log();
            assert_eq!(output_2["message.id"], "1".into());

            let output_3 = out.recv().await.unwrap().into_log();
            assert_eq!(output_3["message.id"], "1".into());

            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
        })
        .await;
    }

    #[tokio::test]
    async fn max_events_3() {
        let reduce_config = toml::from_str::<MezmoReduceConfig>(
            r#"
group_by = [ "request_id" ]
merge_strategies.text = "array"
max_events = 3
            "#,
        )
        .unwrap();

        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);
            let (topology, mut out) = create_topology(ReceiverStream::new(rx), reduce_config).await;

            let mut e_1 = LogEvent::default();
            e_1.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 1")),
                ]),
            );

            let mut e_2 = LogEvent::default();
            e_2.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 2")),
                ]),
            );

            let mut e_3 = LogEvent::default();
            e_3.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 3")),
                ]),
            );

            let mut e_4 = LogEvent::default();
            e_4.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 4")),
                ]),
            );

            let mut e_5 = LogEvent::default();
            e_5.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 5")),
                ]),
            );

            let mut e_6 = LogEvent::default();
            e_6.insert(
                "message",
                BTreeMap::from([
                    ("request_id".into(), Value::from("1")),
                    ("text".into(), Value::from("test 6")),
                ]),
            );

            for event in vec![
                e_1.into(),
                e_2.into(),
                e_3.into(),
                e_4.into(),
                e_5.into(),
                e_6.into(),
            ] {
                tx.send(event).await.unwrap();
            }

            let output_1 = out.recv().await.unwrap().into_log();
            assert_eq!(
                output_1["message.text"],
                vec!["test 1", "test 2", "test 3"].into()
            );

            let output_2 = out.recv().await.unwrap().into_log();
            assert_eq!(
                output_2["message.text"],
                vec!["test 4", "test 5", "test 6"].into()
            );

            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
        })
        .await
    }

    #[tokio::test]
    async fn mezmo_reduce_arrays_in_payload() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        group_by = [ "request_id" ]

        merge_strategies.foo = "array"
        merge_strategies.bar = "concat"

        [ends_when]
          type = "vrl"
          source = "exists(.test_end)"
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
                ("request_id".into(), "1".into()),
                ("foo".into(), json!([1, 3]).into()),
                ("bar".into(), json!([1, 3]).into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "2".into()),
                ("foo".into(), json!([2, 4]).into()),
                ("bar".into(), json!([2, 4]).into()),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "1".into()),
                ("foo".into(), json!([5, 7]).into()),
                ("bar".into(), json!([5, 7]).into()),
            ]),
        );

        let mut e_4 = LogEvent::default();
        e_4.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "1".into()),
                ("foo".into(), json!("done").into()),
                ("bar".into(), json!("done").into()),
                ("test_end".into(), "yep".into()),
            ]),
        );

        let mut e_5 = LogEvent::default();
        e_5.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "2".into()),
                ("foo".into(), json!([6, 8]).into()),
                ("bar".into(), json!([6, 8]).into()),
            ]),
        );

        let mut e_6 = LogEvent::default();
        e_6.insert(
            "message",
            BTreeMap::from([
                ("request_id".into(), "2".into()),
                ("foo".into(), json!("done").into()),
                ("bar".into(), json!("done").into()),
                ("test_end".into(), "yep".into()),
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
          '."ts"' = "%Y-%m-%d %H:%M:%S"
          ".epoch" = "%s"
          ".epoch_str" = "%s"

        [ends_when]
          type = "vrl"
          source = "exists(.test_end)"
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
                ("ts".into(), "2014-11-28 12:00:09".into()),
                ("epoch".into(), 1671134262.into()),
                ("epoch_str".into(), "1671134262".into()),
            ]),
        );

        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("ts".into(), "2014-11-28 13:00:09".into()),
                ("epoch".into(), 1671134263.into()),
                ("epoch_str".into(), "1671134263".into()),
            ]),
        );

        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("ts".into(), "2014-11-28 14:00:09".into()),
                ("epoch".into(), 1671134264.into()),
                ("epoch_str".into(), "1671134264".into()),
                ("test_end".into(), "yup".into()),
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

    #[tokio::test]
    async fn mezmo_reduce_merge_strategies_with_special_paths() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        [merge_strategies]
          "some-retain-field" = "retain"
          "some!array-field" = "array"
          "concat-me!" = "concat"
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
                ("some-retain-field".into(), "one".into()),
                ("some!array-field".into(), "four".into()),
                ("concat-me!".into(), "seven".into()),
            ]),
        );
        let mut e_2 = LogEvent::default();
        e_2.insert(
            "message",
            BTreeMap::from([
                ("some-retain-field".into(), "two".into()),
                ("some!array-field".into(), "five".into()),
                ("concat-me!".into(), "eight".into()),
            ]),
        );
        let mut e_3 = LogEvent::default();
        e_3.insert(
            "message",
            BTreeMap::from([
                ("some-retain-field".into(), "three".into()),
                ("some!array-field".into(), "six".into()),
                ("concat-me!".into(), "nine".into()),
            ]),
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(output_1["message.\"some-retain-field\""], "three".into());
        assert_eq!(
            output_1["message.\"some!array-field\""],
            Value::Array(vec!["four".into(), "five".into(), "six".into()])
        );
        assert_eq!(
            output_1["message.\"concat-me!\""],
            "seven eight nine".into()
        );
    }

    #[assay(
        env = [
          ("REDUCE_BYTE_THRESHOLD_PER_STATE", "30"),
        ]
      )]
    async fn mezmo_reduce_state_exceeds_threshold() {
        // Since `flush_into()` creates the output event, and it's ONLY called via a tokio interval in `transform()`,
        // we must test that code path using `assert_transform_compliance`.
        // Set `flush_period_ms` to fire `flush_into()` regularly, and use `expire_after_ms` to end the test.
        let reduce_config = toml::from_str::<MezmoReduceConfig>(
            r#"
                flush_period_ms = 50
                expire_after_ms = 2000

                [merge_strategies]
                "key1" = "array"
            "#,
        )
        .unwrap();

        assert_transform_compliance(async move {
            let (tx, rx) = mpsc::channel(1);

            let (_topology, mut out_stream) =
                create_topology(ReceiverStream::new(rx), reduce_config).await;
            let message_key_path = log_schema().message_key_target_path().unwrap();
            let message_key = log_schema().message_key().unwrap().to_string();

            let mut e_1 = LogEvent::default();
            e_1.insert(
                message_key_path,
                btreemap! {
                    "key1" => "first one",
                    "key2" => "first"
                },
            );
            let mut e_2 = LogEvent::default();
            e_2.insert(
                message_key_path,
                btreemap! {
                    "key1" => "second one",
                    "key2" => "NOPE"
                },
            );
            let mut e_3 = LogEvent::default();
            e_3.insert(
                // This will cause the threshold to be exceeded
                message_key_path,
                btreemap! {
                    "key1" => "and now you're too big!",
                    "key2" => "NEIGH"
                },
            );
            let mut e_4 = LogEvent::default();
            e_4.insert(
                // This will be a new event
                message_key_path,
                btreemap! {
                    "key1" => "a new reduce event",
                    "key2" => "yep"
                },
            );

            for event in vec![e_1.into(), e_2.into(), e_3.into(), e_4.into()] {
                tx.send(event).await.unwrap();
                // Space out the events so that the internal timer can call `flush_into` which does the size checking.
                sleep(tokio::time::Duration::from_millis(100)).await;
            }

            let output_1 = out_stream.recv().await.unwrap().into_log();
            assert_eq!(
                output_1,
                LogEvent::from(btreemap! {
                    message_key.as_str() => btreemap! {
                        "key1" => json!(["first one", "second one", "and now you're too big!"]),
                        "key2" => "first",
                    }
                })
            );

            let output_2 = out_stream.recv().await.unwrap().into_log();
            assert_eq!(
                output_2,
                LogEvent::from(btreemap! {
                    message_key.as_str() => btreemap! {
                        "key1" => json!(["a new reduce event"]),
                        "key2" => "yep",
                    }
                })
            );
        })
        .await;
    }

    #[assay(
        env = [
          ("REDUCE_BYTE_THRESHOLD_ALL_STATES", "30"),
        ]
      )]
    async fn mezmo_reduce_all_states_total_exceeds_threshold() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
                group_by = [ "request_id" ]

                [merge_strategies]
                "key1" = "array"
            "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key_path = log_schema().message_key_target_path().unwrap();
        let message_key = log_schema().message_key().unwrap().to_string();

        // Different request ids will cause multiple states since each unique `id` is a discriminant and state
        let mut e_1 = LogEvent::default();
        e_1.insert(
            message_key_path,
            btreemap! {
                "request_id" => "1",
                "key1" => "one",
            },
        );
        let mut e_2 = LogEvent::default();
        e_2.insert(
            message_key_path,
            btreemap! {
                "request_id" => "1",
                "key1" => "two",
            },
        );
        let mut e_3 = LogEvent::default();
        e_3.insert(
            message_key_path,
            btreemap! {
                "request_id" => "2",
                "key1" => "one",
            },
        );
        let mut e_4 = LogEvent::default();
        e_4.insert(
            message_key_path,
            btreemap! {
                "request_id" => "2",
                "key1" => "two",
            },
        );
        let mut e_5 = LogEvent::default();
        e_5.insert(
            message_key_path,
            btreemap! {
                "request_id" => "2",
                "key1" => "aaaaaaaaaaaand we're way too long now",
            },
        );

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "key1" => json!(["one", "two"]),
                    "request_id" => "1",
                }
            })
        );

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "key1" => json!(["one", "two", "aaaaaaaaaaaand we're way too long now"]),
                    "request_id" => "2",
                }
            })
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_group_by_number_field() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        group_by = ["status"]

        [merge_strategies]
            "method" = "array"
            "status" = "sum" # Should be IGNORED
        "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "GET",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "POST",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "POST",
            },
        });

        let e_4 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 2,
                "method" => "POST",
            },
        });

        let e_5 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 2,
                "method" => "POST",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "status" => 1,
                    "method" => json!(["GET", "POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "status" => 2,
                    "method" => json!(["POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_group_by_number_field_using_dot_notation() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        group_by = [".status"]

        [merge_strategies]
            "method" = "array"
            "status" = "sum" # Should be IGNORED
        "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "GET",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "POST",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 1,
                "method" => "POST",
            },
        });

        let e_4 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 2,
                "method" => "POST",
            },
        });

        let e_5 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "status" => 2,
                "method" => "POST",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "status" => 1,
                    "method" => json!(["GET", "POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "status" => 2,
                    "method" => json!(["POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_group_by_number_field_with_special_chars() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        group_by = ['."my-status"']

        [merge_strategies]
            "method" = "array"
            "my-status" = "sum" # Should be IGNORED
        "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "my-status" => 1,
                "method" => "GET",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "my-status" => 1,
                "method" => "POST",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "my-status" => 1,
                "method" => "POST",
            },
        });

        let e_4 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "my-status" => 2,
                "method" => "POST",
            },
        });

        let e_5 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "my-status" => 2,
                "method" => "POST",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "my-status" => 1,
                    "method" => json!(["GET", "POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "my-status" => 2,
                    "method" => json!(["POST", "POST"])
                }
            }),
            "group_by did not apply merge strategies to its fields"
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_group_by_with_nested_object() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
        group_by = ['."user.data"."user_ids"[0]']

        [merge_strategies]
            "method" = "array"
        "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "user_ids" => json!([1]),
                    "some_key" => "first",
                    "my_int" => 55
                },
                "method" => "GET",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "user_ids" => json!([1]),
                    "some_key" => "second",
                    "my_int" => 1
                },
                "method" => "POST",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "user_ids" => json!([1]),
                    "some_key" => "third",
                    "my_int" => 2
                },
                "method" => "POST",
            },
        });

        let e_4 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "user_ids" => json!([2]),
                    "some_key" => "first",
                    "my_int" => 66
                },
                "method" => "POST",
            },
        });

        let e_5 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "user_ids" => json!([2]),
                    "some_key" => "second",
                    "my_int" => 4
                },
                "method" => "POST",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into(), e_4.into(), e_5.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        // Nested objects are NOT reduced, so the entire object, although used in group_by, should be a Discard
        // strategy where only the first value is kept in its entirety.
        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "user.data" => btreemap! {
                        "user_ids" => json!([1]),
                        "some_key" => "first",
                        "my_int" => 55
                    },
                    "method" => json!(["GET", "POST", "POST"])
                }
            }),
            "group_by worked using a nested structure and field paths"
        );
        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "user.data" => btreemap! {
                        "user_ids" => json!([2]),
                        "some_key" => "first",
                        "my_int" => 66
                    },
                    "method" => json!(["POST", "POST"])
                }
            }),
            "group_by worked using a nested structure and field paths"
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_finalizers_are_handled_correctly() {
        let reduce = toml::from_str::<MezmoReduceConfig>("")
            .unwrap()
            .build(&TransformContext::default())
            .await
            .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let mut e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "num" => 1,
                "method" => "GET",
            },
        });

        // Add a finalizer to be carried through to `flush()`. We'll use the receiver to make sure
        // this particular finalizer was carried through the reduce process.
        let (batch, receiver) = BatchNotifier::new_with_receiver();
        let finalizers = EventFinalizers::new(EventFinalizer::new(batch));
        e_1.metadata_mut().merge_finalizers(finalizers);
        let inputs: Vec<Event> = vec![e_1.into()];

        let in_stream = Box::pin(stream::iter(inputs));
        let out_stream = reduce.transform_events(in_stream);

        // Since ownership changes too many times, we cannot test that the finalizers are "the same" using memory addresses.
        // Instead, we'll poll the receiver for status updates which should be fired by dropping `res`
        let res: Vec<_> = out_stream
            .take_until(sleep(tokio::time::Duration::from_millis(2_000)))
            .collect()
            .await;

        assert_eq!(res.len(), 1, "Result count is correct");
        drop(res);

        // Turn the receiving channel into a stream and take elements from it for 500 ms, collecting into vec
        let res: Vec<_> = receiver
            .into_stream()
            .take_until(sleep(tokio::time::Duration::from_millis(500)))
            .collect()
            .await;

        assert_eq!(
            res.len(),
            1,
            "Finalizer sent a message through the receiver"
        );
        assert_eq!(res[0], BatchStatus::Delivered, "Batch status is delivered");
    }

    #[tokio::test]
    async fn mezmo_reduce_merge_strategies_with_path_notation() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
            [merge_strategies]
                ".method" = "array"
                '."user.data"' = "retain"
                ".user.data.IGNORED[0]" = "retain"
            "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "some_key" => "first",
                    "my_int" => 55
                },
                "method" => "GET",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "some_key" => "second",
                    "my_int" => 1
                },
                "method" => "POST",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "some_key" => "third",
                    "my_int" => 2
                },
                "method" => "POST",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "user.data" => btreemap! {
                        "some_key" => "third",
                        "my_int" => 2
                    },
                    "method" => json!(["GET", "POST", "POST"])
                }
            }),
            "merge_strategies works with field paths"
        );
    }

    #[tokio::test]
    async fn mezmo_reduce_can_handle_message_schema_prefix() {
        let reduce = toml::from_str::<MezmoReduceConfig>(
            r#"
            group_by = ['message."user.data".id']

            [date_formats]
                "message.epoch" = "%s"

            [merge_strategies]
                "message.str" = "array"
            "#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();
        let reduce = reduce.into_task();
        let message_key = log_schema().message_key().unwrap().to_string();

        let e_1 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "id" => 1
                },
                "epoch" => "1689077395229",
                "num" => 5,
                "str" => "one",
            },
        });

        let e_2 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "id" => 2
                },
                "epoch" => "1689077418279",
                "num" => 10,
                "str" => "two",
            },
        });

        let e_3 = LogEvent::from(btreemap! {
            message_key.as_str() => btreemap! {
                "user.data" => btreemap! {
                    "id" => 1
                },
                "epoch" => "1689077430135",
                "num" => 15,
                "str" => "three",
            },
        });

        let inputs = vec![e_1.into(), e_2.into(), e_3.into()];
        let in_stream = Box::pin(stream::iter(inputs));
        let mut out_stream = reduce.transform_events(in_stream);

        let output_1 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_1,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "user.data" => btreemap! {
                        "id" => 1
                    },
                    "epoch" => "1689077395229",
                    "epoch_end" => "1689077430135",
                    "num" => 20,
                    "str" => vec!["one", "three"],
                }
            }),
            "group_by the first id"
        );

        let output_2 = out_stream.next().await.unwrap().into_log();
        assert_eq!(
            output_2,
            LogEvent::from(btreemap! {
                message_key.as_str() => btreemap! {
                    "user.data" => btreemap! {
                        "id" => 2
                    },
                    "epoch" => "1689077418279",
                    "epoch_end" => "1689077418279",
                    "num" => 10,
                    "str" => vec!["two"],
                }
            }),
            "group_by the second id"
        );
    }

    #[test]
    fn mezmo_reduce_test_get_root_property_name_from_path() {
        let test_cases = [
            ("".into(), None, false, "empty string"),
            (".".into(), None, false, "dot"),
            (
                "does-not-parse".into(),
                Some("does-not-parse".to_string()),
                false,
                "invalid characters",
            ),
            (
                "nope!".into(),
                Some("nope!".to_string()),
                false,
                "invalid characters",
            ),
            (
                "yep".into(),
                Some("yep".to_string()),
                false,
                "root-level property given without dot-notation",
            ),
            (
                ".yep".into(),
                Some("yep".to_string()),
                false,
                "dot-notated parent",
            ),
            (
                ".yep.nested".into(),
                Some("yep".to_string()),
                false,
                "returns parent when nested errors is false",
            ),
            (
                ".yep.nested".into(),
                None,
                true,
                "nested errors true returns none",
            ),
            (
                ".\"special-chars\".nested".into(),
                Some("special-chars".to_string()),
                false,
                "quoting special chars returns parent when error_when_nested is false",
            ),
            (
                ".\"special-chars\".nested".into(),
                None,
                true,
                "quoting special chars returns None when error_when_nested",
            ),
            (
                "thing[1].nested".into(),
                Some("thing".to_string()),
                false,
                "array path returns root when error_when_nested is false",
            ),
            (
                "thing[1].nested".into(),
                None,
                true,
                "array path returns None when error_when_nested is true",
            ),
            (
                "message.not_nested".into(),
                Some("not_nested".to_string()),
                true,
                "message schema prefix is ignored and root-level property returned",
            ),
            (
                "message.thing.nested".into(),
                None,
                true,
                "message schema prefix is ignored and deeply-nested detected",
            ),
            (
                "message.thing.nested".into(),
                Some("thing".to_string()),
                false,
                "message schema prefix is ignored and root-level property returned",
            ),
            (
                "message".into(),
                None,
                true,
                "message schema without a root-level property returns None",
            ),
            (
                "(root | message).field_name".into(),
                Some("(root | message).field_name".into()),
                true,
                "Coalescing VRL has been removed in vector 0.39",
            ),
        ];

        for (input, expected, error_when_nested, desc) in test_cases {
            let result = get_root_property_name_from_path(&input, desc, error_when_nested);
            assert_eq!(
                result, expected,
                "Failed item: {desc}, expected: {expected:?}",
            );
        }
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_persist_state_saves_to_rocksdb() {
        let (mut reduce, _temp_dir) = create_test_reduce_with_persistence().await;

        // Add some events to create state
        let events = vec![
            create_test_event("req-1", 10, "first"),
            create_test_event("req-1", 20, "second"),
            create_test_event("req-2", 5, "other"),
        ];

        for event in events {
            let discriminant =
                Discriminant::from_log_event(event.as_log(), &["request_id".to_string()]);
            let log_event = event.into_log();
            reduce.push_or_new_reduce_state(log_event.clone(), log_event, discriminant);
        }

        // Verify we have state before persistence
        assert_eq!(reduce.reduce_merge_states.len(), 2);

        // Persist the state
        reduce.persist_state().await;

        // Verify state was written to RocksDB
        if let Some(state_persistence) = &reduce.state_persistence {
            let stored_data = state_persistence.get(STATE_PERSISTENCE_KEY).unwrap();
            assert!(stored_data.is_some());

            // Deserialize and verify the stored state
            let stored_state: PersistedState = serde_json::from_str(&stored_data.unwrap()).unwrap();

            assert_eq!(stored_state.merge_state.len(), 2);
            // Check that we have discriminants that contain the expected request IDs
            let has_req1 = stored_state.merge_state.iter().any(|(values, _)| {
                values.iter().any(|opt_val| {
                    if let Some(val) = opt_val {
                        val.to_string_lossy().contains("req-1")
                    } else {
                        false
                    }
                })
            });
            let has_req2 = stored_state.merge_state.iter().any(|(values, _)| {
                values.iter().any(|opt_val| {
                    if let Some(val) = opt_val {
                        val.to_string_lossy().contains("req-2")
                    } else {
                        false
                    }
                })
            });
            assert!(has_req1);
            assert!(has_req2);
        } else {
            panic!("State persistence should be configured");
        }
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_load_initial_state_from_rocksdb() {
        let temp_dir = tempdir().expect("Could not create temp dir");
        let state_persistence_base_path = temp_dir.path().to_str().unwrap();

        // Store mezmo_ctx to reuse for the second instance
        let mezmo_ctx = {
            // Create the first instance with the same base path
            let config_str = format!(
                r#"
                    expire_after_ms = 30000
                    flush_period_ms = 10000
                    group_by = ["request_id"]
                    state_persistence_base_path = "{}"
                    state_persistence_tick_ms = 100

                    [merge_strategies]
                    counter = "sum"
                    message = "concat"
                    "#,
                state_persistence_base_path
            );

            let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
            let mezmo_ctx = test_mezmo_context();
            let ctx = TransformContext {
                mezmo_ctx: Some(mezmo_ctx.clone()),
                ..Default::default()
            };
            let mut reduce = MezmoReduce::new(&reduce_config, &ctx).unwrap();

            // Add some events
            let events = vec![
                create_test_event("req-1", 15, "initial"),
                create_test_event("req-2", 25, "another"),
            ];

            for event in events {
                let discriminant =
                    Discriminant::from_log_event(event.as_log(), &["request_id".to_string()]);
                let log_event = event.into_log();
                reduce.push_or_new_reduce_state(log_event.clone(), log_event, discriminant);
            }

            // Add a custom date kind for testing
            reduce.mezmo_metadata.save_date_kind("timestamp", "integer");

            // Persist the state
            reduce.persist_state().await;

            // Verify persistence completed by checking the database directly
            if let Some(state_persistence) = &reduce.state_persistence {
                let stored_data = state_persistence.get(STATE_PERSISTENCE_KEY).unwrap();
                assert!(stored_data.is_some(), "State was not persisted");
            }

            mezmo_ctx
        };

        // Now create a new instance that should load the persisted state
        let config_str = format!(
            r#"
                expire_after_ms = 30000
                flush_period_ms = 10000
                group_by = ["request_id"]
                state_persistence_base_path = "{}"
                state_persistence_tick_ms = 100

                [merge_strategies]
                counter = "sum"
                message = "concat"
                "#,
            state_persistence_base_path
        );

        let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
        let context = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };
        let new_reduce = MezmoReduce::new(&reduce_config, &context).unwrap();

        // Verify the state was loaded
        assert_eq!(new_reduce.reduce_merge_states.len(), 2);

        // Verify date kinds were loaded
        let loaded_date_kind = new_reduce.mezmo_metadata.get_date_kind("timestamp");
        assert_eq!(loaded_date_kind, "integer");

        // Verify the discriminants match what we expect
        let discriminants: Vec<String> = new_reduce
            .reduce_merge_states
            .keys()
            .map(|d| d.to_string())
            .collect();

        assert!(discriminants.iter().any(|d| d.contains("req-1")));
        assert!(discriminants.iter().any(|d| d.contains("req-2")));
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_load_initial_state_handles_missing_data() {
        let temp_dir = tempdir().expect("Could not create temp dir");
        let state_persistence_base_path = temp_dir.path().to_str().unwrap();

        // Create a MezmoReduce instance pointing to an empty RocksDB
        let config_str = format!(
            r#"
                expire_after_ms = 30000
                group_by = ["request_id"]
                state_persistence_base_path = "{}"
                "#,
            state_persistence_base_path
        );

        let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
        let mezmo_ctx = test_mezmo_context();

        let ctx = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };
        let reduce = MezmoReduce::new(&reduce_config, &ctx).unwrap();

        // Should have empty state when no persisted data exists
        assert_eq!(reduce.reduce_merge_states.len(), 0);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_load_initial_state_handles_corrupted_data() {
        let temp_dir = tempdir().expect("Could not create temp dir");
        let state_persistence_base_path = temp_dir.path().to_str().unwrap();
        let mezmo_ctx = test_mezmo_context();

        // First, write corrupted data to RocksDB
        {
            let db_path = format!("{}/reduce", state_persistence_base_path);
            let state_persistence =
                Arc::new(RocksDBPersistenceConnection::new(&db_path, &mezmo_ctx).unwrap());

            // Write invalid JSON
            let _ = state_persistence.set(STATE_PERSISTENCE_KEY, "invalid json data");
        }

        // Now try to create a MezmoReduce instance
        let config_str = format!(
            r#"
                expire_after_ms = 30000
                group_by = ["request_id"]
                state_persistence_base_path = "{}"
                "#,
            state_persistence_base_path
        );

        let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
        let context = TransformContext::default();
        let reduce = MezmoReduce::new(&reduce_config, &context).unwrap();

        // Should have empty state when corrupted data is encountered
        assert_eq!(reduce.reduce_merge_states.len(), 0);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_periodic_state_persistence() {
        let (mut reduce, _temp_dir) = create_test_reduce_with_persistence().await;

        // Add some events
        let events = vec![
            create_test_event("req-1", 10, "first"),
            create_test_event("req-1", 20, "second"),
        ];

        for event in events {
            let discriminant =
                Discriminant::from_log_event(event.as_log(), &["request_id".to_string()]);
            let log_event = event.into_log();
            reduce.push_or_new_reduce_state(log_event.clone(), log_event, discriminant);
        }

        // Manually trigger persist_state multiple times to simulate periodic persistence
        reduce.persist_state().await;

        // Add more state
        let event = create_test_event("req-2", 5, "additional");
        let discriminant =
            Discriminant::from_log_event(event.as_log(), &["request_id".to_string()]);
        let log_event = event.into_log();
        reduce.push_or_new_reduce_state(log_event.clone(), log_event, discriminant);

        // Persist again
        reduce.persist_state().await;

        // Verify final state was persisted
        if let Some(state_persistence) = &reduce.state_persistence {
            let stored_data = state_persistence.get(STATE_PERSISTENCE_KEY).unwrap();
            assert!(stored_data.is_some());

            let stored_state: PersistedState = serde_json::from_str(&stored_data.unwrap()).unwrap();

            // Should have both discriminants
            assert_eq!(stored_state.merge_state.len(), 2);
        }
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_combined_state_serialization() {
        // Test the PersistedState structure directly
        #[allow(clippy::mutable_key_type)]
        let mut merge_state = HashMap::new();
        let mut date_kinds_state = HashMap::new();

        // Create a mock ReduceState (this is simplified for testing)
        let test_event = create_test_event("test", 1, "msg");
        let discriminant =
            Discriminant::from_log_event(test_event.as_log(), &["request_id".to_string()]);

        let strategies = IndexMap::new();
        let mezmo_metadata = MezmoMetadata::new(HashMap::new(), HashMap::new());
        let reduce_state = ReduceState::new(
            test_event.as_log().clone(),
            test_event.into_log(),
            &strategies,
            mezmo_metadata,
            &[KeyString::from("request_id")],
        );

        merge_state.insert(discriminant, reduce_state);
        date_kinds_state.insert("timestamp".to_string(), "integer".to_string());

        let combined_state = PersistedState::from_runtime_state(&merge_state, &date_kinds_state);

        // Test serialization
        let serialized = serde_json::to_string(&combined_state).unwrap();
        assert!(serialized.contains("merge_state"));
        assert!(serialized.contains("date_kinds_state"));

        // Test deserialization
        let deserialized: PersistedState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.merge_state.len(), 1);
        assert_eq!(deserialized.date_kinds_state.len(), 1);
        assert_eq!(
            deserialized.date_kinds_state.get("timestamp").unwrap(),
            "integer"
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    async fn test_state_persistence_with_different_merge_strategies() {
        let temp_dir = tempdir().expect("Could not create temp dir");
        let state_persistence_base_path = temp_dir.path().to_str().unwrap();

        // Create shared mezmo_ctx for both instances
        let mezmo_ctx = test_mezmo_context();

        let mut reduce = {
            let config_str = format!(
                r#"
                    expire_after_ms = 30000
                    flush_period_ms = 10000
                    group_by = ["request_id"]
                    state_persistence_base_path = "{}"
                    state_persistence_tick_ms = 100

                    [merge_strategies]
                    counter = "sum"
                    message = "concat"
                    "#,
                state_persistence_base_path
            );

            let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
            let ctx = TransformContext {
                mezmo_ctx: Some(mezmo_ctx.clone()),
                ..Default::default()
            };
            MezmoReduce::new(&reduce_config, &ctx).unwrap()
        };

        // Create events that will exercise different merge strategies
        let mut event1 = LogEvent::default();
        event1.insert("request_id", "req-1");
        event1.insert("message.counter", 10); // sum strategy
        event1.insert("message.message", "first"); // concat strategy

        let mut event2 = LogEvent::default();
        event2.insert("request_id", "req-1");
        event2.insert("message.counter", 15); // sum strategy
        event2.insert("message.message", "second"); // concat strategy

        let events = vec![Event::Log(event1), Event::Log(event2)];

        for event in events {
            let discriminant =
                Discriminant::from_log_event(event.as_log(), &["request_id".to_string()]);
            let log_event = event.into_log();
            reduce.push_or_new_reduce_state(log_event.clone(), log_event, discriminant);
        }

        // Persist the state
        reduce.persist_state().await;

        // Verify persistence completed by checking the database directly
        if let Some(state_persistence) = &reduce.state_persistence {
            let stored_data = state_persistence.get(STATE_PERSISTENCE_KEY).unwrap();
            assert!(stored_data.is_some(), "State was not persisted");
        }

        // Load state into new instance
        let config_str = format!(
            r#"
                expire_after_ms = 30000
                group_by = ["request_id"]
                state_persistence_base_path = "{}"

                [merge_strategies]
                counter = "sum"
                message = "concat"
                "#,
            state_persistence_base_path
        );

        let reduce_config = toml::from_str::<MezmoReduceConfig>(&config_str).unwrap();
        let context = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };
        let new_reduce = MezmoReduce::new(&reduce_config, &context).unwrap();

        // Verify state was loaded with correct merge strategies
        assert_eq!(new_reduce.reduce_merge_states.len(), 1);

        // The state should contain the merged values from both events
        let state = new_reduce.reduce_merge_states.values().next().unwrap();
        assert!(state
            .message_fields
            .contains_key(&KeyString::from("request_id")));
        assert!(state
            .message_fields
            .contains_key(&KeyString::from("message")));
    }

    #[tokio::test]
    async fn test_discriminant_serialization_roundtrip() {
        // Create a HashMap with Discriminant keys
        #[allow(clippy::mutable_key_type)]
        let mut merge_state = HashMap::new();
        let mut date_kinds_state = HashMap::new();

        // Create discriminants with different value types
        let discriminant1 = Discriminant::from_values(vec![
            Some(Value::Bytes("req-1".into())),
            Some(Value::Integer(123)),
        ]);
        let discriminant2 =
            Discriminant::from_values(vec![Some(Value::Bytes("req-2".into())), None]);
        let discriminant3 = Discriminant::from_values(vec![
            Some(Value::Boolean(true)),
            Some(Value::Float(ordered_float::NotNan::new(45.67).unwrap())),
        ]);

        // Create test reduce states
        let test_event = create_test_event("test", 1, "msg");
        let strategies = IndexMap::new();
        let mezmo_metadata = MezmoMetadata::new(HashMap::new(), HashMap::new());

        let reduce_state1 = ReduceState::new(
            test_event.as_log().clone(),
            test_event.clone().into_log(),
            &strategies,
            mezmo_metadata.clone(),
            &[KeyString::from("request_id")],
        );

        let reduce_state2 = ReduceState::new(
            test_event.as_log().clone(),
            test_event.clone().into_log(),
            &strategies,
            mezmo_metadata.clone(),
            &[KeyString::from("request_id")],
        );

        let reduce_state3 = ReduceState::new(
            test_event.as_log().clone(),
            test_event.into_log(),
            &strategies,
            mezmo_metadata,
            &[KeyString::from("request_id")],
        );

        merge_state.insert(discriminant1.clone(), reduce_state1);
        merge_state.insert(discriminant2.clone(), reduce_state2);
        merge_state.insert(discriminant3.clone(), reduce_state3);

        date_kinds_state.insert("timestamp".to_string(), "integer".to_string());

        // Convert to serializable format
        let persisted_state = PersistedState::from_runtime_state(&merge_state, &date_kinds_state);

        // Test serialization
        let serialized = serde_json::to_string(&persisted_state).unwrap();

        // Verify JSON structure is valid
        assert!(serialized.contains("merge_state"));
        assert!(serialized.contains("date_kinds_state"));

        // Test deserialization
        let deserialized: PersistedState = serde_json::from_str(&serialized).unwrap();

        // Convert back to runtime format
        let (restored_merge_state, restored_date_kinds_state) = deserialized.to_runtime_state();

        // Verify all discriminants were restored correctly
        assert_eq!(restored_merge_state.len(), 3);
        assert_eq!(restored_date_kinds_state.len(), 1);

        // Check that discriminants are equivalent (they should be equal due to PartialEq impl)
        assert!(restored_merge_state.contains_key(&discriminant1));
        assert!(restored_merge_state.contains_key(&discriminant2));
        assert!(restored_merge_state.contains_key(&discriminant3));

        // Verify date kinds state
        assert_eq!(
            restored_date_kinds_state.get("timestamp").unwrap(),
            "integer"
        );

        // Verify discriminant values are correctly preserved
        for original_discriminant in merge_state.keys() {
            let found = restored_merge_state
                .keys()
                .any(|restored_discriminant| restored_discriminant == original_discriminant);
            assert!(
                found,
                "Discriminant not found after roundtrip: {:?}",
                original_discriminant
            );
        }
    }
}
