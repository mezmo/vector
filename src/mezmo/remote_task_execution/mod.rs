#![cfg(feature = "api-client")]

use chrono::Utc;
use futures_util::StreamExt;
use http::{header, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, Instant},
};
use tokio::time::sleep;
use url::Url;
use vector_api_client::{
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

/// Fetches and executes tasks for local deployments, mainly used for remote tapping.
pub(crate) async fn start_polling_for_tasks(
    config: config::api::Options,
    auth_token: String,
    get_endpoint_url: String,
    post_endpoint_url: String,
) {
    sleep(TASK_INITIAL_POLL_DELAY).await;
    info!("Starting to poll for tasks");
    let client = Client::new();
    loop {
        let start = Instant::now();
        run_task_step(
            &config,
            &client,
            &auth_token,
            &get_endpoint_url,
            &post_endpoint_url,
        )
        .await;

        let elapsed = start.elapsed();
        if elapsed < TASK_POLL_STEP_DELAY {
            sleep(TASK_POLL_STEP_DELAY - elapsed).await;
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
    pipeline_id: String,
    task_parameters: TaskParameters,
    age_secs: isize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskParameters {
    limit: Option<isize>,
    timeout_ms: Option<u64>,
    component_id: Option<String>,
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
    let endpoint_url = endpoint_url
        .replace(":task_id", &task.task_id)
        .replace(":pipeline_id", &task.pipeline_id);

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

    let subscription_client = connect_subscription_client(url)
        .await
        .map_err(|e| format!("Couldn't connect to Vector API via WebSockets: {}", e))?;

    let tap_timeout = task
        .task_parameters
        .timeout_ms
        .map_or(DEFAULT_TAP_TIMEOUT, |timeout_ms| {
            Duration::from_millis(timeout_ms)
        });

    tokio::pin! {
        let stream = subscription_client.output_events_by_component_id_patterns_subscription(
            vec![component_id.to_string()],
            vec![],
            TapEncodingFormat::Json,
            limit as i64,
            tap_timeout.as_millis() as i64, // Continue fetching for the duration of the timeout
        );
    }

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
        matchers::{all_of, contains, json_decoded, request, url_decoded},
        responders::{json_encoded, status_code},
        Expectation, Server,
    };
    use serde_json::json;

    #[tokio::test]
    async fn fetches_tasks_and_reports_errors() {
        let get_path = "/fake/get/url";
        let post_path = "/fake/post/:task_id/url?pipeline_id=:pipeline_id";
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![request::method("GET"), request::path(get_path),])
                .times(1)
                .respond_with(json_encoded(json!({
                    "data": [{
                        "task_id": "task1",
                        "task_type": "tap",
                        "pipeline_id": "pip1",
                        "age_secs": 1,
                        "task_parameters": {"component_id": "comp1", "limit": 1, "timeout_ms": 1000},
                    }]
                }))),
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("POST"),
                request::path("/fake/post/task1/url"),
                request::query(url_decoded(contains(("pipeline_id", "pip1")))),
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
}
