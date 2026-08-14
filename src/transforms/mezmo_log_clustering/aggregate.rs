use std::{collections::HashMap, time::Duration};

use chrono::Utc;
use tokio::{sync::mpsc::UnboundedReceiver, time::sleep};
use vector_lib::{
    event::{LogEvent, Value},
    mezmo::analytics::{self, AnalyticsEventBatch, AnalyticsOutput},
};

use super::{ComponentInfo, LocalId, LogGroupAggregateInfo, LogGroupInfo, store};

const MAX_NEW_TEMPLATES_QUEUED: usize = 100;

pub(crate) async fn aggregate_in_loop(
    mut rx: UnboundedReceiver<LogGroupInfo>,
    agg_window: Duration,
) {
    let conn_str = match store::init_db_pool().await {
        Ok(conn_str) => {
            info!("Starting to store log clustering data in metrics db");
            Some(conn_str)
        }
        Err(err) => {
            error!(message = "There was an error initializing the log clustering db client.", %err);
            error!("No log clustering data will be stored in the db.");
            None
        }
    };

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
                    let map = aggregated.entry(info.key).or_default();
                    if info.template.is_some() {
                        new_templates += 1;
                    }
                    let aggregated_info = map.entry(info.local_id).or_default();
                    aggregated_info.cluster_id = info.cluster_id;
                    aggregated_info.count += 1;
                    aggregated_info.size += info.size;

                    // Template and annotations are conditionally sent
                    // Make sure we don't blindly overwrite the existing value
                    if info.template.is_some() {
                        aggregated_info.template = info.template;
                    }
                    if info.annotation_set.is_some() {
                        aggregated_info.annotation_set = info.annotation_set;
                    }

                    info.samples.iter().for_each(|s| aggregated_info.samples.push(s.clone()));

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

        analytics::publish(|| analytics_batches(&aggregated));

        if let Some(conn_str) = &conn_str {
            store::save(conn_str, aggregated).await;
        }
    }
}

fn analytics_batches(
    aggregated: &HashMap<ComponentInfo, HashMap<LocalId, LogGroupAggregateInfo>>,
) -> Vec<AnalyticsEventBatch> {
    let timestamp = Utc::now();
    let mut clusters = Vec::new();
    let mut samples = Vec::new();
    let mut usage = Vec::new();

    for (component, aggregates) in aggregated {
        let account_id = Value::from(component.account_id.to_string());
        let component_id = Value::from(component.component_id.clone());

        for aggregate in aggregates.values() {
            let common_fields = vector_lib::btreemap!(
                "timestamp" => Value::Timestamp(timestamp),
                "account_id" => account_id.clone(),
                "component_id" => component_id.clone(),
                "log_cluster_id" => Value::from(aggregate.cluster_id.clone())
            );

            if let Some(template) = &aggregate.template {
                let mut cluster_fields = common_fields.clone();
                cluster_fields.insert("template".into(), Value::from(template.clone()));
                cluster_fields.insert("first_seen_at".into(), Value::Timestamp(timestamp));
                cluster_fields.insert(
                    "annotations".into(),
                    aggregate
                        .annotation_set
                        .as_ref()
                        .map_or(Value::Null, |set| {
                            Value::from(
                                serde_json::to_value(set)
                                    .expect("annotation sets should always serialize"),
                            )
                        }),
                );
                clusters.push(LogEvent::from(Value::Object(cluster_fields)));

                for sample in &aggregate.samples {
                    let mut sample_fields = common_fields.clone();
                    sample_fields.insert("sample".into(), sample.clone());
                    samples.push(LogEvent::from(Value::Object(sample_fields)));
                }
            }

            let mut usage_fields = common_fields;
            usage_fields.insert("count".into(), Value::from(aggregate.count));
            usage_fields.insert("size".into(), Value::from(aggregate.size));
            usage.push(LogEvent::from(Value::Object(usage_fields)));
        }
    }

    [
        (AnalyticsOutput::LogClusters, clusters),
        (AnalyticsOutput::LogClusterSamples, samples),
        (AnalyticsOutput::LogClusterUsage, usage),
    ]
    .into_iter()
    .filter_map(|(output, events)| {
        if events.is_empty() {
            None
        } else {
            Some(AnalyticsEventBatch::new(output, events))
        }
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;
    use vector_lib::{event::Value, mezmo::analytics::AnalyticsOutput};

    use super::*;

    #[test]
    fn creates_all_log_cluster_analytics_batches() {
        let component = ComponentInfo {
            account_id: Uuid::nil(),
            component_id: "analysis".into(),
        };
        let aggregate = LogGroupAggregateInfo {
            cluster_id: "cluster".into(),
            count: 2,
            size: 20,
            template: Some("request <*>".into()),
            annotation_set: None,
            samples: vec![Value::from("request 42")],
        };
        let aggregated = HashMap::from([(component, HashMap::from([(1, aggregate)]))]);

        let batches = analytics_batches(&aggregated);
        assert_eq!(batches.len(), 3);

        let cluster = batches
            .iter()
            .find(|batch| batch.output() == AnalyticsOutput::LogClusters)
            .expect("log cluster batch");
        assert_eq!(cluster.events().len(), 1);
        assert_eq!(
            cluster.events()[0].get("log_cluster_id"),
            Some(&Value::from("cluster"))
        );
        assert_eq!(
            cluster.events()[0].get("template"),
            Some(&Value::from("request <*>"))
        );

        let samples = batches
            .iter()
            .find(|batch| batch.output() == AnalyticsOutput::LogClusterSamples)
            .expect("log cluster samples batch");
        assert_eq!(samples.events().len(), 1);
        assert_eq!(
            samples.events()[0].get("sample"),
            Some(&Value::from("request 42"))
        );

        let usage = batches
            .iter()
            .find(|batch| batch.output() == AnalyticsOutput::LogClusterUsage)
            .expect("log cluster usage batch");
        assert_eq!(usage.events().len(), 1);
        assert_eq!(usage.events()[0].get("count"), Some(&Value::from(2)));
        assert_eq!(usage.events()[0].get("size"), Some(&Value::from(20)));
    }
}
