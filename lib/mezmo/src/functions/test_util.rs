use std::collections::{BTreeMap, HashMap};

use enrichment::{Case, Condition, IndexHandle, Table, TableRegistry, TableSearch};
use vrl::{compiler::ProgramInfo, owned_value_path, path::OwnedTargetPath, prelude::*};

// A mock table that we can control for tests. It simulates an enrichment table.
#[derive(Clone, Debug)]
pub(crate) struct MockStateTable {
    state: std::sync::Arc<std::sync::Mutex<BTreeMap<KeyString, Value>>>,
}

impl MockStateTable {
    pub(crate) fn new(initial_state: BTreeMap<KeyString, Value>) -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(initial_state)),
        }
    }
}

impl Table for MockStateTable {
    fn find_table_row<'a>(
        &self,
        _case: Case,
        _conditions: &'a [Condition<'a>],
        select: Option<&'a [String]>,
        _index: Option<IndexHandle>,
    ) -> Result<BTreeMap<KeyString, Value>, String> {
        let state = self.state.lock().unwrap();
        let mut result = BTreeMap::new();

        if let Some(select_fields) = select {
            for field_name in select_fields {
                let key = KeyString::from(field_name.as_str());
                let value = state.get(&key).cloned().unwrap_or(Value::Null);
                result.insert(key, value);
            }
        } else {
            return Err("MockStateTable requires a `select` parameter".to_string());
        }

        Ok(result)
    }

    // The following methods are not currently needed for tests.
    fn find_table_rows<'a>(
        &self,
        _case: Case,
        _condition: &'a [Condition<'a>],
        _select: Option<&'a [String]>,
        _index: Option<IndexHandle>,
    ) -> Result<Vec<BTreeMap<KeyString, Value>>, String> {
        unimplemented!()
    }

    fn add_index(&mut self, _case: Case, _fields: &[&str]) -> Result<IndexHandle, String> {
        Ok(IndexHandle(0))
    }

    fn index_fields(&self) -> Vec<(Case, Vec<String>)> {
        Vec::new()
    }

    fn needs_reload(&self) -> bool {
        false
    }
}

// Create a VRL context to be used in testing.
pub(crate) fn create_test_vrl_context(
    initial_state: BTreeMap<KeyString, Value>,
) -> (TableSearch, ProgramInfo, TimeZone) {
    let info = ProgramInfo {
        fallible: false,
        abortable: false,
        target_queries: vec![
            OwnedTargetPath::event(owned_value_path!("name")),
            OwnedTargetPath::event(owned_value_path!("namespace")),
            OwnedTargetPath::event(owned_value_path!("timestamp")),
            OwnedTargetPath::event(owned_value_path!("kind")),
            OwnedTargetPath::event(owned_value_path!("type")),
            OwnedTargetPath::event(owned_value_path!("tags")),
        ],
        target_assignments: vec![],
    };
    let tz = TimeZone::default();

    let mock_table = MockStateTable::new(initial_state);
    let registry = TableRegistry::default();
    let mut table_map: HashMap<String, Box<dyn Table + Send + Sync>> = HashMap::new();
    table_map.insert("state_variables".to_string(), Box::new(mock_table));
    registry.load(table_map);
    registry.finish_load();
    let enrichment_tables = registry.as_readonly();

    (enrichment_tables, info, tz)
}
