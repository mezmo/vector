use chrono::{DateTime, NaiveDateTime, Utc};
use prometheus_remote_write::prometheus::{
    Label, MetricMetadata, MetricType, Sample, TimeSeries, WriteRequest,
};
use prometheus_remote_write::validation::StaticValidate;
use std::borrow::Cow;
use std::collections::BTreeMap;

use smallvec::SmallVec;

use vector_core::{
    config::log_schema,
    event::{
        metric::mezmo::{
            IntoTagValue, IntoValue, MetricTags, MetricTagsAccessor, MetricToLogEvent,
            MetricValueAccessor, MetricValueSerializable, MezmoMetric,
        },
        Event, MetricKind,
    },
};

use vector_core::config::LogNamespace;

use super::metric_sample_types::{
    BasicMetricValue, Counter, Gauge, HistogramBucketValue, HistogramMetricValue,
    SummaryMetricValue, SummaryQuantileValue, Untyped,
};

#[derive(Debug, snafu::Snafu)]
pub enum ParseError {
    #[snafu(display("Unexpected Summary or Histogram: {message}"))]
    UnknownSummaryOrHistogram { message: String },
    #[snafu(display("Unexpected Metric Builder"))]
    UnexpectedMetricBuilder,
    #[snafu(display("Unexpected Sample Type for Sample Group"))]
    SampleGroupTypeMismatch,
    #[snafu(display("Missing `le` label"))]
    ExpectedLeLabel,
    #[snafu(display("Missing `quantile` label"))]
    ExpectedQuantileLabel,
    #[snafu(display("Missing label {value}"))]
    MissingLabel { value: String },
    ParseFloatError {
        #[snafu(source)]
        source: std::num::ParseFloatError,
    },
    #[snafu(display("Value {value} out of range to be converted to a u64"))]
    F64toU64ValueOutOfRange { value: f64 },
    #[snafu(display("Value {value} out of range to be converted to a i64"))]
    F64toI64ValueOutOfRange { value: f64 },
    #[snafu(display("Duplicate Histogram Sum sample"))]
    DuplicateHistogramSumSample,
    #[snafu(display("Duplicate Histogram Count sample"))]
    DuplicateHistogramCountSample,
    #[snafu(display("Duplicate Summary Sum sample"))]
    DuplicateSummarySumSample,
    #[snafu(display("Duplicate Summary Count sample"))]
    DuplicateSummaryCountSample,
    ProtobufError {
        #[snafu(source)]
        source: prometheus_remote_write::Error,
    },
}

fn try_f64_to_u64(f: f64) -> Result<u64, ParseError> {
    if 0.0 <= f && f <= u64::MAX as f64 {
        Ok(f as u64)
    } else {
        Err(ParseError::F64toU64ValueOutOfRange { value: f })
    }
}

#[derive(Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
enum GroupingStrategy<'a> {
    HistogramBucket(Cow<'a, str>),
    HistogramSum,
    HistogramCount,
    SummaryQuantile(Cow<'a, str>),
    SummarySum,
    SummaryCount,
    Counter,
    Gauge,
    Untyped,
}

impl<'a> GroupingStrategy<'a> {
    fn from_basic_type(mt: &MetricType) -> Self {
        match mt {
            MetricType::COUNTER => Self::Counter,
            MetricType::GAUGE => Self::Gauge,
            _ => Self::Untyped,
        }
    }

    fn new_group_map(&self) -> TypedSampleGroupMap<'a> {
        use GroupingStrategy::*;
        match self {
            HistogramBucket(_) | HistogramSum | HistogramCount => {
                TypedSampleGroupMap::new(GroupedSampleType::Histogram)
            }
            SummaryQuantile(_) | SummarySum | SummaryCount => {
                TypedSampleGroupMap::new(GroupedSampleType::Summary)
            }
            Counter => TypedSampleGroupMap::new(GroupedSampleType::Counter),
            Gauge => TypedSampleGroupMap::new(GroupedSampleType::Gauge),
            Untyped => TypedSampleGroupMap::new(GroupedSampleType::Untyped),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SampleGrouping<'a> {
    name: Cow<'a, str>,
    kind: GroupingStrategy<'a>,
}

enum GroupedSampleType {
    Summary,
    Histogram,
    Gauge,
    Counter,
    Untyped,
}

#[derive(Ord, PartialOrd, Debug, Eq, Hash, PartialEq)]
struct SampleGroupKey<'a> {
    pub timestamp: Option<i64>,
    pub labels: &'a [Label<'a>],
}

type SampleGroupMap<'a, T> = BTreeMap<SampleGroupKey<'a>, T>;

#[derive(Debug)]
enum TypedSampleGroupMap<'a> {
    Summary(SampleGroupMap<'a, SummaryMetricValue>),
    Histogram(SampleGroupMap<'a, HistogramMetricValue>),
    Gauge(SampleGroupMap<'a, BasicMetricValue<Gauge>>),
    Counter(SampleGroupMap<'a, BasicMetricValue<Counter>>),
    Untyped(SampleGroupMap<'a, BasicMetricValue<Untyped>>),
}

fn matching_group<'a, 's, T: Default>(
    values: &'s mut SampleGroupMap<'a, T>,
    group: SampleGroupKey<'a>,
) -> &'s mut T {
    values.entry(group).or_insert_with(T::default)
}

#[derive(Debug)]
enum TypedSampleGroupMapIter<T, U, V, W, X> {
    Summary(T),
    Histogram(U),
    Gauge(V),
    Counter(W),
    Untyped(X),
}

#[derive(Debug)]
enum TypedSample<T, U, V, W, X> {
    Summary(T),
    Histogram(U),
    Gauge(V),
    Counter(W),
    Untyped(X),
}

impl<'a, T, U, V, W, X> MetricValueAccessor<'a> for TypedSample<T, U, V, W, X>
where
    T: MetricValueAccessor<
        'a,
        ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>,
        ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>,
    >,
    U: MetricValueAccessor<
        'a,
        ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>,
        ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>,
    >,
    V: MetricValueAccessor<
        'a,
        ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>,
        ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>,
    >,
    W: MetricValueAccessor<
        'a,
        ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>,
        ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>,
    >,
    X: MetricValueAccessor<
        'a,
        ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>,
        ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>,
    >,
{
    type ArrIter = std::array::IntoIter<&'a dyn IntoValue, 0>;
    type ObjIter = std::array::IntoIter<(&'a dyn ToString, &'a dyn IntoValue), 3>;

    fn metric_type(&'a self) -> Option<Cow<'a, str>> {
        use TypedSample::*;
        match self {
            Summary(val) => val.metric_type(),
            Histogram(val) => val.metric_type(),
            Gauge(val) => val.metric_type(),
            Counter(val) => val.metric_type(),
            Untyped(val) => val.metric_type(),
        }
    }

    fn value(&'a self) -> MetricValueSerializable<'a, Self::ArrIter, Self::ObjIter> {
        use TypedSample::*;
        match self {
            Summary(val) => val.value(),
            Histogram(val) => val.value(),
            Gauge(val) => val.value(),
            Counter(val) => val.value(),
            Untyped(val) => val.value(),
        }
    }
}

impl<'a> TypedSampleGroupMap<'a> {
    fn new(kind: GroupedSampleType) -> Self {
        match kind {
            GroupedSampleType::Summary => Self::Summary(BTreeMap::default()),
            GroupedSampleType::Histogram => Self::Histogram(BTreeMap::default()),
            GroupedSampleType::Counter => Self::Counter(BTreeMap::default()),
            GroupedSampleType::Gauge => Self::Gauge(BTreeMap::default()),
            GroupedSampleType::Untyped => Self::Untyped(BTreeMap::default()),
        }
    }

    /// Logic for inserting a Sample into it's existing group based on
    /// the grouping strategy provided
    fn insert_sample(
        &mut self,
        grouping_strategy: &GroupingStrategy<'a>,
        key: SampleGroupKey<'a>,
        value: f64,
    ) -> Result<(), ParseError> {
        use std::str::FromStr;
        match (self, grouping_strategy) {
            (Self::Counter(ref mut metrics), GroupingStrategy::Counter) => {
                metrics.insert(key, BasicMetricValue::new(value));
            }
            (Self::Gauge(ref mut metrics), GroupingStrategy::Gauge) => {
                metrics.insert(key, BasicMetricValue::new(value));
            }
            (Self::Untyped(ref mut metrics), GroupingStrategy::Untyped) => {
                metrics.insert(key, BasicMetricValue::new(value));
            }
            (
                Self::Histogram(ref mut metrics),
                GroupingStrategy::HistogramBucket(bucket_id_str),
            ) => {
                let upper_limit = f64::from_str(bucket_id_str)
                    .map_err(|e| ParseError::ParseFloatError { source: e })?;
                let count = try_f64_to_u64(value)?;
                matching_group(metrics, key)
                    .buckets
                    .push(HistogramBucketValue { upper_limit, count });
            }
            (Self::Histogram(ref mut metrics), GroupingStrategy::HistogramSum) => {
                matching_group(metrics, key).sum = value;
            }
            (Self::Histogram(ref mut metrics), GroupingStrategy::HistogramCount) => {
                matching_group(metrics, key).count = try_f64_to_u64(value)?;
            }
            (Self::Summary(ref mut metrics), GroupingStrategy::SummaryQuantile(quantile_str)) => {
                let quantile = f64::from_str(quantile_str)
                    .map_err(|e| ParseError::ParseFloatError { source: e })?;
                matching_group(metrics, key)
                    .quantiles
                    .push(SummaryQuantileValue { quantile, value });
            }
            (Self::Summary(ref mut metrics), GroupingStrategy::SummarySum) => {
                matching_group(metrics, key).sum = value;
            }
            (Self::Summary(ref mut metrics), GroupingStrategy::SummaryCount) => {
                matching_group(metrics, key).count = try_f64_to_u64(value)?;
            }
            _ => {
                return Err(ParseError::SampleGroupTypeMismatch);
            }
        }
        Ok(())
    }

    fn to_events(&self, metric_group_name: Cow<'a, str>, out: &mut SmallVec<[Event; 1]>) {
        // Helper struct
        struct MetricTagsWrapper<'a> {
            tags: &'a [Label<'a>],
        }

        impl<'a> MetricTagsAccessor<'a> for MetricTagsWrapper<'a> {
            type Iter = std::iter::Map<
                std::slice::Iter<'a, Label<'a>>,
                fn(&'a Label<'a>) -> (&'a dyn ToString, &'a dyn IntoTagValue),
            >;

            fn tags(&'a self) -> MetricTags<'a, Self::Iter> {
                MetricTags {
                    tags: self.tags.iter().map(|label: &'a Label| {
                        (
                            &label.name as &'a dyn ToString,
                            &label.value as &'a dyn IntoTagValue,
                        )
                    }),
                }
            }
        }

        use TypedSampleGroupMap::*;
        let mut iter = match self {
            Counter(metrics) => TypedSampleGroupMapIter::Counter(
                metrics.iter().map(|(k, v)| (k, TypedSample::Counter(v))),
            ),
            Gauge(metrics) => TypedSampleGroupMapIter::Gauge(
                metrics.iter().map(|(k, v)| (k, TypedSample::Gauge(v))),
            ),
            Untyped(metrics) => TypedSampleGroupMapIter::Untyped(
                metrics.iter().map(|(k, v)| (k, TypedSample::Untyped(v))),
            ),
            Histogram(metrics) => TypedSampleGroupMapIter::Histogram(
                metrics.iter().map(|(k, v)| (k, TypedSample::Histogram(v))),
            ),
            Summary(metrics) => TypedSampleGroupMapIter::Summary(
                metrics.iter().map(|(k, v)| (k, TypedSample::Summary(v))),
            ),
        };
        for (key, value) in std::iter::from_fn(move || {
            use TypedSampleGroupMapIter::*;
            match iter {
                Summary(ref mut it) => it.next(),
                Histogram(ref mut it) => it.next(),
                Gauge(ref mut it) => it.next(),
                Counter(ref mut it) => it.next(),
                Untyped(ref mut it) => it.next(),
            }
        }) {
            let tags = MetricTagsWrapper { tags: key.labels };
            let mut log_event = {
                MezmoMetric {
                    name: metric_group_name.clone(),
                    namespace: None,             // TODO
                    kind: &MetricKind::Absolute, // All prom metrics are Absolute?
                    tags: Some(&tags),
                    value: &value,
                }
                .to_log_event()
            };
            if let (Some(timestamp_key), Some(timestamp)) =
                (log_schema().timestamp_key(), key.timestamp)
            {
                let ts = NaiveDateTime::from_timestamp_millis(timestamp)
                    .expect("timestamp should be a valid timestamp");
                let ts = DateTime::<Utc>::from_utc(ts, Utc);
                log_event.insert((lookup::PathPrefix::Event, timestamp_key), ts);
            }
            out.push(log_event.into());
        }
    }
}

fn process_samples<'a, 's>(
    sample_groups: &'s mut BTreeMap<Cow<'a, str>, TypedSampleGroupMap<'a>>,
    metric_base_name: Cow<'a, str>,
    labels: &'a [Label<'a>],
    grouping_strategy: &GroupingStrategy<'a>,
    samples: &[Sample],
) -> Result<(), ParseError> {
    for Sample { value, timestamp } in samples {
        let sample_group_key = SampleGroupKey {
            timestamp: Some(*timestamp),
            labels,
        };
        let group = sample_groups
            .entry(metric_base_name.clone())
            .or_insert_with(|| grouping_strategy.new_group_map());

        group.insert_sample(grouping_strategy, sample_group_key, *value)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
pub(crate) struct MetricMetadataGroups<'a>(BTreeMap<Cow<'a, str>, MetricType>);

// and we'll implement FromIterator
impl<'a> std::iter::FromIterator<&'a MetricMetadata<'a>> for MetricMetadataGroups<'a> {
    fn from_iter<I: IntoIterator<Item = &'a MetricMetadata<'a>>>(iter: I) -> Self {
        iter.into_iter().fold(
            MetricMetadataGroups::default(),
            |mut acc,
             MetricMetadata {
                 type_pb,
                 metric_family_name,
                 ..
             }| {
                acc.insert(metric_family_name.clone(), *type_pb);
                acc
            },
        )
    }
}

impl<'a> MetricMetadataGroups<'a> {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert(&mut self, metric_family_name: Cow<'a, str>, type_pb: MetricType) {
        self.0.insert(metric_family_name.clone(), type_pb);
    }

    fn find_and_prep_name(
        &self,
        name: &mut Cow<'a, str>,
        suffix_len: usize,
    ) -> Option<&MetricType> {
        let len = name.len();
        let prefix = &name[..len - suffix_len];
        if let Some((k, v)) = self.0.get_key_value(prefix) {
            *name = k.clone();
            Some(v)
        } else {
            name.to_mut().truncate(len - suffix_len);
            None
        }
    }

    fn get_grouping_strategy(
        &self,
        labels: &mut Vec<Label<'a>>,
        name: String,
    ) -> Result<(Cow<'a, str>, GroupingStrategy<'a>), ParseError> {
        use MetricType::*;
        const LE_LABEL: &str = "le";
        const QUANTILE_LABEL: &str = "quantile";

        let mut name = Cow::from(name);
        let grouping_strategy = if name.ends_with("_bucket") {
            match self.find_and_prep_name(&mut name, 7) {
                Some(HISTOGRAM | GAUGEHISTOGRAM) => GroupingStrategy::HistogramBucket(
                    extract_label(LE_LABEL, labels)
                        .ok_or(ParseError::ExpectedLeLabel)?
                        .value
                        .clone(),
                ),
                Some(t) => GroupingStrategy::from_basic_type(t),
                _ => GroupingStrategy::Untyped,
            }
        } else if name.ends_with("_sum") {
            match self.find_and_prep_name(&mut name, 4) {
                Some(HISTOGRAM | GAUGEHISTOGRAM) => GroupingStrategy::HistogramSum,
                Some(SUMMARY) => GroupingStrategy::SummarySum,
                Some(t) => GroupingStrategy::from_basic_type(t),
                _ => GroupingStrategy::Untyped,
            }
        } else if name.ends_with("_count") {
            match self.find_and_prep_name(&mut name, 6) {
                Some(HISTOGRAM | GAUGEHISTOGRAM) => GroupingStrategy::HistogramCount,
                Some(SUMMARY) => GroupingStrategy::SummaryCount,
                Some(t) => GroupingStrategy::from_basic_type(t),
                _ => GroupingStrategy::Untyped,
            }
        } else {
            match self.find_and_prep_name(&mut name, 0) {
                Some(SUMMARY) => GroupingStrategy::SummaryQuantile(
                    extract_label(QUANTILE_LABEL, labels)
                        .ok_or(ParseError::ExpectedQuantileLabel)?
                        .value
                        .clone(),
                ),
                Some(t) => GroupingStrategy::from_basic_type(t),
                _ => GroupingStrategy::Untyped,
            }
        };
        Ok((name, grouping_strategy))
    }
}

fn extract_label<'a>(label: &str, labels: &mut Vec<Label<'a>>) -> Option<Label<'a>> {
    labels
        .iter()
        .position(|x| x.name == label)
        .map(|name_index| labels.remove(name_index))
}

pub fn parse_write_req(
    bytes: &[u8],
    _log_namespace: LogNamespace,
) -> vector_common::Result<SmallVec<[Event; 1]>> {
    let mut write_req = WriteRequest::try_from(bytes)?;

    write_req
        .validate()
        .map_err(|e| ParseError::ProtobufError { source: e })?;

    let WriteRequest {
        ref metadata,
        ref mut timeseries,
    } = write_req;

    // Parse the metadata for groups.
    let metric_types_lookup: MetricMetadataGroups = metadata.iter().collect();

    // Group the samples
    let grouped_samples = timeseries.iter_mut().try_fold(
        BTreeMap::new(),
        |mut acc,
         TimeSeries {
             ref mut labels,
             ref samples,
             ..
         }| {
            let name = extract_label(METRIC_NAME_LABEL, labels)
                .map(|label| label.value.to_string())
                .ok_or_else(|| ParseError::MissingLabel {
                    value: METRIC_NAME_LABEL.to_string(),
                })?;
            let (base_name, grouping_strategy) =
                metric_types_lookup.get_grouping_strategy(labels, name)?;
            process_samples(&mut acc, base_name, labels, &grouping_strategy, samples)?;
            Ok::<_, ParseError>(acc)
        },
    )?;

    let mut res = smallvec::smallvec!();
    let grouped_samples_iter = grouped_samples.into_iter();
    for (metric_group_name, metric_group) in grouped_samples_iter {
        metric_group.to_events(metric_group_name, &mut res);
    }
    Ok(res)
}

#[cfg(test)]
mod test {

    use super::parse_write_req;
    use quick_protobuf::Writer;
    use vector_core::config::LogNamespace;

    use prometheus_remote_write::prometheus::{
        Label, MetricMetadata, MetricType, Sample, TimeSeries, WriteRequest,
    };

    use bytes::{BufMut, BytesMut};
    use std::borrow::Cow;

    #[test]
    fn test_count() {
        let out = BytesMut::new();
        let mut out_writer = out.writer();

        let test_label = Label {
            name: Cow::Borrowed("__name__"),
            value: Cow::Borrowed("unknown"),
        };

        let message = WriteRequest {
            timeseries: vec![TimeSeries {
                exemplars: vec![],
                histograms: vec![],
                labels: vec![
                    test_label.clone(),
                    Label {
                        name: Cow::Borrowed("test_label"),
                        value: Cow::Borrowed("test_value"),
                    },
                ],
                samples: vec![Sample::default()],
            }],
            metadata: vec![MetricMetadata {
                help: Cow::from("help"),
                metric_family_name: Cow::from("unknown"),
                type_pb: MetricType::COUNTER,
                unit: Cow::from("unit"),
            }],
        };
        {
            let mut writer = Writer::new(&mut out_writer);
            writer.write_message(&message).expect("failed to write");
        }

        let out = out_writer.into_inner();

        let ret = parse_write_req(&out[1..], LogNamespace::Legacy).expect("Failed to parse");
        assert_eq!(ret.len(), 1);
    }
}
