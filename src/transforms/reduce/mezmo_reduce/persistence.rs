use std::collections::HashMap;
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime},
};

use serde::{Deserialize, Serialize};
use vector_lib::event::discriminant::Discriminant;

use crate::mezmo::persistence::PersistenceConnection;
use crate::transforms::reduce::mezmo_reduce::{
    EventMetadata, KeyString, MezmoMetadata, ReduceState, ReduceValueMerger,
    SerializableReduceValueMerger,
};
use crate::{event::Value, Error};

// The key for the state persistence db.
const STATE_PERSISTENCE_KEY: &str = "state";

/// Combined state structure for serialization to RocksDB
/// Uses a JSON-compatible format by converting Discriminant to Vec<Value>
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PersistedState {
    /// Store discriminant map as Vec of (discriminant_values, reduce_state) pairs
    merge_state: Vec<(Vec<Option<Value>>, ReduceState)>,
    date_kinds_state: HashMap<String, String>,
}

impl PersistedState {
    /// Convert from the runtime HashMap format to the serializable format
    #[allow(clippy::mutable_key_type)]
    pub(crate) fn from_runtime_state(
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
    pub(crate) fn to_runtime_state(
        &self,
    ) -> (HashMap<Discriminant, ReduceState>, HashMap<String, String>) {
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

impl Serialize for MezmoMetadata {
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

impl<'de> Deserialize<'de> for MezmoMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
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

impl Serialize for ReduceState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

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
            .map_err(|err| serde::ser::Error::custom(format!("SystemTime error: {err}")))?
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

impl<'de> Deserialize<'de> for ReduceState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
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
            + Duration::new(helper.started_at_epoch_secs, helper.started_at_epoch_nanos);
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

/// Handles loading initial state from persistent storage, returning an appropriate
/// default value if the state is not found or cannot be deserialized.
#[allow(clippy::borrowed_box)]
pub(crate) fn load_initial_state(
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
                    tracing::error!(
                        "Failed to deserialize state from persistence: {}, component_id",
                        err
                    );
                    (HashMap::new(), HashMap::new())
                }
            },
            None => {
                tracing::debug!("MezmoReduce: no existing state found");
                (HashMap::new(), HashMap::new())
            }
        },
        Err(err) => {
            tracing::error!(
                "Failed to load state from persistence: {}, component_id",
                err
            );
            (HashMap::new(), HashMap::new())
        }
    }
}

/// Persists the state data to storage.
///
/// This function handles the serialization and storage of the reduce state,
/// returning a Result to indicate success or failure.
#[allow(clippy::mutable_key_type)]
pub(crate) async fn persist_runtime_state(
    state: PersistedState,
    state_persistence: &Arc<dyn PersistenceConnection>,
) -> Result<(), Error> {
    let state_persistence = Arc::clone(state_persistence);
    let handle = tokio::task::spawn_blocking(move || {
        let serialized_state = serde_json::to_string(&state)?;
        state_persistence.set(STATE_PERSISTENCE_KEY, &serialized_state)
    });

    match handle.await {
        Ok(result) => match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from(format!("Failed to persist state: {err}"))),
        },
        Err(join_err) => Err(Error::from(format!(
            "Failed to execute state persistence task: {join_err}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::config::TransformContext;
    use crate::event::{discriminant::Discriminant, Event, KeyString, Value};
    use crate::mezmo::persistence::RocksDBPersistenceConnection;
    use crate::transforms::reduce::mezmo_reduce::{
        MezmoMetadata, MezmoReduce, MezmoReduceConfig, ReduceState,
    };
    use assay::assay;
    use chrono::Utc;
    use indexmap::IndexMap;
    use mezmo::MezmoContext;
    use tempfile::tempdir;
    use vector_lib::event::LogEvent;

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
                state_persistence_base_path = "{state_persistence_base_path}"
                state_persistence_tick_ms = 100

                [merge_strategies]
                counter = "sum"
                message = "concat"
            "#
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
                    state_persistence_base_path = "{state_persistence_base_path}"
                    state_persistence_tick_ms = 100

                    [merge_strategies]
                    counter = "sum"
                    message = "concat"
                "#
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
                state_persistence_base_path = "{state_persistence_base_path}"
                state_persistence_tick_ms = 100

                [merge_strategies]
                counter = "sum"
                message = "concat"
            "#
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
                state_persistence_base_path = "{state_persistence_base_path}"
            "#
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
            let db_path = format!("{state_persistence_base_path}/reduce");
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
                state_persistence_base_path = "{state_persistence_base_path}"
            "#
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
                    state_persistence_base_path = "{state_persistence_base_path}"
                    state_persistence_tick_ms = 100

                    [merge_strategies]
                    counter = "sum"
                    message = "concat"
                "#
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
                state_persistence_base_path = "{state_persistence_base_path}"

                [merge_strategies]
                counter = "sum"
                message = "concat"
            "#
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
                "Discriminant not found after roundtrip: {original_discriminant:?}"
            );
        }
    }
}
