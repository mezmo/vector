use deadpool_postgres::PoolConfig;
use std::borrow::Cow;
use std::time::{Duration, SystemTime};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    future::ready,
    num::NonZeroUsize,
};

use crate::{
    config::{
        schema::Definition, DataType, Input, LogNamespace, OutputId, TransformConfig,
        TransformContext,
    },
    event::Event,
    transforms::{TaskTransform, Transform},
};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, OnceCell};
use uuid::Uuid;
use vector_config::configurable_component;
use vector_core::config::{log_schema, TransformOutput};

use crate::mezmo::MezmoContext;
use crate::transforms::mezmo_log_clustering::drain::{LocalId, LogUpdateStatus};
use crate::transforms::mezmo_log_clustering::store::save_in_loop;
use vector_core::event::LogEvent;
use vector_core::usage_metrics::{get_annotations, get_db_config, AnnotationSet};
use vrl::value::Value;

mod drain;
mod store;

const DEFAULT_DB_CONNECTION_POOL_SIZE: usize = 2;

/// Configuration for the `mezmo_log_clustering` transform.
#[configurable_component(transform("mezmo_log_clustering"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MezmoLogClusteringConfig {
    /// Max number of log clusters to keep in LRU cache
    #[serde(default = "default_max_clusters")]
    pub max_clusters: usize,

    /// A threshold of range 0.0 to 1.0 of how similar a log needs to be to a
    /// log cluster to be considered a match.
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,

    /// Maximum depth of the prefix tree
    #[serde(default = "default_max_node_depth")]
    pub max_node_depth: usize,

    /// Maximum number of cluster groups allowed in a leaf node.
    #[serde(default = "default_max_children")]
    pub max_children: usize,

    /// The field to cluster. If not provide then ".message" will be used
    pub cluster_field: Option<String>,

    /// Determines whether it should store data in the metrics database
    #[serde(default)]
    pub store_metrics: bool,

    /// When `store_metrics` is set, it determines the beginning of the window when data is stored.
    pub sample_start: Option<i64>,

    /// When `store_metrics` is set, it determines the end of the window when data is stored.
    pub sample_end: Option<i64>,

    /// When `store_metrics` is enabled, it determines the flush interval.
    #[serde(default = "default_store_metrics_flush_interval")]
    pub store_metrics_flush_interval: Duration,
}

const fn default_max_clusters() -> usize {
    1000
}

const fn default_similarity_threshold() -> f64 {
    0.5
}

const fn default_max_node_depth() -> usize {
    6
}

const fn default_max_children() -> usize {
    40
}

const fn default_store_metrics_flush_interval() -> Duration {
    Duration::from_secs(20)
}

impl_generate_config_from_default!(MezmoLogClusteringConfig);

type DbTransmitter = UnboundedSender<LogGroupInfo>;
static ONCE: OnceCell<DbTransmitter> = OnceCell::const_new();

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_log_clustering")]
impl TransformConfig for MezmoLogClusteringConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        // Create a channel with a db connection pool only once
        let mut key = None;
        let db_tx = if self.store_metrics {
            let db_url = env::var("MEZMO_METRICS_DB_URL").ok();
            let Some(db_url) = db_url else {
                return Err(
                    "Cannot store log clustering metrics without MEZMO_METRICS_DB_URL being set"
                        .into(),
                );
            };

            let Ok(mut config) = get_db_config(db_url.as_str()) else {
                return Err("Invalid db url".into());
            };
            config.pool = Some(PoolConfig::new(DEFAULT_DB_CONNECTION_POOL_SIZE));
            let Some(mezmo_ctx) = context.mezmo_ctx.as_ref() else {
                return Err("Cannot store log clustering metrics without a component key".into());
            };
            key = get_component_info(mezmo_ctx);
            if key.is_none() {
                return Err("Component key is not valid".into());
            }

            let store_metrics_flush_interval = self.store_metrics_flush_interval;
            let tx = ONCE
                .get_or_init(move || async move {
                    let (tx, rx) = mpsc::unbounded_channel();
                    // Start saving in the background
                    // This task will be running forever, topology changes should not affect it
                    tokio::spawn(async move {
                        save_in_loop(rx, config, store_metrics_flush_interval).await;
                    });

                    tx
                })
                .await;

            Some(tx.clone())
        } else {
            None
        };

        Ok(Transform::event_task(MezmoLogClustering::new(
            self, key, db_tx,
        )))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: enrichment::TableRegistry,
        _: &[(OutputId, Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Log, HashMap::new())]
    }

    fn enable_concurrency(&self) -> bool {
        false
    }
}

struct MezmoLogClustering {
    parser: drain::LogParser<'static>,
    cluster_field: Option<String>,
    store_metrics: bool,
    sample_start: Option<i64>,
    sample_end: Option<i64>,
    transform_status: Option<TransformStatus>,
    key: Option<ComponentInfo>,
    db_tx: Option<DbTransmitter>,
}

#[derive(Copy, Clone, PartialEq)]
enum TransformStatus {
    Noop,
    Store,
    AnnotateEvent,
}

pub(crate) struct LogGroupInfo {
    local_id: LocalId,
    cluster_id: String,
    size: i64,
    template: Option<String>,
    annotation_set: Option<AnnotationSet>,
    key: ComponentInfo,
}

impl MezmoLogClustering {
    pub fn new(
        config: &MezmoLogClusteringConfig,
        key: Option<ComponentInfo>,
        db_tx: Option<DbTransmitter>,
    ) -> Self {
        let similarity_threshold = if config.similarity_threshold > 1.0
            || config.similarity_threshold < 0.0
        {
            warn!("Similarity threshold should be between 0.0 and 1.0, but received {}. Using the default: {}",
            config.similarity_threshold,
            default_similarity_threshold());
            default_similarity_threshold()
        } else {
            config.similarity_threshold
        };

        let max_node_depth = if config.max_node_depth == 0 {
            warn!(
                "Attempted to use a max node depth of zero. Using the default: {}",
                default_max_node_depth()
            );
            default_max_node_depth()
        } else {
            config.max_node_depth
        };

        let max_children = if config.max_children == 0 {
            warn!(
                "Attempted to use a max children of zero. Using the default: {}",
                default_max_children()
            );
            default_max_children()
        } else {
            config.max_children
        };

        MezmoLogClustering {
            parser: drain::LogParser::new(NonZeroUsize::new(config.max_clusters).unwrap_or_else(
                || {
                    warn!(
                        "Attempted to use a max clusters of zero. Using the default: {}",
                        default_max_clusters()
                    );
                    NonZeroUsize::new(default_max_clusters()).unwrap()
                },
            ))
            .sim_threshold(similarity_threshold)
            .max_node_depth(max_node_depth)
            .max_children(max_children),
            store_metrics: config.store_metrics,
            sample_start: config.sample_start,
            sample_end: config.sample_end,
            transform_status: None,
            key,
            db_tx,
            cluster_field: config.cluster_field.clone(),
        }
    }

    /// Determines whether the Transform is:
    /// - Modifying the event with cluster information
    /// - Storing the event
    /// - Passing through the event (noop)
    fn get_transform_status(&self) -> TransformStatus {
        if !self.store_metrics {
            return TransformStatus::AnnotateEvent;
        }

        if let (Some(start), Some(end)) = (self.sample_start, self.sample_end) {
            let ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|t| t.as_millis() as i64)
                .unwrap_or(0);

            if ts > start && ts < end {
                TransformStatus::Store
            } else {
                // Outside the sample window
                TransformStatus::Noop
            }
        } else {
            // No sample window
            TransformStatus::Store
        }
    }

    fn transform_one(&mut self, mut event: Event) -> Option<Event> {
        let status = self.get_transform_status();
        let last_status = self.transform_status;
        self.transform_status = Some(status);

        if status == TransformStatus::Noop {
            if last_status == Some(TransformStatus::Store) {
                // We are no longer storing data from this component.
                // Free the parser memory by dropping the previous parser.
                self.parser = drain::LogParser::new(NonZeroUsize::new(1).unwrap());
            }

            return Some(event);
        }

        let log = event.as_mut_log();
        let (Some(field_name), Some(line)) = self.get_cluster_line(log) else {
            return Some(event);
        };
        let field_name = if status == TransformStatus::AnnotateEvent {
            // Copy for the AnnotateEvent case
            Some(field_name.to_string())
        } else {
            // Not needed
            None
        };

        let (group, group_status) = self.parser.add_log_line(line.as_ref());

        if status == TransformStatus::Store {
            let mut info = LogGroupInfo {
                local_id: group.local_id(),
                cluster_id: group.cluster_id(),
                size: line.as_ref().len() as i64,
                template: None,
                annotation_set: None,
                // The component_key was already validated to be "some" for the Store case
                key: self.key.as_ref().unwrap().clone(),
            };

            // Send the full cluster information only when it has added/changed
            if group_status != LogUpdateStatus::None {
                info.template = Some(format!("{}", group));

                if group_status == LogUpdateStatus::CreatedCluster {
                    info.annotation_set = log.as_map().and_then(get_annotations);
                }
            }

            if let Err(_) = self.db_tx.as_ref().expect("can't fail").send(info) {
                error!("Db channel closed");
            }
        } else if status == TransformStatus::AnnotateEvent {
            let mut cluster = BTreeMap::new();

            cluster.insert(
                "cluster_id".to_string(),
                Value::Bytes(group.cluster_id().into()),
            );
            cluster.insert(
                "match_count".to_string(),
                Value::Integer(group.match_count() as i64),
            );
            cluster.insert(
                "template".to_string(),
                Value::Bytes(format!("{}", group).into()),
            );

            log.insert(
                field_name.expect("to be set for annotate case").as_str(),
                Value::Object(cluster),
            );
        }

        Some(event)
    }

    /// Tries to get the line string from the log event using the cluster_field configured or
    /// using the .annotations.message_key or checking the value of .message.message or .message.
    fn get_cluster_line<'a>(
        &self,
        log: &'a LogEvent,
    ) -> (Option<Cow<'a, str>>, Option<Cow<'a, str>>) {
        let field_name: Option<Cow<'a, str>> = if let Some(field_name) = self.cluster_field.as_ref()
        {
            Some(Cow::Owned(field_name.as_str().to_string()))
        } else if let Some(field_name) =
            log.get((log_schema().annotations_key().to_string() + ".message_key").as_str())
        {
            field_name.as_str()
        } else {
            None
        };

        if field_name.is_none() {
            // Check .message property for backward compatibility
            let field_name = log_schema().message_key();
            if let Some(field) = log.get(field_name) {
                if field.is_bytes() {
                    return (Some(Cow::Owned(field_name.to_string())), field.as_str());
                }
            }
            return (None, None);
        }

        let line = field_name
            .as_ref()
            .and_then(|name| log.get(name.as_ref()))
            .and_then(|f| f.as_str());

        (field_name, line)
    }
}

impl TaskTransform<Event> for MezmoLogClustering {
    fn transform(
        self: Box<Self>,
        task: std::pin::Pin<Box<dyn futures_util::Stream<Item = Event> + Send>>,
    ) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Event> + Send>> {
        let mut inner = self;
        Box::pin(task.filter_map(move |v| ready(inner.transform_one(v))))
    }
}

#[derive(Default)]
struct LogGroupAggregateInfo {
    cluster_id: String,
    count: i64,
    size: i64,
    template: Option<String>,
    annotation_set: Option<AnnotationSet>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct ComponentInfo {
    account_id: Uuid,
    // The id of the shared route/source or other component that is being tracked
    component_id: String,
}

fn get_component_info(mezmo_ctx: &MezmoContext) -> Option<ComponentInfo> {
    Some(ComponentInfo {
        account_id: mezmo_ctx.account_id()?,
        component_id: mezmo_ctx.component_id().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::{default_store_metrics_flush_interval, MezmoLogClusteringConfig};

    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    use crate::{
        event::{Event, LogEvent},
        test_util::components::assert_transform_compliance,
        transforms::test::create_topology,
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<MezmoLogClusteringConfig>();
    }

    const fn make_transform_config() -> MezmoLogClusteringConfig {
        MezmoLogClusteringConfig {
            max_clusters: 100,
            similarity_threshold: 0.5,
            max_node_depth: 5,
            max_children: 100,
            cluster_field: None,
            store_metrics: false,
            sample_start: None,
            sample_end: None,
            store_metrics_flush_interval: default_store_metrics_flush_interval(),
        }
    }

    fn verify_cluster(
        event: Event,
        expected_template: &str,
        expected_cluster_id: &str,
        expected_match_count: usize,
    ) {
        let log = event.as_log();
        assert_eq!(
            expected_template,
            &log.get(".message.template").unwrap().to_string_lossy()
        );
        assert_eq!(
            expected_cluster_id,
            &log.get(".message.cluster_id").unwrap().to_string_lossy()
        );
        assert_eq!(
            expected_match_count as i64,
            log.get(".message.match_count")
                .unwrap()
                .as_integer()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn clustering() {
        let transform_config = make_transform_config();
        assert_transform_compliance(async {
            let (tx, rx) = mpsc::channel(1);
            let (topology, mut out) =
                create_topology(ReceiverStream::new(rx), transform_config).await;

            let mut log_parser = super::drain::LogParser::new(NonZeroUsize::new(1000).unwrap());

            tx.send(Event::Log(LogEvent::from("hi there 1")))
                .await
                .unwrap();
            let (cluster, _) = log_parser.add_log_line("hi there 1");
            let new_event = out.recv().await.unwrap();
            verify_cluster(new_event, "hi there <*>", &cluster.cluster_id(), 1);

            tx.send(Event::Log(LogEvent::from("hi there 2")))
                .await
                .unwrap();
            let new_event = out.recv().await.unwrap();
            let (cluster, _) = log_parser.add_log_line("hi there 2");
            verify_cluster(new_event, "hi there <*>", &cluster.cluster_id(), 2);

            tx.send(Event::Log(LogEvent::from("hi there 3")))
                .await
                .unwrap();
            let (cluster, _) = log_parser.add_log_line("hi there 3");
            let new_event = out.recv().await.unwrap();
            verify_cluster(new_event, "hi there <*>", &cluster.cluster_id(), 3);

            drop(tx);
            topology.stop().await;
            assert_eq!(out.recv().await, None);
        })
        .await;
    }
}
