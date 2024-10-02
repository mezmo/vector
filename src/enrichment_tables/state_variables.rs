use crate::config::EnrichmentTableConfig;
use moka::sync::Cache;
use snafu::Snafu;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use vector_lib::configurable::configurable_component;
use vector_lib::enrichment::{Case, Condition, IndexHandle, Table};
use vector_lib::mezmo;
use vrl::value::{KeyString, Value};

const QUERY_ALL_STATE_VARIABLES: &str =
    "SELECT account_id::text, pipeline_id::text, state::text FROM pipeline_state_variables";

const MAX_CACHE_ENTRIES: u64 = 100_000;
const POLL_DELAY: Duration = Duration::from_secs(5);

/// Potential postgres connection error
#[derive(Debug, Snafu)]
pub enum StateVariablesDBError {
    #[snafu(display("Postgres URL is invalid for state variables enrichment table"))]
    /// Error for when the database connnection string is invalid
    UrlInvalid,
    /// Error when a database connection cannot be made
    #[snafu(display(
        "Can't connect to postgres for state variables encrichment table: {message}"
    ))]
    ConnectionError {
        /// A message detailing the exact error
        message: String,
    },
    /// Error when a query error has been encountered
    #[snafu(display("There was a query error. {message}"))]
    QueryError {
        /// A message detailing the exact error
        message: String,
    },
}

fn get_cache_key(account_id: &str, pipeline_id: &str) -> String {
    format!("{}:{}", account_id, pipeline_id)
}

async fn fetch_states_from_db(
    cache: &Arc<Cache<String, String>>,
) -> Result<usize, StateVariablesDBError> {
    let Ok(conn_str) = mezmo::postgres::get_connection_string("pipeline") else {
        error!(message = "Can't connect to postgres DB. URL not found.",);
        return Err(StateVariablesDBError::UrlInvalid);
    };
    match mezmo::postgres::db_connection(&conn_str).await {
        Ok(client) => {
            let result = client.query(QUERY_ALL_STATE_VARIABLES, &[]).await;
            match result {
                Ok(rows) => {
                    for row in rows.iter() {
                        let account_id: String = row.get(0);
                        let pipeline_id: String = row.get(1);
                        let state: String = row.get(2);
                        let key = get_cache_key(&account_id, &pipeline_id);
                        debug!("CACHE INSERT: {key} / {state}");
                        cache.insert(key, state);
                    }
                    Ok(rows.len())
                }
                Err(err) => Err(StateVariablesDBError::QueryError {
                    message: format!("Query execution failed: {err:?}"),
                }),
            }
        }
        Err(err) => Err(StateVariablesDBError::ConnectionError {
            message: format!("Could not get a DB pool connection: {err:?}"),
        }),
    }
}

/// Empty config for state variables. It's only used to implement `.build()`
#[configurable_component(enrichment_table("state_variables"))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StateVariablesConfig {}
impl_generate_config_from_default!(StateVariablesConfig);

impl EnrichmentTableConfig for StateVariablesConfig {
    async fn build(
        &self,
        _globals: &crate::config::GlobalOptions,
    ) -> crate::Result<Box<dyn Table + Send + Sync>> {
        Ok(Box::new(StateVariables::new().await?))
    }
}

/// A struct that implements [enrichment::Table] to handle loading data from postgres.
#[derive(Clone)]
pub struct StateVariables {
    _state_poller: Option<Arc<JoinHandle<()>>>, // Saved here only to prevent dropping. It's not really used directly.
    cache: Arc<Cache<String, String>>,
}

impl StateVariables {
    /// Impl for the state variables enrichment table
    pub async fn new() -> Result<Self, StateVariablesDBError> {
        let cache = Arc::new(Cache::new(MAX_CACHE_ENTRIES));
        let row_count = fetch_states_from_db(&cache).await?;
        if row_count == 0 {
            warn!("Warning: The state variables DB table appears to be empty");
        }

        let spawn_cache = Arc::clone(&cache);

        let state_poller = tokio::task::spawn(async move {
            loop {
                match fetch_states_from_db(&spawn_cache).await {
                    Ok(0) => {
                        warn!("Warning: The state variables DB table appears to be empty")
                    }
                    Ok(row_len) => debug!("Loaded {row_len} entries"),
                    Err(err) => error!("Error polling state variables DB table: {err:?}"),
                };

                tokio::time::sleep(POLL_DELAY).await;
            }
        });

        Ok(Self {
            _state_poller: Some(Arc::new(state_poller)),
            cache,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            _state_poller: None,
            cache: Arc::new(Cache::new(1000)),
        }
    }

    /// Iterates the conditions and validates the parameters we need.
    /// It forms a simple key/val HashMap where the keys are db columns and the values
    /// have been converted to simple strings. Later, this will be used to form
    /// cache keys for the data.
    pub fn gather_query_parameters(
        &self,
        conditions: &[Condition],
    ) -> Result<HashMap<String, String>, String> {
        let mut param_names: HashMap<String, String> = HashMap::new();

        for cond in conditions {
            match cond {
                Condition::Equals { field, value } => {
                    let key = field.to_string();
                    // Use lossy to get the value, otherwise the quotes will be part of it "\"my value\""
                    let val = value.to_string_lossy().to_string();
                    param_names.insert(key, val);
                }
                _ => return Err("Unsupported query condition".to_owned()),
            }
        }

        if param_names.is_empty() {
            Err("Conditions for `account_id` and `pipeline_id` are required".to_owned())
        } else if !param_names.contains_key("account_id") {
            Err("Missing required condition, `account_id`".to_owned())
        } else if !param_names.contains_key("pipeline_id") {
            Err("Missing required condition, `pipeline_id`".to_owned())
        } else {
            Ok(param_names)
        }
    }

    /// Finds a serialized state string in cache, then returns the requested select fields in a BTreeMap.
    fn lookup(
        &self,
        param_names: &HashMap<String, String>,
        select: Option<&[String]>,
    ) -> Result<BTreeMap<KeyString, Value>, String> {
        let account_id = param_names
            .get("account_id")
            .expect("Condition field `account_id` not found");
        let pipeline_id = param_names
            .get("pipeline_id")
            .expect("Condition field `pipeline_id` not found");
        let key = get_cache_key(account_id, pipeline_id);

        let state = self.cache.get(&key).unwrap_or("{}".to_owned());

        let json: BTreeMap<KeyString, serde_json::Value> =
            match serde_json::from_str(state.as_str()) {
                Ok(Some(variables)) => variables,
                Ok(None) => {
                    return Ok(BTreeMap::new());
                }
                Err(err) => {
                    return Err(err.to_string());
                }
            };

        // If the user is specifying `select` fields, then prune the rest. Otherwise, iterate
        // the whole object into the result. Create the return sructure containing Value objects.

        let mut result: BTreeMap<KeyString, Value> = BTreeMap::new();

        match select {
            Some(select) if !select.is_empty() => {
                for field in select.iter() {
                    match json.get(field.as_str()) {
                        Some(serde_value) => {
                            result.insert(field.clone().into(), Value::from(serde_value))
                        }
                        _ => result.insert(field.clone().into(), Value::Null),
                    };
                }
            }
            _ => {
                // No select fields - iterate all
                for (field, serde_value) in json.iter() {
                    result.insert(field.clone().into(), Value::from(serde_value));
                }
            }
        }
        // TODO: consider caching this result, keyed by {conditions, select}
        Ok(result)
    }
}

impl Table for StateVariables {
    /// Look up state variables by account_id and pipeline_id. Based on `select`, return those fields.
    fn find_table_row<'a>(
        &self,
        _: Case,
        conditions: &'a [Condition<'a>],
        select: Option<&'a [String]>,
        _: Option<IndexHandle>,
    ) -> Result<BTreeMap<KeyString, Value>, String> {
        let param_names = self.gather_query_parameters(conditions)?;
        let result = self.lookup(&param_names, select)?;

        Ok(result)
    }

    fn find_table_rows<'a>(
        &self,
        _case: Case,
        _condition: &'a [Condition<'a>],
        _select: Option<&'a [String]>,
        _index: Option<IndexHandle>,
    ) -> Result<Vec<BTreeMap<KeyString, Value>>, String> {
        // This can be implemented if/when we look up variables for all pipelines of an account
        Ok(vec![BTreeMap::new()])
    }

    /// Not needed for this implementation, but the return value needs to be valid
    fn add_index(&mut self, _case: Case, _fields: &[&str]) -> Result<IndexHandle, String> {
        Ok(IndexHandle(0))
    }

    /// Not used in this implementation
    fn index_fields(&self) -> Vec<(Case, Vec<String>)> {
        Vec::new()
    }

    /// Not used in this implementation
    fn needs_reload(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_query_params() {
        let state_variables = StateVariables::new_test();

        let conditions = vec![
            Condition::Equals {
                field: "account_id",
                value: "dc6fb299-9cbb-44c5-ba86-e0529cd6ce95".into(),
            },
            Condition::Equals {
                field: "pipeline_id",
                value: "e497d246-9ac8-44ec-a765-cc5d884725e3".into(),
            },
        ];
        let expected: HashMap<String, String> = HashMap::from([
            (
                "account_id".to_string(),
                "dc6fb299-9cbb-44c5-ba86-e0529cd6ce95".to_string(),
            ),
            (
                "pipeline_id".to_string(),
                "e497d246-9ac8-44ec-a765-cc5d884725e3".to_string(),
            ),
        ]);

        let result = state_variables.gather_query_parameters(&conditions);

        assert_eq!(
            Ok(expected),
            result,
            "Both parameters are extracted correctly"
        );

        // Test missing parameters

        let conditions = vec![];
        let result = state_variables.gather_query_parameters(&conditions);
        assert_eq!(
            Err("Conditions for `account_id` and `pipeline_id` are required".to_owned()),
            result,
            "No conditions supplied"
        );
        let conditions = vec![Condition::Equals {
            field: "account_id",
            value: "dc6fb299-9cbb-44c5-ba86-e0529cd6ce95".into(),
        }];
        let result = state_variables.gather_query_parameters(&conditions);
        assert_eq!(
            Err("Missing required condition, `pipeline_id`".to_owned()),
            result,
            "Errors if pipeline_id is missing"
        );

        let conditions = vec![Condition::Equals {
            field: "pipeline_id",
            value: "e497d246-9ac8-44ec-a765-cc5d884725e3".into(),
        }];
        let result = state_variables.gather_query_parameters(&conditions);
        assert_eq!(
            Err("Missing required condition, `account_id`".to_owned()),
            result,
            "Errors if account_id is missing"
        );
    }

    #[test]
    fn test_lookup() {
        let state_variables = StateVariables::new_test();
        let account_id = String::from("dc6fb299-9cbb-44c5-ba86-e0529cd6ce95");
        let pipeline_id = String::from("e497d246-9ac8-44ec-a765-cc5d884725e3");
        let param_names: HashMap<String, String> = HashMap::from([
            ("account_id".into(), account_id.clone()),
            ("pipeline_id".into(), pipeline_id.clone()),
        ]);
        let key = get_cache_key(&account_id, &pipeline_id);
        let state = String::from(
            r#"{
                "var_1": "my first value",
                "var_2": "my second value"
            }"#,
        );
        state_variables.cache.insert(key.clone(), state);

        // Begin assertions
        let select = None;
        let expected: BTreeMap<KeyString, Value> = BTreeMap::from([
            ("var_1".into(), "my first value".into()),
            ("var_2".into(), "my second value".into()),
        ]);
        let result = state_variables.lookup(&param_names, select);

        assert_eq!(Ok(expected), result, "No select fields returns everything");

        let select: Option<&[String]> = Some(&[]);
        let expected: BTreeMap<KeyString, Value> = BTreeMap::from([
            ("var_1".into(), "my first value".into()),
            ("var_2".into(), "my second value".into()),
        ]);
        let result = state_variables.lookup(&param_names, select);

        assert_eq!(
            Ok(expected),
            result,
            "Empty select fields returns everything"
        );

        let fields = ["var_2".to_string()];
        let select: Option<&[String]> = Some(&fields);
        let expected: BTreeMap<KeyString, Value> =
            BTreeMap::from([("var_2".into(), "my second value".into())]);
        let result = state_variables.lookup(&param_names, select);

        assert_eq!(Ok(expected), result, "selecting specific fields works");

        // Errors
        let state = String::from(
            r#"{
                "bad_json":
            }"#,
        );
        state_variables.cache.insert(key.clone(), state);
        let result = state_variables.lookup(&param_names, None);
        assert_eq!(
            Err("expected value at line 3 column 13".to_string()),
            result,
            "Bad JSON"
        );

        let state = String::from("{}");
        state_variables.cache.insert(key.clone(), state);
        let result = state_variables.lookup(&param_names, None);
        assert_eq!(Ok(BTreeMap::new()), result, "Empty JSON from cache");

        state_variables.cache.invalidate(&key);
        let result = state_variables.lookup(&param_names, None);
        assert_eq!(
            Ok(BTreeMap::new()),
            result,
            "Empty JSON when no cache is found"
        );
    }
}
