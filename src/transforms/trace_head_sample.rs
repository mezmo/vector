use vector_lib::config::{log_schema, LogNamespace, OutputId, TransformOutput};
use vector_lib::configurable::configurable_component;

use crate::mezmo::persistence::PersistenceConnection;
use crate::{
    config::{schema::Definition, DataType, Input, TransformConfig, TransformContext},
    event::Event,
    transforms::{FunctionTransform, OutputBuffer, Transform},
};

use crate::mezmo::persistence::RocksDBPersistenceConnection;
use mezmo::MezmoContext;
use mezmo::{user_log_warn, user_trace::MezmoUserLog};
use serde::ser::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use vrl::value::Value;

static TRACE_HEAD_SAMPLE_COUNT_KEY: &str = "trace_head_sample_count";
const DEFAULT_RATE: u64 = 10;
const MINIMUM_TTL_SECS: u64 = 60;
const DEFAULT_TTL_SECS: u64 = 15 * 60; // 15 min

/// Configuration for the `trace_head_sample` transform.
#[configurable_component(transform("trace_head_sample"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceHeadSampleConfig {
    /// the key of the event to evaluate
    #[serde(default = "default_trace_id_field")]
    trace_id_field: String,

    /// the sampling rate for the given key
    /// example: 10 means 1 of 10 are forwarded, the remainder are dropped
    #[serde(default = "default_rate")]
    rate: u64,

    /// the minimum ttl for a trace_id key to be cached
    #[serde(default = "default_ttl_secs")]
    ttl_secs: u64,

    /// the base path on disk to maintain keys and data while tracking traces
    state_persistence_base_path: Option<String>,
}

fn default_trace_id_field() -> String {
    ".trace_id".to_owned()
}

const fn default_rate() -> u64 {
    DEFAULT_RATE
}

const fn default_ttl_secs() -> u64 {
    DEFAULT_TTL_SECS
}

impl TraceHeadSampleConfig {
    pub fn new(config: &TraceHeadSampleConfig) -> Self {
        TraceHeadSampleConfig {
            trace_id_field: config.trace_id_field.clone(),
            rate: config.rate,
            ttl_secs: config.ttl_secs,
            state_persistence_base_path: config.state_persistence_base_path.clone(),
        }
    }
}

impl_generate_config_from_default!(TraceHeadSampleConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "trace_head_sample")]
impl TransformConfig for TraceHeadSampleConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        // generate a unique path for this component to store its data
        let mezmo_ctx = context.mezmo_ctx.clone().unwrap();
        let sample_path = "trace_head_sample".to_owned();
        let base_path = if let Some(p) = self.state_persistence_base_path.clone() {
            format!("{}/{}", p, sample_path)
        } else {
            sample_path
        };
        let persistence =
            RocksDBPersistenceConnection::new_with_ttl(&base_path, &mezmo_ctx, self.ttl_secs)?;
        Ok(Transform::function(TraceHeadSample::new(
            self.clone(),
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
pub struct TraceHeadSample {
    config: TraceHeadSampleConfig,
    mezmo_ctx: MezmoContext,
    count: u64,
    persistence: Arc<dyn PersistenceConnection>,
}

impl TraceHeadSample {
    pub(crate) fn new(
        config: TraceHeadSampleConfig,
        mezmo_ctx: MezmoContext,
        persistence: Arc<dyn PersistenceConnection>,
    ) -> Self {
        let mut sampler = Self {
            config,
            mezmo_ctx,
            count: 0,
            persistence,
        };
        sampler.intialize();
        sampler
    }

    fn intialize(&mut self) {
        // determine if we're re-initializing this particular component
        // so we can continue where we left off
        let count = self.get_value(TRACE_HEAD_SAMPLE_COUNT_KEY);
        self.count = count.unwrap_or_default();

        // handles bad configurations
        if self.config.rate == 0 {
            self.config.rate = DEFAULT_RATE;
        }
        if self.config.ttl_secs < MINIMUM_TTL_SECS {
            self.config.ttl_secs = DEFAULT_TTL_SECS;
        }
    }

    fn get_value<T: FromStr>(&mut self, key: &str) -> Option<T> {
        match self.persistence.get(key) {
            Ok(Some(value)) => value.parse::<T>().ok(),
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

    fn set_value<T>(&mut self, key: &str, value: T)
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

    fn log_warning(&mut self, value: String) {
        let msg = Value::from(value);
        user_log_warn!(Some(self.mezmo_ctx.clone()), msg);
    }
}

impl FunctionTransform for TraceHeadSample {
    fn transform(&mut self, output: &mut OutputBuffer, event: Event) {
        // TODO: update when we're ready to handle TraceEvent types with datadog traces
        let maybe_log = event.maybe_as_log();
        if maybe_log.is_none() {
            self.log_warning("Event dropped. Not a log event".to_string());
            return;
        }

        let log = maybe_log.unwrap();
        if let Some(message) = log.get(log_schema().message_key_target_path().unwrap()) {
            if !message.is_object() {
                self.log_warning("Event dropped. Message is not an object".to_string());
                return;
            }

            if let Some(Value::Bytes(b)) = message.get(self.config.trace_id_field.as_str()) {
                let trace_id = String::from_utf8_lossy(b);
                if let Some(value) = self.get_value(&trace_id) {
                    if value {
                        output.push(event);
                    }
                } else {
                    // otherwise evaluate the key
                    self.count = (self.count + 1) % self.config.rate;
                    if self.count == 1 {
                        self.set_value(&trace_id, true);
                        output.push(event);
                    } else {
                        self.set_value(&trace_id, false);
                    }

                    self.set_value(TRACE_HEAD_SAMPLE_COUNT_KEY, self.count);
                }
            } else {
                self.log_warning(format!(
                    "Event dropped. Trace ID is not a string: {}",
                    self.config.trace_id_field
                ));
            }
        } else {
            self.log_warning(format!(
                "Event dropped. Trace ID not found: {}",
                self.config.trace_id_field
            ));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::Event;
    use crate::transforms::trace_head_sample::TraceHeadSampleConfig;
    use assay::assay;
    use mezmo::MezmoContext;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use uuid::Uuid;
    use vector_lib::btreemap;
    use vector_lib::event::{LogEvent, TraceEvent};
    use vector_lib::transform::OutputBuffer;
    use vrl::event_path;

    fn test_ctx() -> MezmoContext {
        let account_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        MezmoContext::try_from(format!(
            "v1:trace_head_sample:transform:{component_id}:pipeline_id:{account_id}"
        ))
        .unwrap()
    }

    #[allow(warnings)]
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
        crate::test_util::test_generate_config::<super::TraceHeadSampleConfig>();
    }

    #[test]
    fn test_deserialize_default_config() {
        let default_config: TraceHeadSampleConfig = serde_json::from_str(r#"{ }"#).unwrap();

        assert_eq!(
            default_config.trace_id_field,
            ".trace_id".to_owned(),
            "default trace_id field is correct"
        );
        assert_eq!(default_config.rate, DEFAULT_RATE, "default rate is correct");
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
        let config = TraceHeadSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            rate: 0, // a rate of 0 would never sample and cause division errors
            ttl_secs: MINIMUM_TTL_SECS - 1,
            state_persistence_base_path: Some("/some-path".to_owned()),
        };

        let test_ctx = test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let sampler = TraceHeadSample::new(config, test_ctx, connection);
        assert_eq!(sampler.config.rate, DEFAULT_RATE, "rate is corrected");
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
    async fn build_transform() {
        let mezmo_ctx = test_ctx();
        let ctx = TransformContext {
            mezmo_ctx: Some(mezmo_ctx),
            ..Default::default()
        };

        let config = TraceHeadSampleConfig::default();
        match config.build(&ctx).await {
            Ok(_) => {}
            Err(e) => panic!("Failed to generate config: {}", e),
        }
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn log_not_object() {
        let config = TraceHeadSampleConfig::default();
        let test_ctx = test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceHeadSample::new(config, test_ctx, connection);

        let event1 = Event::Log(LogEvent::from("just a string"));

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.into());

        assert!(output.is_empty(), "Expected no events: {:?}", output);
        assert_eq!(sampler.count, 0, "No event counted");
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn event_is_not_a_log() {
        let config = TraceHeadSampleConfig::default();
        let test_ctx = test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceHeadSample::new(config, test_ctx, connection);

        let mut log = LogEvent::default();
        log.insert(event_path!("tags", "foo"), "x");
        let event1 = TraceEvent::from(log);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.into());

        assert!(output.is_empty(), "Expected no events: {:?}", output);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn field_not_found() {
        let config = TraceHeadSampleConfig {
            trace_id_field: ".prop1".to_owned(),
            rate: 2,
            ttl_secs: 300,
            state_persistence_base_path: None,
        };
        let test_ctx = test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceHeadSample::new(config, test_ctx, connection);

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
        assert_eq!(sampler.count, 0, "No event counted");
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn first_trace_id_encounter() {
        let event1 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello",
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
                    "span_id" => "e274a55e-39bd-4626-9270-295b170c7c5f"
                }
            }
        }));

        let config = TraceHeadSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            rate: 2,
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = test_ctx();

        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceHeadSample::new(config, test_ctx, connection);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, event1.clone());
        sampler.transform(&mut output, event2.clone());

        assert_eq!(output.len(), 2, "both events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual[0], event1);
        assert_eq!(actual[1], event2);

        assert_eq!(sampler.count, 1, "One distinct trace_id");
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
                    "span_id" => "e274a55e-39bd-4626-9270-295b170c7c5f"
                }
            }
        }));
        let traceid_3a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "d7c1972a-b5d2-4049-98a5-c6893a8b539b"
                }
            }
        }));
        let traceid_3b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "4378a819-fe06-4f87-addd-d03b37acb7fb"
                }
            }
        }));
        let traceid_5 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello4",
                "context" => btreemap! {
                    "trace_id" => "b1e3c528-1065-43e5-b18a-baa727c3a1f2",
                    "span_id" => "7cc912cb-26b6-49b4-bff7-7ef750ba0839"
                }
            }
        }));
        let traceid_6 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello5",
                "context" => btreemap! {
                    "trace_id" => "6bb596dc-534c-4141-aa39-272145d7fc11",
                    "span_id" => "1109c26a-2316-4ab6-a601-023af90d1782"
                }
            }
        }));

        let config = TraceHeadSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            rate: 2,
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = test_ctx();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, None);
        let mut sampler = TraceHeadSample::new(config, test_ctx, connection);

        let mut output = OutputBuffer::default();
        sampler.transform(&mut output, traceid_1a.clone());
        sampler.transform(&mut output, traceid_1b.clone());
        sampler.transform(&mut output, traceid_3a.clone());
        sampler.transform(&mut output, traceid_3b.clone());
        sampler.transform(&mut output, traceid_5.clone());
        sampler.transform(&mut output, traceid_6.clone());

        assert_eq!(output.len(), 3, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual[0], traceid_1a);
        assert_eq!(actual[1], traceid_1b);
        assert_eq!(actual[2], traceid_5);

        assert_eq!(sampler.count, 0, "reset back to 0");
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    #[test]
    fn vector_restart_continue_rate() {
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
                    "span_id" => "e274a55e-39bd-4626-9270-295b170c7c5f"
                }
            }
        }));
        let traceid_3a = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "d7c1972a-b5d2-4049-98a5-c6893a8b539b"
                }
            }
        }));
        let traceid_3b = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello3",
                "context" => btreemap! {
                    "trace_id" => "c69f13ea-4c0e-4a67-b20e-b70c04eb0ea7",
                    "span_id" => "4378a819-fe06-4f87-addd-d03b37acb7fb"
                }
            }
        }));
        let traceid_5 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello4",
                "context" => btreemap! {
                    "trace_id" => "b1e3c528-1065-43e5-b18a-baa727c3a1f2",
                    "span_id" => "7cc912cb-26b6-49b4-bff7-7ef750ba0839"
                }
            }
        }));
        let traceid_6 = Event::Log(LogEvent::from(btreemap! {
            "message" => btreemap! {
                "name" => "hello5",
                "context" => btreemap! {
                    "trace_id" => "6bb596dc-534c-4141-aa39-272145d7fc11",
                    "span_id" => "1109c26a-2316-4ab6-a601-023af90d1782"
                }
            }
        }));

        // run a transform against some samples
        let config = TraceHeadSampleConfig {
            trace_id_field: ".context.trace_id".to_owned(),
            rate: 2,
            ttl_secs: 300,
            state_persistence_base_path: Some("/data/component-state".to_owned()),
        };
        let test_ctx = test_ctx();
        #[allow(deprecated)]
        let base_dir = tempdir().expect("Could not create temp dir").into_path();
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, Some(base_dir.clone()));
        let mut sampler = TraceHeadSample::new(config.clone(), test_ctx.clone(), connection);

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
        let connection = test_connection(test_ctx.clone(), config.ttl_secs, Some(base_dir));
        let mut output = OutputBuffer::default();
        let mut sampler = TraceHeadSample::new(config, test_ctx.clone(), connection);
        sampler.transform(&mut output, traceid_3a.clone());
        sampler.transform(&mut output, traceid_3b.clone());
        sampler.transform(&mut output, traceid_5.clone());
        sampler.transform(&mut output, traceid_6.clone());

        assert_eq!(output.len(), 1, "events caught");
        let actual = output.into_events().collect::<Vec<_>>();
        assert_eq!(actual[0], traceid_5);

        assert_eq!(sampler.count, 0, "reset back to 0");
    }
}
