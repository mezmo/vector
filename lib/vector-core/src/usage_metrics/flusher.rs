use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use futures::future::join_all;
use std::cmp;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_postgres::types::ToSql;
use tokio_postgres::NoTls;
use url::Url;

use super::{UsageMetricsKey, UsageMetricsValue};

const INSERT_QUERY: &str =
    "INSERT INTO usage_metrics (event_ts, account_id, pipeline_id, component_id, processor, metric, value) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING";
const DB_MAX_PARALLEL_EXECUTIONS: usize = 32;

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
}

impl DbFlusher {
    pub(crate) async fn new(endpoint_url: String) -> Result<Self, DbFlusherError> {
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

        Ok(Self { pool })
    }

    fn get_config(endpoint_url: &str) -> Result<Config, DbFlusherError> {
        // The library deadpool postgres does not support urls like tokio_postgres::connect
        // Implement our own
        let url = Url::parse(endpoint_url).map_err(|_| DbFlusherError::UrlInvalid)?;
        if url.scheme() != "postgresql" {
            error!(
                message = "Invalid scheme for metrics db",
                scheme = url.scheme()
            );
            return Err(DbFlusherError::UrlInvalid);
        }
        let mut cfg = Config::new();
        cfg.host = url.host().map(|h| h.to_string());
        cfg.user = if url.username().len() > 0 {
            Some(url.username().to_string())
        } else {
            None
        };
        cfg.password = url.password().map(Into::into);
        if let Some(mut path_segments) = url.path_segments() {
            if let Some(first) = path_segments.next() {
                cfg.dbname = Some(first.into());
            }
        }

        Ok(cfg)
    }
}

impl DbFlusher {
    async fn insert_metrics(&self, k: UsageMetricsKey, v: UsageMetricsValue) {
        let time_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as i64;
        let processor = "vector".to_string();
        let measurement = "count".to_string();
        let value = v.total_count as i32;
        let params_count: Vec<&(dyn ToSql + Sync)> = vec![
            &time_micros,
            &k.account_id,
            &k.pipeline_id,
            &k.component_id,
            &processor,
            &measurement,
            &value,
        ];

        let value = v.total_size as i32;
        let measurement = "byte_size".to_string();
        let params_size: Vec<&(dyn ToSql + Sync)> = vec![
            &time_micros,
            &k.account_id,
            &k.pipeline_id,
            &k.component_id,
            &processor,
            &measurement,
            &value,
        ];

        match self.pool.get().await {
            Ok(client) => {
                if let Ok(stmt) = client.prepare_cached(INSERT_QUERY).await {
                    let result_count = client.execute(&stmt, &params_count);
                    let result_size = client.execute(&stmt, &params_size);

                    match tokio::try_join!(result_count, result_size) {
                        Ok(_) => {}
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
        while let Some((k, v)) = DbFlusher::get_next(iter.clone()) {
            self.insert_metrics(k, v).await;
        }
    }

    fn get_next(
        iter: Arc<Mutex<IntoIter<UsageMetricsKey, UsageMetricsValue>>>,
    ) -> Option<(UsageMetricsKey, UsageMetricsValue)> {
        if let Ok(mut i) = iter.lock() {
            if let Some((k, v)) = i.next() {
                return Some((k, v));
            }
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

        let config = DbFlusher::get_config("postgresql://user1:pass@server/db1?zzz=1").unwrap();
        assert_eq!(config.host, Some("server".to_string()));
        assert_eq!(config.user, Some("user1".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.dbname, Some("db1".to_string()));

        assert!(DbFlusher::get_config("http://abc.com/sa").is_err());
        assert!(DbFlusher::get_config("http://abc.com/sa").is_err());
    }
}
