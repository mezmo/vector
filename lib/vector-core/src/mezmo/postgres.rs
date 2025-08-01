use deadpool_postgres::{Config, Object, Pool, PoolConfig, Runtime};
use snafu::Snafu;
use std::collections::HashMap;
use std::env;
use std::sync::LazyLock;
use tokio::sync::RwLock;
use tokio_postgres::NoTls;
use url::Url;

#[derive(Debug, Snafu, PartialEq)]
pub enum DbError {
    #[snafu(display("DB endpoint {name} is not set in the environment."))]
    DbEndpointNotSet { name: String },

    #[snafu(display("DB endpoint url is not in a valid format."))]
    DbEndpointUrlInvalid,

    #[snafu(display("{cause}"))]
    PoolError { cause: String },
}

const DEFAULT_DB_CONNECTION_POOL_SIZE: usize = 4;

fn connection_pool_size() -> usize {
    if let Ok(value) = env::var("DB_CONNECTION_POOL_SIZE") {
        if let Ok(value) = value.parse::<usize>() {
            return value;
        }
    }
    DEFAULT_DB_CONNECTION_POOL_SIZE
}

/// Fetch a database connection string value from the environment variables if that
/// variable follows the Mezmo standard of `MEZMO_{name}_DB_URL`.
///
/// # Errors
///
/// Will return `Err` if the environment variable is not found.
pub fn get_connection_string(db_name: &str) -> Result<String, DbError> {
    let var_name = format!("MEZMO_{}_DB_URL", db_name.to_ascii_uppercase());
    env::var(var_name).map_err(|_| DbError::DbEndpointNotSet {
        name: db_name.to_string(),
    })
}

fn parse_endpoint_url(endpoint_url: &str) -> Result<Config, DbError> {
    // The library deadpool postgres does not support urls like tokio_postgres::connect
    // Implement our own
    let url = Url::parse(endpoint_url).map_err(|_| DbError::DbEndpointUrlInvalid)?;
    if url.scheme() != "postgresql" && url.scheme() != "postgres" {
        error!(
            message = "Invalid scheme for metrics db",
            scheme = url.scheme()
        );
        return Err(DbError::DbEndpointUrlInvalid);
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

    // Postgres client supports request pipelining (aka pipeline mode).
    // We can have a small pool and send a large number of requests in parallel
    cfg.pool = Some(PoolConfig::new(connection_pool_size()));

    Ok(cfg)
}

static DB_POOLS: LazyLock<RwLock<HashMap<String, Pool>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

async fn get_conn(pool: &Pool) -> Result<Object, DbError> {
    pool.get().await.map_err(|err| DbError::PoolError {
        cause: err.to_string(),
    })
}

fn create_pool(config: &Config) -> Result<Pool, DbError> {
    let runtime = Some(Runtime::Tokio1);
    config
        .create_pool(runtime, NoTls)
        .map_err(|err| DbError::PoolError {
            cause: err.to_string(),
        })
}

/// # Errors
///
/// Will return `Err` if a connection from `db_name` could not be created. This can happen because
/// the connection string isn't set in the environment, the connection string is incorrect or the
/// connection can't be established.
pub async fn db_connection(conn_str: &str) -> Result<Object, DbError> {
    // Most of the code paths are going to attempt to read an existing entry. Use
    // a read lock allowing the most common situation to occur in parallel.
    let read_lock = DB_POOLS.read().await;
    if let Some(pool) = read_lock.get(conn_str) {
        return get_conn(pool).await;
    }

    // Make sure the read lock is dropped and then obtain a write lock. This ensures
    // the code only holds a single lock through this function.
    drop(read_lock);
    let mut write_lock = DB_POOLS.write().await;
    // It could be possible for two threads not find an entry and both then attempt an init.
    // Check once we have the write lock one more time to see if anything needs to be done.
    if !write_lock.contains_key(conn_str) {
        let pool_cfg = parse_endpoint_url(conn_str)?;
        let pool = create_pool(&pool_cfg)?;
        write_lock.insert(conn_str.to_string(), pool);
    }

    get_conn(
        write_lock
            .get(conn_str)
            .expect("write lock should have initialized pool"),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use assay::assay;

    #[assay(env = [("MEZMO_UNIT_TEST_DB_URL", "db-conn-str")])]
    fn get_connection_string_test() {
        let actual = get_connection_string("unit_test").expect("should load a value");
        assert_eq!(actual, "db-conn-str");
    }

    #[assay(env = [("MEZMO_METRICS_DB_URL", "db-conn-str")])]
    fn get_connection_string_not_found() {
        let actual = get_connection_string("unit_test");
        let expected = Err(DbError::DbEndpointNotSet {
            name: "unit_test".to_string(),
        });
        assert_eq!(actual, expected);
    }

    #[assay(env = [("DB_CONNECTION_POOL_SIZE", "100")])]
    fn connection_pool_size_test() {
        assert_eq!(connection_pool_size(), 100);
    }

    #[assay(env = [("DB_CONNECTION_POOL_SIZE", "abc")])]
    fn connection_pool_size_invalid_test() {
        assert_eq!(connection_pool_size(), DEFAULT_DB_CONNECTION_POOL_SIZE);
    }

    #[test]
    fn connection_pool_size_default_test() {
        assert_eq!(connection_pool_size(), DEFAULT_DB_CONNECTION_POOL_SIZE);
    }

    #[test]
    fn parse_endpoint_url_test() {
        let config = parse_endpoint_url("postgresql://user1:pass@server/db1").unwrap();
        assert_eq!(config.host, Some("server".to_string()));
        assert_eq!(config.user, Some("user1".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.dbname, Some("db1".to_string()));

        let config = parse_endpoint_url("postgres://user1:pass@server/db1?zzz=1").unwrap();
        assert_eq!(config.host, Some("server".to_string()));
        assert_eq!(config.user, Some("user1".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.dbname, Some("db1".to_string()));

        assert!(parse_endpoint_url("http://abc.com/sa").is_err());
        assert!(parse_endpoint_url("http://abc.com/sa").is_err());

        let config = parse_endpoint_url("postgresql://metrics:A_%3EBX%2FWN%3E%3CZZ%3BBg@pipeline-primary.pipeline.svc:5432/metrics").unwrap();
        assert_eq!(config.port, Some(5432));
        assert_eq!(
            config.host,
            Some("pipeline-primary.pipeline.svc".to_string())
        );
        assert_eq!(config.user, Some("metrics".to_string()));
        assert_eq!(config.password, Some("A_>BX/WN><ZZ;Bg".to_string()));
        assert_eq!(config.dbname, Some("metrics".to_string()));
    }

    #[test]
    fn parse_endpoint_url_failures() {
        let bad_inputs = vec![
            "",
            " ",
            "this is not a valid url!!!!!!!!!",
            "http://user1:pass@server/db1",
        ];

        for input in bad_inputs {
            let actual = parse_endpoint_url(input);
            assert!(matches!(actual, Err(DbError::DbEndpointUrlInvalid)));
        }
    }
}
