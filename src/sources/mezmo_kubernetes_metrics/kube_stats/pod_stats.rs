use chrono::Utc;
use k8s_openapi::{api::core::v1::Pod, apimachinery::pkg::apis::meta::v1::OwnerReference};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PodStats {
    pub resource: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub controller: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub controller_type: String,
    pub created: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub namespace: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub node: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phase: String,
    pub pod_age: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pod: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub priority_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub qos_class: String,
}

impl From<&Pod> for PodStats {
    fn from(p: &Pod) -> Self {
        let (controller, controller_type) = get_controller_details(&p.metadata.owner_references);

        let namespace = p.metadata.namespace.clone().unwrap_or_default();
        let pod = p.metadata.name.clone().unwrap_or_default();

        let mut priority_class = String::new();
        let mut node = String::new();
        let mut ip = String::new();
        let mut phase = String::new();
        let mut qos_class = String::new();
        let mut pod_age = 0i64;
        let mut created = 0i64;
        let mut priority = None;

        if let Some(spec) = &p.spec {
            priority = spec.priority;

            if let Some(name) = &spec.priority_class_name {
                priority_class.clone_from(name);
            }

            if let Some(name) = &spec.node_name {
                node.clone_from(name);
            }
        }

        if let Some(status) = &p.status {
            if let Some(pod_created) = status.start_time.clone() {
                pod_age = Utc::now()
                    .signed_duration_since(pod_created.0)
                    .num_milliseconds();
                created = pod_created.0.timestamp_millis();
            }

            if let Some(pod_ip) = &status.pod_ip {
                ip.clone_from(pod_ip);
            }

            if let Some(p) = &status.phase {
                phase.clone_from(p);
            }

            if let Some(qos) = &status.qos_class {
                qos_class.clone_from(qos);
            }
        }

        PodStats {
            controller,
            controller_type,
            created,
            ip,
            namespace,
            node,
            phase,
            pod_age,
            pod,
            priority_class,
            priority,
            qos_class,
            resource: "container".to_string(),
            r#type: "metric".to_string(),
        }
    }
}

fn get_controller_details(owners: &Option<Vec<OwnerReference>>) -> (String, String) {
    if let Some(owners) = owners {
        for owner in owners {
            if owner.controller == Some(true) {
                return (owner.name.clone(), owner.kind.clone());
            }
        }
    }
    (String::new(), String::new())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use k8s_openapi::{
        api::core::v1::{Pod, PodSpec, PodStatus},
        apimachinery::pkg::apis::meta::v1::Time,
    };
    use kube::api::ObjectMeta;

    use super::PodStats;

    #[tokio::test]
    async fn test_create_pod() {
        let spec = create_spec();
        let status = create_status();
        let pod = create_pod(Some(spec), Some(status));

        let result = PodStats::from(&pod);

        assert_eq!(result.ip, "ip".to_string());
        assert_eq!(result.phase, "phase".to_string());
        assert_eq!(result.priority_class, "p_class".to_string());
        assert_eq!(result.node, "node_name".to_string());
        assert_eq!(result.qos_class, "class".to_string());
        assert_eq!(result.namespace, "namespace".to_string());
        assert_eq!(result.pod, "name".to_string());
        assert_eq!(result.priority.unwrap(), 222);
    }

    #[tokio::test]
    async fn test_create_no_spec() {
        let status = create_status();
        let pod = create_pod(None, Some(status));

        let result = PodStats::from(&pod);

        assert_eq!(result.node, "".to_string());
    }

    #[tokio::test]
    async fn test_create_no_status() {
        let spec = create_spec();
        let pod = create_pod(Some(spec), None);

        let result = PodStats::from(&pod);

        assert_eq!(result.phase, "".to_string());
    }

    fn create_pod(spec: Option<PodSpec>, status: Option<PodStatus>) -> Pod {
        let meta = ObjectMeta {
            annotations: None,
            creation_timestamp: Some(Time(Utc::now())),
            deletion_grace_period_seconds: None,
            deletion_timestamp: None,
            finalizers: None,
            generate_name: None,
            generation: None,
            labels: None,
            managed_fields: None,
            name: Some("name".to_string()),
            namespace: Some("namespace".to_string()),
            owner_references: None,
            resource_version: None,
            self_link: None,
            uid: None,
        };

        Pod {
            metadata: meta,
            spec,
            status,
        }
    }

    fn create_spec() -> PodSpec {
        PodSpec {
            active_deadline_seconds: None,
            affinity: None,
            automount_service_account_token: None,
            containers: Vec::new(),
            dns_config: None,
            dns_policy: None,
            enable_service_links: None,
            ephemeral_containers: None,
            host_aliases: None,
            host_ipc: None,
            host_network: None,
            host_pid: None,
            hostname: None,
            image_pull_secrets: None,
            init_containers: None,
            node_name: Some("node_name".to_string()),
            node_selector: None,
            overhead: None,
            preemption_policy: None,
            priority: Some(222),
            priority_class_name: Some("p_class".to_string()),
            readiness_gates: None,
            restart_policy: None,
            runtime_class_name: None,
            scheduler_name: None,
            security_context: None,
            service_account: None,
            service_account_name: None,
            share_process_namespace: None,
            subdomain: None,
            termination_grace_period_seconds: None,
            tolerations: None,
            topology_spread_constraints: None,
            volumes: None,
            set_hostname_as_fqdn: None,
            host_users: None,
            os: None,
            resource_claims: None,
            scheduling_gates: None,
        }
    }

    fn create_status() -> PodStatus {
        PodStatus {
            conditions: None,
            container_statuses: None,
            ephemeral_container_statuses: None,
            host_ip: None,
            init_container_statuses: None,
            message: None,
            nominated_node_name: None,
            phase: Some("phase".to_string()),
            pod_ip: Some("ip".to_string()),
            pod_ips: None,
            qos_class: Some("class".to_string()),
            reason: None,
            start_time: Some(Time(Utc::now())),
        }
    }
}
