use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::sinks::mezmo_opentelemetry::{
    config::OpentelemetryMetricConfig,
    models::{
        value_to_otlp_value, value_to_system_time, OpentelemetryModelMatch, OpentelemetryModelType,
        OpentelemetryResource, OpentelemetryScope, OpentelemetrySpanId, OpentelemetryTraceId,
    },
    sink::OpentelemetrySinkError,
};
use opentelemetry_sdk::{
    metrics::data::{
        Aggregation, DataPoint, Exemplar, Gauge, Histogram, HistogramDataPoint, Metric,
        ResourceMetrics, ScopeMetrics, Sum, Temporality,
    },
    Resource,
};

use opentelemetry::{metrics::Unit, KeyValue};

use vector_lib::{
    config::log_schema,
    event::{
        metric::{samples_to_buckets, Metric as MezmoMetric},
        Event, KeyString, MetricKind, MetricValue, StatisticKind, Value,
    },
};

static WORD_TO_UCUM: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    vec![
        // Time
        ("days", "d"),
        ("hours", "h"),
        ("minutes", "min"),
        ("seconds", "s"),
        ("milliseconds", "ms"),
        ("microseconds", "us"),
        ("nanoseconds", "ns"),
        // Bytes
        ("bytes", "By"),
        ("kibibytes", "KiBy"),
        ("mebibytes", "MiBy"),
        ("gibibytes", "GiBy"),
        ("tibibytes", "TiBy"),
        ("kilobytes", "KBy"),
        ("megabytes", "MBy"),
        ("gigabytes", "GBy"),
        ("terabytes", "TBy"),
        // SI
        ("kilometers", "km"),
        ("meters", "m"),
        ("volts", "V"),
        ("amperes", "A"),
        ("joules", "J"),
        ("watts", "W"),
        ("grams", "g"),
        // Misc
        ("celsius", "Cel"),
        ("C", "°C"),
        ("fahrenheit", "°F"),
        ("F", "°F"),
        ("hertz", "Hz"),
        ("ratio", "1"),
        ("percent", "%"),
        ("packets", "{packets}"),
        ("requests", "{requests}"),
    ]
    .into_iter()
    .collect()
});

static WORD_TO_UCUM_INVERT: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| WORD_TO_UCUM.iter().map(|(_, v)| (*v, *v)).collect());

static PER_WORD_TO_UCUM: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    vec![
        ("second", "s"),
        ("minute", "m"),
        ("hour", "h"),
        ("day", "d"),
        ("week", "w"),
        ("month", "mo"),
        ("year", "y"),
    ]
    .into_iter()
    .collect()
});

static PER_WORD_TO_UCUM_INVERT: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| PER_WORD_TO_UCUM.iter().map(|(_, v)| (*v, *v)).collect());

fn get_property<'a>(
    root: &'a BTreeMap<KeyString, Value>,
    property_name: &'a str,
) -> Result<&'a Value, OpentelemetrySinkError> {
    match root.get(property_name) {
        None => Err(OpentelemetrySinkError::new(
            format!("FieldNotFound field: {}", property_name).as_str(),
        )),
        Some(Value::Null) => Err(OpentelemetrySinkError::new(
            format!("FieldBadValue field: {} is Null", property_name).as_str(),
        )),
        Some(value) => Ok(value),
    }
}

fn get_float(
    value_object: &BTreeMap<KeyString, Value>,
    name: &str,
) -> Result<f64, OpentelemetrySinkError> {
    let value = get_property(value_object, name)?;
    parse_float(value, name)
}

fn parse_float(value: &Value, field_name: &str) -> Result<f64, OpentelemetrySinkError> {
    // Depending on the serialization format and input value (which we don't control)
    // a float value might appear as a Value::Float or Value::Integer
    match value {
        Value::Float(v) => Ok(v.into_inner()),
        Value::Integer(v) => {
            if v < &(2i64.pow(52)) {
                #[allow(clippy::cast_precision_loss)]
                Ok(*v as f64)
            } else {
                Err(OpentelemetrySinkError::new(
                    format!("NumberTruncation field: {} is Null", field_name).as_str(),
                ))
            }
        }
        _ => Err(OpentelemetrySinkError::new(
            format!("FieldInvalidType field: {}", field_name).as_str(),
        )),
    }
}

fn parse_u64(value: &Value, field_name: &str) -> Result<u64, OpentelemetrySinkError> {
    let val = value.as_integer().ok_or_else(|| {
        OpentelemetrySinkError::new(
            format!("FieldInvalidType field: {} is no integer", field_name).as_str(),
        )
    })?;

    if val < 0 {
        // Internally represented as a i64, any negative value overflows
        // field: name.into()
        return Err(OpentelemetrySinkError::new(
            format!("ParseIntOverflow field: {} is less than 0", field_name).as_str(),
        ));
    }

    Ok(val as u64)
}

fn get_object(value_object: &BTreeMap<KeyString, Value>, name: &str) -> BTreeMap<KeyString, Value> {
    get_property(value_object, name)
        .unwrap_or(&Value::Object(BTreeMap::new()))
        .as_object()
        .unwrap_or(&BTreeMap::new())
        .clone()
}

fn get_string_or_defailt<'a>(value_object: &BTreeMap<KeyString, Value>, key: &'a str) -> String {
    match value_object.get(key) {
        Some(Value::Bytes(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
        _ => String::new(),
    }
}

fn get_word_unit_from_name(name: &str, is_counter: bool) -> String {
    let mut unit = String::new();

    let mut per_token = None;
    let name_tokens: Vec<&str> = name.splitn(2, "_per_").collect();

    if name_tokens.len() > 1 {
        per_token = Some(name_tokens[1]);
    }

    let name_tokens: Vec<&str> = name_tokens[0].split('_').collect();
    let tokens_len = name_tokens.len();
    let name_tokens = remove_type_suffixes(name_tokens, is_counter);

    if tokens_len > name_tokens.len() {
        unit.push_str(&String::from("total".to_owned()));
    }

    let tokens_len = name_tokens.len();

    if WORD_TO_UCUM.get(name_tokens[tokens_len - 1]).is_some()
        || WORD_TO_UCUM_INVERT
            .get(name_tokens[tokens_len - 1])
            .is_some()
    {
        if !unit.is_empty() {
            unit = String::from("_".to_owned() + unit.as_str());
        }

        unit = String::from(name_tokens[tokens_len - 1].to_owned() + unit.as_str());

        if PER_WORD_TO_UCUM.get(per_token.unwrap_or("")).is_some()
            || PER_WORD_TO_UCUM_INVERT
                .get(per_token.unwrap_or(""))
                .is_some()
        {
            unit.push_str(&String::from("_per_".to_owned() + per_token.unwrap()));
        }
    }

    unit.to_string()
}

// UnitWordToUCUM converts english unit words to UCUM units:
// https://ucum.org/ucum#section-Alphabetic-Index-By-Symbol
// It also handles rates, such as meters_per_second, by translating the first
// word to UCUM, and the "per" word to UCUM. It joins them with a "/" between.
fn unit_word_to_ucum(unit: &str, is_counter: bool) -> String {
    let unit_tokens: Vec<&str> = unit.splitn(2, "_per_").collect();

    if unit_tokens.is_empty() {
        return "".to_string();
    }

    let mut per_token = String::new();
    let mut ucum_token = String::new();

    if unit_tokens.len() > 1 && !unit_tokens[1].is_empty() {
        per_token = String::from("/".to_owned() + per_word_to_ucum_or_default(unit_tokens[1]))
    }

    let ucum_tokens: Vec<&str> = unit_tokens[0].split('_').collect();
    let ucum_tokens = remove_type_suffixes(ucum_tokens, is_counter);

    if !ucum_tokens.is_empty() {
        ucum_token = word_to_ucum_or_default(ucum_tokens[0]);
    }

    ucum_token + &per_token
}

// wordToUCUMOrDefault retrieves the Prometheus "basic" unit corresponding to
// the specified "basic" unit. Returns the specified unit if not found in
// wordToUCUM.
fn word_to_ucum_or_default(unit_word: &str) -> String {
    if let Some(prom_unit) = WORD_TO_UCUM.get(unit_word) {
        return prom_unit.to_owned().to_string();
    } else if let Some(prom_unit) = WORD_TO_UCUM_INVERT.get(unit_word) {
        return prom_unit.to_owned().to_string();
    }

    unit_word.to_owned()
}

// perWordToUCUMOrDefault retrieve the Prometheus "per" unit corresponding to
// the specified "per" unit. Returns the specified unit if not found in perWordToUCUM.
fn per_word_to_ucum_or_default(per_unit: &str) -> &str {
    if let Some(prom_per_unit) = PER_WORD_TO_UCUM.get(per_unit) {
        return prom_per_unit;
    } else if let Some(prom_per_unit) = PER_WORD_TO_UCUM_INVERT.get(per_unit) {
        return prom_per_unit;
    }

    per_unit
}

fn remove_type_suffixes<'a>(tokens: Vec<&'a str>, is_counter: bool) -> Vec<&'a str> {
    if is_counter {
        // Only counters/sum are expected to have a type suffix at this point.
        // for other types, suffixes are removed during scrape.
        return remove_suffix(tokens, "total");
    }

    tokens
}

fn remove_suffix<'a>(mut tokens: Vec<&'a str>, suffix: &'a str) -> Vec<&'a str> {
    let len = tokens.len();

    if tokens[len - 1] == suffix {
        tokens.truncate(len - 1);
    }

    tokens
}

fn sanitize_name(name: &str, unit_word: &str) -> String {
    name.chars().take(name.len() - unit_word.len()).collect()
}

#[derive(Debug)]
pub struct OpentelemetryMetricsModel {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub temporality: Option<Temporality>,
    pub is_monotonic: Option<bool>,
    pub data_point: OpentelemetryDataPoint,
    pub scope: OpentelemetryScope,
    pub resource: OpentelemetryResource,
}

impl OpentelemetryModelMatch for OpentelemetryMetricsModel {
    fn maybe_match(event: &Event) -> Option<OpentelemetryModelType> {
        if let Some(metric) = event.clone().try_into_metric() {
            match metric.value() {
                // Skip Summary and Sketch metrics
                MetricValue::Distribution {
                    statistic: StatisticKind::Summary,
                    ..
                }
                | MetricValue::AggregatedSummary { .. }
                | MetricValue::Sketch { .. } => {
                    return None;
                }
                _ => {
                    let arbitrary = metric.arbitrary_value().value();
                    let user_metadata = get_object(arbitrary, log_schema().user_metadata_key());
                    let resource = get_object(&user_metadata, "resource");
                    let partitioner_key: OpentelemetrySpanId = resource.get("uniq_id").into();

                    return Some(OpentelemetryModelType::Metrics {
                        partitioner_key: partitioner_key.into(),
                    });
                }
            }
        }

        None
    }
}

impl TryFrom<(Event, &OpentelemetryMetricConfig)> for OpentelemetryMetricsModel {
    type Error = OpentelemetrySinkError;

    fn try_from(
        input: (Event, &OpentelemetryMetricConfig), /*, metric_config: OpentelemetryMetricConfig*/
    ) -> Result<Self, Self::Error> {
        // https://github.com/open-telemetry/opentelemetry-rust/blob/936c46639aa1521bf49dbffba49bbd9795f8ea58/opentelemetry-sdk/src/metrics/data/mod.rs#L15

        let (buf_event, metric_config) = input;

        let metric = buf_event.as_metric();
        let arbitrary = metric.arbitrary_value().value();

        let resource = OpentelemetryResource::from(metric);
        let scope = OpentelemetryScope::from(metric);
        let data_point: OpentelemetryDataPoint =
            OpentelemetryDataPoint::try_from((metric, metric_config))?;

        let mut temporality =
            if let Some(Value::Integer(val)) = arbitrary.get("aggregation_temporality") {
                match val {
                    1 => Some(Temporality::Delta),
                    2 => Some(Temporality::Cumulative),
                    _ => Some(Temporality::Delta),
                }
            } else {
                None
            };

        let mut is_monotonic = if let Some(Value::Boolean(val)) = arbitrary.get("is_monotonic") {
            Some(*val)
        } else {
            None
        };

        match data_point {
            OpentelemetryDataPoint::Sum(..) => {
                if is_monotonic.is_none() {
                    is_monotonic = match metric.kind() {
                        MetricKind::Incremental => Some(true),
                        MetricKind::Absolute => Some(false),
                    };
                }

                if temporality.is_none() {
                    temporality = Some(Temporality::Cumulative);
                }
            }
            OpentelemetryDataPoint::Histogram(..) => {
                if temporality.is_none() {
                    temporality = Some(Temporality::Cumulative);
                }
            }
            OpentelemetryDataPoint::Gauge(..) => {}
        };

        let mut unit_word = String::new();
        let is_counter = is_monotonic.unwrap_or(false);

        let name = if let Some(Value::Bytes(bytes)) = arbitrary.get("name") {
            String::from_utf8_lossy(bytes).into_owned()
        } else {
            unit_word = get_word_unit_from_name(metric.name(), is_counter);
            sanitize_name(metric.name(), unit_word.as_str())
        };

        let unit = if let Some(Value::Bytes(bytes)) = arbitrary.get("unit") {
            String::from_utf8_lossy(bytes).into_owned()
        } else {
            unit_word_to_ucum(unit_word.as_str(), is_counter)
        };

        let description = get_string_or_defailt(arbitrary, "description");

        Ok(Self {
            name: name.to_string(),
            description,
            unit: unit.to_string(),
            temporality,
            is_monotonic,
            data_point,
            scope,
            resource,
        })
    }
}

#[derive(Debug, Clone)]
pub enum OpentelemetryDataPoint {
    Gauge(DataPoint<f64>),
    Sum(DataPoint<f64>),
    Histogram(HistogramDataPoint<f64>),
}

impl From<&OpentelemetryDataPoint> for String {
    fn from(data_point: &OpentelemetryDataPoint) -> Self {
        match data_point {
            OpentelemetryDataPoint::Gauge(..) => "gauge".into(),
            OpentelemetryDataPoint::Sum(..) => "sum".into(),
            OpentelemetryDataPoint::Histogram(..) => "histogram".into(),
        }
    }
}

impl TryFrom<(&MezmoMetric, &OpentelemetryMetricConfig)> for OpentelemetryDataPoint {
    type Error = OpentelemetrySinkError;

    fn try_from(input: (&MezmoMetric, &OpentelemetryMetricConfig)) -> Result<Self, Self::Error> {
        let (metric, metric_config) = input;

        let arbitrary = metric.arbitrary_value().value();
        let user_metadata = get_object(arbitrary, log_schema().user_metadata_key());
        // If an event is an aggregation result it shouldn't be considered as OTLP event
        let is_otlp_event =
            get_string_or_defailt(&user_metadata, "data_provider").as_str() == "otlp";
        let original_type = get_string_or_defailt(&user_metadata, "original_type");

        let start_time = match arbitrary.get("start_time_unix") {
            Some(time) => Some(value_to_system_time(time)),
            None => Some(value_to_system_time(&Value::Timestamp(
                metric.timestamp().unwrap(),
            ))),
        };

        let time = match arbitrary.get("time_unix") {
            Some(time) => Some(value_to_system_time(time)),
            None => Some(value_to_system_time(&Value::Timestamp(
                metric.timestamp().unwrap(),
            ))),
        };

        let attributes = get_object(&user_metadata, "attributes")
            .iter()
            .map(|(key, value)| KeyValue::new(key.to_string(), value_to_otlp_value(value.clone())))
            .collect::<Vec<KeyValue>>();

        let mut exemplars: Vec<Exemplar<f64>> = Vec::new();

        if let Some(Value::Array(exemplars_set)) = arbitrary.get("exemplars") {
            for exemplar_obj in exemplars_set {
                if let Value::Object(exemplar) = exemplar_obj {
                    let filtered_attributes = get_object(exemplar, "filtered_attributes")
                        .iter()
                        .map(|(key, value)| {
                            KeyValue::new(key.to_string(), value_to_otlp_value(value.clone()))
                        })
                        .collect::<Vec<KeyValue>>();

                    let time = match exemplar.get("time_unix") {
                        Some(time) => value_to_system_time(time),
                        None => {
                            value_to_system_time(&Value::Timestamp(metric.timestamp().unwrap()))
                        }
                    };

                    let value = get_float(exemplar, "value")?;

                    let trace_id: OpentelemetryTraceId = exemplar.get("trace_id").into();
                    let span_id: OpentelemetrySpanId = exemplar.get("span_id").into();

                    exemplars.push(Exemplar {
                        filtered_attributes,
                        time,
                        value,
                        span_id: span_id.into(),
                        trace_id: trace_id.into(),
                    });
                }
            }
        }

        let exemplars = exemplars;

        let min = if let Some(Value::Float(min)) = arbitrary.get("min") {
            Some(min.into_inner())
        } else {
            None
        };

        let max = if let Some(Value::Float(max)) = arbitrary.get("max") {
            Some(max.into_inner())
        } else {
            None
        };

        match metric.value() {
            MetricValue::Gauge { value } => {
                if is_otlp_event && original_type == "sum" {
                    // OTLP source converts non monotonic Sum into Absolute Gauge.
                    // It has to be converted back into a non monotonic Sum here.
                    Ok(Self::Sum(DataPoint {
                        attributes: attributes.as_slice().into(),
                        start_time,
                        time,
                        value: *value,
                        exemplars,
                    }))
                } else {
                    Ok(Self::Gauge(DataPoint {
                        attributes: attributes.as_slice().into(),
                        start_time,
                        time,
                        value: *value,
                        exemplars,
                    }))
                }
            }
            MetricValue::Counter { value } => {
                // OTLP source converts monotonic Sum into Incremental Counter
                // It does not meter if it's OTLP event or not in any case
                // it's gonna be a monotonic Sum metric.
                Ok(Self::Sum(DataPoint {
                    attributes: attributes.as_slice().into(),
                    start_time,
                    time,
                    value: *value,
                    exemplars,
                }))
            }
            MetricValue::Set { values } => {
                // There is no OTLP Set metric.
                // It has to be converted into Absolute Gauge
                // Value has to be a lenght of the values array.
                Ok(Self::Gauge(DataPoint {
                    attributes: attributes.as_slice().into(),
                    start_time,
                    time,
                    value: values.len() as f64,
                    exemplars,
                }))
            }
            MetricValue::Distribution {
                statistic: StatisticKind::Histogram,
                samples,
            } => {
                // convert distributions into aggregated histograms
                let (buckets, count, sum) = samples_to_buckets(samples, &metric_config.buckets);

                let mut bounds: Vec<f64> = vec![];
                let mut bucket_counts: Vec<u64> = vec![];

                for bucket in buckets {
                    bounds.push(bucket.upper_limit);
                    bucket_counts.push(bucket.count);
                }

                bounds.push(0 as f64);

                Ok(Self::Histogram(HistogramDataPoint {
                    attributes: attributes.as_slice().into(),
                    start_time: start_time.unwrap(),
                    time: time.unwrap(),
                    count,
                    bounds,
                    bucket_counts,
                    sum,
                    min,
                    max,
                    exemplars,
                }))
            }
            MetricValue::AggregatedHistogram {
                buckets,
                count,
                sum,
            } => {
                let mut bounds: Vec<f64> = vec![];
                let mut bucket_counts: Vec<u64> = vec![];

                if is_otlp_event && original_type == "histogram" {
                    let parsed_bounds: Result<Vec<f64>, Self::Error> =
                        get_property(arbitrary, "explicit_bounds")?
                            .as_array()
                            .ok_or_else(|| {
                                Self::Error::new(
                                    "FieldInvalidType field: explicit_bounds is not an array",
                                )
                            })?
                            .iter()
                            .map(|value| parse_float(value, "explicit_bounds"))
                            .collect();
                    bounds = parsed_bounds?;

                    let parsed_bucket_counts: Result<Vec<u64>, Self::Error> =
                        get_property(arbitrary, "bucket_counts")?
                            .as_array()
                            .ok_or_else(|| Self::Error::new("FieldInvalidType"))?
                            .iter()
                            .map(|value| parse_u64(value, "bucket_counts"))
                            .collect();
                    bucket_counts = parsed_bucket_counts?;
                } else {
                    for bucket in buckets {
                        bounds.push(bucket.upper_limit);
                        bucket_counts.push(bucket.count);
                    }

                    bounds.push(0 as f64);
                }

                Ok(Self::Histogram(HistogramDataPoint {
                    attributes: attributes.as_slice().into(),
                    start_time: start_time.unwrap(),
                    time: time.unwrap(),
                    count: *count,
                    bounds,
                    bucket_counts,
                    sum: *sum,
                    min,
                    max,
                    exemplars,
                }))
            }
            MetricValue::Distribution {
                statistic: StatisticKind::Summary,
                ..
            } => Err(Self::Error::new(
                "Summary metric type does is not supported",
            )),
            MetricValue::AggregatedSummary { .. } => Err(Self::Error::new(
                "Summary metric type does is not supported",
            )),
            MetricValue::Sketch { .. } => {
                Err(Self::Error::new("Sketch metric type does is not supported"))
            }
        }
    }
}

#[derive(Debug)]
pub struct OpentelemetryResourceMetrics(pub ResourceMetrics);

impl TryFrom<Vec<OpentelemetryMetricsModel>> for OpentelemetryResourceMetrics {
    type Error = OpentelemetrySinkError;

    fn try_from(models: Vec<OpentelemetryMetricsModel>) -> Result<Self, Self::Error> {
        let mut scope_metrics = Vec::new();

        if models.is_empty() {
            return Ok(Self(ResourceMetrics {
                resource: Resource::empty(),
                scope_metrics,
            }));
        }

        let mut scope_group = OpentelemetryScopeGroup::new(models[0].resource.clone());

        for model in &models {
            scope_group.insert(model);
        }

        for scope_metric in scope_group.scopes.values() {
            let scope = &scope_metric.scope;

            let mut metrics: Vec<Metric> = vec![];

            for metric in scope_metric.metrics.values() {
                let data: Box<dyn Aggregation> = match metric.kind.as_str() {
                    "gauge" => Box::new(Gauge {
                        data_points: metric
                            .data_points
                            .iter()
                            .filter_map(|data_point| match data_point {
                                OpentelemetryDataPoint::Gauge(v) => Some(v.clone()),
                                _ => None,
                            })
                            .collect::<Vec<DataPoint<f64>>>(),
                    }),
                    "sum" => Box::new(Sum {
                        data_points: metric
                            .data_points
                            .iter()
                            .filter_map(|data_point| match data_point {
                                OpentelemetryDataPoint::Sum(v) => Some(v.clone()),
                                _ => None,
                            })
                            .collect::<Vec<DataPoint<f64>>>(),
                        temporality: metric.temporality.unwrap_or(Temporality::Cumulative),
                        is_monotonic: metric.is_monotonic.unwrap_or(true),
                    }),
                    "histogram" => Box::new(Histogram {
                        data_points: metric
                            .data_points
                            .iter()
                            .filter_map(|data_point| match data_point {
                                OpentelemetryDataPoint::Histogram(v) => Some(v.clone()),
                                _ => None,
                            })
                            .collect::<Vec<HistogramDataPoint<f64>>>(),
                        // Maybe we should set Temporality::Delta by default
                        temporality: metric.temporality.unwrap_or(Temporality::Cumulative),
                    }),
                    &_ => todo!(),
                };

                metrics.push(Metric {
                    name: metric.name.clone().into(),
                    description: metric.description.clone().into(),
                    unit: Unit::new(Cow::from(metric.unit.clone())),
                    data,
                });
            }

            scope_metrics.push(ScopeMetrics {
                scope: (*scope).clone().into(),
                metrics,
            });
        }

        Ok(Self(ResourceMetrics {
            resource: scope_group.resource.into(),
            scope_metrics,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct OpentelemetryScopeGroup {
    pub resource: OpentelemetryResource,
    pub scopes: HashMap<Value, OpentelemetryMetricGroup>,
}

impl OpentelemetryScopeGroup {
    pub fn new(resource: OpentelemetryResource) -> Self {
        Self {
            resource,
            scopes: HashMap::new(),
        }
    }

    pub fn build_key(&mut self, scope: &OpentelemetryScope) -> Value {
        Value::Object(BTreeMap::from([
            ("name".into(), scope.name.clone().into_owned().into()),
            (
                "version".into(),
                match &scope.version {
                    Some(version) => version.clone().into_owned().into(),
                    None => Value::Null,
                },
            ),
        ]))
    }

    pub fn insert(&mut self, metric: &OpentelemetryMetricsModel) -> &mut Self {
        let key = self.build_key(&metric.scope);

        if !self.scopes.contains_key(&key) {
            self.scopes.insert(
                key.clone(),
                OpentelemetryMetricGroup::new(metric.scope.clone()),
            );
        }

        self.scopes.get_mut(&key).unwrap().insert(metric);
        self
    }
}

#[derive(Debug, Clone)]
pub struct OpentelemetryMetricGroup {
    pub scope: OpentelemetryScope,
    pub metrics: HashMap<Value, OpentelemetryDataPointGroup>,
}

impl OpentelemetryMetricGroup {
    pub fn new(scope: OpentelemetryScope) -> Self {
        Self {
            scope,
            metrics: HashMap::new(),
        }
    }

    pub fn build_key(&mut self, metric: &OpentelemetryMetricsModel) -> Value {
        Value::Object(BTreeMap::from([
            (
                "kind".into(),
                String::from(&metric.data_point).as_str().into(),
            ),
            ("name".into(), metric.name.as_str().into()),
            (
                "temporality".into(),
                match metric.temporality {
                    Some(value) => match value {
                        Temporality::Delta => 1.into(),
                        Temporality::Cumulative => 2.into(),
                        _ => Value::Null,
                    },
                    None => Value::Null,
                },
            ),
            (
                "is_monotonic".into(),
                match metric.is_monotonic {
                    Some(value) => value.into(),
                    None => Value::Null,
                },
            ),
        ]))
    }

    pub fn insert(&mut self, metric: &OpentelemetryMetricsModel) -> &mut Self {
        let key = self.build_key(metric);

        if !self.metrics.contains_key(&key) {
            self.metrics
                .insert(key.clone(), OpentelemetryDataPointGroup::new(metric));
        }

        self.metrics
            .get_mut(&key)
            .unwrap()
            .insert(&metric.data_point);
        self
    }
}

#[derive(Debug, Clone)]
pub struct OpentelemetryDataPointGroup {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub unit: String,
    pub temporality: Option<Temporality>,
    pub is_monotonic: Option<bool>,
    pub data_points: Vec<OpentelemetryDataPoint>,
}

impl OpentelemetryDataPointGroup {
    pub fn new(metric: &OpentelemetryMetricsModel) -> Self {
        Self {
            kind: String::from(&metric.data_point),
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            temporality: metric.temporality,
            is_monotonic: metric.is_monotonic,
            data_points: vec![],
        }
    }

    pub fn insert(&mut self, data_point: &OpentelemetryDataPoint) -> &mut Self {
        self.data_points.push(data_point.clone());
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::Value;
    use crate::sinks::mezmo_opentelemetry::default_histogram_buckets;
    use chrono::DateTime;
    use std::collections::BTreeMap;
    use std::time::SystemTime;
    use vector_lib::config::log_schema;
    use vector_lib::event::metric::mezmo::{from_f64_or_zero, to_metric};
    use vector_lib::event::{Event, LogEvent};
    use vector_lib::lookup::PathPrefix;

    #[derive(Debug, Clone)]
    enum TestMetricGenerator {
        Counter,
        Gauge,
        Set,
        DistributionHistogram,
        DistributionSummary,
        AggregatedHistogram,
        AggregatedSummary,
    }

    impl TestMetricGenerator {
        pub fn generate_value(
            &self,
            index: usize,
            otlp_event: bool,
            original_otlp_type: Option<&str>,
            resource_uniq_id: Option<[u8; 8]>,
        ) -> BTreeMap<KeyString, Value> {
            let trace_id = Value::from(faster_hex::hex_string(&[
                95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
            ]));
            let span_id = Value::from(faster_hex::hex_string(&[
                76, 114, 27, 243, 62, 60, 175, 143,
            ]));

            if otlp_event {
                original_otlp_type.expect("original_otlp_type is not specified");
                resource_uniq_id.expect("resource_uniq_id is not specified");
            }

            let mut value = match self {
                Self::Gauge => {
                    let mut value = btreemap! {
                        KeyString::from("type") => Value::from("gauge"),
                        KeyString::from("value") => Value::from(index as f64 * 11.0),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            KeyString::from("name") => Value::from("system.filesystem.usage"),
                            KeyString::from("description") => Value::from("test_description"),
                            KeyString::from("unit") => Value::from("GiBy/s"),
                            KeyString::from("exemplars") => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    KeyString::from("filtered_attributes") => btreemap! {"foo" => Value::from("bar")},
                                    KeyString::from("span_id") => span_id,
                                    KeyString::from("trace_id") => trace_id,
                                    KeyString::from("time_unix") => Value::from(
                                        DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp")
                                    ),
                                    KeyString::from("value") => Value::Integer(10),
                                }

                            )])),
                            KeyString::from("flags") => Value::Integer(1),
                            KeyString::from("start_time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                        };

                        if original_otlp_type.unwrap() == "sum" {
                            arbitrary.insert("aggregation_temporality".into(), Value::Integer(2));
                            arbitrary.insert("is_monotonic".into(), Value::Boolean(false));
                        }

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::Counter => {
                    let mut value = btreemap! {
                        KeyString::from("type") => Value::from("counter"),
                        KeyString::from("value") => Value::from(index as f64 * 11.0),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            KeyString::from("name") => Value::from("system.filesystem.usage"),
                            KeyString::from("description") => Value::from("test_description"),
                            KeyString::from("unit") => Value::from("GiBy/s"),
                            KeyString::from("exemplars") => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    KeyString::from("filtered_attributes") => btreemap! {KeyString::from("foo") => Value::from("bar")},
                                    KeyString::from("span_id") => span_id,
                                    KeyString::from("trace_id") => trace_id,
                                    KeyString::from("time_unix") => Value::from(
                                        DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp")
                                    ),
                                    KeyString::from("value") => Value::Integer(10),
                                }

                            )])),
                            KeyString::from("flags") => Value::Integer(1),
                            KeyString::from("start_time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("aggregation_temporality") => Value::Integer(2),
                            KeyString::from("is_monotonic") => Value::Boolean(true),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::Set => {
                    let mut values = vec![];
                    for value in 0..(index + 1) {
                        values.push(Value::from(String::from((value as i32).to_string())));
                    }
                    btreemap! {
                        KeyString::from("type") => Value::from("set"),
                        KeyString::from("value") => Value::Object(btreemap! {
                            KeyString::from("values") => Value::from(values),
                        }),
                    }
                }
                Self::AggregatedHistogram => {
                    let mut value = btreemap! {
                        KeyString::from("type") => Value::from("histogram"),
                        KeyString::from("value") => Value::Object(btreemap! {
                            KeyString::from("count") => Value::Integer(10),
                            KeyString::from("sum") => from_f64_or_zero(3.7),
                            KeyString::from("buckets") => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.005,
                                    KeyString::from("count") => 214,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.01,
                                    KeyString::from("count") => 6,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.025,
                                    KeyString::from("count") => 1,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.05,
                                    KeyString::from("count") => 1,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.075,
                                    KeyString::from("count") => 2,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.1,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.25,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.5,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 0.75,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 1.0,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 2.5,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 5.0,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 7.5,
                                    KeyString::from("count") => 0,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("upper_limit") => 10.0,
                                    KeyString::from("count") => 0,
                                }),
                            ]))
                        }),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            KeyString::from("name") => Value::from("system.filesystem.usage"),
                            KeyString::from("description") => Value::from("test_description"),
                            KeyString::from("unit") => Value::from("GiBy/s"),
                            KeyString::from("exemplars") => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    KeyString::from("filtered_attributes") => btreemap! {KeyString::from("foo") => Value::from("bar")},
                                    KeyString::from("span_id") => span_id,
                                    KeyString::from("trace_id") => trace_id,
                                    KeyString::from("time_unix") => Value::from(
                                        DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp")
                                    ),
                                    KeyString::from("value") => Value::Integer(10),
                                }

                            )])),
                            KeyString::from("flags") => Value::Integer(1),
                            KeyString::from("bucket_counts") => Value::Array(Vec::from([
                                Value::Integer(214),
                                Value::Integer(6),
                                Value::Integer(1),
                                Value::Integer(1),
                                Value::Integer(2),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                                Value::Integer(0),
                            ])),

                            KeyString::from("explicit_bounds") => Value::Array(Vec::from([
                                from_f64_or_zero(0.005),
                                from_f64_or_zero(0.01),
                                from_f64_or_zero(0.025),
                                from_f64_or_zero(0.05),
                                from_f64_or_zero(0.075),
                                from_f64_or_zero(0.1),
                                from_f64_or_zero(0.25),
                                from_f64_or_zero(0.5),
                                from_f64_or_zero(0.75),
                                from_f64_or_zero(1.0),
                                from_f64_or_zero(2.5),
                                from_f64_or_zero(5.0),
                                from_f64_or_zero(7.5),
                                from_f64_or_zero(10.0),
                            ])),
                            KeyString::from("max") => from_f64_or_zero(9.9),
                            KeyString::from("min") => from_f64_or_zero(0.1),
                            KeyString::from("start_time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("aggregation_temporality") => Value::Integer(2),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::DistributionHistogram => {
                    btreemap! {
                        KeyString::from("type") => Value::from("distribution"),
                        KeyString::from("value") => Value::Object(btreemap! {
                            KeyString::from("samples") => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 1.0,
                                    KeyString::from("rate") => 3,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 2.0,
                                    KeyString::from("rate") => 3,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 3.0,
                                    KeyString::from("rate") => 2,
                                })
                            ])),
                            KeyString::from("statistic") => Value::from("histogram"),
                        })
                    }
                }
                Self::AggregatedSummary => {
                    let mut value = btreemap! {
                        KeyString::from("type") => Value::from("summary"),
                        KeyString::from("value") => Value::Object(btreemap! {
                            KeyString::from("count") => Value::Integer(10),
                            KeyString::from("sum") => from_f64_or_zero(3.7),
                            KeyString::from("quantiles") => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    KeyString::from("quantile") => 0.005,
                                    KeyString::from("value") => 10,
                                })
                            ]))
                        }),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            KeyString::from("name") => Value::from("system.filesystem.usage"),
                            KeyString::from("description") => Value::from("test_description"),
                            KeyString::from("unit") => Value::from("GiBy/s"),
                            KeyString::from("count") => Value::Integer(10),
                            KeyString::from("sum") => from_f64_or_zero(10.0),
                            KeyString::from("flags") => Value::Integer(1),
                            KeyString::from("quantile_values") => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    KeyString::from("quantile") => 0.005,
                                    KeyString::from("value") => 10,
                                })
                            ])),
                            KeyString::from("start_time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                            KeyString::from("time_unix") => Value::from(
                                DateTime::from_timestamp(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp")
                            ),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::DistributionSummary => {
                    btreemap! {
                        KeyString::from("type") => Value::from("distribution"),
                        KeyString::from("value") => Value::Object(btreemap! {
                            KeyString::from("samples") => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 1.0,
                                    KeyString::from("rate") => 3,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 2.0,
                                    KeyString::from("rate") => 3,
                                }),
                                Value::Object(btreemap! {
                                    KeyString::from("value") => 3.0,
                                    KeyString::from("rate") => 2,
                                })
                            ])),
                            KeyString::from("statistic") => Value::from("summary"),
                        })
                    }
                }
            };

            if otlp_event {
                value.insert(
                    log_schema().user_metadata_key().into(),
                    Value::Object(btreemap! {
                        KeyString::from("original_type") => Value::from(original_otlp_type.unwrap()),
                        KeyString::from("data_provider") => Value::from("otlp"),
                        KeyString::from("resource") => Value::Object(btreemap! {
                            KeyString::from("attributes") => btreemap! {KeyString::from("foo") => Value::from("bar")},
                            KeyString::from("dropped_attributes_count") => Value::Integer(1),
                            KeyString::from("uniq_id") => Value::from(faster_hex::hex_string(&resource_uniq_id.unwrap())),
                        }),
                        KeyString::from("scope") => Value::Object(btreemap! {
                            KeyString::from("attributes") => btreemap! {KeyString::from("foo") => Value::from("bar")},
                            KeyString::from("dropped_attributes_count") => Value::Integer(1),
                            KeyString::from("name") => Value::from("test_name"),
                            KeyString::from("version") => Value::Null,
                        }),
                        KeyString::from("attributes") => btreemap! {"foo" => Value::from("bar")},
                    })
                );
            }

            value
        }
    }

    fn metric_event_generator(
        index: usize,
        metric_type: TestMetricGenerator,
        metric_kind: &str,
        otlp_event: bool,
        original_otlp_type: Option<&str>,
        resource_uniq_id: Option<[u8; 8]>,
    ) -> Event {
        let mut event_data = BTreeMap::<KeyString, Value>::new();
        let value =
            metric_type.generate_value(index, otlp_event, original_otlp_type, resource_uniq_id);

        event_data.insert(
            "message".into(),
            Value::Object(btreemap! {
                KeyString::from("kind") => Value::from(metric_kind),
                KeyString::from("name") => Value::from("system_filesystem_usage_gibibytes_per_second"),
                KeyString::from("tags") => btreemap! {KeyString::from("foo") => "bar"},
                KeyString::from("value") => Value::from(value.clone())
            }),
        );

        let mut log_event = LogEvent::from(event_data);

        if let Some(user_metadata) = value.get(log_schema().user_metadata_key()) {
            log_event.insert(
                (PathPrefix::Event, log_schema().user_metadata_key()),
                user_metadata.clone(),
            );
        }

        Event::Log(log_event)
    }

    pub fn generate_events<Gen: FnMut(usize) -> Event>(generator: Gen, count: usize) -> Vec<Event> {
        (0..count).map(generator).collect::<Vec<_>>()
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_gauge_events() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let gen_settings = vec![
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("gauge"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("gauge"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("gauge"),
                Some(uniq_id),
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        let expected_time =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(1_579_134_612_000_000_011);
        let expected_trace_id: [u8; 16] = [
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ];
        let expected_span_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(gauge) = metric.data.as_any().downcast_ref::<Gauge<f64>>() {
                    assert_eq!(gauge.data_points.len(), 3);

                    for (i, data_point) in gauge.data_points.iter().enumerate() {
                        assert_eq!(data_point.time.unwrap(), expected_time);
                        assert_eq!(data_point.start_time.unwrap(), expected_time);
                        assert_eq!(data_point.value, { i as f64 * 11.0 });

                        assert_eq!(data_point.exemplars.len(), 1);
                        assert_eq!(data_point.exemplars[0].value, 10.0_f64);
                        assert_eq!(data_point.exemplars[0].time, expected_time);
                        assert_eq!(data_point.exemplars[0].trace_id, expected_trace_id);
                        assert_eq!(data_point.exemplars[0].span_id, expected_span_id);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_sum_not_monotonic_events() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let gen_settings = vec![
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        let expected_time =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(1_579_134_612_000_000_011);
        let expected_trace_id: [u8; 16] = [
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ];
        let expected_span_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(sum) = metric.data.as_any().downcast_ref::<Sum<f64>>() {
                    assert_eq!(sum.data_points.len(), 3);
                    assert!(!sum.is_monotonic);
                    assert_eq!(sum.temporality, Temporality::Cumulative);

                    for (i, data_point) in sum.data_points.iter().enumerate() {
                        assert_eq!(data_point.time.unwrap(), expected_time);
                        assert_eq!(data_point.start_time.unwrap(), expected_time);
                        assert_eq!(data_point.value, { i as f64 * 11.0 });

                        assert_eq!(data_point.exemplars.len(), 1);
                        assert_eq!(data_point.exemplars[0].value, 10.0_f64);
                        assert_eq!(data_point.exemplars[0].time, expected_time);
                        assert_eq!(data_point.exemplars[0].trace_id, expected_trace_id);
                        assert_eq!(data_point.exemplars[0].span_id, expected_span_id);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_sum_monotonic_events() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let gen_settings = vec![
            (
                &TestMetricGenerator::Counter,
                "incremental",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Counter,
                "incremental",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Counter,
                "incremental",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        let expected_time =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(1_579_134_612_000_000_011);
        let expected_trace_id: [u8; 16] = [
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ];
        let expected_span_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(sum) = metric.data.as_any().downcast_ref::<Sum<f64>>() {
                    assert_eq!(sum.data_points.len(), 3);
                    assert!(sum.is_monotonic);
                    assert_eq!(sum.temporality, Temporality::Cumulative);

                    for (i, data_point) in sum.data_points.iter().enumerate() {
                        assert_eq!(data_point.time.unwrap(), expected_time);
                        assert_eq!(data_point.start_time.unwrap(), expected_time);
                        assert_eq!(data_point.value, { i as f64 * 11.0 });

                        assert_eq!(data_point.exemplars.len(), 1);
                        assert_eq!(data_point.exemplars[0].value, 10.0_f64);
                        assert_eq!(data_point.exemplars[0].time, expected_time);
                        assert_eq!(data_point.exemplars[0].trace_id, expected_trace_id);
                        assert_eq!(data_point.exemplars[0].span_id, expected_span_id);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_set_events() {
        let gen_settings = vec![
            (&TestMetricGenerator::Set, "absolute", false, None, None),
            (&TestMetricGenerator::Set, "absolute", false, None, None),
            (&TestMetricGenerator::Set, "absolute", false, None, None),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(gauge) = metric.data.as_any().downcast_ref::<Gauge<f64>>() {
                    assert_eq!(gauge.data_points.len(), 3);

                    for (i, data_point) in gauge.data_points.iter().enumerate() {
                        assert!(data_point.time.is_some());
                        assert!(data_point.start_time.is_some());
                        assert_eq!(data_point.value, i as f64 + 1.0);

                        assert_eq!(data_point.exemplars.len(), 0);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_aggregated_histogram_events() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let gen_settings = vec![
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                true,
                Some("histogram"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                true,
                Some("histogram"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                true,
                Some("histogram"),
                Some(uniq_id),
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        let expected_time =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(1_579_134_612_000_000_011);
        let expected_trace_id: [u8; 16] = [
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ];
        let expected_span_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(histogram) = metric.data.as_any().downcast_ref::<Histogram<f64>>() {
                    assert_eq!(histogram.data_points.len(), 3);

                    for data_point in &histogram.data_points {
                        assert_eq!(data_point.time, expected_time);
                        assert_eq!(data_point.start_time, expected_time);

                        assert_eq!(data_point.count, 10);
                        assert_eq!(data_point.min.unwrap(), 0.1_f64);
                        assert_eq!(data_point.max.unwrap(), 9.9_f64);
                        assert_eq!(data_point.sum, 3.7_f64);

                        assert_eq!(
                            data_point.bounds,
                            [
                                0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5,
                                5.0, 7.5, 10.0
                            ]
                        );
                        assert_eq!(
                            data_point.bucket_counts,
                            [214, 6, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                        );

                        assert_eq!(data_point.exemplars.len(), 1);
                        assert_eq!(data_point.exemplars[0].value, 10.0_f64);
                        assert_eq!(data_point.exemplars[0].time, expected_time);
                        assert_eq!(data_point.exemplars[0].trace_id, expected_trace_id);
                        assert_eq!(data_point.exemplars[0].span_id, expected_span_id);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_distribution_histogram_events() {
        let gen_settings = vec![
            (
                &TestMetricGenerator::DistributionHistogram,
                "absolute",
                false,
                None,
                None,
            ),
            (
                &TestMetricGenerator::DistributionHistogram,
                "absolute",
                false,
                None,
                None,
            ),
            (
                &TestMetricGenerator::DistributionHistogram,
                "absolute",
                false,
                None,
                None,
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(histogram) = metric.data.as_any().downcast_ref::<Histogram<f64>>() {
                    assert_eq!(histogram.data_points.len(), 3);

                    for data_point in &histogram.data_points {
                        assert_eq!(data_point.count, 8);
                        assert!(data_point.min.is_none());
                        assert!(data_point.max.is_none());
                        assert_eq!(data_point.sum, 15_f64);

                        assert_eq!(
                            data_point.bounds,
                            [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 0.0]
                        );
                        assert_eq!(data_point.bucket_counts, [0, 0, 0, 0, 0, 0, 0, 3, 3, 2, 0]);

                        assert_eq!(data_point.exemplars.len(), 0);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_mixed_otlp_events() {
        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
        let gen_settings = vec![
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("gauge"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("gauge"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Gauge,
                "absolute",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Counter,
                "incremental",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::Counter,
                "incremental",
                true,
                Some("sum"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                true,
                Some("histogram"),
                Some(uniq_id),
            ),
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                true,
                Some("histogram"),
                Some(uniq_id),
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(sum) = metric.data.as_any().downcast_ref::<Sum<f64>>() {
                    assert_eq!(sum.data_points.len(), 2);
                } else if let Some(gauge) = metric.data.as_any().downcast_ref::<Gauge<f64>>() {
                    assert_eq!(gauge.data_points.len(), 2);
                } else if let Some(histogram) =
                    metric.data.as_any().downcast_ref::<Histogram<f64>>()
                {
                    assert_eq!(histogram.data_points.len(), 2);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_mixed_not_otlp_events() {
        let gen_settings = vec![
            (&TestMetricGenerator::Gauge, "absolute", false, None, None),
            (
                &TestMetricGenerator::Counter,
                "incremental",
                false,
                None,
                None,
            ),
            (
                &TestMetricGenerator::AggregatedHistogram,
                "absolute",
                false,
                None,
                None,
            ),
            (
                &TestMetricGenerator::DistributionHistogram,
                "absolute",
                false,
                None,
                None,
            ),
            (&TestMetricGenerator::Set, "absolute", false, None, None),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let mut metrics: Vec<OpentelemetryMetricsModel> = vec![];
        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            match OpentelemetryMetricsModel::try_from((event.clone(), &config)) {
                Ok(m) => metrics.push(m),
                Err(err) => panic!("Metric event cannot be converted to a model: {:#?}", err),
            }
        }

        let resource_metrics = OpentelemetryResourceMetrics::try_from(metrics)
            .expect("Failed to convert metrics to OpentelemetryResourceMetrics");

        for scope_metrics in resource_metrics.0.scope_metrics {
            for metric in scope_metrics.metrics {
                if let Some(sum) = metric.data.as_any().downcast_ref::<Sum<f64>>() {
                    assert_eq!(sum.data_points.len(), 1);
                } else if let Some(gauge) = metric.data.as_any().downcast_ref::<Gauge<f64>>() {
                    assert_eq!(gauge.data_points.len(), 2);
                } else if let Some(histogram) =
                    metric.data.as_any().downcast_ref::<Histogram<f64>>()
                {
                    assert_eq!(histogram.data_points.len(), 2);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_unsupported_events() {
        let gen_settings = vec![
            (
                &TestMetricGenerator::AggregatedSummary,
                "absolute",
                false,
                None,
                None,
            ),
            (
                &TestMetricGenerator::DistributionSummary,
                "absolute",
                false,
                None,
                None,
            ),
        ];

        let generator = |idx| {
            let (metric_type, metric_kind, otlp_event, original_otlp_type, uniq_id) =
                gen_settings[idx];
            let event = metric_event_generator(
                idx,
                metric_type.clone(),
                metric_kind,
                otlp_event,
                original_otlp_type,
                uniq_id,
            );
            let log = event.as_log();
            Event::Metric(to_metric(log).expect("Failed to convert log to metric"))
        };

        let config = OpentelemetryMetricConfig {
            buckets: default_histogram_buckets(),
        };
        for event in generate_events(generator, gen_settings.len()) {
            assert!(OpentelemetryMetricsModel::try_from((event.clone(), &config)).is_err());
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_unit_word_to_ucum() {
        let unit_map = HashMap::from([
            ("", ""),
            ("days", "d"),
            ("seconds", "s"),
            ("kibibytes", "KiBy"),
            ("volts", "V"),
            ("bananas_per_day", "bananas/d"),
            ("meters_per_hour", "m/h"),
            ("ratio", "1"),
            ("percent", "%"),
            ("kibibytes_total", "KiBy"),
            ("bytes", "By"),
            ("bytes_total", "By"),
            ("", ""),
            ("total", ""),
            ("packets", "{packets}"),
            ("hertz", "Hz"),
            ("km_per_hour", "km/h"),
            ("meters_per_second", "m/s"),
            ("percent", "%"),
            ("ratio", "1"),
            ("F", "°F"),
            ("C", "°C"),
            ("celsius", "Cel"),
            ("fahrenheit", "°F"),
            ("seconds_total", "s"),
            ("requests", "{requests}"),
            ("kilobytes", "KBy"),
            ("microseconds", "us"),
        ]);

        for (input, expected) in unit_map.iter() {
            assert_eq!(unit_word_to_ucum(input, true), expected.to_string());
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_get_unit_sum_metric_type() {
        let name_map = HashMap::from([
            ("system_filesystem_usage_bytes", "bytes"),
            ("system_io_bytes_total", "bytes_total"),
            ("system_network_dropped", ""),
            ("system_network_dropped_total", "total"),
            ("system_network_dropped_packets", "packets"),
            ("hw_gpu_memory_utilization_ratio", "ratio"),
            ("hw_cpu_speed_limit_hertz", "hertz"),
            ("broken_metric_speed_km_per_hour", "km_per_hour"),
            (
                "astro_light_speed_limit_meters_per_second",
                "meters_per_second",
            ),
            ("broken_metric_success_ratio_percent", "percent"),
            ("broken_metric_success_percent", "percent"),
            ("broken_metric_success_ratio", "ratio"),
            ("unsupported_metric_temperature_F", "F"),
            ("unsupported_metric_temperature_C", "C"),
            ("unsupported_metric_temperature_celsius", "celsius"),
            ("unsupported_metric_temperature_fahrenheit", "fahrenheit"),
            ("system_disk_operation_time_seconds_total", "seconds_total"),
            ("nginx_requests", "requests"),
            ("nsxt_node_memory_usage_kilobytes", "kilobytes"),
            ("redis_latest_fork_microseconds", "microseconds"),
        ]);

        for (input, expected) in name_map.iter() {
            assert_eq!(get_word_unit_from_name(input, true), expected.to_string());
        }
    }

    #[tokio::test]
    async fn test_otlp_sink_event_to_metric_model_get_unit_non_metric_type() {
        let name_map = HashMap::from([
            ("system_filesystem_usage_bytes", "bytes"),
            ("system_io_bytes_total", ""),
            ("system_network_dropped", ""),
            ("system_network_dropped_total", ""),
            ("system_network_dropped_packets", "packets"),
            ("hw_gpu_memory_utilization_ratio", "ratio"),
            ("hw_cpu_speed_limit_hertz", "hertz"),
            ("broken_metric_speed_km_per_hour", "km_per_hour"),
            (
                "astro_light_speed_limit_meters_per_second",
                "meters_per_second",
            ),
            ("broken_metric_success_ratio_percent", "percent"),
            ("broken_metric_success_percent", "percent"),
            ("broken_metric_success_ratio", "ratio"),
            ("unsupported_metric_temperature_F", "F"),
            ("unsupported_metric_temperature_C", "C"),
            ("unsupported_metric_temperature_celsius", "celsius"),
            ("unsupported_metric_temperature_fahrenheit", "fahrenheit"),
            ("system_disk_operation_time_seconds_total", ""),
            ("nginx_requests", "requests"),
            ("nsxt_node_memory_usage_kilobytes", "kilobytes"),
            ("redis_latest_fork_microseconds", "microseconds"),
        ]);

        for (input, expected) in name_map.iter() {
            assert_eq!(get_word_unit_from_name(input, false), expected.to_string());
        }
    }
}
