use chrono::Utc;
use futures_util::StreamExt;
use reqwest::{
    header::{self, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, Instant},
};
use tokio::time::sleep;
use url::Url;
use vector_lib::api_client::{
    connect_subscription_client,
    gql::{
        output_events_by_component_id_patterns_subscription::OutputEventsByComponentIdPatternsSubscriptionOutputEventsByComponentIdPatterns,
        TapEncodingFormat, TapSubscriptionExt,
    },
};

use crate::config;

const TASK_INITIAL_POLL_DELAY: Duration = Duration::from_secs(5);
const TASK_POLL_STEP_DELAY: Duration = Duration::from_millis(500);
const TASK_MAX_AGE_SECS: isize = 120;
const DEFAULT_TAP_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_TAP_LIMIT_PER_INTERVAL: isize = 10;
const DEFAULT_TASK_CYCLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Flush buffered events eagerly, at the expense of I/O overhead.
/// See [vector::api::schema::events::create_events_stream] for more info.
const SUBSCRIPTION_FLUSH_INTERVAL_MS: i64 = 100;

/// Fetches and executes tasks for local deployments, mainly used for remote tapping.
pub(crate) async fn start_polling_for_tasks(
    config: config::api::Options,
    auth_token: String,
    get_endpoint_url: String,
    post_endpoint_url: String,
) {
    let task_execution_timeout = std::env::var("MEZMO_REMOTE_TASK_EXECUTION_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok().map(Duration::from_secs))
        .unwrap_or(DEFAULT_TASK_CYCLE_TIMEOUT);

    sleep(TASK_INITIAL_POLL_DELAY).await;
    info!("Starting to poll for tasks (task_execution_timeout = {task_execution_timeout:?}");
    let mut client = Client::new();
    loop {
        let start = Instant::now();
        let task_fut = run_task_step(
            &config,
            &client,
            &auth_token,
            &get_endpoint_url,
            &post_endpoint_url,
        );

        if let Err(_) = tokio::time::timeout(task_execution_timeout, task_fut).await {
            warn!("Remote task execution timed out");
            client = Client::new();
        }

        let elapsed = start.elapsed();
        if elapsed < TASK_POLL_STEP_DELAY {
            let sleep_duration = TASK_POLL_STEP_DELAY - elapsed;
            debug!("sleeping for {sleep_duration:?}");
            sleep(sleep_duration).await;
        }
    }
}

async fn run_task_step(
    config: &config::api::Options,
    client: &Client,
    auth_token: &str,
    get_endpoint_url: &str,
    post_endpoint_url: &str,
) {
    let tasks = fetch_tasks(client, auth_token, get_endpoint_url)
        .await
        .unwrap_or_else(|e| {
            warn!("Remote task fetch failed: {e}");
            vec![]
        });

    if !tasks.is_empty() {
        info!("Obtained {} task(s) for execution", tasks.len());
    }

    for t in tasks.into_iter() {
        if t.age_secs > TASK_MAX_AGE_SECS {
            // The user is likely not waiting anymore for this task to complete.
            // The user can retry and, by ignoring old tasks, we will be able to catch up faster.
            info!(
                "Remote task {} ignored due to age of {} secs",
                t.task_id, t.age_secs
            );
            continue;
        }

        let results = execute_task(&t, config).await;
        if let Err(e) = post_task_results(client, auth_token, post_endpoint_url, &t, &results).await
        {
            warn!(
                "There was an error when posting task results for {}: {}",
                t.task_id, e
            );
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    task_id: String,
    task_type: String,
    task_parameters: TaskParameters,
    age_secs: isize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskParameters {
    limit: Option<isize>,
    timeout_ms: Option<u64>,
    component_id: Option<String>,
    filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskFetchResponse {
    data: Vec<Task>,
}

type Err = String;
type TaskResult = Vec<HashMap<String, String>>;

trait TaskPostRequest {
    fn to_json(&self) -> Value;
}

impl TaskPostRequest for Result<TaskResult, Err> {
    fn to_json(&self) -> Value {
        match self {
            Ok(results) => json!({
                "data": {
                    "events": results,
                }
            }),
            Err(e) => {
                json!({ "errors": [e] })
            }
        }
    }
}

enum TaskType {
    Tap,
}

impl FromStr for TaskType {
    type Err = String;

    fn from_str(input: &str) -> Result<TaskType, Self::Err> {
        match input {
            "tap" => Ok(TaskType::Tap),
            v => Err(format!("Unknown task type: {v}")),
        }
    }
}

fn get_headers(auth_token: &str) -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("Mezmo Pulse"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Token {auth_token}")).unwrap(),
    );

    headers
}

async fn fetch_tasks(
    client: &Client,
    auth_token: &str,
    endpoint_url: &str,
) -> Result<Vec<Task>, Err> {
    let resp = client
        .get(endpoint_url)
        .headers(get_headers(auth_token))
        .send()
        .await
        .map_err(|e| format!("Connection error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Response error - {:?}", resp.status()));
    }

    let r = resp
        .json::<TaskFetchResponse>()
        .await
        .map_err(|e| format!("JSON deserialization error - {:?}", e))?;

    Ok(r.data)
}

async fn post_task_results(
    client: &Client,
    auth_token: &str,
    endpoint_url: &str,
    task: &Task,
    results: &Result<TaskResult, Err>,
) -> Result<(), Err> {
    let endpoint_url = endpoint_url.replace(":task_id", &task.task_id);

    let resp = client
        .post(&endpoint_url)
        .json(&results.to_json())
        .headers(get_headers(auth_token))
        .send()
        .await
        .map_err(|e| format!("Connection error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Response error when posting results - {:?}",
            resp.status()
        ));
    }

    Ok(())
}

async fn execute_task(task: &Task, config: &config::api::Options) -> Result<TaskResult, Err> {
    let task_type = TaskType::from_str(&task.task_type)?;
    match task_type {
        TaskType::Tap => tap(task, config).await,
    }
}

async fn tap(task: &Task, config: &config::api::Options) -> Result<TaskResult, Err> {
    let addr = config.address.expect("API address not to be empty");

    // For the MVP, we only support a single pod at a time
    // In the future, we should query the K8s API to get the deployment pods
    let url = Url::parse(&format!("ws://{}/graphql", addr)).expect("Couldn't parse API URL.");

    let component_id = task
        .task_parameters
        .component_id
        .as_ref()
        .ok_or_else(|| "component_id not set in parameters".to_string())?;

    let limit = task
        .task_parameters
        .limit
        .unwrap_or(DEFAULT_TAP_LIMIT_PER_INTERVAL);

    let filter = task.task_parameters.filter.clone();

    let subscription_client = connect_subscription_client(url)
        .await
        .map_err(|e| format!("Couldn't connect to Vector API via WebSockets: {}", e))?;

    tokio::pin! {
        let stream = subscription_client.output_events_by_component_id_patterns_subscription(
            vec![component_id.to_string()],
            vec![],
            filter,
            TapEncodingFormat::Json,
            limit as i64,
            SUBSCRIPTION_FLUSH_INTERVAL_MS,
        );
    };

    let tap_timeout = task
        .task_parameters
        .timeout_ms
        .map_or(DEFAULT_TAP_TIMEOUT, |timeout_ms| {
            Duration::from_millis(timeout_ms)
        });

    let mut result = Vec::new();
    let sleep_future = sleep(tap_timeout);
    tokio::pin!(sleep_future);

    loop {
        tokio::select! {
            biased;
            _ = &mut sleep_future => {
                // Exit loop and drop
                info!("Remote tap finished after {:?}", tap_timeout);
                break;
            }
            message = stream.next() => {
                if let Some(Some(res)) = message {
                    if let Some(d) = res.data {
                        for tap_event in d.output_events_by_component_id_patterns.iter() {
                            match tap_event {
                                OutputEventsByComponentIdPatternsSubscriptionOutputEventsByComponentIdPatterns::Log(ev) => {
                                    result.push(HashMap::from([
                                        ("type".to_string(), "Log".to_string()),
                                        ("timestamp".to_string(), ev.timestamp.unwrap_or_else(Utc::now).to_string()),
                                        ("message".to_string(), ev.mezmo_message.clone().unwrap_or_default()),
                                        ("metadata".to_string(), ev.mezmo_metadata.clone().unwrap_or_default()),
                                    ]));
                                },
                                OutputEventsByComponentIdPatternsSubscriptionOutputEventsByComponentIdPatterns::Metric(ev) => {
                                    result.push(HashMap::from([
                                        ("type".to_string(), "Metric".to_string()),
                                        ("message".to_string(), ev.string.clone()),
                                    ]));
                                },
                                OutputEventsByComponentIdPatternsSubscriptionOutputEventsByComponentIdPatterns::Trace(ev) => {
                                    result.push(HashMap::from([
                                        ("type".to_string(), "Trace".to_string()),
                                        ("message".to_string(), ev.string.clone()),
                                    ]));
                                },
                                OutputEventsByComponentIdPatternsSubscriptionOutputEventsByComponentIdPatterns::EventNotification(ev) => {
                                    debug!("TAP event notification: {}", ev.message);
                                },
                            }
                        }
                    }
                } else {
                    warn!("There was a temporary failure when running tap");
                    break;
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use httptest::{
        matchers::{all_of, json_decoded, request},
        responders::{json_encoded, status_code},
        Expectation, Server,
    };
    use serde_json::json;

    #[tokio::test]
    async fn fetches_tasks_and_reports_errors() {
        let get_path = "/fake/get/url";
        let post_path = "/fake/post/:task_id/url";
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![request::method("GET"), request::path(get_path),])
                .times(1)
                .respond_with(json_encoded(json!({
                    "data": [{
                        "task_id": "task1",
                        "task_type": "tap",
                        "age_secs": 1,
                        "task_parameters": {"component_id": "comp1", "limit": 1, "timeout_ms": 1000},
                    }]
                }))),
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("POST"),
                request::path("/fake/post/task1/url"),
                request::body(json_decoded(|v: &serde_json::Value| -> bool {
                    if let Some(errors) = v["errors"].as_array() {
                        // API is not enabled in the test so the graphQL query should fail
                        // with a single entry containing the error
                        return errors.len() == 1 && errors[0].is_string();
                    }
                    false
                })),
            ])
            .times(1)
            .respond_with(status_code(200)),
        );

        let get_url = format!("http://{}{}", server.addr(), get_path);
        let post_url = format!("http://{}{}", server.addr(), post_path);
        let client = Client::new();

        run_task_step(&Default::default(), &client, "token", &get_url, &post_url).await;
    }

    #[tokio::test]
    async fn fetches_tasks_including_filters() {
        let get_path = "/fake/get/url";
        let post_path = "/fake/post/:task_id/url";
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![request::method("GET"), request::path(get_path),])
                .times(1)
                .respond_with(json_encoded(json!({
                    "data": [{
                        "task_id": "task1",
                        "task_type": "tap",
                        "age_secs": 1,
                        "task_parameters": {
                            "component_id": "comp1",
                            "limit": 1,
                            "timeout_ms": 1000,
                            "filter": ".message == \"hello\""
                        },
                    }]
                }))),
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("POST"),
                request::path("/fake/post/task1/url"),
            ])
            .times(1)
            .respond_with(status_code(200)),
        );

        let get_url = format!("http://{}{}", server.addr(), get_path);
        let post_url = format!("http://{}{}", server.addr(), post_path);
        let client = Client::new();

        run_task_step(&Default::default(), &client, "token", &get_url, &post_url).await;
    }
}
