use super::{AnnotationMap, AnnotationSet, UsageMetricsKey, UsageMetricsValue};
use crate::mezmo;
use async_trait::async_trait;
use chrono::Utc;
use futures::future::join_all;
use http::{HeaderName, HeaderValue, header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::IntoIter;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use std::{cmp, time::Duration};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_postgres::types::ToSql;
use uuid::Uuid;
use vector_common::internal_event::emit;
use vector_common::internal_event::usage_metrics::InsertFailed;

const INSERT_BILLING_QUERY: &str = "INSERT INTO usage_metrics (event_ts, account_id, pipeline_id, component_id, processor, metric, value) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING";
const INSERT_PROFILES_QUERY: &str = "INSERT INTO usage_metrics_by_annotations (ts, account_id, component_id, count, size, annotations) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING";

const DB_MAX_PARALLEL_EXECUTIONS: usize = 8;
const VECTOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_trait]
pub(crate) trait MetricsFlusher: Sync {
    async fn save_billing_metrics(&self, metrics: HashMap<UsageMetricsKey, UsageMetricsValue>);

    async fn save_profile_metrics(&self, metrics: HashMap<UsageMetricsKey, AnnotationMap>);
}

#[derive(Debug)]
pub(crate) enum DbFlusherError {
    UrlInvalid,
    ConnectionError,
    QueryError,
}

pub(crate) struct DbFlusher {
    conn_str: String,
    processor_name: String,
}

impl DbFlusher {
    pub(crate) async fn new(pod_name: &str) -> Result<Self, DbFlusherError> {
        let Ok(conn_str) = mezmo::postgres::get_connection_string("metrics") else {
            return Err(DbFlusherError::UrlInvalid);
        };

        let client = mezmo::postgres::db_connection(&conn_str)
            .await
            .map_err(|error| {
                error!(message = "There was an error connecting to usage metrics db", %error);
                DbFlusherError::ConnectionError
            })?;

        // Check that the query is valid on init
        client
            .prepare_cached(INSERT_BILLING_QUERY)
            .await
            .map_err(|error| {
                error!(message = "There was an error preparing usage metrics query", %error);
                DbFlusherError::QueryError
            })?;

        let processor_name = format!("app=vector,pod={pod_name},version={VECTOR_VERSION}");
        Ok(Self {
            conn_str,
            processor_name,
        })
    }
}

impl DbFlusher {
    async fn insert_billing_metrics(&self, k: UsageMetricsKey, v: UsageMetricsValue) {
        let event_ts = Utc::now();
        let metric = "count".to_string();
        let value = v.total_count as i64;
        let params_count: Vec<&(dyn ToSql + Sync)> = vec![
            &event_ts,
            &k.account_id,
            &k.pipeline_id,
            &k.component_id,
            &self.processor_name,
            &metric,
            &value,
        ];

        let value = v.total_size as i64;
        let metric = "byte_size".to_string();
        let params_size: Vec<&(dyn ToSql + Sync)> = vec![
            &event_ts,
            &k.account_id,
            &k.pipeline_id,
            &k.component_id,
            &self.processor_name,
            &metric,
            &value,
        ];

        match mezmo::postgres::db_connection(&self.conn_str).await {
            Ok(client) => {
                if let Ok(stmt) = client.prepare_cached(INSERT_BILLING_QUERY).await {
                    let result_count = client.execute(&stmt, &params_count);
                    let result_size = client.execute(&stmt, &params_size);

                    match tokio::try_join!(result_count, result_size) {
                        Ok(_) => {
                            trace!(
                                message = "Flushed usage metrics records for component",
                                component_id = k.component_id
                            );
                        }
                        Err(error) => {
                            emit(InsertFailed {
                                error: error.to_string(),
                            });
                        }
                    }
                } else {
                    error!("There was an error preparing usage metrics query");
                }
            }
            Err(error) => {
                error!(message = "Usage metrics db pool failed to obtain a connection", %error);
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    async fn insert_profile_metrics(
        &self,
        k: UsageMetricsKey,
        set: AnnotationSet,
        v: UsageMetricsValue,
    ) {
        let Ok(account_id) = Uuid::try_parse(&k.account_id) else {
            error!("account_id could not be parsed for profile metrics");
            return;
        };

        // ts, account_id, component_id, count, size, annotations
        let ts = Utc::now();
        let count = v.total_count as i64;
        let size = v.total_size as i64;
        let json_set = tokio_postgres::types::Json(set);
        let params: Vec<&(dyn ToSql + Sync)> =
            vec![&ts, &account_id, &k.component_id, &count, &size, &json_set];

        match mezmo::postgres::db_connection(&self.conn_str).await {
            Ok(client) => {
                if let Ok(stmt) = client.prepare_cached(INSERT_PROFILES_QUERY).await {
                    match client.execute(&stmt, &params).await {
                        Ok(_) => {}
                        Err(error) => {
                            error!(message = "Usage metric profiles insert failed", %error);
                        }
                    }
                } else {
                    error!("There was an error preparing usage metric profiles query");
                }
            }
            Err(error) => {
                error!(message = "Usage metrics db pool failed to obtain a connection", %error);
            }
        }
    }

    async fn insert_billing_metrics_sequentially(
        &self,
        iter: Arc<Mutex<IntoIter<UsageMetricsKey, UsageMetricsValue>>>,
    ) {
        while let Some((k, v)) = DbFlusher::get_next(iter.clone()).await {
            self.insert_billing_metrics(k, v).await;
        }
    }

    async fn insert_profile_metrics_sequentially(
        &self,
        iter: Arc<
            Mutex<
                Box<
                    dyn Iterator<Item = (UsageMetricsKey, AnnotationSet, UsageMetricsValue)> + Send,
                >,
            >,
        >,
    ) {
        while let Some((k, set, v)) = DbFlusher::get_next(iter.clone()).await {
            self.insert_profile_metrics(k, set, v).await;
        }
    }

    async fn get_next<Item>(iter: Arc<Mutex<impl Iterator<Item = Item>>>) -> Option<Item> {
        let mut i = iter.lock().await;
        if let Some(v) = i.next() {
            return Some(v);
        }
        None
    }
}

#[async_trait]
impl MetricsFlusher for DbFlusher {
    async fn save_billing_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        let items_len = metrics_map.len();
        let iter = Arc::new(Mutex::new(metrics_map.into_iter()));
        let concurrency_level = cmp::min(DB_MAX_PARALLEL_EXECUTIONS, items_len);
        // Start n inserts in parallel and continue until the iterator is exhausted
        let futures =
            (0..concurrency_level).map(|_| self.insert_billing_metrics_sequentially(iter.clone()));

        join_all(futures).await;
    }

    async fn save_profile_metrics(&self, metrics: HashMap<UsageMetricsKey, AnnotationMap>) {
        let iter: Box<
            dyn Iterator<Item = (UsageMetricsKey, AnnotationSet, UsageMetricsValue)> + Send,
        > = Box::new(metrics.into_iter().flat_map(|(k, v)| {
            v.into_iter()
                .map(move |(set, value)| (k.clone(), set, value))
        }));
        let iter = Arc::new(Mutex::new(iter));
        // Start n inserts in parallel and continue until the iterator is exhausted
        let futures = (0..DB_MAX_PARALLEL_EXECUTIONS)
            .map(|_| self.insert_profile_metrics_sequentially(iter.clone()));

        join_all(futures).await;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HttpFlusher {
    processor_name: String,
    client: Client,
    url: String,
    headers: HashMap<String, String>,
    base_delay: Duration,
    max_delay: Duration,
}

impl HttpFlusher {
    pub(crate) fn new(
        pod_name: &str,
        url: String,
        headers: HashMap<String, String>,
        max_delay: Duration,
    ) -> Self {
        let processor_name = format!("app=vector,pod={pod_name},version={VECTOR_VERSION}");

        HttpFlusher {
            client: Client::new(),
            processor_name,
            url,
            headers,
            base_delay: Duration::from_millis(200),
            max_delay,
        }
    }

    async fn save_metrics_with_retries(
        &self,
        metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>,
    ) {
        let event_ts = Utc::now().timestamp_millis();
        let metrics: Vec<_> = metrics_map
            .into_iter()
            .map(|(k, v)| HttpFlusherRequestBodyItem {
                event_ts,
                pipeline_id: k
                    .pipeline_id
                    .expect("to be set for billing usage metrics in Edge"),
                component_id: k.component_id,
                processor: self.processor_name.clone(),
                total_count: v.total_count,
                total_size: v.total_size,
            })
            .collect();

        let mut attempt = 0;
        let start = Instant::now();
        while start.elapsed() < self.max_delay {
            attempt += 1;
            match self.http_request(&metrics).await {
                Ok(()) => {
                    return;
                }
                Err(e) => {
                    if let &HttpError::Client = &e {
                        error!("Usage metrics could not be stored due to a client error");
                        break;
                    }
                    if attempt == 1 {
                        warn!(
                            message = format!(
                                "Usage metrics could not be stored due to a {:?} on the first attempt, retrying",
                                e
                            )
                        );
                    }
                    if attempt % 10 == 0 {
                        error!(
                            message = format!(
                                "Usage metrics could not be stored due to a {:?} after {} attempts, retrying",
                                e, attempt
                            )
                        );
                    }
                }
            }
            sleep(self.base_delay).await;
        }

        error!("Usage metrics failed to be stored");
    }

    async fn http_request(&self, body: &Vec<HttpFlusherRequestBodyItem>) -> Result<(), HttpError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("Mezmo Pulse"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        for (k, v) in &self.headers {
            headers.insert(
                HeaderName::from_str(k).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }

        // Make the request
        let resp = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|_| HttpError::Connection)?;

        if !resp.status().is_success() {
            if resp.status().is_client_error() {
                return Err(HttpError::Client);
            }
            if resp.status().is_server_error() {
                return Err(HttpError::Server);
            }
            return Err(HttpError::Connection);
        }

        Ok(())
    }
}

#[async_trait]
impl MetricsFlusher for HttpFlusher {
    async fn save_billing_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        let flusher = self.clone();
        tokio::spawn(async move {
            // Store metrics in the background to avoid having the caller
            // await for long periods of time that can cause excessive buffering
            // of the usage metrics
            flusher.save_metrics_with_retries(metrics_map).await;
        });
    }

    async fn save_profile_metrics(&self, _metrics: HashMap<UsageMetricsKey, AnnotationMap>) {
        // Http flusher is intended for Edge/Pulse
        // Do nothing: the volume of the data for profiles is too high to be sent over internet
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpFlusherRequestBodyItem {
    event_ts: i64,
    pipeline_id: String,
    component_id: String,
    processor: String,
    total_count: usize,
    total_size: usize,
}

enum HttpError {
    Connection,
    Client,
    Server,
}

impl fmt::Debug for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Connection => write!(f, "Connection error"),
            HttpError::Client => write!(f, "Client error"),
            HttpError::Server => write!(f, "Server error"),
        }
    }
}

pub(crate) struct NoopFlusher {}

#[async_trait]
impl MetricsFlusher for NoopFlusher {
    async fn save_billing_metrics(
        &self,
        _metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>,
    ) {
        // Do nothing
    }

    async fn save_profile_metrics(&self, _metrics: HashMap<UsageMetricsKey, AnnotationMap>) {
        // Do nothing
    }
}

pub(crate) struct StdErrFlusher {}

#[allow(clippy::print_stderr)]
#[async_trait]
impl MetricsFlusher for StdErrFlusher {
    async fn save_billing_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        for (k, v) in metrics_map {
            let key = serde_json::to_string(&k).unwrap();
            let value = serde_json::to_string(&v).unwrap();
            eprintln!("usage_metrics_billing: {{\"key\":{key}, \"value\":{value}}}");
        }
    }

    async fn save_profile_metrics(&self, metrics: HashMap<UsageMetricsKey, AnnotationMap>) {
        for (k, v) in metrics {
            let key = serde_json::to_string(&k).unwrap();
            let value = serde_json::to_string(&v).unwrap();
            eprintln!("usage_metrics_profile: {{\"key\":{key}, \"value\":{value}}}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::usage_metrics::ComponentKind;

    use super::*;
    use httptest::{
        Expectation, Server,
        matchers::{all_of, json_decoded, request},
        responders::{cycle, status_code},
    };

    static HTTP_FLUSHER_PATH: &str = "/v1/http-flusher-test";
    static HTTP_FLUSHER_MAX_DELAY: Duration = Duration::from_millis(400);

    #[tokio::test]
    async fn http_flusher_should_not_retry_client_errors() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("POST", HTTP_FLUSHER_PATH))
                .times(1)
                .respond_with(status_code(403)),
        );

        let flusher = HttpFlusher {
            client: Client::new(),
            processor_name: "test processor".to_string(),
            url: format!("http://{}{HTTP_FLUSHER_PATH}", server.addr()),
            headers: HashMap::new(),
            base_delay: Duration::from_millis(20),
            max_delay: HTTP_FLUSHER_MAX_DELAY,
        };

        flusher.save_billing_metrics(HashMap::new()).await;
        sleep(HTTP_FLUSHER_MAX_DELAY).await;
        // httptest takes care of assertions
    }

    #[tokio::test]
    async fn http_flusher_should_retry_server_errors() {
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method("POST"),
                request::path(HTTP_FLUSHER_PATH),
                request::body(json_decoded(match_values)),
            ])
            .times(3)
            .respond_with(cycle![
                status_code(500),
                status_code(501),
                status_code(200),
            ]),
        );

        let flusher = HttpFlusher {
            client: Client::new(),
            processor_name: "test processor".to_string(),
            url: format!("http://{}{HTTP_FLUSHER_PATH}", server.addr()),
            headers: HashMap::new(),
            base_delay: Duration::from_millis(20),
            max_delay: HTTP_FLUSHER_MAX_DELAY,
        };

        let key = UsageMetricsKey {
            account_id: "account1".to_string(),
            pipeline_id: Some("pipeline1".to_string()),
            component_id: "component1".to_string(),
            component_type: "type1".to_string(),
            component_kind: ComponentKind::Sink,
        };
        let value = UsageMetricsValue {
            total_count: 101,
            total_size: 12345,
        };
        flusher
            .save_billing_metrics(HashMap::from([(key, value)]))
            .await;
        sleep(HTTP_FLUSHER_MAX_DELAY).await;
    }

    #[allow(clippy::ptr_arg)]
    fn match_values(v: &Vec<HttpFlusherRequestBodyItem>) -> bool {
        if v.len() != 1 {
            return false;
        }

        let item = &v[0];

        item.pipeline_id == "pipeline1"
            && item.component_id == "component1"
            && item.event_ts > 0
            && item.total_count == 101
            && item.total_size == 12345
    }
}
