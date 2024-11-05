use crate::mezmo::choose_weighted;
use chrono::Utc;
use faker_rand::{en_us::internet::Domain, lorem::Word};
use rand::{thread_rng, Rng};
use serde::Serialize;

const LEVELS: [(&str, f32); 2] = [("INFO", 15.0), ("ERROR", 1.0)];
const APPS: [(&str, f32); 3] = [
    ("user-service", 8.0),
    ("notification-service", 3.0),
    ("analytics-engine", 0.5),
];
const CONTAINERS: [(&str, f32); 3] = [
    ("liveness-probe", 8.0),
    ("sysdig-runtime-scanner", 2.5),
    ("kafka-broker", 1.5),
];
const INFO_MESSAGES: [(&str, f32); 10] = [
    (
        "Successfully created the pod 'user-service' with 2 replicas.",
        16.0,
    ),
    ("Deployment 'frontend' scaled down to 3 replicas.", 10.0),
    (
        "Node 'node-05' became ready and is now available for scheduling.",
        6.0,
    ),
    (
        "Service 'backend-api' is now accessible at endpoint 'http://backend-api:8080'.",
        2.0,
    ),
    ("ConfigMap 'app-config' updated with new settings.", 1.0),
    (
        "Pod 'order-service' started successfully and is now running.",
        1.0,
    ),
    (
        "PersistentVolumeClaim 'data-claim' successfully bound to 'data-volume'.",
        0.5,
    ),
    (
        "Ingress 'web-ingress' has been updated with new routing rules.",
        1.0,
    ),
    (
        "Pod 'task-processor' completed execution and is terminating.",
        1.0,
    ),
    (
        "Service 'frontend' has been assigned the ClusterIP '10.0.0.10'.",
        1.0,
    ),
];
const ERROR_MESSAGES: [(&str, f32); 10] = [
    (
        "Failed to create the pod 'user-service' due to resource limits.",
        10.0,
    ),
    (
        "Deployment 'frontend' failed to scale due to insufficient nodes.",
        7.0,
    ),
    (
        "Node 'node-05' is not responding; marking as unhealthy.",
        3.0,
    ),
    (
        "Error pulling image 'nginx:latest' for pod 'frontend-pod'.",
        1.0,
    ),
    (
        "PersistentVolumeClaim 'data-claim' could not be bound.",
        1.0,
    ),
    ("Pod 'order-service' crashed with exit code 1.", 1.0),
    (
        "Failed to update ConfigMap 'app-config' due to invalid syntax.",
        0.5,
    ),
    (
        "Service 'backend-api' is not reachable from the cluster.",
        0.0,
    ),
    ("Error while applying Ingress 'web-ingress' rules.", 1.0),
    ("Pod 'task-processor' is in CrashLoopBackOff state.", 1.0),
];

pub fn kubernetes_log_line() -> KubernetesLog {
    KubernetesLog::new()
}

#[derive(Debug, Serialize)]
pub struct KubernetesLog {
    app: String,
    container: String,
    host: String,
    node: String,
    pod: String,
    namespace: String,
    level: String,
    line: String,
}

impl KubernetesLog {
    fn new() -> Self {
        let app = choose_weighted(&APPS);
        let level = choose_weighted(&LEVELS);
        Self {
            app: app.to_string(),
            container: choose_weighted(&CONTAINERS).to_string(),
            host: thread_rng().gen::<Domain>().to_string(),
            node: thread_rng().gen::<Word>().to_string(),
            pod: thread_rng().gen::<Word>().to_string(),
            namespace: thread_rng().gen::<Word>().to_string(),
            level: level.to_string(),
            line: kubernetes_message(level, app),
        }
    }
}

fn kubernetes_message(level: &str, app: &str) -> String {
    let msg = match level {
        "ERROR" => choose_weighted(&ERROR_MESSAGES).to_string(),
        _ => choose_weighted(&INFO_MESSAGES).to_string(),
    };
    // message format:
    // <datetime:YYYY-MM-DDTHH:MM:SS.MMMZ> <level> <component>: <message>
    format!(
        "{} {} {}: {}",
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        level,
        app,
        msg
    )
}
