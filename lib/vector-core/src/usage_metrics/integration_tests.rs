#![cfg(all(test, feature = "pgbouncer-integration-tests"))]

use super::*;
use crate::mezmo::postgres;
use assay::assay;
use std::collections::HashMap;

const INIT_TABLE: &str = "CREATE TABLE IF NOT EXISTS usage_metrics (
    event_ts timestamp with time zone NOT NULL,
    account_id text,
    pipeline_id text,
    component_id text NOT NULL,
    processor text NOT NULL,
    metric text NOT NULL,
    value bigint NOT NULL,
    edge_id text
);";

// Write billing metrics to database using prepared statements.
#[assay(env = [("MEZMO_METRICS_DB_URL", "postgres://vector:vector@localhost:6432/postgres")])]
async fn save_billing_metrics() {
    // Create table if not exists
    let conn = "postgres://vector:vector@localhost:6432/postgres";
    let client = postgres::db_connection(conn).await.unwrap();
    client.execute(INIT_TABLE, &[]).await.unwrap();

    // Mock usage metrics
    let key = UsageMetricsKey {
        account_id: "test_account".to_string(),
        pipeline_id: Some("test_pipeline".to_string()),
        component_id: "test_component".to_string(),
        component_type: "test_type".to_string(),
        component_kind: ComponentKind::Sink,
    };
    let value = UsageMetricsValue {
        total_count: 1,
        total_size: 1024,
    };
    let mut metrics = HashMap::new();
    metrics.insert(key, value);

    // Write metrics to db with prepared statements
    let flusher = DbFlusher::new("foobar").await.unwrap();
    flusher.save_billing_metrics(metrics).await;
}
