use async_trait::async_trait;
use chrono::Utc;
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use futures::future::join_all;
use std::cmp;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
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
        cfg.user = if url.username().len() > 0 {
            Some(
                urlencoding::decode(url.username())
                    .expect("UTF-8")
                    .to_string(),
            )
        } else {
            None
        };
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
        let value = v.total_count as i32;
        let params_count: Vec<&(dyn ToSql + Sync)> = vec![
            &event_ts,
            &k.account_id,
            &k.pipeline_id,
            &k.component_id,
            &self.processor_name,
            &metric,
            &value,
        ];

        let value = v.total_size as i32;
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
                            )
                        }
                        Err(error) => {
                            error!(message = "Usage metrics insert failed", %error)
                        }
                    }
                } else {
                    error!("There was an error preparing usage metrics query");
                }
            }
            Err(error) => {
                error!(message = "Usage metrics db pool failed to obtain a connection", %error)
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

pub(crate) struct NoopFlusher {}

#[async_trait]
impl MetricsFlusher for NoopFlusher {
    async fn save_metrics(&self, _metrics_map: HashMap<UsageMetricsKey, UsageMetricsValue>) {
        // Do nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
