use std::{
    collections::{HashMap, hash_map::Entry},
    pin::Pin,
    time::Duration,
};

use async_stream::stream;
use futures::{Stream, StreamExt};
use vector_lib::configurable::configurable_component;
use vector_lib::{
    config::LogNamespace,
    event::{Metric, metric::mezmo::to_metric},
};
use vector_lib::{
    config::{OutputId, TransformOutput},
    event::metric::mezmo::from_metric,
};

use crate::{
    config::{DataType, Input, TransformConfig, TransformContext},
    event::{Event, EventMetadata, metric},
    internal_events::{
        MezmoAggregateEventRecorded, MezmoAggregateFlushed, MezmoAggregateUpdateFailed,
    },
    schema,
    transforms::{TaskTransform, Transform},
};
use mezmo::{MezmoContext, user_trace::handle_transform_error};

/// Configuration for the `mezmo_aggregate` transform.
#[configurable_component(transform("mezmo_aggregate"))]
#[derive(Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct AggregateConfig {
    /// The interval between flushes, in milliseconds.
    ///
    /// Over this period metrics with the same series data (name, namespace, tags, …) will be aggregated.
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

const fn default_interval_ms() -> u64 {
    10 * 1000
}

impl_generate_config_from_default!(AggregateConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_aggregate")]
impl TransformConfig for AggregateConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Aggregate::new_with_mezmo_ctx(self, context.mezmo_ctx.clone()).map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(
        &self,
        _: vector_lib::enrichment::TableRegistry,
        _: &[(OutputId, schema::Definition)],
        _: LogNamespace,
    ) -> Vec<TransformOutput> {
        vec![TransformOutput::new(DataType::Log, HashMap::new())]
    }
}

type MetricEntry = (metric::MetricData, EventMetadata);

#[derive(Debug)]
pub struct Aggregate {
    interval: Duration,
    map: HashMap<metric::MetricSeries, MetricEntry>,

    /// The mezmo context used to surface errors
    mezmo_ctx: Option<MezmoContext>,
}

impl Aggregate {
    pub fn new(config: &AggregateConfig) -> crate::Result<Self> {
        Self::new_with_mezmo_ctx(config, None)
    }

    pub fn new_with_mezmo_ctx(
        config: &AggregateConfig,
        mezmo_ctx: Option<MezmoContext>,
    ) -> crate::Result<Self> {
        Ok(Self {
            interval: Duration::from_millis(config.interval_ms),
            map: Default::default(),
            mezmo_ctx,
        })
    }

    fn record(&mut self, event: Event) {
        let event = to_metric(&event.into_log());
        match event {
            Ok(event) => {
                let (series, data, metadata) = event.into_parts();
                match data.kind {
                    metric::MetricKind::Incremental => match self.map.entry(series) {
                        Entry::Occupied(mut entry) => {
                            let existing = entry.get_mut();
                            // In order to update (add) the new and old kind's must match
                            if existing.0.kind == data.kind && existing.0.update(&data) {
                                existing.1.merge(metadata);
                            } else {
                                emit!(MezmoAggregateUpdateFailed);
                                *existing = (data, metadata);
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((data, metadata));
                        }
                    },
                    metric::MetricKind::Absolute => {
                        // Always replace/store
                        self.map.insert(series, (data, metadata));
                    }
                };

                emit!(MezmoAggregateEventRecorded);
            }
            Err(err) => {
                handle_transform_error(&self.mezmo_ctx, err);
            }
        }
    }

    fn flush_into(&mut self, output: &mut Vec<Event>) {
        let map = std::mem::take(&mut self.map);
        for (series, entry) in map.into_iter() {
            output.push(Event::Log(from_metric(&Metric::from_parts(
                series, entry.0, entry.1,
            ))));
        }

        emit!(MezmoAggregateFlushed);
    }
}

impl TaskTransform<Event> for Aggregate {
    fn transform(
        mut self: Box<Self>,
        mut input_rx: Pin<Box<dyn Stream<Item = Event> + Send>>,
    ) -> Pin<Box<dyn Stream<Item = Event> + Send>>
    where
        Self: 'static,
    {
        let mut flush_stream = tokio::time::interval(self.interval);

        Box::pin(stream! {
            let mut output = Vec::new();
            let mut done = false;
            while !done {
                tokio::select! {
                    _ = flush_stream.tick() => {
                        self.flush_into(&mut output);
                    },
                    maybe_event = input_rx.next() => {
                        match maybe_event {
                            None => {
                                self.flush_into(&mut output);
                                done = true;
                            }
                            Some(event) => self.record(event),
                        }
                    }
                };
                for event in output.drain(..) {
                    yield event;
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, task::Poll};

    use futures::stream;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use vector_lib::event::{
        StatisticKind,
        metric::{Bucket, Quantile, Sample},
    };

    use super::*;
    use crate::{
        event::{Event, Metric, metric},
        test_util::components::assert_transform_compliance,
        transforms::test::create_topology,
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<AggregateConfig>();
    }

    fn make_metric(name: &str, kind: metric::MetricKind, value: metric::MetricValue) -> Event {
        Event::Log(metric::mezmo::from_metric(&Metric::new(name, kind, value)))
    }

    #[test]
    fn incremental() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let counter_a_1 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 42.0 },
        );
        let counter_a_2 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 43.0 },
        );
        let counter_a_summed = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 85.0 },
        );

        // Single item, just stored regardless of kind
        agg.record(counter_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item counter_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&counter_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two increments with the same series, should sum into 1
        agg.record(counter_a_1.clone());
        agg.record(counter_a_2);
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&counter_a_summed, &out[0]);

        let counter_b_1 = make_metric(
            "counter_b",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 44.0 },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(counter_a_1.clone());
        agg.record(counter_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "counter_a" => assert_eq!(counter_a_1, event),
                "counter_b" => assert_eq!(counter_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }

    #[test]
    fn absolute() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let gauge_a_1 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 42.0 },
        );
        let gauge_a_2 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 43.0 },
        );

        // Single item, just stored regardless of kind
        agg.record(gauge_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item gauge_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&gauge_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two absolutes with the same series, should get the 2nd (last) back.
        agg.record(gauge_a_1.clone());
        agg.record(gauge_a_2.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&gauge_a_2, &out[0]);

        let gauge_b_1 = make_metric(
            "gauge_b",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 44.0 },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(gauge_a_1.clone());
        agg.record(gauge_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "gauge_a" => assert_eq!(gauge_a_1, event),
                "gauge_b" => assert_eq!(gauge_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }

    #[test]
    fn conflicting_value_type() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let counter = make_metric(
            "the-thing",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 42.0 },
        );
        let mut values = BTreeSet::<String>::new();
        values.insert("a".into());
        values.insert("b".into());
        let set = make_metric(
            "the-thing",
            metric::MetricKind::Incremental,
            metric::MetricValue::Set { values },
        );
        let summed = make_metric(
            "the-thing",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 84.0 },
        );

        // when types conflict the new values replaces whatever is there

        // Start with an counter
        agg.record(counter.clone());
        // Another will "add" to it
        agg.record(counter.clone());
        // Then an set will replace it due to a failed update
        agg.record(set.clone());
        // Then a set union would be a noop
        agg.record(set.clone());
        let mut out = vec![];
        // We should flush 1 item counter
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&set, &out[0]);

        // Start out with an set
        agg.record(set.clone());
        // Union with itself, a noop
        agg.record(set);
        // Send an counter with the same name, will replace due to a failed update
        agg.record(counter.clone());
        // Send another counter will "add"
        agg.record(counter);
        let mut out = vec![];
        // We should flush 1 item counter
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&summed, &out[0]);
    }

    #[test]
    fn conflicting_kinds() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let incremental = make_metric(
            "the-thing",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 42.0 },
        );
        let absolute = make_metric(
            "the-thing",
            metric::MetricKind::Absolute,
            metric::MetricValue::Counter { value: 43.0 },
        );
        let summed = make_metric(
            "the-thing",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 84.0 },
        );

        // when types conflict the new values replaces whatever is there

        // Start with an incremental
        agg.record(incremental.clone());
        // Another will "add" to it
        agg.record(incremental.clone());
        // Then an absolute will replace it with a failed update
        agg.record(absolute.clone());
        // Then another absolute will replace it normally
        agg.record(absolute.clone());
        let mut out = vec![];
        // We should flush 1 item incremental
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&absolute, &out[0]);

        // Start out with an absolute
        agg.record(absolute.clone());
        // Replace it normally
        agg.record(absolute);
        // Send an incremental with the same name, will replace due to a failed update
        agg.record(incremental.clone());
        // Send another incremental will "add"
        agg.record(incremental);
        let mut out = vec![];
        // We should flush 1 item incremental
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&summed, &out[0]);
    }

    #[tokio::test]
    async fn transform_shutdown() {
        let agg = toml::from_str::<AggregateConfig>(
            r#"
interval_ms = 999999
"#,
        )
        .unwrap()
        .build(&TransformContext::default())
        .await
        .unwrap();

        let agg = agg.into_task();

        let counter_a_1 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 42.0 },
        );
        let counter_a_2 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 43.0 },
        );
        let counter_a_summed = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 85.0 },
        );
        let gauge_a_1 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 42.0 },
        );
        let gauge_a_2 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 43.0 },
        );
        let inputs = vec![counter_a_1, counter_a_2, gauge_a_1, gauge_a_2.clone()];

        // Queue up some events to be consumed & recorded
        let in_stream = Box::pin(stream::iter(inputs));
        // Kick off the transform process which should consume & record them
        let mut out_stream = agg.transform_events(in_stream);

        // B/c the input stream has ended we will have gone through the `input_rx.next() => None`
        // part of the loop and do the shutting down final flush immediately. We'll already be able
        // to read our expected bits on the output.
        let mut count = 0_u8;
        while let Some(event) = out_stream.next().await {
            count += 1;
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "counter_a" => assert_eq!(counter_a_summed, event),
                "gauge_a" => assert_eq!(gauge_a_2, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            };
        }
        // There were only 2
        assert_eq!(2, count);
    }

    #[tokio::test]
    async fn transform_interval() {
        let transform_config = toml::from_str::<AggregateConfig>("").unwrap();

        let counter_a_1 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 42.0 },
        );
        let counter_a_2 = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 43.0 },
        );
        let counter_a_summed = make_metric(
            "counter_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Counter { value: 85.0 },
        );
        let gauge_a_1 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 42.0 },
        );
        let gauge_a_2 = make_metric(
            "gauge_a",
            metric::MetricKind::Absolute,
            metric::MetricValue::Gauge { value: 43.0 },
        );

        assert_transform_compliance(async {
            let (tx, rx) = mpsc::channel(10);
            let (topology, out) = create_topology(ReceiverStream::new(rx), transform_config).await;
            let mut out = ReceiverStream::new(out);

            tokio::time::pause();

            // tokio interval is always immediately ready, so we poll once to make sure
            // we trip it/set the interval in the future
            assert_eq!(Poll::Pending, futures::poll!(out.next()));

            // Now send our events
            tx.send(counter_a_1).await.unwrap();
            tx.send(counter_a_2).await.unwrap();
            tx.send(gauge_a_1).await.unwrap();
            tx.send(gauge_a_2.clone()).await.unwrap();
            // We won't have flushed yet b/c the interval hasn't elapsed, so no outputs
            assert_eq!(Poll::Pending, futures::poll!(out.next()));
            // Now fast forward time enough that our flush should trigger.
            tokio::time::advance(Duration::from_secs(11)).await;
            // We should have had an interval fire now and our output aggregate events should be
            // available.
            let mut count = 0_u8;
            while count < 2 {
                if let Some(event) = out.next().await {
                    match event
                        .as_log()
                        .get(".message.name")
                        .unwrap()
                        .to_string_lossy()
                        .as_ref()
                    {
                        "counter_a" => assert_eq!(counter_a_summed, event),
                        "gauge_a" => assert_eq!(gauge_a_2, event),
                        _ => panic!("Unexpected metric name in aggregate output"),
                    };
                    count += 1;
                } else {
                    panic!("Unexpectedly received None in output stream");
                }
            }
            // We should be back to pending, having nothing waiting for us
            assert_eq!(Poll::Pending, futures::poll!(out.next()));

            drop(tx);
            topology.stop().await;
            assert_eq!(out.next().await, None);
        })
        .await;
    }

    #[test]
    fn incremental_histogram() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let histogram_a_1 = make_metric(
            "histogram_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedHistogram {
                buckets: vec![Bucket {
                    upper_limit: 2.0,
                    count: 1,
                }],
                count: 1,
                sum: 1.0,
            },
        );
        let histogram_a_2 = make_metric(
            "histogram_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedHistogram {
                buckets: vec![Bucket {
                    upper_limit: 2.0,
                    count: 2,
                }],
                count: 2,
                sum: 4.0,
            },
        );
        let histogram_a_aggregated = make_metric(
            "histogram_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedHistogram {
                buckets: vec![Bucket {
                    upper_limit: 2.0,
                    count: 3,
                }],
                count: 3,
                sum: 5.0,
            },
        );

        // Single item, just stored regardless of kind
        agg.record(histogram_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item histogram_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&histogram_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two increments with the same series, should sum into 1
        agg.record(histogram_a_1.clone());
        agg.record(histogram_a_2);
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&histogram_a_aggregated, &out[0]);

        let histogram_b_1 = make_metric(
            "histogram_b",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedHistogram {
                buckets: vec![Bucket {
                    upper_limit: 2.0,
                    count: 2,
                }],
                count: 2,
                sum: 4.0,
            },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(histogram_a_1.clone());
        agg.record(histogram_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "histogram_a" => assert_eq!(histogram_a_1, event),
                "histogram_b" => assert_eq!(histogram_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }

    #[test]
    fn incremental_set() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let set_a_1 = make_metric(
            "set_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Set {
                values: BTreeSet::from(["a".to_owned()]),
            },
        );
        let set_a_2 = make_metric(
            "set_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Set {
                values: BTreeSet::from(["b".to_owned()]),
            },
        );
        let set_a_aggregated = make_metric(
            "set_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Set {
                values: BTreeSet::from(["a".to_owned(), "b".to_owned()]),
            },
        );

        // Single item, just stored regardless of kind
        agg.record(set_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item set_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&set_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two increments with the same series, should sum into 1
        agg.record(set_a_1.clone());
        agg.record(set_a_2);
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&set_a_aggregated, &out[0]);

        let set_b_1 = make_metric(
            "set_b",
            metric::MetricKind::Incremental,
            metric::MetricValue::Set {
                values: BTreeSet::from(["c".to_owned()]),
            },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(set_a_1.clone());
        agg.record(set_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "set_a" => assert_eq!(set_a_1, event),
                "set_b" => assert_eq!(set_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }

    #[test]
    fn incremental_distribution() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let distribution_a_1 = make_metric(
            "distribution_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Distribution {
                samples: vec![Sample {
                    rate: 2,
                    value: 1.2,
                }],
                statistic: StatisticKind::Summary,
            },
        );
        let distribution_a_2 = make_metric(
            "distribution_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Distribution {
                samples: vec![Sample {
                    rate: 3,
                    value: 1.3,
                }],
                statistic: StatisticKind::Summary,
            },
        );
        let distribution_a_aggregated = make_metric(
            "distribution_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::Distribution {
                samples: vec![
                    Sample {
                        rate: 2,
                        value: 1.2,
                    },
                    Sample {
                        rate: 3,
                        value: 1.3,
                    },
                ],
                statistic: StatisticKind::Summary,
            },
        );

        // Single item, just stored regardless of kind
        agg.record(distribution_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item distribution_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&distribution_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two increments with the same series, should sum into 1
        agg.record(distribution_a_1.clone());
        agg.record(distribution_a_2);
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&distribution_a_aggregated, &out[0]);

        let distribution_b_1 = make_metric(
            "distribution_b",
            metric::MetricKind::Incremental,
            metric::MetricValue::Distribution {
                samples: vec![Sample {
                    rate: 4,
                    value: 1.4,
                }],
                statistic: StatisticKind::Summary,
            },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(distribution_a_1.clone());
        agg.record(distribution_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "distribution_a" => assert_eq!(distribution_a_1, event),
                "distribution_b" => assert_eq!(distribution_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }

    #[test]
    fn incremental_summary() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let summary_a_1 = make_metric(
            "summary_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedSummary {
                quantiles: vec![Quantile {
                    quantile: 1.1,
                    value: 1.1,
                }],
                count: 1,
                sum: 1.0,
            },
        );
        let summary_a_2 = make_metric(
            "summary_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedSummary {
                quantiles: vec![Quantile {
                    quantile: 2.2,
                    value: 2.2,
                }],
                count: 2,
                sum: 2.0,
            },
        );
        // Summary are not incremental and will replace that last metric
        let summary_a_aggregated = make_metric(
            "summary_a",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedSummary {
                quantiles: vec![Quantile {
                    quantile: 2.2,
                    value: 2.2,
                }],
                count: 2,
                sum: 2.0,
            },
        );

        // Single item, just stored regardless of kind
        agg.record(summary_a_1.clone());
        let mut out = vec![];
        // We should flush 1 item summary_a_1
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&summary_a_1, &out[0]);

        // A subsequent flush doesn't send out anything
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // One more just to make sure that we don't re-see from the other buffer
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(0, out.len());

        // Two increments with the same series, should sum into 1
        agg.record(summary_a_1.clone());
        agg.record(summary_a_2);
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(1, out.len());
        assert_eq!(&summary_a_aggregated, &out[0]);

        let summary_b_1 = make_metric(
            "summary_b",
            metric::MetricKind::Incremental,
            metric::MetricValue::AggregatedSummary {
                quantiles: vec![Quantile {
                    quantile: 3.3,
                    value: 3.3,
                }],
                count: 3,
                sum: 3.0,
            },
        );
        // Two increments with the different series, should get each back as-is
        agg.record(summary_a_1.clone());
        agg.record(summary_b_1.clone());
        out.clear();
        agg.flush_into(&mut out);
        assert_eq!(2, out.len());
        // B/c we don't know the order they'll come back
        for event in out {
            match event
                .as_log()
                .get(".message.name")
                .unwrap()
                .to_string_lossy()
                .as_ref()
            {
                "summary_a" => assert_eq!(summary_a_1, event),
                "summary_b" => assert_eq!(summary_b_1, event),
                _ => panic!("Unexpected metric name in aggregate output"),
            }
        }
    }
}
