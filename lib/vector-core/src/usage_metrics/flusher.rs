use async_trait::async_trait;
use chrono::Utc;
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use futures::future::join_all;
use http::{header, HeaderName, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use std::{cmp, time::Duration};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_postgres::types::ToSql;
use tokio_postgres::NoTls;
use url::Url;

use super::{UsageMetricsKey, UsageMetricsValue};

const INSERT_QUERY: &str =
    "INSERT INTO usage_metrics (event_ts, account_id, pipeline_id, component_id, processor, metric, value) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING";
const DB_MAX_PARALLEL_EXECUTIONS: usize = 4;
const VECTOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_trait]
pub(crate) trait MetricsFlusher: Sync {
    async fn save_metrics(&self, metrics: HashMap<UsageMetricsKey, UsageMetricsValue>);
}

#[derive(Debug)]
pub(crate) enum DbFlusherError {
    UrlInvalid,
    ConnectionError,
    QueryError,
}

pub(crate) struct DbFlusher {
    pool: Pool,
    processor_name: String,
}

impl DbFlusher {
    pub(crate) async fn new(
        endpoint_url: String,
        pod_name: &String,
    ) -> Result<Self, DbFlusherError> {
        let cfg = DbFlusher::get_config(&endpoint_url)?;
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|_| DbFlusherError::ConnectionError)?;

        // Preemptively try to connect to the db to fail fast
        let client = pool.get().await.map_err(|error| {
            error!(message = "There was an error connecting to usage metrics db", %error);
            DbFlusherError::ConnectionError
        })?;
        // Check that the query is valid on init
        client.prepare_cached(INSERT_QUERY).await.map_err(|error| {
            error!(message = "There was an error preparing usage metrics query", %error);
            DbFlusherError::QueryError
        })?;

        let processor_name = format!("app=vector,pod={pod_name},version={VECTOR_VERSION}");
        Ok(Self {
            pool,
            processor_name,
        })
    }

    fn get_config(endpoint_url: &str) -> Result<Config, DbFlusherError> {
        // The library deadpool postgres does not support urls like tokio_postgres::connect
        // Implement our own
        let url = Url::parse(endpoint_url).map_err(|_| DbFlusherError::UrlInvalid)?;
        if url.scheme() != "postgresql" && url.scheme() != "postgres" {
            error!(
                message = "Invalid scheme for metrics db",
                scheme = url.scheme()
            );
            return Err(DbFlusherError::UrlInvalid);
        }

        let mut cfg = Config::new();
        cfg.host = url.host().map(|h| h.to_string());
        cfg.port = url.port();
        cfg.user = (!url.username().is_empty()).then(|| {
            urlencoding::decode(url.username())
                .expect("UTF-8")
                .to_string()
        });
        cfg.password = url
            .password()
            .map(|v| urlencoding::decode(v).expect("UTF-8").to_string());
        if let Some(mut path_segments) = url.path_segments() {
            if let Some(first) = path_segments.next() {
                cfg.dbname = Some(first.into());
            }
        }

        // Set the max_size on the pool to match the number of parallel inserts
        cfg.pool = Some(PoolConfig::new(DB_MAX_PARALLEL_EXECUTIONS));

        Ok(cfg)
    }
}

impl DbFlusher {
    async fn insert_metrics(&self, k: UsageMetricsKey, v: UsageMetricsValue) {
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

        match self.pool.get().await {
            Ok(client) => {
                if let Ok(stmt) = client.prepare_cached(INSERT_QUERY).await {
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
                            error!(message = "Usage metrics insert failed", %error);
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

    async fn execute_one_at_a_time(
        &self,
        iter: Arc<Mutex<IntoIter<UsageMetricsKey, UsageMetricsValue>>>,
    ) {
        while let Some((k, v)) = DbFlusher::get_next(iter.clone()).await {
            self.insert_metrics(k, v).await;
        }
    }

    async fn get_next(
        iter: Arc<Mutex<IntoIter<UsageMetricsKey, UsageMetricsValue>>>,
    ) -> Option<(UsageMetricsKey, UsageMetricsValue)> {
        let mut i = iter.lock().await;
        if let Some((k, v)) = i.next() {
            return Some((k, v));
        }
        None
    }
}

#[async_trait]
impl MetricsFlusher for DbFlusher {
    async fn save_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        let items_len = metrics_map.len();
        let iter = Arc::new(Mutex::new(metrics_map.into_iter()));
        let concurrency_level = cmp::min(DB_MAX_PARALLEL_EXECUTIONS, items_len);
        // Start n inserts in parallel and continue until the iterator is exhausted
        let futures = (0..concurrency_level).map(|_| self.execute_one_at_a_time(iter.clone()));

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
                pipeline_id: k.pipeline_id,
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
                Ok(_) => {
                    return;
                }
                Err(e) => {
                    if let &HttpError::Client = &e {
                        error!("Usage metrics could not be stored due to a client error");
                        break;
                    }
                    if attempt == 1 {
                        warn!(message = format!(
                            "Usage metrics could not be stored due to a {:?} on the first attempt, retrying",
                            e
                        ));
                    }
                    if attempt % 10 == 0 {
                        error!(
                            message = format!(
                                "Usage metrics could not be stored due to a {:?} after {} attempts, retrying",
                                e,
                                attempt
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
    async fn save_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        let flusher = self.clone();
        tokio::spawn(async move {
            // Store metrics in the background to avoid having the caller
            // await for long periods of time that can cause excessive buffering
            // of the usage metrics
            flusher.save_metrics_with_retries(metrics_map).await;
        });
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
    async fn save_metrics(&self, _metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        // Do nothing
    }
}

pub(crate) struct StdErrFlusher {}

#[async_trait]
impl MetricsFlusher for StdErrFlusher {
    #[allow(clippy::print_stderr)]
    async fn save_metrics(&self, metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        for (k, v) in metrics_map {
            let key = serde_json::to_string(&k).unwrap();
            let value = serde_json::to_string(&v).unwrap();
            eprintln!("usage_metrics: {{\"key\":{key}, \"value\":{value}}}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::usage_metrics::ComponentKind;

    use super::*;
    use httptest::{
        matchers::{all_of, json_decoded, request},
        responders::{cycle, status_code},
        Expectation, Server,
    };

    static HTTP_FLUSHER_PATH: &str = "/v1/http-flusher-test";
    static HTTP_FLUSHER_MAX_DELAY: Duration = Duration::from_millis(400);

    #[test]
    fn get_config_test() {
        let config = DbFlusher::get_config("postgresql://user1:pass@server/db1").unwrap();
        assert_eq!(config.host, Some("server".to_string()));
        assert_eq!(config.user, Some("user1".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.dbname, Some("db1".to_string()));

        let config = DbFlusher::get_config("postgres://user1:pass@server/db1?zzz=1").unwrap();
        assert_eq!(config.host, Some("server".to_string()));
        assert_eq!(config.user, Some("user1".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.dbname, Some("db1".to_string()));

        assert!(DbFlusher::get_config("http://abc.com/sa").is_err());
        assert!(DbFlusher::get_config("http://abc.com/sa").is_err());

        let config = DbFlusher::get_config("postgresql://metrics:A_%3EBX%2FWN%3E%3CZZ%3BBg@pipeline-primary.pipeline.svc:5432/metrics").unwrap();
        assert_eq!(config.port, Some(5432));
        assert_eq!(
            config.host,
            Some("pipeline-primary.pipeline.svc".to_string())
        );
        assert_eq!(config.user, Some("metrics".to_string()));
        assert_eq!(config.password, Some("A_>BX/WN><ZZ;Bg".to_string()));
        assert_eq!(config.dbname, Some("metrics".to_string()));
    }

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

        flusher.save_metrics(HashMap::new()).await;
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
            pipeline_id: "pipeline1".to_string(),
            component_id: "component1".to_string(),
            component_type: "type1".to_string(),
            component_kind: ComponentKind::Sink,
        };
        let value = UsageMetricsValue {
            total_count: 101,
            total_size: 12345,
        };
        flusher.save_metrics(HashMap::from([(key, value)])).await;
        sleep(HTTP_FLUSHER_MAX_DELAY).await;
    }

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
