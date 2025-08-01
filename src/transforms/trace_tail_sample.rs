use chrono::{DateTime, Utc};
use vector_lib::config::{log_schema, LogNamespace, OutputId, TransformOutput};
use vector_lib::configurable::configurable_component;

use crate::mezmo::persistence::PersistenceConnection;
use crate::{
    conditions::{AnyCondition, Condition},
    config::{schema::Definition, DataType, Input, TransformConfig, TransformContext},
    event::Event,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

use crate::mezmo::persistence::RocksDBPersistenceConnection;
use mezmo::MezmoContext;
use mezmo::{user_log_warn, user_trace::MezmoUserLog};
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use vrl::value::Value;

const MINIMUM_TTL_SECS: u64 = 15;
const DEFAULT_TTL_SECS: u64 = 5 * 60; // 5 min

/// A conditional as defined by the customer
#[configurable_component]
#[derive(Clone, Debug)]
pub struct TailSampleConditional {
    /// the rate at which matching traces should be kept
    rate: u64,
    /// the condition to compare against the trace
    condition: AnyCondition,
    /// the unique output name of the condition
    output_name: String,
}

/// Configuration for the `trace_head_sample` transform.
#[configurable_component(transform("trace_tail_sample"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceTailSampleConfig {
    /// the key of the event to evaluate
    #[serde(default = "default_trace_id_field")]
    trace_id_field: String,

    /// the key of the event to evaluate
    #[serde(default = "default_parent_span_id_field")]
    parent_span_id_field: String,

    /// list of conditionals and rates at which to apply
    #[serde(default = "default_conditionals")]
    conditionals: Vec<TailSampleConditional>,

    /// the minimum ttl for a trace_id key to be cached
    #[serde(default = "default_ttl_secs")]
    ttl_secs: u64,

    /// the base path on disk to maintain keys and data while tracking traces
    state_persistence_base_path: Option<String>,
}

fn default_trace_id_field() -> String {
    ".trace_id".to_owned()
}

fn default_parent_span_id_field() -> String {
    ".parent_span_id".to_owned()
}

const fn default_conditionals() -> Vec<TailSampleConditional> {
    Vec::new()
}

const fn default_ttl_secs() -> u64 {
    DEFAULT_TTL_SECS
}

impl TraceTailSampleConfig {
    pub fn new(config: &TraceTailSampleConfig) -> Self {
        TraceTailSampleConfig {
            trace_id_field: config.trace_id_field.clone(),
            parent_span_id_field: config.parent_span_id_field.clone(),
            conditionals: config.conditionals.clone(),
            ttl_secs: config.ttl_secs,
            state_persistence_base_path: config.state_persistence_base_path.clone(),
        }
    }
}

impl_generate_config_from_default!(TraceTailSampleConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "trace_tail_sample")]
impl TransformConfig for TraceTailSampleConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        // generate a unique path for this component to store its data
        let mezmo_ctx = context.mezmo_ctx.clone().unwrap();
        let sample_path = "trace_tail_sample".to_owned();
        let base_path = if let Some(p) = self.state_persistence_base_path.clone() {
            format!("{}/{}", p, sample_path)
        } else {
            sample_path
        };
        let persistence =
            RocksDBPersistenceConnection::new_with_ttl(&base_path, &mezmo_ctx, self.ttl_secs)?;

        //build all the conditions from their configs to be used in evaluations
        let conditions = self
            .conditionals
            .iter()
            .map(|c| {
                let built_condition = c
                    .condition
                    .build(&context.enrichment_tables, context.mezmo_ctx.clone())
                    .unwrap();
                (c.output_name.clone(), built_condition)
            })
            .collect();

        Ok(Transform::function(TraceTailSample::new(
            self.clone(),
            conditions,
            mezmo_ctx,
            Arc::new(persistence),
        )))
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

#[derive(Clone, Debug)]
pub struct TraceTailSample {
    config: TraceTailSampleConfig,
    conditions: Vec<(String, Condition)>,
    rates_map: HashMap<String, u64>,
    mezmo_ctx: MezmoContext,
    persistence: Arc<dyn PersistenceConnection>,
}

impl TraceTailSample {
    pub(crate) fn new(
        config: TraceTailSampleConfig,
        conditions: Vec<(String, Condition)>,
        mezmo_ctx: MezmoContext,
        persistence: Arc<dyn PersistenceConnection>,
    ) -> Self {
        let mut sampler = Self {
            config,
            conditions,
            rates_map: HashMap::new(),
            mezmo_ctx,
            persistence,
        };
        sampler.intialize();
        sampler
    }

    fn intialize(&mut self) {
        if self.config.ttl_secs < MINIMUM_TTL_SECS {
            self.config.ttl_secs = DEFAULT_TTL_SECS;
        }

        // initialize all the counts for the conditionals
        // based on previous runs and setup the output rates map
        let conditionals = self.config.conditionals.clone();
        for conditional in conditionals.iter() {
            self.rates_map
                .insert(conditional.output_name.clone(), conditional.rate);
        }
    }

    fn build_key(&mut self, data_name: &str, trace_id: Option<&str>) -> String {
        if trace_id.is_none() {
            data_name.to_string()
        } else {
            format!("{}:{}", trace_id.unwrap(), data_name)
        }
    }

    fn get_cache_value<T: FromStr>(&mut self, key: &str) -> Option<T> {
        match self.persistence.get(key) {
            Ok(Some(value)) => match value.parse::<T>() {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            Ok(None) => None,
            Err(e) => {
                error!(
                    message = "Failed to get key/value",
                    key,
                    component_id = self.mezmo_ctx.component_id(),
                    error = e
                );
                None
            }
        }
    }

    fn set_cache_value<T>(&mut self, key: &str, value: T)
    where
        T: Serialize,
    {
        let serialized = { serde_json::to_string(&value).unwrap() };
        if let Err(e) = self.persistence.set(key, serialized.as_str()) {
            error!(
                message = "Failed to set key/value",
                key,
                component_id = self.mezmo_ctx.component_id(),
                error = e
            );
        }
    }

    fn get_events(&mut self, trace_id: &str) -> Vec<Event> {
        let key = self.build_key("events", Some(trace_id));

        let events: Vec<Event> = match self.persistence.get(&key) {
            Ok(Some(value)) => serde_json::from_str(value.as_str()).unwrap_or_else(|_| {
                error!(
                    message = "Failed to deserialize events",
                    trace_id,
                    component_id = self.mezmo_ctx.component_id()
                );
                Vec::new()
            }),
            Ok(None) => Vec::new(),
            Err(e) => {
                error!(
                    message = "Failed to get key/value",
                    key,
                    component_id = self.mezmo_ctx.component_id(),
                    error = e
                );
                Vec::new()
            }
        };

        events
    }

    fn append_event(&mut self, trace_id: &str, event: Event) {
        let mut events = self.get_events(trace_id);
        events.push(event);

        let key = self.build_key("events", Some(trace_id));
        self.set_cache_value(&key, events);
    }

    fn delete_events(&mut self, trace_id: &str) {
        let key = self.build_key("events", Some(trace_id));
        if let Err(e) = self.persistence.delete(&key) {
            error!(
                message = "Failed to delete key",
                key,
                component_id = self.mezmo_ctx.component_id(),
                error = e
            );
        }
    }

    fn log_user_warning(&mut self, value: String) {
        let msg = Value::from(value);
        user_log_warn!(Some(self.mezmo_ctx.clone()), msg);
    }

    fn get_message_field_value(&mut self, message: &Value, field: &str) -> Option<String> {
        if let Some(Value::Bytes(b)) = message.get(field) {
            let field_value = String::from_utf8_lossy(b);
            if field_value.len() > 0 {
                return Some(field_value.to_string());
            }
        }

        None
    }
}

impl FunctionTransform for TraceTailSample {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        // TODO: update when we're ready to handle TraceEvent types with datadog traces
        let maybe_log = event.maybe_as_log();
        if maybe_log.is_none() {
            self.log_user_warning("Event dropped. Not a log event".to_string());
            return;
        }

        let log = maybe_log.unwrap();
        if let Some(message) = log.get(log_schema().message_key_target_path().unwrap()) {
            if !message.is_object() {
                self.log_user_warning("Event dropped. Message is not an object".to_string());
                return;
            }

            let trace_id_field = self.config.trace_id_field.clone();
            let trace_id: String;
            if let Some(extracted_trace_id) = self.get_message_field_value(message, &trace_id_field)
            {
                trace_id = extracted_trace_id;
            } else {
                self.log_user_warning(format!(
                    "Event dropped as trace_id not found: {}",
                    self.config.trace_id_field
                ));
                return;
            }

            // check if we've evaluated the trace_id before. if so and it was a positive result,
            // send it down the line
            let eval_result_key = self.build_key("result", Some(&trace_id));
            if let Some(evaluation) = self.get_cache_value(&eval_result_key) {
                if evaluation {
                    output.push(event);
                }
                return;
            }

            // if not the head of the trace, then save the event for later evaluation
            let parent_span_id_field = self.config.parent_span_id_field.clone();
            let parent_span_id = self.get_message_field_value(message, &parent_span_id_field);
            if parent_span_id.is_some() {
                self.append_event(trace_id.as_str(), event);
                return;
            }

            // must be the head event, so perform the evaluation logic
            // if matches the logic, determine if we should flush the events downstream
            // a rate of 1 means to always flush whereas a rate of 10 means to flush 1 of every 10
            let mut flush_events_downstream = false;
            for (output_name, condition) in self.conditions.iter() {
                let (result, _) = condition.check(event.clone());
                if result {
                    let rate = self.rates_map.get(output_name.as_str()).unwrap().to_owned();
                    if rate == 1 {
                        flush_events_downstream = true;
                    } else {
                        let condition_count_key =
                            self.build_key(output_name.clone().as_str(), None);
                        let mut current_count: u64 = self
                            .get_cache_value(&condition_count_key)
                            .unwrap_or_default();
                        current_count = (current_count + 1) % rate;

                        flush_events_downstream = current_count == 1;
                        self.set_cache_value(&condition_count_key, current_count);
                    }

                    break;
                }
            }

            // flush all the corresponding events downstream evaluated positively
            // and make sure the longterm cache is updated to handle restarts
            if flush_events_downstream {
                output.push(event);
                let associated_events = self.get_events(trace_id.as_str());

                // since we've serialized/deserialized, the type of the timestamp was lost
                // and is now a string. Convert this back to a DateTime type
                let timestamp_key = log_schema().timestamp_key_target_path().unwrap();
                for ae in associated_events.iter() {
                    let mut cloned_event = ae.clone();
                    let log = cloned_event.as_mut_log();
                    if let Some(existing_timestamp) = log.get(timestamp_key) {
                        let timestamp: DateTime<Utc> =
                            existing_timestamp.to_string_lossy().parse().unwrap();
                        log.insert(timestamp_key, Value::Timestamp(timestamp));
                    }

                    output.push(Event::Log((*log).clone()));
                }
            }

            // delete the events cache
            self.delete_events(trace_id.as_str());

            // mark the result of the evaluation
            self.set_cache_value(&eval_result_key, flush_events_downstream);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::Event;
    use crate::transforms::trace_tail_sample::TraceTailSampleConfig;
    use assay::assay;
    use chrono::prelude::*;
    use mezmo::MezmoContext;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use uuid::Uuid;
    use vector_lib::btreemap;
    use vector_lib::event::{LogEvent, TraceEvent};
    use vector_lib::transform::OutputBuffer;
    use vrl::event_path;

    fn get_test_ctx() -> MezmoContext {
        let account_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        MezmoContext::try_from(format!(
            "v1:trace_tail_sample:transform:{component_id}:pipeline_id:{account_id}"
        ))
        .unwrap()
    }

    fn test_connection(
        mezmo_ctx: MezmoContext,
        ttl_secs: u64,
        tmp_dir: Option<PathBuf>,
    ) -> Arc<dyn PersistenceConnection> {
        let tmp_path = {
            if let Some(dir) = tmp_dir {
                dir
            } else {
                tempdir().expect("Could not create temp dir").into_path()
            }
        };

        let base_path = tmp_path.to_str().unwrap();
        let persistence =
            RocksDBPersistenceConnection::new_with_ttl(base_path, &mezmo_ctx, ttl_secs).unwrap();
        Arc::new(persistence)
    }

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<super::TraceTailSampleConfig>();
    }

    #[test]
    fn test_deserialize_default_config() {
        let default_config: TraceTailSampleConfig = serde_json::from_str(r#"{ }"#).unwrap();

        assert_eq!(
            default_config.trace_id_field,
            ".trace_id".to_owned(),
            "default trace_id field is correct"
        );
        assert_eq!(
            default_config.parent_span_id_field,
            ".parent_span_id".to_owned(),
            "default parent_span_id field is correct"
        );

        assert_eq!(
            default_config.conditionals.len(),
            0,
            "default conditionals field is empty"
        );

        assert_eq!(
            default_config.ttl_secs, DEFAULT_TTL_SECS,
            "default ttl_secs is correct"
        );
        assert_eq!(
            default_config.state_persistence_base_path, None,
            "default state_persistence_base_path is correct"
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn patch_invalid_values() {
        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![],
            ttl_secs: MINIMUM_TTL_SECS - 1,
            state_persistence_base_path: Some("/some-path".to_owned()),
        };

        let test_ctx = get_test_ctx();
        let conditions = Vec::new();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let sampler = TraceTailSample::new(config, conditions, test_ctx, connection);
        assert_eq!(
            sampler.config.ttl_secs, DEFAULT_TTL_SECS,
            "ttl_secs is corrected"
        );
        assert_eq!(
            sampler.config.state_persistence_base_path,
            Some("/some-path".to_owned()),
            "state_persistence_base_path is correct"
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    async fn build_transform_from_config() {
        let mezmo_ctx = get_test_ctx();
        let ctx = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };

        let config = TraceTailSampleConfig::default();
        match config.build(&ctx).await {
            Ok(transform) => match transform {
                Transform::Function(_) => {}
                _ => panic!("Expected a Function transform"),
            },
            Err(e) => panic!("Failed to generate config: {}", e),
        }
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn log_not_object() {
        let config = TraceTailSampleConfig::default();
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceTailSample::new(config, Vec::new(), test_ctx, connection);

        let event1 = Event::Log(LogEvent::from("just a string"));

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.into());

        assert!(output.is_empty(), "Expected no events: {:?}", output);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn event_is_not_a_log() {
        let config = TraceTailSampleConfig::default();
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceTailSample::new(config, Vec::new(), test_ctx, connection);

        let mut log = LogEvent::default();
        log.insert(event_path!("tags", "foo"), "x");
        let event1 = TraceEvent::from(log);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.into());

        assert!(output.is_empty(), "Expected no events: {:?}", output);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    async fn trace_id_not_found() {
        let config = TraceTailSampleConfig {
            trace_id_field: ".prop1".to_owned(),
            parent_span_id_field: ".prop2".to_owned(),
            conditionals: vec![],
            ttl_secs: 300,
            state_persistence_base_path: None,
        };

        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceTailSample::new(config, Vec::new(), test_ctx, connection);

        let event1 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
                "context" => btreemap! {
                    "trace_id" => "1ebddd15-314e-4f5c-bacc-782dd746883d",
                    "span_id" => "4ffd5acd-45bf-47e9-91cf-852e622b1468"
                }
            }
        }));
        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.into());

        assert!(output.is_empty(), "Expected no events: {:?}", output);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn entire_trace_output_head_received_last() {
        let event1 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
                "status_code" => 200,
                "context" => btreemap! {
                    "trace_id" => "1ebddd15-314e-4f5c-bacc-782dd746883d",
                    "span_id" => "4ffd5acd-45bf-47e9-91cf-852e622b1468"
                }
            }
        }));
        let event2 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello2",
                "context" => btreemap! {
                    "trace_id" => "1ebddd15-314e-4f5c-bacc-782dd746883d",
                    "span_id" => "e274a55e-39bd-4626-9270-295b170c7c5f",
                    "parent_span_id" => "4ffd5acd-45bf-47e9-91cf-852e622b1468"
                }
            }
        }));

        let condition_config: TailSampleConditional = toml::from_str(
            r#"
            rate = 1
            condition = "exists(.message.name)"
            output_name = "hello"
        "#,
        )
        .unwrap();

        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);

        let mut output = OutputBuffer::default();
        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx, connection);
        sampler.transform(&mut output, event2.clone());
        sampler.transform(&mut output, event1.clone());

        assert_eq!(output.len(), 2, "both events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual[0], event1);
        assert_eq!(actual[1], event2);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn entire_trace_output_head_received_first() {
        let event1 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
                "status_code" => 200,
                "context" => btreemap! {
                    "trace_id" => "431b53e9-a335-46f7-b0d5-4a0ed58c9ae3",
                    "span_id" => "a34ba1c0-a936-4c2d-8841-1f13763725a1"
                }
            }
        }));
        let event2 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello2",
                "context" => btreemap! {
                    "trace_id" => "431b53e9-a335-46f7-b0d5-4a0ed58c9ae3",
                    "span_id" => "eb3aa314-7943-49da-ba24-19f65d250dff",
                    "parent_span_id" => "a34ba1c0-a936-4c2d-8841-1f13763725a1"
                }
            }
        }));

        let condition_config: TailSampleConditional = toml::from_str(
            r#"
            rate = 1
            condition = "exists(.message.name)"
            output_name = "hello"
        "#,
        )
        .unwrap();

        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);

        let mut output = OutputBuffer::default();

        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx, connection);
        sampler.transform(&mut output, event1.clone());
        sampler.transform(&mut output, event2.clone());

        assert_eq!(output.len(), 2, "both events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual[0], event1);
        assert_eq!(actual[1], event2);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn rate_sampling() {
        let traceid_1a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
                "context" => btreemap! {
                    "trace_id" => "1ebddd15-314e-4f5c-bacc-782dd746883d",
                    "span_id" => "4ffd5acd-45bf-47e9-91cf-852e622b1468"
                }
            }
        }));
        let traceid_1b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello2",
                "context" => btreemap! {
                    "trace_id" => "1ebddd15-314e-4f5c-bacc-782dd746883d",
                    "span_id" => "e274a55e-39bd-4626-9270-295b170c7c5f",
                    "parent_span_id" => "4ffd5acd-45bf-47e9-91cf-852e622b1468"
                }
            }
        }));
        let traceid_2a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name2" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "d7c1972a-b5d2-4049-98a5-c6893a8b539b"
                }
            }
        }));
        let traceid_2b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name2" => "hello4",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "4378a819-fe06-4f87-addd-d03b37acb7fb",
                    "parent_span_id" => "d7c1972a-b5d2-4049-98a5-c6893a8b539b"
                }
            }
        }));
        let traceid_3a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello5",
                "context" => btreemap! {
                    "trace_id" => "b1e3c528-1065-43e5-b18a-baa727c3a1f2",
                    "span_id" => "7cc912cb-26b6-49b4-bff7-7ef750ba0839"
                }
            }
        }));
        let traceid_3b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello6",
                "context" => btreemap! {
                    "trace_id" => "b1e3c528-1065-43e5-b18a-baa727c3a1f2",
                    "span_id" => "6b5029ea-4580-4c4b-ba61-e7feb9d8ac1b",
                    "parent_span_id" => "7cc912cb-26b6-49b4-bff7-7ef750ba0839"
                }
            }
        }));
        // 3 span trace received in reverse
        let traceid_4c = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello9",
                "context" => btreemap! {
                    "trace_id" => "6bb596dc-534c-4141-aa39-272145d7fc11",
                    "span_id" => "a1316b35-6c25-486b-a425-6382a88d3913",
                    "parent_span_id" => "6ffc22ae-ce15-4982-a118-c7b4832329b7"
                }
            }
        }));
        let traceid_4b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello8",
                "context" => btreemap! {
                    "trace_id" => "6bb596dc-534c-4141-aa39-272145d7fc11",
                    "span_id" => "6ffc22ae-ce15-4982-a118-c7b4832329b7",
                    "parent_span_id" => "1109c26a-2316-4ab6-a601-023af90d1782"
                }
            }
        }));
        let traceid_4a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello7",
                "context" => btreemap! {
                    "trace_id" => "6bb596dc-534c-4141-aa39-272145d7fc11",
                    "span_id" => "1109c26a-2316-4ab6-a601-023af90d1782"
                }
            }
        }));

        let condition_config: TailSampleConditional = toml::from_str(
            r#"
            rate = 2
            condition = "exists(.message.name)"
            output_name = "hello"
        "#,
        )
        .unwrap();

        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);

        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx, connection);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, traceid_1a.clone());
        sampler.transform(&mut output, traceid_1b.clone());
        sampler.transform(&mut output, traceid_2a.clone());
        sampler.transform(&mut output, traceid_2b.clone());
        sampler.transform(&mut output, traceid_3a.clone());
        sampler.transform(&mut output, traceid_3b.clone());
        sampler.transform(&mut output, traceid_4c.clone());
        sampler.transform(&mut output, traceid_4b.clone());
        sampler.transform(&mut output, traceid_4a.clone());

        assert_eq!(output.len(), 5, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();

        assert_eq!(actual[0], traceid_1a);
        assert_eq!(actual[1], traceid_1b);
        assert_eq!(actual[2], traceid_4a);
        assert_eq!(actual[3], traceid_4c);
        assert_eq!(actual[4], traceid_4b);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn vector_restart_continue_rate() {
        let base_dir = tempdir().expect("Could not create temp dir").into_path();

        let traceid_1a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
                "context" => btreemap! {
                    "trace_id" => "2a15bbef-9d17-4294-bad8-dc7f1059c2b9",
                    "span_id" => "607c42e6-059f-4fe1-a296-763b6bfc53d2"
                }
            }
        }));
        let traceid_1b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello2",
                "context" => btreemap! {
                    "trace_id" => "2a15bbef-9d17-4294-bad8-dc7f1059c2b9",
                    "span_id" => "9e68fce0-7a17-43b7-b098-87dde0cfb1d7",
                    "parent_span_id" => "607c42e6-059f-4fe1-a296-763b6bfc53d2"
                }
            }
        }));
        let traceid_2a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name2" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "8c67c7ba-2dd8-419a-ac03-ddf55e837832",
                    "span_id" => "bb562ec6-598d-426b-b297-e29dafccf395"
                }
            }
        }));
        let traceid_2b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name2" => "hello4",
                "context" => btreemap! {
                    "trace_id" => "8c67c7ba-2dd8-419a-ac03-ddf55e837832",
                    "span_id" => "b9b68ac1-ace0-4d61-88b8-45a409ed3cf6",
                    "parent_span_id" => "bb562ec6-598d-426b-b297-e29dafccf395"
                }
            }
        }));
        let traceid_3a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello5",
                "context" => btreemap! {
                    "trace_id" => "afb466ba-7469-4500-808d-caff16df9196",
                    "span_id" => "9a57b8e8-a884-470e-a998-778b705b0808"
                }
            }
        }));
        let traceid_3b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello6",
                "context" => btreemap! {
                    "trace_id" => "afb466ba-7469-4500-808d-caff16df9196",
                    "span_id" => "9147d7d2-4501-448b-8f13-d6cbf0f3de20",
                    "parent_span_id" => "9a57b8e8-a884-470e-a998-778b705b0808"
                }
            }
        }));
        // 3 span trace received in reverse
        let traceid_4c = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello9",
                "context" => btreemap! {
                    "trace_id" => "85030a5a-c39a-434e-b416-56dd4c4d4157",
                    "span_id" => "0552fe39-31ca-4b07-9792-adfc6272426f",
                    "parent_span_id" => "55bed825-0aec-4f0b-8a72-e4b3afa6c582"
                }
            }
        }));
        let traceid_4b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello8",
                "context" => btreemap! {
                    "trace_id" => "85030a5a-c39a-434e-b416-56dd4c4d4157",
                    "span_id" => "55bed825-0aec-4f0b-8a72-e4b3afa6c582",
                    "parent_span_id" => "c0affb71-d2c7-4b36-bfff-11d4d79e3955"
                }
            }
        }));
        let traceid_4a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello7",
                "context" => btreemap! {
                    "trace_id" => "85030a5a-c39a-434e-b416-56dd4c4d4157",
                    "span_id" => "c0affb71-d2c7-4b36-bfff-11d4d79e3955"
                }
            }
        }));

        let condition_config: TailSampleConditional = toml::from_str(
            r#"
            rate = 2
            condition = "exists(.message.name)"
            output_name = "hello"
        "#,
        )
        .unwrap();

        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, Some(base_dir.clone()));

        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx.clone(), connection);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, traceid_1a.clone());
        sampler.transform(&mut output, traceid_1b.clone());

        assert_eq!(output.len(), 2, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();

        assert_eq!(actual[0], traceid_1a);
        assert_eq!(actual[1], traceid_1b);

        // now act as if vector crashed, reloaded the pipeline, or whatever
        // now start a new sampler, connecting to the same source,
        // and make sure it rejects the first items with the continued count
        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };

        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];

        let connection = test_connection(test_ctx.clone(), config.ttl_secs, Some(base_dir));
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx, connection);
        let mut output = OutputBuffer::default();

        sampler.transform(&mut output, traceid_2a.clone());
        sampler.transform(&mut output, traceid_2b.clone());
        sampler.transform(&mut output, traceid_3a.clone());
        sampler.transform(&mut output, traceid_3b.clone());
        sampler.transform(&mut output, traceid_4c.clone());
        sampler.transform(&mut output, traceid_4b.clone());
        sampler.transform(&mut output, traceid_4a.clone());

        assert_eq!(output.len(), 3, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();

        assert_eq!(actual[0], traceid_4a);
        assert_eq!(actual[1], traceid_4c);
        assert_eq!(actual[2], traceid_4b);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn timestamp_is_always_present() {
        let base_dir = tempdir().expect("Could not create temp dir").into_path();

        let traceid_1a = Event::Log(LogEvent::from(btreemap! {
            "timestamp" => Utc.with_ymd_and_hms(2022, 2, 19, 3, 10, 37).unwrap(),
            "message" => btreemap! {
                "name" => "hello",
                "context" => btreemap! {
                    "trace_id" => "2a15bbef-9d17-4294-bad8-dc7f1059c2b9",
                    "span_id" => "607c42e6-059f-4fe1-a296-763b6bfc53d2"
                }
            }
        }));
        let traceid_1b = Event::Log(LogEvent::from(btreemap! {
            "timestamp" => Utc.with_ymd_and_hms(2022, 2, 19, 3, 12, 59).unwrap(),
            "message" => btreemap! {
                "name" => "hello2",
                "context" => btreemap! {
                    "trace_id" => "2a15bbef-9d17-4294-bad8-dc7f1059c2b9",
                    "span_id" => "9e68fce0-7a17-43b7-b098-87dde0cfb1d7",
                    "parent_span_id" => "607c42e6-059f-4fe1-a296-763b6bfc53d2"
                }
            }
        }));
        let traceid_1c = Event::Log(LogEvent::from(btreemap! {
            "timestamp" => Utc.with_ymd_and_hms(2022, 2, 19, 3, 13, 2).unwrap(),
            "message" => btreemap! {
                "name" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "2a15bbef-9d17-4294-bad8-dc7f1059c2b9",
                    "span_id" => "bea3666e-55c1-4b9e-844a-6fe6ab53a1bf",
                    "parent_span_id" => "9e68fce0-7a17-43b7-b098-87dde0cfb1d7"
                }
            }
        }));

        let condition_config: TailSampleConditional = toml::from_str(
            r#"
            rate = 2
            condition = "exists(.message.name)"
            output_name = "hello"
        "#,
        )
        .unwrap();

        let config = TraceTailSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            parent_span_id_field: ".context.parent_span_id".to_owned(),
            conditionals: vec![condition_config.clone()],
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = get_test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, Some(base_dir.clone()));

        let conditions = vec![(
            "hello".to_owned(),
            condition_config
                .condition
                .build(&Default::default(), Some(test_ctx.clone()))
                .unwrap(),
        )];
        let mut sampler = TraceTailSample::new(config, conditions, test_ctx.clone(), connection);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, traceid_1c.clone());
        sampler.transform(&mut output, traceid_1b.clone());
        sampler.transform(&mut output, traceid_1a.clone());

        assert_eq!(output.len(), 3, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual.len(), 3, "actual events caught");

        // validate the timestamp is present on all events, serialized or not
        // and is an actual timestamp type
        for e in actual.iter() {
            let log = e.as_log();
            assert!(
                log.get_timestamp().is_some(),
                "timestamp is present as expected"
            );

            let ts = log.get_timestamp().unwrap();
            assert_eq!("timestamp", ts.kind_str(), "is a timestamp type");
        }
    }
}
