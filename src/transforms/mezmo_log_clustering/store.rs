use crate::internal_events::mezmo_log_clustering::MezmoLogClusteringStore;
use crate::transforms::mezmo_log_clustering::{
    ComponentInfo, LocalId, LogGroupAggregateInfo, LogGroupInfo,
};
use chrono::Utc;
use deadpool_postgres::{Config, Object, Pool, Runtime};
use futures_util::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::vec::IntoIter;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_postgres::types::{Json, ToSql};
use tokio_postgres::{NoTls, Statement};

const MAX_NEW_TEMPLATES_QUEUED: usize = 100;
const DB_MAX_PARALLEL_EXECUTIONS: usize = 8;

const INSERT_USAGE_QUERY: &str = "INSERT INTO usage_metrics_log_cluster (ts, component_id, log_cluster_id, count, size) VALUES ($1, $2, $3, $4, $5)";
const INSERT_LOG_CLUSTER_QUERY: &str = "INSERT INTO log_clusters (ts, account_id, component_id, log_cluster_id, template, first_seen_at, annotations) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING";

async fn init_db_pool(config: Config) -> crate::Result<Pool> {
    let pool = config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|_| "Init pool fail")?;

    // Preemptively try to connect to the db to fail fast
    let client = pool.get().await.map_err(|e| {
        format!(
            "There was an error connecting to usage metrics db for log clustering: {:?}",
            e
        )
    })?;

    // Check that the queries are valid on init
    client
        .prepare_cached(INSERT_USAGE_QUERY)
        .await
        .map_err(|e| {
            format!(
                "There was an error preparing log clustering usage query: {:?}",
                e
            )
        })?;

    client
        .prepare_cached(INSERT_LOG_CLUSTER_QUERY)
        .await
        .map_err(|e| format!("There was an error preparing log clusters query: {:?}", e))?;

    Ok(pool)
}

pub(crate) async fn save_in_loop(
    mut rx: UnboundedReceiver<LogGroupInfo>,
    config: Config,
    agg_window: Duration,
) {
    let pool = match init_db_pool(config).await {
        Ok(p) => p,
        Err(error) => {
            error!(message = "There was error initializing log clustering db client", %error);
            error!("No log clustering data will be stored in the db");
            // Dequeue and ignore
            while let Some(_) = rx.recv().await {
                // Do nothing
            }
            return;
        }
    };

    info!("Starting to store log clustering data in metrics db");

    let mut finished = false;
    while !finished {
        let mut aggregated: HashMap<ComponentInfo, HashMap<LocalId, LogGroupAggregateInfo>> =
            HashMap::new();
        let timeout = sleep(agg_window);
        tokio::pin!(timeout);
        let mut new_templates = 0;

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    // Break the inner loop, start a new timer
                    break;
                },
                Some(info) = rx.recv() => {
                    let map = aggregated.entry(info.key).or_insert_with(HashMap::new);
                    if info.template.is_some() {
                        new_templates += 1;
                    }
                    let aggregated_info = map.entry(info.local_id).or_insert_with(Default::default);
                    aggregated_info.cluster_id = info.cluster_id;
                    aggregated_info.count += 1;
                    aggregated_info.size += info.size;
                    aggregated_info.template = info.template;
                    aggregated_info.annotation_set = info.annotation_set;

                    if new_templates > MAX_NEW_TEMPLATES_QUEUED {
                        break;
                    }
                },
                else => {
                    // Channel closed
                    finished = true;
                    break;
                }
            }
        }

        save(&pool, aggregated).await;
    }
}

async fn save(
    pool: &Pool,
    aggregated: HashMap<ComponentInfo, HashMap<LocalId, LogGroupAggregateInfo>>,
) {
    if aggregated.is_empty() {
        return;
    }

    let start = Instant::now();
    let client = match pool.get().await {
        Ok(client) => client,
        Err(error) => {
            error!(message = "Could not get a client from pool for log clustering", %error);
            return;
        }
    };

    let (Ok(usage_stmt), Ok(log_cluster_stmt)) = (
        client.prepare_cached(INSERT_USAGE_QUERY).await,
        client.prepare_cached(INSERT_LOG_CLUSTER_QUERY).await,
    ) else {
        error!("Could not prepare statement for log clustering");
        return;
    };

    // Use references, avoid copying
    let mut usage: Vec<(&ComponentInfo, &LogGroupAggregateInfo)> = Vec::new();
    let mut log_clusters = Vec::new();
    for (k, v) in aggregated.iter() {
        for (_, aggregate_info) in v.iter() {
            usage.push((k, aggregate_info));

            if aggregate_info.template.is_some() {
                log_clusters.push((k, aggregate_info));
            }
        }
    }

    let total_usage_records = usage.len();

    if !log_clusters.is_empty() {
        let iter = Arc::new(Mutex::new(log_clusters.into_iter()));
        let futures = (0..DB_MAX_PARALLEL_EXECUTIONS).map(|_| {
            insert_log_clusters_sequentially(&client, &log_cluster_stmt, Arc::clone(&iter))
        });

        join_all(futures).await;
    }

    let iter = Arc::new(Mutex::new(usage.into_iter()));
    let futures = (0..DB_MAX_PARALLEL_EXECUTIONS)
        .map(|_| insert_usage_sequentially(&client, &usage_stmt, Arc::clone(&iter)));

    join_all(futures).await;

    emit!(MezmoLogClusteringStore {
        elapsed: start.elapsed(),
        total_usage_records
    });
}

async fn insert_log_clusters_sequentially(
    client: &Object,
    stmt: &Statement,
    iter: Arc<Mutex<IntoIter<(&ComponentInfo, &LogGroupAggregateInfo)>>>,
) {
    while let Some((k, v)) = get_next(Arc::clone(&iter)).await {
        insert_log_clusters(client, stmt, k, v).await;
    }
}

async fn insert_log_clusters(
    client: &Object,
    stmt: &Statement,
    component_info: &ComponentInfo,
    aggregate_info: &LogGroupAggregateInfo,
) {
    let json_set = aggregate_info.annotation_set.as_ref().map(Json);
    let ts = Utc::now();
    let params: Vec<&(dyn ToSql + Sync)> = vec![
        &ts,
        &component_info.account_id,
        &component_info.component_id,
        &aggregate_info.cluster_id,
        aggregate_info.template.as_ref().expect("Already validated"),
        &ts,
        &json_set,
    ];

    if let Err(error) = client.execute(stmt, &params).await {
        error!(message = "Log cluster insert failed", %error);
    }
}

async fn insert_usage_sequentially(
    client: &Object,
    stmt: &Statement,
    iter: Arc<Mutex<IntoIter<(&ComponentInfo, &LogGroupAggregateInfo)>>>,
) {
    while let Some((k, v)) = get_next(Arc::clone(&iter)).await {
        insert_usage(client, stmt, k, v).await;
    }
}

async fn insert_usage(
    client: &Object,
    stmt: &Statement,
    component_info: &ComponentInfo,
    aggregate_info: &LogGroupAggregateInfo,
) {
    let ts = Utc::now();
    let params: Vec<&(dyn ToSql + Sync)> = vec![
        &ts,
        &component_info.component_id,
        &aggregate_info.cluster_id,
        &aggregate_info.count,
        &aggregate_info.size,
    ];

    if let Err(error) = client.execute(stmt, &params).await {
        error!(message = "Log cluster insert failed", %error);
    }
}

async fn get_next<Item>(iter: Arc<Mutex<impl Iterator<Item = Item>>>) -> Option<Item> {
    let mut i = iter.lock().await;
    if let Some(v) = i.next() {
        return Some(v);
    }
    None
}
