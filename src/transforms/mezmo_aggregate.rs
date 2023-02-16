use std::{
    borrow::{Borrow, Cow},
    collections::{hash_map::Entry, BTreeMap, BTreeSet, HashMap},
    pin::Pin,
    time::Duration,
};

use async_stream::stream;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use lookup::{lookup_v2::ValuePath, path};
use std::hash::{Hash, Hasher};
use value::Value;
use vector_config::configurable_component;
use vector_core::config::LogNamespace;
use vector_core::{
    config::log_schema,
    event::{metric::mezmo::TransformError, LogEvent},
};

use ordered_float::NotNan;

use crate::{
    config::{DataType, Input, Output, TransformConfig, TransformContext},
    event::{Event, EventMetadata},
    internal_events::{
        MezmoAggregateEventRecorded, MezmoAggregateFlushed, MezmoAggregateUpdateFailed,
    },
    mezmo::{user_trace::handle_transform_error, MezmoContext},
    schema,
    transforms::{TaskTransform, Transform},
};

type Error = TransformError;

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
impl TransformConfig for AggregateConfig {
    async fn build(&self, context: &TransformContext) -> crate::Result<Transform> {
        Aggregate::new_with_mezmo_ctx(self, context.mezmo_ctx.clone()).map(Transform::event_task)
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn outputs(&self, _: &schema::Definition, _: LogNamespace) -> Vec<Output> {
        vec![Output::default(DataType::Log)]
    }
}

#[derive(Debug)]
pub struct Aggregate {
    interval: Duration,
    aggregated: HashMap<LogMetricSeries, LogMetric>,

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
            aggregated: Default::default(),
            mezmo_ctx: mezmo_ctx,
        })
    }

    fn record(&mut self, event: Event) {
        match self.inner_record(event.into_log()) {
            Ok(_) => {}
            Err(err) => handle_transform_error(&self.mezmo_ctx, err),
        }
        emit!(MezmoAggregateEventRecorded);
    }

    fn inner_record(&mut self, event: LogEvent) -> Result<(), Error> {
        let (series, metric) = into_parts(event)?;
        match metric.kind()?.borrow() {
            "incremental" => match self.aggregated.entry(series) {
                Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    // In order to update (add) the new and old kind's must match
                    if existing.kind()? == metric.kind()? && existing.update(&metric)? {
                        existing.metadata.merge(metric.metadata)
                    } else {
                        emit!(MezmoAggregateUpdateFailed);
                        *existing = metric
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(metric);
                }
            },
            "absolute" => {
                self.aggregated.insert(series, metric);
            }
            _ => {
                return Err(Error::InvalidKind {
                    kind: metric.kind()?.to_string(),
                })
            }
        }

        Ok(())
    }

    fn flush_into(&mut self, output: &mut Vec<Event>) {
        let aggregated = std::mem::take(&mut self.aggregated);
        for value in aggregated.into_values() {
            output.push(value.into());
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

fn into_parts(event: LogEvent) -> Result<(LogMetricSeries, LogMetric), Error> {
    let (fields, metadata) = event.into_parts();

    let message = fields
        .get(log_schema().message_key())
        .ok_or_else(|| Error::FieldNotFound {
            field: log_schema().message_key().into(),
        })?
        .as_object()
        .ok_or(Error::FieldInvalidType {
            field: log_schema().message_key().into(),
        })?;

    let name = if let Some(name) = message.get("name") {
        if name.is_bytes() {
            Ok(name.to_owned())
        } else {
            Err(Error::FieldInvalidType {
                field: "name".to_string(),
            })
        }
    } else {
        Err(Error::FieldNotFound {
            field: "name".to_string(),
        })
    }?;

    let namespace = if let Some(namespace) = message.get("namespace") {
        if namespace.is_bytes() {
            Ok(Some(namespace.to_owned()))
        } else {
            Err(Error::FieldInvalidType {
                field: "namespace".to_string(),
            })
        }
    } else {
        Ok(None)
    }?;

    let tags = if let Some(tags) = message.get("tags") {
        if let Some(object) = tags.as_object() {
            if object.values().all(|v| v.is_bytes()) {
                Ok(Some(tags.to_owned()))
            } else {
                Err(Error::FieldInvalidType {
                    field: "tags".to_string(),
                })
            }
        } else {
            Err(Error::FieldInvalidType {
                field: "tags".to_string(),
            })
        }
    } else {
        Ok(None)
    }?;

    return Ok((
        LogMetricSeries {
            name,
            namespace,
            tags,
        },
        LogMetric { fields, metadata },
    ));
}

#[derive(Debug, Eq)]
struct LogMetricSeries {
    name: Value,
    namespace: Option<Value>,
    tags: Option<Value>,
}

impl LogMetricSeries {
    fn name(&self) -> Cow<str> {
        self.name.as_str().unwrap()
    }

    fn namespace(&self) -> Option<Cow<str>> {
        if let Some(ns) = &self.namespace {
            return Some(ns.as_str().unwrap());
        }
        None
    }

    fn tags(&self) -> Option<&BTreeMap<String, Value>> {
        if let Some(tags) = &self.tags {
            return Some(tags.as_object().unwrap());
        }
        None
    }
}

impl PartialEq for LogMetricSeries {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.namespace() == other.namespace()
            && self.tags() == other.tags()
    }
}

impl Hash for LogMetricSeries {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.namespace().hash(state);
        self.tags().hash(state);
    }
}

#[derive(Debug)]
struct LogMetric {
    fields: Value,
    metadata: EventMetadata,
}

/// This is wrapper emulates [`vector_core::event::Metric`] and implements the
/// `add()` logic from [`vector_core::event::MetricValue`].
impl LogMetric {
    /// See [`vector_core::event::metric::MetricData::update()`]
    fn update(&mut self, other: &LogMetric) -> Result<bool, Error> {
        let new_ts = match (self.timestamp()?, other.timestamp()?) {
            (Some(t1), Some(t2)) => Some(t1.max(t2).to_owned()),
            (Some(t), None) | (None, Some(t)) => Some(t.to_owned()),
            (_, _) => None,
        };

        Ok(self.add(other)? && {
            if let Some(new_ts) = new_ts {
                self.fields
                    .insert(log_schema().timestamp_key(), Value::Timestamp(new_ts));
            }
            true
        })
    }

    /// See [`vector_core::event::metric::MetricValue::add()`]
    fn add(&mut self, other: &LogMetric) -> Result<bool, Error> {
        Ok(
            match (self.value_type()?.borrow(), other.value_type()?.borrow()) {
                ("counter", "counter") | ("gauge", "gauge") => {
                    self.add_counter_or_gauge(other)?;
                    true
                }
                ("set", "set") => {
                    self.add_set(other)?;
                    true
                }
                ("distribution", "distribution") => {
                    self.add_distribution(other)?;
                    true
                }
                ("histogram", "histogram") => {
                    self.add_histogram(other)?;
                    true
                }
                (_, _) => false,
            },
        )
    }

    fn add_counter_or_gauge(&mut self, other: &LogMetric) -> Result<(), Error> {
        let c1 = self.value_mut()?;
        let c2 = other.value()?;
        *c1 = Value::Float(check_float(c1, "value")? + check_float(c2, "value")?);
        Ok(())
    }

    fn add_set(&mut self, other: &LogMetric) -> Result<(), Error> {
        let mut s = self.values()?;
        s.extend(other.values()?);
        *self.values_mut()? = s
            .into_iter()
            .map(|v| Value::Bytes(Bytes::copy_from_slice(v.as_bytes())))
            .collect();
        Ok(())
    }

    fn add_distribution(&mut self, other: &LogMetric) -> Result<(), Error> {
        let d1 = self.value_mut()?;
        let d2 = other.value()?;
        let statistic1 = get(d1, "statistic", "statistic")?;
        let statistic2 = get(d2, "statistic", "statistic")?;
        if statistic1 == statistic2 {
            let samples1 = as_array_mut(d1, "samples", "samples")?;
            let samples2 = as_array(d2, "samples", "samples")?;
            samples1.extend_from_slice(samples2);
        }
        Ok(())
    }

    fn add_histogram(&mut self, other: &LogMetric) -> Result<(), Error> {
        let h1 = self.value_mut()?;
        let h2 = other.value()?;
        let buckets1 = as_array_mut(h1, "buckets", "buckets")?;
        let buckets2 = as_array(h2, "buckets", "buckets")?;

        let check_upper_limits = || -> Result<bool, Error> {
            for (b1, b2) in buckets1.iter().zip(buckets2) {
                let ul1 = as_float(b1, "upper_limit", "upper_limit")?;
                let ul2 = as_float(b2, "upper_limit", "upper_limit")?;
                if ul1 != ul2 {
                    return Ok(false);
                }
            }
            return Ok(true);
        };

        if buckets1.len() == buckets2.len() && check_upper_limits()? {
            for (b1, b2) in buckets1.iter_mut().zip(buckets2) {
                let count1 = get_mut(b1, "count", "count")?;
                let count2 = as_integer(b2, "count", "count")?;
                *count1 = Value::Integer(check_integer(count1, "count")? + count2);
            }

            let total_count1 = get_mut(h1, "count", "count")?;
            let total_count2 = as_integer(h2, "count", "count")?;
            *total_count1 = Value::Integer(check_integer(total_count1, "count")? + total_count2);

            let total_sum1 = get_mut(h1, "sum", "sum")?;
            let total_sum2 = as_float(h2, "sum", "sum")?;

            *total_sum1 = Value::Float(check_float(total_sum1, "sum")? + total_sum2);
        }
        Ok(())
    }

    fn timestamp(&self) -> Result<Option<&DateTime<Utc>>, Error> {
        self.fields
            .get(log_schema().timestamp_key())
            .map_or(Ok(None), |timestamp| {
                timestamp.as_timestamp().map_or(
                    Err(Error::FieldInvalidType {
                        field: "timestamp".to_string(),
                    }),
                    |timestamp| Ok(Some(timestamp)),
                )
            })
    }

    fn kind(&self) -> Result<Cow<str>, Error> {
        as_str(
            &self.fields,
            path!(log_schema().message_key(), "kind"),
            "kind",
        )
    }

    fn value_type(&self) -> Result<Cow<str>, Error> {
        as_str(
            &self.fields,
            path!(log_schema().message_key(), "value", "type"),
            "value.type",
        )
    }

    fn values(&self) -> Result<BTreeSet<String>, Error> {
        let values = as_array(
            &self.fields,
            path!(log_schema().message_key(), "value", "value", "values"),
            "value.value.values",
        )?;
        let mut set = BTreeSet::new();
        for value in values {
            set.insert(check_str(value, "values")?.to_string());
        }
        Ok(set)
    }

    fn values_mut(&mut self) -> Result<&mut Vec<Value>, Error> {
        as_array_mut(
            &mut self.fields,
            path!(log_schema().message_key(), "value", "value", "values"),
            "value.value.values",
        )
    }

    fn value(&self) -> Result<&Value, Error> {
        get(
            &self.fields,
            path!(log_schema().message_key(), "value", "value"),
            "value.value",
        )
    }

    fn value_mut(&mut self) -> Result<&mut Value, Error> {
        get_mut(
            &mut self.fields,
            path!(log_schema().message_key(), "value", "value"),
            "value.value",
        )
    }
}

fn check_str<'a>(value: &'a Value, name: &str) -> Result<Cow<'a, str>, Error> {
    value.as_str().ok_or(Error::FieldInvalidType {
        field: name.to_string(),
    })
}

fn as_str<'a>(
    root: &'a Value,
    path: impl ValuePath<'a>,
    name: &str,
) -> Result<Cow<'a, str>, Error> {
    check_str(get(root, path, name)?, name)
}

fn check_float<'a>(value: &'a Value, name: &str) -> Result<NotNan<f64>, Error> {
    value.as_float().ok_or(Error::FieldInvalidType {
        field: name.to_string(),
    })
}

fn as_float<'a>(
    root: &'a Value,
    path: impl ValuePath<'a>,
    name: &str,
) -> Result<NotNan<f64>, Error> {
    check_float(get(root, path, name)?, name)
}

fn check_integer<'a>(value: &'a Value, name: &str) -> Result<i64, Error> {
    value.as_integer().ok_or(Error::FieldInvalidType {
        field: name.to_string(),
    })
}

fn as_integer<'a>(root: &'a Value, path: impl ValuePath<'a>, name: &str) -> Result<i64, Error> {
    check_integer(get(root, path, name)?, name)
}

fn as_array<'a>(
    root: &'a Value,
    path: impl ValuePath<'a>,
    name: &str,
) -> Result<&'a [Value], Error> {
    get(root, path, name)?
        .as_array()
        .ok_or(Error::FieldInvalidType {
            field: name.to_string(),
        })
}

fn as_array_mut<'a>(
    root: &'a mut Value,
    path: impl ValuePath<'a>,
    name: &str,
) -> Result<&'a mut Vec<Value>, Error> {
    get_mut(root, path, name)?
        .as_array_mut()
        .ok_or(Error::FieldInvalidType {
            field: name.to_string(),
        })
}

fn get<'a>(root: &'a Value, path: impl ValuePath<'a>, name: &str) -> Result<&'a Value, Error> {
    match root.get(path) {
        None => Err(Error::FieldNotFound {
            field: name.to_string(),
        }),
        Some(Value::Null) => Err(Error::FieldNull {
            field: name.to_string(),
        }),
        Some(value) => Ok(value),
    }
}

fn get_mut<'a>(
    root: &'a mut Value,
    path: impl ValuePath<'a>,
    name: &str,
) -> Result<&'a mut Value, Error> {
    match root.get_mut(path) {
        None => Err(Error::FieldNotFound {
            field: name.to_string(),
        }),
        Some(Value::Null) => Err(Error::FieldNull {
            field: name.to_string(),
        }),
        Some(value) => Ok(value),
    }
}

impl From<LogMetric> for Event {
    fn from(metric: LogMetric) -> Self {
        Event::Log(LogEvent::from_parts(metric.fields, metric.metadata))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, task::Poll};

    use futures::stream;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;
    use crate::{
        event::Event, test_util::components::assert_transform_compliance,
        transforms::test::create_topology,
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<AggregateConfig>();
    }

    fn make_event(s: &str) -> Event {
        Event::Log(
            serde_json::from_str::<BTreeMap<String, Value>>(s)
                .unwrap()
                .into(),
        )
    }

    #[test]
    fn incremental() {
        let mut agg = Aggregate::new(&AggregateConfig {
            interval_ms: 1000_u64,
        })
        .unwrap();

        let counter_a_1 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "counter", "value": 42.0 }
            }
        }"#,
        );

        let counter_a_2 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "counter", "value": 43.0 }
            }
        }"#,
        );

        let counter_a_summed = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "counter", "value": 85.0 }
            }
        }"#,
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

        let counter_b_1 = make_event(
            r#"{
                "message": {
                "name": "counter_b",
                "kind": "incremental",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "counter", "value": 44.0 }
            }
        }"#,
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

        let gauge_a_1 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "gauge", "value": 42.0 }
            }
        }"#,
        );

        let gauge_a_2 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "gauge", "value": 43.0 }
            }
        }"#,
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

        let gauge_b_1 = make_event(
            r#"{
                "message": {
                "name": "gauge_b",
                "kind": "absolute",
                "namespace": "ns",
                "tags": {"k1": "v1"},
                "value": { "type": "gauge", "value": 44.0 }
            }
        }"#,
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

        let counter = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "incremental",
                "value": { "type": "counter", "value": 42.0 }
            }
        }"#,
        );

        let set = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "incremental",
                "value": { "type": "set", "value": { "values": ["a", "b"] } }
            }
        }"#,
        );

        let summed = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "incremental",
                "value": { "type": "counter", "value": 84.0 }
            }
        }"#,
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

        let incremental = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "incremental",
                "value": { "type": "counter", "value": 42.0 }
            }
        }"#,
        );

        let absolute = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "absolute",
                "value": { "type": "counter", "value": 43.0 }
            }
        }"#,
        );

        let summed = make_event(
            r#"{
                "message": {
                "name": "the-thing",
                "kind": "incremental",
                "value": { "type": "counter", "value": 84.0 }
            }
        }"#,
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

        let counter_a_1 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 42.0 }
            }
        }"#,
        );
        let counter_a_2 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 43.0 }
            }
        }"#,
        );
        let counter_a_summed = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 85.0 }
            }
        }"#,
        );
        let gauge_a_1 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "value": { "type": "gauge", "value": 42.0 }
            }
        }"#,
        );
        let gauge_a_2 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "value": { "type": "gauge", "value": 43.0 }
            }
        }"#,
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

        let counter_a_1 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 42.0 }
            }
        }"#,
        );
        let counter_a_2 = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 43.0 }
            }
        }"#,
        );
        let counter_a_summed = make_event(
            r#"{
                "message": {
                "name": "counter_a",
                "kind": "incremental",
                "value": { "type": "counter", "value": 85.0 }
            }
        }"#,
        );
        let gauge_a_1 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "value": { "type": "gauge", "value": 42.0 }
            }
        }"#,
        );
        let gauge_a_2 = make_event(
            r#"{
                "message": {
                "name": "gauge_a",
                "kind": "absolute",
                "value": { "type": "gauge", "value": 43.0 }
            }
        }"#,
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

        let histogram_a_1 = make_event(
            r#"{
                "message": {
                "name": "histogram_a",
                "kind": "incremental",
                "value": {
                    "type": "histogram",
                    "value": {
                        "buckets": [ { "upper_limit": 2.0, "count": 1 } ],
                        "count": 1,
                        "sum": 1.0
                    }
                }
            }
        }"#,
        );

        let histogram_a_2 = make_event(
            r#"{
                "message": {
                "name": "histogram_a",
                "kind": "incremental",
                "value": {
                    "type": "histogram",
                    "value": {
                        "buckets": [ { "upper_limit": 2.0, "count": 2 } ],
                        "count": 2,
                        "sum": 4.0
                    }
                }
            }
        }"#,
        );

        let histogram_a_aggregated = make_event(
            r#"{
                "message": {
                "name": "histogram_a",
                "kind": "incremental",
                "value": {
                    "type": "histogram",
                    "value": {
                        "buckets": [ { "upper_limit": 2.0, "count": 3 } ],
                        "count": 3,
                        "sum": 5.0
                    }
                }
            }
        }"#,
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

        let histogram_b_1 = make_event(
            r#"{
                "message": {
                "name": "histogram_b",
                "kind": "incremental",
                "value": {
                    "type": "histogram",
                    "value": {
                        "buckets": [ { "upper_limit": 2.0, "count": 2 } ],
                        "count": 2,
                        "sum": 4.0
                    }
                }
            }
        }"#,
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

        let set_a_1 = make_event(
            r#"{
                "message": {
                "name": "set_a",
                "kind": "incremental",
                "value": {
                    "type": "set",
                    "value": { "values": ["a"] }
                }
            }
            }"#,
        );

        let set_a_2 = make_event(
            r#"{
                "message": {
                "name": "set_a",
                "kind": "incremental",
                "value": {
                    "type": "set",
                    "value": { "values": ["b"] }
                }
            }
            }"#,
        );

        let set_a_aggregated = make_event(
            r#"{
                "message": {
                "name": "set_a",
                "kind": "incremental",
                "value": {
                    "type": "set",
                    "value": { "values": ["a", "b"] }
                }
            }
            }"#,
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

        let set_b_1 = make_event(
            r#"{
                "message": {
                "name": "set_b",
                "kind": "incremental",
                "value": {
                    "type": "set",
                    "value": { "values": ["c"] }
                }
            }
            }"#,
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

        let distribution_a_1 = make_event(
            r#"{
                "message": {
                "name": "distribution_a",
                "kind": "incremental",
                "value": {
                    "type": "distribution",
                    "value": {
                        "samples": [
                            {"value": 1.2, "rate": 2}
                        ],
                        "statistic": "summary"
                    }
                }
            }
        }"#,
        );
        let distribution_a_2 = make_event(
            r#"{
                "message": {
                "name": "distribution_a",
                "kind": "incremental",
                "value": {
                    "type": "distribution",
                    "value": {
                        "samples": [
                            {"value": 1.3, "rate": 3}
                        ],
                        "statistic": "summary"
                    }
                }
            }
        }"#,
        );
        let distribution_a_aggregated = make_event(
            r#"{
                "message": {
                "name": "distribution_a",
                "kind": "incremental",
                "value": {
                    "type": "distribution",
                    "value": {
                        "samples": [
                            {"value": 1.2, "rate": 2},
                            {"value": 1.3, "rate": 3}
                        ],
                        "statistic": "summary"
                    }
                }
            }
        }"#,
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

        let distribution_b_1 = make_event(
            r#"{
                "message": {
                "name": "distribution_b",
                "kind": "incremental",
                "value": {
                    "type": "distribution",
                    "value": {
                        "samples": [
                            {"value": 1.4, "rate": 4}
                        ],
                        "statistic": "summary"
                    }
                }
            }
        }"#,
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

        let summary_a_1 = make_event(
            r#"{
                "message": {
                "name": "summary_a",
                "kind": "incremental",
                "value": {
                    "type": "summary",
                    "value": {
                        "quantiles": [ { "quantile": 1.1, "value": 1.1 } ],
                        "count": 1,
                        "sum": 1.0
                    }
                }
            }
        }"#,
        );
        let summary_a_2 = make_event(
            r#"{
                "message": {
                "name": "summary_a",
                "kind": "incremental",
                "value": {
                    "type": "summary",
                    "value": {
                        "quantiles": [ { "quantile": 2.2, "value": 2.2 } ],
                        "count": 2,
                        "sum": 2.0
                    }
                }
            }
        }"#,
        );
        // Summaries are not incremental and will replace that last metric
        let summary_a_aggregated = summary_a_2.clone();

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

        let summary_b_1 = make_event(
            r#"{
                "message": {
                "name": "summary_b",
                "kind": "incremental",
                "value": {
                    "type": "summary",
                    "value": {
                        "quantiles": [ { "quantile": 3.3, "value": 3.3 } ],
                        "count": 3,
                        "sum": 3.0
                    }
                }
            }
        }"#,
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
