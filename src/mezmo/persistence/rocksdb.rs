use rocksdb::statistics::{Histogram, StatsLevel, Ticker};
use rocksdb::{BlockBasedOptions, Cache, DB, DBCompactionStyle, Options};
use snafu::{ResultExt, Snafu};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::mezmo_env_config;
use crate::{
    Error,
    internal_events::mezmo_persistence::{
        MezmoPersistenceRocksDBHistogram, MezmoPersistenceRocksDBTicker,
    },
};
use mezmo::MezmoContext;

use super::PersistenceConnection;

const POD_NAME_ENV_VAR: &str = "POD_NAME";

const MAX_LOG_FILE_SIZE_ENV_VAR: &str = "MAX_ROCKSDB_LOG_FILE_SIZE";
const MAX_LOG_FILE_SIZE_DEFAULT: usize = 1024 * 1024 * 10; // 10 MB max

const MAX_LOG_FILE_NUM_ENV_VAR: &str = "MAX_ROCKSDB_LOG_FILE_NUM";
const MAX_LOG_FILE_NUM_DEFAULT: usize = 5; // 5 logs max

// Minimum allowed blockcache size is 512KB (default 32MB)
// https://github.com/facebook/rocksdb/wiki/Block-Cache
const ROCKSDB_BLOCK_CACHE_SIZE: usize = 1024 * 500;

// Global TTL for all RocksDB connections.
// Records will live for at least as long at this TTL, and will be expired by the storage
// backend in a best-effort as soon as possible after the TTL elapses.
// In the future this may need to be configurable per-component, but for now all records
// are subject to the same TTL.
const ROCKSDB_TTL_SECS: u64 = 90_000; // 25 hours

// Expired records are only removed from disk when the files holding them are compacted.
// Periodic compaction recompacts files once they are older than the TTL, so space used by
// expired records is reclaimed even when write volume is too low to trigger compaction.
// The floor keeps databases with a short TTL from being recompacted too often.
const PERIODIC_COMPACTION_MIN_SECS: u64 = 60 * 60; // 1 hour

// RocksDB writes stats to the LOG file of every open database at this interval, so an open
// database is never idle on disk for longer than this. The stale database cleanup relies on it.
const ROCKSDB_STATS_DUMP_PERIOD_SECS: u32 = 600; // 10 minutes

// Databases are named after the pod that opened them, so databases created under a previous
// pod name, or for accounts that no longer have stateful components, are never opened again.
// When a base path is first used, databases under it that have not been modified for at least
// the minimum age (or the TTL, if greater) are removed.
const STALE_DB_CLEANUP_ENABLED_ENV_VAR: &str = "ROCKSDB_STALE_DB_CLEANUP_ENABLED";
const STALE_DB_CLEANUP_ENABLED_DEFAULT: bool = true;

const STALE_DB_MIN_AGE_SECS_ENV_VAR: &str = "ROCKSDB_STALE_DB_MIN_AGE_SECS";
const STALE_DB_MIN_AGE_SECS_DEFAULT: u64 = ROCKSDB_TTL_SECS * 2;

// Global registry of RocksDB connections.
// Connections/databases are partitioned by account. Each component for a given account
// will operate on a reference to the same DB connection.
static ROCKSDB_CONNECTION_REGISTRY: LazyLock<RocksDBConnectionRegistry> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

// Base paths that have already been checked for stale databases by this process.
static STALE_DB_CLEANUP_BASE_PATHS: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

type RocksDB = DB;
type RocksDBConnectionRegistry = Arc<Mutex<HashMap<String, Arc<RocksDBConnection>>>>;

#[derive(Debug)]
pub struct RocksDBConnection {
    pub db: RocksDB,
    pub db_opts: RocksDBOptions,
}

#[derive(Debug, Snafu)]
enum RocksDBPersistenceError {
    Io {
        #[snafu(source)]
        source: std::io::Error,
    },
    RocksDB {
        #[snafu(source)]
        source: rocksdb::Error,
    },
    Conversion {
        #[snafu(source)]
        source: std::string::FromUtf8Error,
    },
    #[snafu(display("Invalid context: {mezmo_ctx:?}"))]
    InvalidContext { mezmo_ctx: MezmoContext },
    #[snafu(display("Missing required environment variable: {var}"))]
    MissingEnvironmentVariable { var: String },
}

#[derive(Default)]
pub struct RocksDBOptions(Options);

impl Deref for RocksDBOptions {
    type Target = Options;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RocksDBOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for RocksDBOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDBOptions").finish()
    }
}

#[derive(Debug)]
pub(crate) struct RocksDBPersistenceConnection {
    connection: Arc<RocksDBConnection>,
    mezmo_ctx: MezmoContext,
}

impl RocksDBPersistenceConnection {
    /// Emits metrics for RocksDB Tickers and Histograms into the internal_metrics event stream
    fn report_metrics(&self) {
        emit!(MezmoPersistenceRocksDBTicker {
            ticker: Ticker::BytesRead,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::DbGet,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::BytesPerRead,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::DecompressionTimesNanos,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBTicker {
            ticker: Ticker::BytesWritten,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::BytesPerWrite,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::TableSyncMicros,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::WriteStall,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });

        emit!(MezmoPersistenceRocksDBHistogram {
            histogram: Histogram::CompressionTimesNanos,
            connection: &self.connection,
            mezmo_ctx: &self.mezmo_ctx,
        });
    }
}

/// Implementation of [PersistenceConnection] that uses RocksDB as its underlying data store.
/// Each account has its own RocksDB database derived from the provided [MezmoContext], and keys
/// are namespaced with the [MezmoContext] component_id. The account DB instance is shared across
/// threads/components.
impl PersistenceConnection for RocksDBPersistenceConnection {
    /// Creates a new [RocksDBPersistenceConnection] instance, either by creating a new RocksDB
    /// database connection or reusing an existing connection.
    /// New connections use the default TTL for Mezmo
    fn new(base_path: &str, mezmo_ctx: &MezmoContext) -> Result<Self, Error> {
        Self::new_with_ttl(base_path, mezmo_ctx, ROCKSDB_TTL_SECS)
    }

    /// Creates a new [RocksDBPersistenceConnection] instance, either by creating a new RocksDB
    /// database connection or reusing an existing connection with the specified record TTL
    fn new_with_ttl(
        base_path: &str,
        mezmo_ctx: &MezmoContext,
        ttl_secs: u64,
    ) -> Result<Self, Error> {
        let account_id = match mezmo_ctx.account_id() {
            Some(account_id) => account_id,
            None => {
                return Err(Box::new(RocksDBPersistenceError::InvalidContext {
                    mezmo_ctx: mezmo_ctx.clone(),
                }));
            }
        };

        let pod_name = match std::env::var(POD_NAME_ENV_VAR) {
            Ok(name) => name,
            Err(_) => {
                return Err(Box::new(
                    RocksDBPersistenceError::MissingEnvironmentVariable {
                        var: POD_NAME_ENV_VAR.to_string(),
                    },
                ));
            }
        };

        let max_log_file_size: usize =
            mezmo_env_config!(MAX_LOG_FILE_SIZE_ENV_VAR, MAX_LOG_FILE_SIZE_DEFAULT);
        let max_log_file_num: usize =
            mezmo_env_config!(MAX_LOG_FILE_NUM_ENV_VAR, MAX_LOG_FILE_NUM_DEFAULT);
        let stale_db_cleanup_enabled: bool = mezmo_env_config!(
            STALE_DB_CLEANUP_ENABLED_ENV_VAR,
            STALE_DB_CLEANUP_ENABLED_DEFAULT
        );
        let stale_db_min_age_secs: u64 =
            mezmo_env_config!(STALE_DB_MIN_AGE_SECS_ENV_VAR, STALE_DB_MIN_AGE_SECS_DEFAULT);

        let mut path = PathBuf::from(base_path);
        path.push(format!("{account_id}.{pod_name}.db"));

        let mut registry = ROCKSDB_CONNECTION_REGISTRY
            .lock()
            .expect("Could not acquire lock on RocksDB persistence registry");
        let conn = match registry.get(path.to_string_lossy().as_ref()) {
            Some(conn) => Arc::clone(conn),
            None => {
                match path.try_exists() {
                    Ok(true) => {
                        debug!("RocksDB db directory exists at: {}", path.to_string_lossy());
                    }
                    Ok(false) => {
                        debug!(
                            "Creating RocksDB db directory at: {}",
                            path.to_string_lossy()
                        );
                        std::fs::create_dir_all(&path).context(IoSnafu)?;
                    }
                    Err(err) => {
                        return Err(Box::new(RocksDBPersistenceError::Io { source: err }));
                    }
                }

                if stale_db_cleanup_enabled
                    && STALE_DB_CLEANUP_BASE_PATHS
                        .lock()
                        .expect("Could not acquire lock on RocksDB stale database cleanup paths")
                        .insert(PathBuf::from(base_path))
                {
                    let min_age = Duration::from_secs(stale_db_min_age_secs.max(ttl_secs));
                    remove_stale_databases(Path::new(base_path), min_age, |db_path| {
                        db_path == path || registry.contains_key(db_path.to_string_lossy().as_ref())
                    });
                }

                let cache = Cache::new_lru_cache(ROCKSDB_BLOCK_CACHE_SIZE);
                let mut block_options = BlockBasedOptions::default();
                block_options.set_block_cache(&cache);

                let mut db_opts = RocksDBOptions::default();
                db_opts.create_if_missing(true);
                db_opts.set_compaction_style(DBCompactionStyle::Universal);
                db_opts.optimize_universal_style_compaction(ROCKSDB_BLOCK_CACHE_SIZE);
                db_opts.set_block_based_table_factory(&block_options);
                db_opts.enable_statistics();
                db_opts.set_statistics_level(StatsLevel::All);
                db_opts.set_log_file_time_to_roll(60 * 60 * 24); // 1 day
                db_opts.set_keep_log_file_num(max_log_file_num);
                db_opts.set_max_log_file_size(max_log_file_size);
                db_opts.set_periodic_compaction_seconds(ttl_secs.max(PERIODIC_COMPACTION_MIN_SECS));
                db_opts.set_stats_dump_period_sec(ROCKSDB_STATS_DUMP_PERIOD_SECS);

                let db = DB::open_with_ttl(&db_opts, &path, Duration::from_secs(ttl_secs))?;
                let conn = Arc::new(RocksDBConnection { db, db_opts });
                registry.insert(path.to_string_lossy().to_string(), Arc::clone(&conn));
                conn
            }
        };

        Ok(Self {
            connection: Arc::clone(&conn),
            mezmo_ctx: mezmo_ctx.clone(),
        })
    }

    /// Gets the value for a given key from the database.
    fn get(&self, key: &str) -> Result<Option<String>, Error> {
        let value = self
            .connection
            .db
            .get(namespaced_key(&self.mezmo_ctx, key))
            .context(RocksDBSnafu)?;

        self.report_metrics();

        match value {
            Some(bytes) => String::from_utf8(bytes)
                .map(|s| Ok(Some(s)))
                .context(ConversionSnafu)?,
            None => Ok(None),
        }
    }

    /// Sets the value for a given key from the database.
    fn set(&self, key: &str, value: &str) -> Result<(), Error> {
        self.connection
            .db
            .put(namespaced_key(&self.mezmo_ctx, key), value)
            .context(RocksDBSnafu)?;

        self.report_metrics();

        Ok(())
    }

    /// Delete the key from the database
    fn delete(&self, key: &str) -> Result<(), Error> {
        self.connection
            .db
            .delete(namespaced_key(&self.mezmo_ctx, key))
            .context(RocksDBSnafu)?;

        self.report_metrics();

        Ok(())
    }
}

fn namespaced_key(mezmo_ctx: &MezmoContext, key: &str) -> String {
    format!("{}:{}", mezmo_ctx.component_id(), key)
}

/// Removes RocksDB databases directly under `base_path` that have not been modified for at
/// least `min_age`, skipping any database for which `in_use` returns true.
///
/// Only directories with a `.db` extension that contain a RocksDB `CURRENT` file are
/// considered. A database open in another process writes to its LOG file every
/// [ROCKSDB_STATS_DUMP_PERIOD_SECS], so it is never idle for long enough to be removed.
fn remove_stale_databases(base_path: &Path, min_age: Duration, in_use: impl Fn(&Path) -> bool) {
    let entries = match std::fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                message = "Unable to check for stale RocksDB databases.",
                path = %base_path.display(),
                %error,
            );
            return;
        }
    };

    let now = SystemTime::now();
    for entry in entries.flatten() {
        let db_path = entry.path();
        if !is_rocksdb_database(&db_path) || in_use(&db_path) {
            continue;
        }

        // Skip the database if its age can't be determined.
        let Some(idle) =
            last_modified(&db_path).and_then(|modified| now.duration_since(modified).ok())
        else {
            continue;
        };
        if idle < min_age {
            continue;
        }

        match std::fs::remove_dir_all(&db_path) {
            Ok(()) => info!(
                message = "Removed stale RocksDB database.",
                path = %db_path.display(),
                idle_secs = idle.as_secs(),
            ),
            Err(error) => warn!(
                message = "Unable to remove stale RocksDB database.",
                path = %db_path.display(),
                %error,
            ),
        }
    }
}

fn is_rocksdb_database(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "db")
        && path.is_dir()
        && path.join("CURRENT").is_file()
}

/// Returns the most recent modification time of a database directory or any file in it.
fn last_modified(path: &Path) -> Option<SystemTime> {
    let mut latest = path.metadata().ok()?.modified().ok()?;
    for entry in std::fs::read_dir(path).ok()? {
        let modified = entry.ok()?.metadata().ok()?.modified().ok()?;
        latest = latest.max(modified);
    }
    Some(latest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_downcast_matches;
    use assay::assay;
    use serde::Serialize;
    use std::{collections::BTreeMap, thread};
    use tempfile::tempdir;
    use uuid::Uuid;

    fn test_mezmo_context() -> MezmoContext {
        let account_id = Uuid::new_v4();
        test_mezmo_context_for_account(&account_id.to_string())
    }

    fn test_mezmo_context_for_account(account_id: &str) -> MezmoContext {
        MezmoContext::try_from(format!(
            "v1:reduce:transform:component_id:pipeline_id:{account_id}"
        ))
        .unwrap()
    }

    fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
        serde_json::to_string(value).unwrap()
    }

    fn create_db_dir(base_path: &Path, name: &str) -> PathBuf {
        let db_path = base_path.join(name);
        std::fs::create_dir_all(&db_path).unwrap();
        std::fs::write(db_path.join("CURRENT"), "MANIFEST-000001\n").unwrap();
        db_path
    }

    /// Sets the modification time of a database directory and its files to `age` ago.
    fn set_db_dir_age(db_path: &Path, age: Duration) {
        let modified = SystemTime::now() - age;
        for entry in std::fs::read_dir(db_path).unwrap() {
            let file = std::fs::File::options()
                .write(true)
                .open(entry.unwrap().path())
                .unwrap();
            file.set_modified(modified).unwrap();
        }
        std::fs::File::open(db_path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
    }

    #[test]
    fn test_remove_stale_databases_removes_unused() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let stale = create_db_dir(tmp_dir.path(), "account.old-pod.db");

        remove_stale_databases(tmp_dir.path(), Duration::ZERO, |_| false);

        assert!(!stale.exists(), "stale database was not removed");
    }

    #[test]
    fn test_remove_stale_databases_keeps_in_use() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let in_use = create_db_dir(tmp_dir.path(), "account.current-pod.db");

        remove_stale_databases(tmp_dir.path(), Duration::ZERO, |path| path == in_use);

        assert!(in_use.exists(), "in-use database was removed");
    }

    #[test]
    fn test_remove_stale_databases_keeps_recently_modified() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let recent = create_db_dir(tmp_dir.path(), "account.other-pod.db");

        remove_stale_databases(tmp_dir.path(), Duration::from_secs(60 * 60), |_| false);

        assert!(recent.exists(), "recently modified database was removed");
    }

    #[test]
    fn test_remove_stale_databases_ignores_other_paths() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let base_path = tmp_dir.path();

        // A .db directory that isn't a RocksDB database
        let not_rocksdb = base_path.join("not-rocksdb.db");
        std::fs::create_dir_all(&not_rocksdb).unwrap();
        // A RocksDB database without a .db extension
        let no_extension = create_db_dir(base_path, "trace_head_sample");
        // A .db file
        let file = base_path.join("file.db");
        std::fs::write(&file, "").unwrap();

        remove_stale_databases(base_path, Duration::ZERO, |_| false);

        assert!(not_rocksdb.exists(), "non-RocksDB directory was removed");
        assert!(
            no_extension.exists(),
            "directory without .db extension was removed"
        );
        assert!(file.exists(), ".db file was removed");
    }

    #[test]
    fn test_remove_stale_databases_missing_base_path() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        remove_stale_databases(&tmp_dir.path().join("missing"), Duration::ZERO, |_| false);
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_new_connection_removes_stale_databases() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let base_path = tmp_dir.path();
        let stale = create_db_dir(base_path, "account.old-pod.db");
        set_db_dir_age(
            &stale,
            Duration::from_secs(STALE_DB_MIN_AGE_SECS_DEFAULT + 60),
        );
        let recent = create_db_dir(base_path, "account.other-pod.db");

        let ctx = test_mezmo_context();
        let db = RocksDBPersistenceConnection::new(base_path.to_str().unwrap(), &ctx).unwrap();

        assert!(!stale.exists(), "stale database was not removed");
        assert!(recent.exists(), "recently modified database was removed");
        assert!(db.set("key", "value").is_ok(), "new database is usable");
    }

    #[assay(env = [
        ("POD_NAME", "vector-test0-0"),
        ("ROCKSDB_STALE_DB_CLEANUP_ENABLED", "false"),
    ])]
    fn test_new_connection_stale_database_cleanup_disabled() {
        let tmp_dir = tempdir().expect("Could not create temp dir");
        let base_path = tmp_dir.path();
        let stale = create_db_dir(base_path, "account.old-pod.db");
        set_db_dir_age(
            &stale,
            Duration::from_secs(STALE_DB_MIN_AGE_SECS_DEFAULT + 60),
        );

        let ctx = test_mezmo_context();
        RocksDBPersistenceConnection::new(base_path.to_str().unwrap(), &ctx).unwrap();

        assert!(
            stale.exists(),
            "stale database was removed with cleanup disabled"
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_namespaced_key() {
        let ctx = test_mezmo_context();
        assert_eq!(namespaced_key(&ctx, "key"), "component_id:key");
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_invalid_context() {
        #[allow(deprecated)]
        let tmp_path = tempdir().expect("Could not create temp dir").into_path();
        let base_path = tmp_path.to_str().unwrap();

        let ctx = test_mezmo_context_for_account("not_a_valid_account_uuid");
        let res = RocksDBPersistenceConnection::new(base_path, &ctx);

        assert!(res.is_err());
        assert_downcast_matches!(
            res.unwrap_err(),
            RocksDBPersistenceError,
            RocksDBPersistenceError::InvalidContext { .. }
        );
    }

    #[assay]
    fn test_missing_pod_name_env() {
        #[allow(deprecated)]
        let tmp_path = tempdir().expect("Could not create temp dir").into_path();
        let base_path = tmp_path.to_str().unwrap();

        let ctx = test_mezmo_context();
        let res = RocksDBPersistenceConnection::new(base_path, &ctx);

        assert!(res.is_err());
        assert_downcast_matches!(
            res.unwrap_err(),
            RocksDBPersistenceError,
            RocksDBPersistenceError::MissingEnvironmentVariable { .. }
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_invalid_base_path_not_a_directory() {
        #[allow(deprecated)]
        let tmp_path = tempdir()
            .expect("Could not create temp dir")
            .into_path()
            .join("exists-but-is-not-a-directory");
        let base_path = tmp_path.to_str().unwrap();

        std::fs::File::create(base_path).unwrap();

        let ctx = test_mezmo_context();
        let res = RocksDBPersistenceConnection::new(base_path, &ctx);

        assert!(res.is_err());
        assert_downcast_matches!(
            res.unwrap_err(),
            RocksDBPersistenceError,
            RocksDBPersistenceError::Io { .. }
        );
    }

    // This test does not pass under CI/Docker, but does pass locally
    #[cfg(not(ci))]
    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_invalid_base_path_exists_but_not_writable() {
        use std::fs::DirBuilder;
        use std::os::unix::fs::DirBuilderExt;

        let base_path = tempdir()
            .expect("Could not create temp dir")
            .keep()
            .join("exists-but-not-writable");

        assert!(
            !base_path.exists(),
            "test prereq failed: {base_path:?} reported as already existing"
        );

        DirBuilder::new().mode(0o000).create(&base_path).unwrap();

        assert!(
            base_path.exists(),
            "test preqreq failed: failed to create {base_path:?}"
        );
        assert!(
            base_path.metadata().unwrap().permissions().readonly(),
            "test prereq failed: {base_path:?} is not read only"
        );

        let ctx = test_mezmo_context();
        let res = RocksDBPersistenceConnection::new(base_path.to_str().unwrap(), &ctx);
        assert!(res.is_err());
        assert_downcast_matches!(
            res.unwrap_err(),
            RocksDBPersistenceError,
            RocksDBPersistenceError::Io { .. }
        );
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_set_get_delete_scalar() {
        #[allow(deprecated)]
        let tmp_path = tempdir().expect("Could not create temp dir").into_path();
        let base_path = tmp_path.to_str().unwrap();

        let ctx = test_mezmo_context();
        let db = RocksDBPersistenceConnection::new(base_path, &ctx).unwrap();

        assert!(db.set("integer", to_json(&123).as_str()).is_ok());
        assert!(db.set("string", to_json("foo").as_str()).is_ok());
        assert!(db.set("boolean", to_json(&false).as_str()).is_ok());

        let int = db.get("integer").unwrap();
        assert!(int.is_some());
        assert_eq!(int.unwrap(), to_json(&123).as_str());

        let str = db.get("string").unwrap();
        assert!(str.is_some());
        assert_eq!(str.unwrap(), to_json("foo"));

        let bool = db.get("boolean").unwrap();
        assert!(bool.is_some());
        assert_eq!(bool.unwrap(), to_json(&false));

        assert!(db.delete("boolean").is_ok());
        let obj_actual = db.get("boolean").unwrap();
        assert!(obj_actual.is_none());
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_set_get_delete_complex() {
        #[allow(deprecated)]
        let tmp_path = tempdir().expect("Could not create temp dir").into_path();
        let base_path = tmp_path.to_str().unwrap();

        let ctx = test_mezmo_context();
        let db = RocksDBPersistenceConnection::new(base_path, &ctx).unwrap();

        let array_expected = to_json(vec!["foo".to_owned(), "bar".to_owned()].as_slice());
        assert!(db.set("array", &array_expected).is_ok());

        let array_actual = db.get("array").unwrap();
        assert!(array_actual.is_some());
        assert_eq!(array_actual.unwrap(), array_expected);

        let mut obj = BTreeMap::new();
        obj.insert("baz".to_owned(), "123".to_owned());
        obj.insert("qux".to_owned(), "456".to_owned());
        let obj_expected = to_json(&obj);
        assert!(db.set("object", &obj_expected).is_ok());

        let obj_actual = db.get("object").unwrap();
        assert!(obj_actual.is_some());
        assert_eq!(obj_actual.unwrap(), obj_expected);

        assert!(db.delete("object").is_ok());
        let obj_actual = db.get("object").unwrap();
        assert!(obj_actual.is_none());
    }

    #[assay(env = [("POD_NAME", "vector-test0-0")])]
    fn test_from_multiple_threads_for_the_same_account() {
        #[allow(deprecated)]
        let base_path_thread_1 = tempdir().expect("Could not create temp dir").into_path();
        let base_path_thread_2 = base_path_thread_1.clone();

        let ctx_thread_1 = test_mezmo_context();
        let ctx_thread_2 = ctx_thread_1.clone();

        let thread1 = thread::spawn(move || {
            RocksDBPersistenceConnection::new(base_path_thread_1.to_str().unwrap(), &ctx_thread_1)
                .unwrap();
        });

        let thread2 = thread::spawn(move || {
            RocksDBPersistenceConnection::new(base_path_thread_2.to_str().unwrap(), &ctx_thread_2)
                .unwrap();
        });

        thread1.join().unwrap();
        thread2.join().unwrap();
    }
}
