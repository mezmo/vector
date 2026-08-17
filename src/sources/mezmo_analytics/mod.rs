mod config;

pub use config::MezmoAnalyticsConfig;

use futures::{StreamExt, future::try_join_all};
use vector_lib::{
    EstimatedJsonEncodedSizeOf, event::Event, mezmo::analytics::AnalyticsSubscription,
};

use crate::{
    SourceSender,
    internal_events::{InternalLogsBytesReceived, InternalLogsEventsReceived, StreamClosedError},
    shutdown::ShutdownSignal,
};

async fn forward_output(
    subscription: AnalyticsSubscription,
    mut out: SourceSender,
    shutdown: ShutdownSignal,
) -> Result<(), ()> {
    let mut batches = subscription.into_stream().take_until(shutdown);

    while let Some(batch) = batches.next().await {
        let (output, events) = batch.into_parts();
        let count = events.len();
        let byte_size = events.estimated_json_encoded_size_of().get();

        emit!(InternalLogsBytesReceived { byte_size });
        emit!(InternalLogsEventsReceived {
            count,
            byte_size: byte_size.into(),
        });

        if out
            .send_batch_named(output.as_str(), events.into_iter().map(Event::Log))
            .await
            .is_err()
        {
            emit!(StreamClosedError { count });
            return Err(());
        }
    }

    Ok(())
}

async fn run(
    subscriptions: Vec<AnalyticsSubscription>,
    out: SourceSender,
    shutdown: ShutdownSignal,
) -> Result<(), ()> {
    let forwarders = subscriptions
        .into_iter()
        .map(|subscription| forward_output(subscription, out.clone(), shutdown.clone()));

    try_join_all(forwarders).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use serial_test::serial;
    use tokio::time::{Duration, timeout};
    use vector_lib::{
        config::LogNamespace,
        event::{EventStatus, LogEvent, array::EventContainer},
        mezmo::analytics::{AnalyticsEventBatch, AnalyticsOutput, AnalyticsSubscription, publish},
    };

    use super::*;
    use crate::{
        config::SourceConfig,
        test_util::components::{SOURCE_TAGS, SOURCE_TESTS, init_test},
    };

    #[test]
    fn generates_config() {
        crate::test_util::test_generate_config::<MezmoAnalyticsConfig>();
    }

    #[test]
    fn exposes_named_outputs() {
        let outputs = MezmoAnalyticsConfig {}.outputs(LogNamespace::Legacy);
        let names = outputs
            .into_iter()
            .map(|output| output.port.expect("all outputs should be named"))
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "usage_metrics",
                "usage_metrics_by_annotations",
                "log_clusters",
                "log_cluster_samples",
                "log_cluster_usage",
            ]
        );
    }

    #[tokio::test]
    #[serial]
    async fn forwards_batches_to_their_named_output() {
        init_test();
        let (mut out, _default_rx) = SourceSender::new_test();
        let mut usage_rx = out
            .add_outputs(EventStatus::Delivered, "usage_metrics".to_owned())
            .flat_map(|item| futures::stream::iter(item.events.into_events()));
        let subscriptions = AnalyticsSubscription::subscribe_all();
        let (trigger_shutdown, shutdown, shutdown_done) = ShutdownSignal::new_wired();
        let source = tokio::spawn(run(subscriptions, out, shutdown));

        publish(|| {
            [AnalyticsEventBatch::new(
                AnalyticsOutput::UsageMetrics,
                vec![LogEvent::from("usage")],
            )]
        });

        let event = timeout(Duration::from_secs(1), usage_rx.next())
            .await
            .expect("source should forward the batch")
            .expect("named output should remain open");
        assert_eq!(
            event
                .into_log()
                .get_message()
                .expect("forwarded log should contain a message")
                .to_string_lossy(),
            "usage"
        );

        drop(trigger_shutdown);
        shutdown_done.await;
        assert_eq!(source.await.expect("source task should complete"), Ok(()));
        SOURCE_TESTS.assert(&SOURCE_TAGS);
    }

    #[tokio::test]
    #[serial]
    async fn blocked_output_does_not_block_other_outputs() {
        const USAGE_BATCHES: usize = 200;

        let (mut out, _default_rx) = SourceSender::new_test();
        let mut usage_rx = out
            .add_outputs(
                EventStatus::Delivered,
                AnalyticsOutput::UsageMetrics.as_str().to_owned(),
            )
            .flat_map(|item| futures::stream::iter(item.events.into_events()));
        let mut cluster_rx = out
            .add_outputs(
                EventStatus::Delivered,
                AnalyticsOutput::LogClusters.as_str().to_owned(),
            )
            .flat_map(|item| futures::stream::iter(item.events.into_events()));
        let subscriptions = AnalyticsSubscription::subscribe_all();
        let (trigger_shutdown, shutdown, shutdown_done) = ShutdownSignal::new_wired();
        let source = tokio::spawn(run(subscriptions, out, shutdown));

        publish(|| {
            let mut batches = Vec::with_capacity(USAGE_BATCHES + 1);
            for _ in 0..USAGE_BATCHES {
                batches.push(AnalyticsEventBatch::new(
                    AnalyticsOutput::UsageMetrics,
                    vec![LogEvent::from("usage")],
                ));
            }
            batches.push(AnalyticsEventBatch::new(
                AnalyticsOutput::LogClusters,
                vec![LogEvent::from("cluster")],
            ));
            batches
        });

        let cluster = timeout(Duration::from_secs(1), cluster_rx.next())
            .await
            .expect("cluster output should not be blocked by usage metrics")
            .expect("cluster output should remain open");
        assert_eq!(
            cluster
                .into_log()
                .get_message()
                .expect("cluster log should contain a message")
                .to_string_lossy(),
            "cluster"
        );

        timeout(Duration::from_secs(5), async {
            for _ in 0..USAGE_BATCHES {
                usage_rx
                    .next()
                    .await
                    .expect("usage metrics output should remain open");
            }
        })
        .await
        .expect("usage metrics should drain after it resumes");

        drop(trigger_shutdown);
        shutdown_done.await;
        assert_eq!(source.await.expect("source task should complete"), Ok(()));
    }
}
