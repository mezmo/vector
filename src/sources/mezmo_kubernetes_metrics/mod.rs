use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1::{Container, ContainerStatus, Node, Pod};
use kube::{
    Client,
    api::{Api, DynamicObject, GroupVersionKind, ListParams, ObjectList},
    discovery,
};
use serde_json::Value;
use serde_with::serde_as;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tracing::{error, info, trace, warn};
use vector_lib::{
    config::{LogNamespace, log_schema},
    configurable::configurable_component,
    event::{Event, LogEvent},
    schema,
};

use crate::{
    SourceSender,
    config::{DataType, SourceConfig, SourceContext, SourceOutput},
    shutdown::ShutdownSignal,
    sources::util::http_client::warn_if_interval_too_low,
};

use self::kube_stats::{
    cluster_stats::ClusterStats,
    container_stats::ContainerStats,
    controller_stats::ControllerStats,
    extended_pod_stats::ExtendedPodStats,
    node_stats::{NodeContainerStats, NodePodStats, NodeStats},
    pod_stats::PodStats,
};

mod kube_stats;

const SELF_NODE_NAME_ENV_KEY: &str = "VECTOR_SELF_NODE_NAME";

const fn default_scrape_interval() -> Duration {
    Duration::from_secs(30)
}

const fn default_scrape_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_self_node_name_env_template() -> String {
    format!("${{{SELF_NODE_NAME_ENV_KEY}}}")
}

/// Configuration for the `mezmo_kubernetes_metrics` source.
///
/// Collects pod, node, and cluster-level metrics from the Kubernetes API and
/// the metrics-server (`metrics.k8s.io/v1beta1`), emitting one JSON log event
/// per container, per node, and one cluster-aggregate event on each collection
/// interval.
#[serde_as]
#[configurable_component(source(
    "mezmo_kubernetes_metrics",
    "Collect Kubernetes pod, node, and cluster metrics via the Kubernetes API."
))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct MezmoKubernetesMetricsConfig {
    /// The interval between metric collection runs.
    ///
    /// Defaults to 30 seconds.
    #[serde(default = "default_scrape_interval")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(rename = "scrape_interval_secs")]
    pub interval: Duration,

    /// The timeout for each metric collection run.
    ///
    /// Defaults to 10 seconds.
    #[serde(default = "default_scrape_timeout")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(rename = "scrape_timeout_secs")]
    pub timeout: Duration,

    /// The name of the Kubernetes Node this pod is running on.
    ///
    /// Scopes metric collection to only the pods and node on which this
    /// agent is running. In a DaemonSet deployment, inject this via the
    /// Downward API:
    ///
    /// ```yaml
    /// env:
    ///   - name: VECTOR_SELF_NODE_NAME
    ///     valueFrom:
    ///       fieldRef:
    ///         fieldPath: spec.nodeName
    /// ```
    #[serde(default = "default_self_node_name_env_template")]
    pub self_node_name: String,
}

impl Default for MezmoKubernetesMetricsConfig {
    fn default() -> Self {
        Self {
            interval: default_scrape_interval(),
            timeout: default_scrape_timeout(),
            self_node_name: default_self_node_name_env_template(),
        }
    }
}

impl_generate_config_from_default!(MezmoKubernetesMetricsConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_kubernetes_metrics")]
impl SourceConfig for MezmoKubernetesMetricsConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let client = Client::try_default()
            .await
            .map_err(|e| format!("Failed to build Kubernetes client: {e}"))?;

        let node_name = if self.self_node_name.is_empty()
            || self.self_node_name == default_self_node_name_env_template()
        {
            std::env::var(SELF_NODE_NAME_ENV_KEY).map_err(|_| {
                format!(
                    "self_node_name config value or {SELF_NODE_NAME_ENV_KEY} env var is not set"
                )
            })?
        } else {
            self.self_node_name.clone()
        };

        warn_if_interval_too_low(self.timeout, self.interval);

        Ok(Box::pin(run(
            client,
            self.interval,
            self.timeout,
            node_name,
            cx.out,
            cx.shutdown,
        )))
    }

    fn outputs(&self, _global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        vec![SourceOutput::new_maybe_logs(
            DataType::Log,
            schema::Definition::default_legacy_namespace(),
        )]
    }

    fn can_acknowledge(&self) -> bool {
        false
    }
}

async fn run(
    client: Client,
    interval: Duration,
    timeout: Duration,
    node_name: String,
    mut out: SourceSender,
    shutdown: ShutdownSignal,
) -> Result<(), ()> {
    info!(message = "Starting Kubernetes metrics collection.", %node_name);

    let mut ticker = IntervalStream::new(time::interval(interval)).take_until(shutdown);

    while ticker.next().await.is_some() {
        match tokio::time::timeout(timeout, process_reporter_info(client.clone(), &node_name)).await
        {
            Ok(Ok((pods, nodes, cluster))) => {
                let now = Utc::now();
                let events = pods
                    .into_iter()
                    .chain(nodes)
                    .chain(std::iter::once(cluster))
                    .filter_map(|line| {
                        serde_json::from_str::<serde_json::Value>(&line)
                            .ok()
                            .map(|v| {
                                let mut log = LogEvent::default();
                                log.insert(log_schema().message_key_target_path().unwrap(), v);
                                LogNamespace::Legacy.insert_standard_vector_source_metadata(
                                    &mut log,
                                    MezmoKubernetesMetricsConfig::NAME,
                                    now,
                                );
                                Event::Log(log)
                            })
                    })
                    .collect::<Vec<Event>>();

                if out.send_batch(events).await.is_err() {
                    error!(
                        message =
                            "Failed to send Kubernetes metrics; downstream may have shut down."
                    );
                    return Err(());
                }
            }
            Ok(Err(e)) => error!(message = "Failed to gather Kubernetes metrics.", error = %e),
            Err(_) => warn!(message = "Kubernetes metrics collection timed out."),
        }
    }

    Ok(())
}

async fn process_reporter_info(
    client: Client,
    node_name: &str,
) -> Result<(Vec<String>, Vec<String>, String)> {
    trace!("Generating Kubernetes metrics report.");

    let pods = get_all_pods(client.clone(), node_name).await?;
    let nodes = get_all_nodes(client.clone(), node_name).await?;
    let pod_metrics = call_metric_api("PodMetrics", client.clone(), None).await?;
    let node_metrics = call_metric_api(
        "NodeMetrics",
        client.clone(),
        Some(&format!("metadata.name={node_name}")),
    )
    .await?;

    let mut controller_map: HashMap<String, ControllerStats> = HashMap::new();
    let mut node_pod_counts_map: HashMap<String, NodePodStats> = HashMap::new();
    let mut node_container_counts_map: HashMap<String, NodeContainerStats> = HashMap::new();
    let mut pod_usage_map: HashMap<String, Value> = HashMap::new();
    let mut node_usage_map: HashMap<String, Value> = HashMap::new();
    let mut extended_pod_stats: Vec<ExtendedPodStats> = Vec::new();
    let mut node_stats: Vec<NodeStats> = Vec::new();

    build_pod_metric_map(pod_metrics, &mut pod_usage_map);
    process_pods(
        pods,
        &mut controller_map,
        pod_usage_map,
        &mut extended_pod_stats,
        &mut node_pod_counts_map,
        &mut node_container_counts_map,
    );
    let pods_strings = format_pod_str(extended_pod_stats, controller_map);

    build_node_metric_map(node_metrics, &mut node_usage_map);
    process_nodes(
        nodes,
        node_usage_map,
        &mut node_stats,
        &mut node_pod_counts_map,
        &mut node_container_counts_map,
    );

    let node_strings = format_node_str(&node_stats);
    let cluster_stats = build_cluster_stats(&node_stats);
    let cluster_stats_string = format_cluster_str(&cluster_stats);

    Ok((pods_strings, node_strings, cluster_stats_string))
}

fn build_pod_metric_map(
    pod_metrics: ObjectList<DynamicObject>,
    pod_usage_map: &mut HashMap<String, Value>,
) {
    for pod_metric in pod_metrics {
        let pod_name = pod_metric.metadata.name.as_deref().unwrap_or("");
        let namespace = pod_metric.metadata.namespace.as_deref().unwrap_or("");

        if let Some(containers) = pod_metric.data["containers"].as_array() {
            for container in containers {
                if let Some(container_name) = container["name"].as_str() {
                    let key = format!("{namespace}/{pod_name}/{container_name}");
                    pod_usage_map.insert(key, container["usage"].clone());
                }
            }
        }
    }
}

fn build_node_metric_map(
    node_metrics: ObjectList<DynamicObject>,
    node_usage_map: &mut HashMap<String, Value>,
) {
    for node_metric in node_metrics {
        let node_name = node_metric
            .metadata
            .name
            .unwrap_or_else(|| "NONE".to_string());
        node_usage_map.insert(node_name, node_metric.data["usage"].clone());
    }
}

fn build_cluster_stats(node_stats: &[NodeStats]) -> ClusterStats {
    macro_rules! aggregate_stat {
        ($acc:ident, $node:ident, $field:ident) => {
            $acc.$field = $acc.$field.map_or($node.$field, |current| {
                $node.$field.map(|new| current + new)
            });
        };
    }

    let mut cluster = ClusterStats::new();

    for node in node_stats {
        cluster.containers_init += node.containers_init;
        cluster.containers_ready += node.containers_ready;
        cluster.containers_running += node.containers_running;
        cluster.containers_terminated += node.containers_terminated;
        cluster.containers_total += node.containers_total;
        cluster.containers_waiting += node.containers_waiting;
        cluster.pods_failed += node.pods_failed;
        cluster.pods_pending += node.pods_pending;
        cluster.pods_running += node.pods_running;
        cluster.pods_succeeded += node.pods_succeeded;
        cluster.pods_total += node.pods_total;
        cluster.pods_unknown += node.pods_unknown;
        cluster.nodes_total += 1;

        aggregate_stat!(cluster, node, cpu_usage);
        aggregate_stat!(cluster, node, memory_usage);
        aggregate_stat!(cluster, node, cpu_allocatable);
        aggregate_stat!(cluster, node, cpu_capacity);
        aggregate_stat!(cluster, node, memory_allocatable);
        aggregate_stat!(cluster, node, memory_capacity);
        aggregate_stat!(cluster, node, pods_allocatable);
        aggregate_stat!(cluster, node, pods_capacity);

        if node.ready.unwrap_or(false) {
            cluster.nodes_ready += 1;
        } else {
            cluster.nodes_notready += 1;
        }

        if node.unschedulable.unwrap_or(false) {
            cluster.nodes_unschedulable += 1;
        }
    }

    cluster
}

fn format_pod_str(
    extended_pod_stats: Vec<ExtendedPodStats>,
    controller_map: HashMap<String, ControllerStats>,
) -> Vec<String> {
    extended_pod_stats
        .into_iter()
        .map(|mut stat| {
            let key = format!(
                "{}.{}.{}",
                stat.pod_stats.namespace, stat.pod_stats.controller_type, stat.pod_stats.controller
            );
            if let Some(controller) = controller_map.get(&key) {
                stat.controller_stats.copy_stats(controller);
            }
            format!(
                r#"{{"kube":{}}}"#,
                serde_json::to_string(&stat).unwrap_or_default()
            )
        })
        .inspect(|s| trace!("{}", s))
        .collect()
}

fn format_node_str(nodes: &[NodeStats]) -> Vec<String> {
    nodes
        .iter()
        .map(|node| {
            let s = format!(
                r#"{{"kube":{}}}"#,
                serde_json::to_string(node).unwrap_or_default()
            );
            trace!("{}", s);
            s
        })
        .collect()
}

fn format_cluster_str(cluster: &ClusterStats) -> String {
    let s = format!(
        r#"{{"kube":{}}}"#,
        serde_json::to_string(cluster).unwrap_or_default()
    );
    trace!("{}", s);
    s
}

fn process_pods(
    pods: ObjectList<Pod>,
    controller_map: &mut HashMap<String, ControllerStats>,
    pod_usage_map: HashMap<String, Value>,
    extended_pod_stats: &mut Vec<ExtendedPodStats>,
    node_pod_counts_map: &mut HashMap<String, NodePodStats>,
    node_container_counts_map: &mut HashMap<String, NodeContainerStats>,
) {
    let empty_vec = Vec::new();

    for pod in pods {
        let (Some(status), Some(spec)) = (pod.status.as_ref(), pod.spec.as_ref()) else {
            continue;
        };

        if status.conditions.is_none() || status.container_statuses.is_none() {
            continue;
        }

        let translated_pod = PodStats::from(&pod);
        let node = translated_pod.node.clone();
        let phase = translated_pod.phase.clone();

        node_pod_counts_map
            .entry(node.clone())
            .or_default()
            .inc(&phase);

        let controller_key = format!(
            "{}.{}.{}",
            translated_pod.namespace, translated_pod.controller_type, translated_pod.controller
        );

        let controller = controller_map.entry(controller_key.clone()).or_default();

        if let Some(conditions) = &status.conditions
            && conditions
                .iter()
                .any(|c| c.status.to_lowercase() == "true" && c.type_.to_lowercase() == "ready")
        {
            controller.inc_pods_ready();
        }
        controller.inc_pods_total();

        let mut container_status_map = HashMap::new();

        for cs in status
            .container_statuses
            .as_ref()
            .unwrap_or(&empty_vec)
            .iter()
            .chain(
                status
                    .init_container_statuses
                    .as_ref()
                    .unwrap_or(&empty_vec)
                    .iter(),
            )
        {
            container_status_map.insert(cs.name.clone(), cs.clone());

            let controller = controller_map.entry(controller_key.clone()).or_default();
            controller.inc_containers_total();
            if cs.ready {
                controller.inc_containers_ready();
            }
        }

        for container in &spec.containers {
            if container.name.is_empty()
                || container.image.is_none()
                || container.resources.is_none()
            {
                continue;
            }

            let container_status = container_status_map.get(&container.name);
            if container_status.is_none() {
                continue;
            }

            if let Some(stat) = build_extended_pod_stat(
                &pod_usage_map,
                container,
                container_status,
                &translated_pod,
            ) {
                node_container_counts_map
                    .entry(node.clone())
                    .or_default()
                    .inc(
                        &stat.container_stats.state,
                        stat.container_stats.ready,
                        false,
                    );
                extended_pod_stats.push(stat);
            }
        }

        let empty_containers = Vec::new();
        for init_container in spec.init_containers.as_ref().unwrap_or(&empty_containers) {
            if init_container.name.is_empty()
                || init_container.image.is_none()
                || init_container.resources.is_none()
            {
                continue;
            }

            let container_status = container_status_map.get(&init_container.name);
            if container_status.is_none() {
                continue;
            }

            if let Some(stat) = build_extended_pod_stat(
                &pod_usage_map,
                init_container,
                container_status,
                &translated_pod,
            ) {
                node_container_counts_map
                    .entry(node.clone())
                    .or_default()
                    .inc(
                        &stat.container_stats.state,
                        stat.container_stats.ready,
                        true,
                    );
                extended_pod_stats.push(stat);
            }
        }
    }
}

fn build_extended_pod_stat(
    pod_usage_map: &HashMap<String, Value>,
    container: &Container,
    container_status: Option<&ContainerStatus>,
    translated_pod: &PodStats,
) -> Option<ExtendedPodStats> {
    let key = format!(
        "{}/{}/{}",
        translated_pod.namespace, translated_pod.pod, container.name
    );
    let usage = pod_usage_map.get(&key)?;
    let c_status = container_status?;

    let translated_container = ContainerStats::new(
        container,
        c_status,
        c_status.state.as_ref()?,
        usage["cpu"].as_str().unwrap_or(""),
        usage["memory"].as_str().unwrap_or(""),
    );

    Some(ExtendedPodStats::new(
        translated_pod.clone(),
        translated_container,
    ))
}

fn process_nodes(
    nodes: ObjectList<Node>,
    node_usage_map: HashMap<String, Value>,
    output_node_vec: &mut Vec<NodeStats>,
    node_pod_counts_map: &mut HashMap<String, NodePodStats>,
    node_container_counts_map: &mut HashMap<String, NodeContainerStats>,
) {
    for node in nodes {
        if node.spec.is_none() || node.status.is_none() || node.metadata.name.is_none() {
            continue;
        }

        let name = node.metadata.name.as_ref().unwrap();
        let default_container_stats = NodeContainerStats::new();
        let default_pod_stats = NodePodStats::new();

        let node_container_stats = node_container_counts_map
            .get(name)
            .unwrap_or(&default_container_stats);
        let node_pod_stats = node_pod_counts_map.get(name).unwrap_or(&default_pod_stats);

        if let Some(usage) = node_usage_map.get(name) {
            output_node_vec.push(NodeStats::new(
                &node,
                node_pod_stats,
                node_container_stats,
                usage["cpu"].as_str().unwrap_or(""),
                usage["memory"].as_str().unwrap_or(""),
            ));
        }
    }
}

async fn call_metric_api(
    kind: &str,
    client: Client,
    field_selector: Option<&str>,
) -> Result<ObjectList<DynamicObject>, kube::Error> {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", kind);
    let (ar, _caps) = discovery::pinned_kind(&client, &gvk).await?;
    let api = Api::<DynamicObject>::all_with(client, &ar);
    let params = match field_selector {
        Some(selector) => ListParams::default().fields(selector),
        None => ListParams::default(),
    };
    api.list(&params).await
}

async fn get_all_nodes(client: Client, node_name: &str) -> Result<ObjectList<Node>, kube::Error> {
    Api::<Node>::all(client)
        .list(&ListParams::default().fields(&format!("metadata.name={node_name}")))
        .await
}

async fn get_all_pods(client: Client, node_name: &str) -> Result<ObjectList<Pod>, kube::Error> {
    Api::<Pod>::all(client)
        .list(&ListParams::default().fields(&format!("spec.nodeName={node_name}")))
        .await
}
