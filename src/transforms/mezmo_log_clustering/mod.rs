use std::{
    collections::{BTreeMap, HashMap},
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
use vector_config::configurable_component;
use vector_core::config::{log_schema, TransformOutput};

use vrl::value::Value;

mod drain;

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
}

const fn default_max_clusters() -> usize {
    1000
}

const fn default_similarity_threshold() -> f64 {
    0.5
}

const fn default_max_node_depth() -> usize {
    5
}

const fn default_max_children() -> usize {
    100
}

impl_generate_config_from_default!(MezmoLogClusteringConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_log_clustering")]
impl TransformConfig for MezmoLogClusteringConfig {
    async fn build(&self, _context: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::event_task(MezmoLogClustering::new(self)))
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
        true
    }
}

pub struct MezmoLogClustering {
    parser: drain::LogParser<'static>,
    cluster_field: String,
}

impl MezmoLogClustering {
    pub fn new(config: &MezmoLogClusteringConfig) -> Self {
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
            cluster_field: config
                .cluster_field
                .as_deref()
                .unwrap_or_else(|| log_schema().message_key())
                .to_string(),
        }
    }

    fn transform_one(&mut self, mut event: Event) -> Option<Event> {
        let log = event.as_mut_log();
        let field = log.get_mut(self.cluster_field.as_str());

        if let Some(field) = field {
            let line = field.to_string_lossy();
            let (group, _) = self.parser.add_log_line(line.as_ref());

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

            log.insert(self.cluster_field.as_str(), Value::Object(cluster));
        }

        Some(event)
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::MezmoLogClusteringConfig;

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
