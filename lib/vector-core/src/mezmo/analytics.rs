//! In-process delivery for Mezmo analytics records.

use std::sync::OnceLock;

use futures::{Stream, StreamExt, future::ready};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

use crate::event::LogEvent;

const ANALYTICS_CHANNEL_CAPACITY: usize = 1_000;
const ANALYTICS_OUTPUT_COUNT: usize = 5;

type AnalyticsSenders = [Sender<AnalyticsEventBatch>; ANALYTICS_OUTPUT_COUNT];

static ANALYTICS: OnceLock<AnalyticsSenders> = OnceLock::new();

/// A named output exposed by the `mezmo_analytics` source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalyticsOutput {
    UsageMetrics,
    UsageMetricsByAnnotations,
    LogClusters,
    LogClusterSamples,
    LogClusterUsage,
}

impl AnalyticsOutput {
    /// Returns the source output name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UsageMetrics => "usage_metrics",
            Self::UsageMetricsByAnnotations => "usage_metrics_by_annotations",
            Self::LogClusters => "log_clusters",
            Self::LogClusterSamples => "log_cluster_samples",
            Self::LogClusterUsage => "log_cluster_usage",
        }
    }

    /// Returns every named output in declaration order.
    pub const fn all() -> [Self; 5] {
        [
            Self::UsageMetrics,
            Self::UsageMetricsByAnnotations,
            Self::LogClusters,
            Self::LogClusterSamples,
            Self::LogClusterUsage,
        ]
    }
}

/// A batch of analytics records destined for one named source output.
#[derive(Clone, Debug)]
pub struct AnalyticsEventBatch {
    output: AnalyticsOutput,
    events: Vec<LogEvent>,
}

impl AnalyticsEventBatch {
    /// Creates a batch for one named output.
    pub fn new(output: AnalyticsOutput, events: Vec<LogEvent>) -> Self {
        Self { output, events }
    }

    /// Returns the named output associated with this batch.
    pub const fn output(&self) -> AnalyticsOutput {
        self.output
    }

    /// Returns the records in this batch.
    pub fn events(&self) -> &[LogEvent] {
        &self.events
    }

    /// Splits the batch into its named output and records.
    pub fn into_parts(self) -> (AnalyticsOutput, Vec<LogEvent>) {
        (self.output, self.events)
    }
}

fn senders() -> &'static AnalyticsSenders {
    ANALYTICS
        .get_or_init(|| std::array::from_fn(|_| broadcast::channel(ANALYTICS_CHANNEL_CAPACITY).0))
}

/// Builds and publishes batches if a `mezmo_analytics` source is subscribed.
pub fn publish<F, B>(build_batches: F)
where
    F: FnOnce() -> B,
    B: IntoIterator<Item = AnalyticsEventBatch>,
{
    let Some(senders) = ANALYTICS.get() else {
        return;
    };
    if senders.iter().all(|sender| sender.receiver_count() == 0) {
        return;
    }

    for batch in build_batches() {
        let sender = &senders[batch.output() as usize];
        if sender.receiver_count() > 0 {
            let _ = sender.send(batch);
        }
    }
}

/// A subscription to analytics batches produced inside Vector.
pub struct AnalyticsSubscription {
    receiver: Receiver<AnalyticsEventBatch>,
}

impl AnalyticsSubscription {
    /// Subscribes to analytics batches for one output produced after this call.
    pub fn subscribe(output: AnalyticsOutput) -> Self {
        Self {
            receiver: senders()[output as usize].subscribe(),
        }
    }

    /// Subscribes to analytics batches for every output.
    pub fn subscribe_all() -> Vec<Self> {
        AnalyticsOutput::all()
            .into_iter()
            .map(Self::subscribe)
            .collect()
    }

    /// Converts this subscription into its analytics batch stream.
    pub fn into_stream(self) -> impl Stream<Item = AnalyticsEventBatch> + Unpin {
        BroadcastStream::new(self.receiver).filter_map(|received| {
            ready(match received {
                Ok(batch) => Some(batch),
                Err(error) => {
                    warn!(message = "Mezmo analytics source lagged behind.", %error);
                    None
                }
            })
        })
    }
}
