use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use ilog::IntLog;
use rand::{Rng, thread_rng};
use serde::{
    Serialize,
    ser::{SerializeMap, SerializeSeq},
};

pub struct Generator {
    generators: Vec<Metric>,
    index: usize,
}

#[derive(Default)]
pub struct GeneratorBuilder {
    generators: Vec<Metric>,
}

#[macro_export]
macro_rules! metric_tags {
    ({}) => {
        BTreeMap::default()
    };

    ({$($k:literal: $v:expr),+ $(,)?}) => ({
        vec![$((String::from($k), String::from($v))),+]
            .into_iter()
            .collect::<::std::collections::BTreeMap<String, String>>()
    });
}

impl GeneratorBuilder {
    pub fn new() -> Self {
        GeneratorBuilder::default()
    }

    pub fn add_counter(
        mut self,
        name: &str,
        kind: Kind,
        params: CounterParams,
        namespace: Option<&str>,
        tag_generator: Option<Box<dyn TagGenerator + Send>>,
    ) -> Self {
        let metric = Metric {
            name: name.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            kind,
            tag_generator,
            value: Value::Counter {
                value: params.start,
                params,
            },
        };
        self.generators.push(metric);
        self
    }

    pub fn add_gauge(
        mut self,
        name: &str,
        kind: Kind,
        params: GaugeParams,
        namespace: Option<&str>,
        tag_generator: Option<Box<dyn TagGenerator + Send>>,
    ) -> Self {
        let metric = Metric {
            name: name.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            kind,
            tag_generator,
            value: Value::Gauge { value: 0, params },
        };
        self.generators.push(metric);
        self
    }

    pub fn add_histogram(
        mut self,
        name: &str,
        kind: Kind,
        params: HistogramParams,
        namespace: Option<&str>,
        tag_generator: Option<Box<dyn TagGenerator + Send>>,
    ) -> Self {
        let metric = Metric {
            name: name.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            kind,
            tag_generator,
            value: Value::Histogram {
                value: Histogram::new(&params),
                params,
            },
        };
        self.generators.push(metric);
        self
    }

    pub fn add_summary(
        mut self,
        name: &str,
        kind: Kind,
        params: SummaryParams,
        namespace: Option<&str>,
        tag_generator: Option<Box<dyn TagGenerator + Send>>,
    ) -> Self {
        let metric = Metric {
            name: name.to_string(),
            namespace: namespace.map(|s| s.to_string()),
            kind,
            tag_generator,
            value: Value::Summary {
                value: Summary::new(&params),
                params,
            },
        };
        self.generators.push(metric);
        self
    }

    pub fn build(self) -> Generator {
        if self.generators.is_empty() {
            panic!("there should be at least one metric")
        }
        Generator {
            generators: self.generators,
            index: 0,
        }
    }

    pub fn build_http() -> Generator {
        let mut hosts = vec![];
        let mut host_ops = vec![];

        for i in 1..=10 {
            let hostname = format!("host{i}.example.com");
            hosts.push(metric_tags!({ "hostname": &hostname }));
            host_ops.push(metric_tags!({"hostname": &hostname, "method": "GET", "path": "/"}));
            host_ops
                .push(metric_tags!({"hostname": &hostname, "method": "GET", "path": "/example"}));
            host_ops.push(metric_tags!({"hostname": &hostname, "method": "POST", "path": "/"}));
            host_ops
                .push(metric_tags!({"hostname": &hostname, "method": "POST", "path": "/example"}));
        }

        GeneratorBuilder::new()
            .add_counter(
                "sent_kilobytes_total",
                Kind::Incremental,
                CounterParams {
                    start: 0,
                    min_increment: 1,
                    max_increment: 1024,
                },
                Some("http.server"),
                Some(Box::new(RoundRobinTagGenerator::new(host_ops.clone()))),
            )
            .add_counter(
                "uptime_seconds",
                Kind::Incremental,
                CounterParams {
                    start: 0,
                    min_increment: 1,
                    max_increment: 1,
                },
                Some("http.server"),
                Some(Box::new(RoundRobinTagGenerator::new(hosts.clone()))),
            )
            .add_gauge(
                "cpu_load",
                Kind::Absolute,
                GaugeParams {
                    lower_bound: 1,
                    upper_bound: 16,
                },
                Some("http.server"),
                Some(Box::new(RoundRobinTagGenerator::new(hosts.clone()))),
            )
            .add_gauge(
                "memory_usage_bytes",
                Kind::Absolute,
                GaugeParams {
                    lower_bound: 1_073_741_824,
                    upper_bound: 34_359_738_368,
                },
                Some("http.server"),
                Some(Box::new(RoundRobinTagGenerator::new(hosts))),
            )
            .add_histogram(
                "request_latency_milliseconds",
                Kind::Absolute,
                HistogramParams {
                    min: 100,
                    max: 2000,
                    count: 1,
                },
                Some("http.server.request"),
                Some(Box::new(RoundRobinTagGenerator::new(host_ops.clone()))),
            )
            .add_histogram(
                "response_latency_milliseconds",
                Kind::Absolute,
                HistogramParams {
                    min: 200,
                    max: 1000,
                    count: 1,
                },
                Some("http.server.response"),
                Some(Box::new(RoundRobinTagGenerator::new(host_ops.clone()))),
            )
            .add_summary(
                "request_size_kilobytes",
                Kind::Absolute,
                SummaryParams {
                    min: 100,
                    max: 2000,
                    count: 1,
                },
                Some("http.server.request"),
                Some(Box::new(RoundRobinTagGenerator::new(host_ops.clone()))),
            )
            .add_summary(
                "response_size_kilobytes",
                Kind::Absolute,
                SummaryParams {
                    min: 100,
                    max: 200,
                    count: 1,
                },
                Some("http.server.response"),
                Some(Box::new(RoundRobinTagGenerator::new(host_ops))),
            )
            .build()
    }

    pub fn build_generic() -> Generator {
        GeneratorBuilder::new()
            .add_counter(
                "counter1",
                Kind::Incremental,
                CounterParams {
                    start: 0,
                    min_increment: 1,
                    max_increment: 1,
                },
                Some("namespace1"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 10))),
            )
            .add_counter(
                "counter2",
                Kind::Incremental,
                CounterParams {
                    start: 0,
                    min_increment: 1,
                    max_increment: 100,
                },
                Some("namespace2"),
                Some(Box::new(RandomTagGenerator::new(1, 5, 100))),
            )
            .add_counter(
                "counter3",
                Kind::Incremental,
                CounterParams {
                    start: 0,
                    min_increment: 1,
                    max_increment: 100,
                },
                Some("namespace3"),
                Some(Box::new(RandomTagGenerator::new(1, 5, 1000))),
            )
            .add_gauge(
                "gauge1",
                Kind::Absolute,
                GaugeParams {
                    lower_bound: 100,
                    upper_bound: 200,
                },
                Some("namespace1"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 10))),
            )
            .add_gauge(
                "gauge2",
                Kind::Absolute,
                GaugeParams {
                    lower_bound: 1000,
                    upper_bound: 2000,
                },
                Some("namespace2"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 100))),
            )
            .add_gauge(
                "gauge3",
                Kind::Absolute,
                GaugeParams {
                    lower_bound: 10000,
                    upper_bound: 20000,
                },
                Some("namespace3"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 1000))),
            )
            .add_histogram(
                "histogram1",
                Kind::Incremental,
                HistogramParams {
                    min: 1,
                    max: 10000,
                    count: 1,
                },
                Some("namespace1"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 10))),
            )
            .add_histogram(
                "histogram2",
                Kind::Incremental,
                HistogramParams {
                    min: 1,
                    max: 100000,
                    count: 1,
                },
                Some("namespace2"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 100))),
            )
            .add_histogram(
                "histogram3",
                Kind::Incremental,
                HistogramParams {
                    min: 1,
                    max: 1000000,
                    count: 1,
                },
                Some("namespace3"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 1000))),
            )
            .add_summary(
                "summary1",
                Kind::Incremental,
                SummaryParams {
                    min: 1,
                    max: 10000,
                    count: 1,
                },
                Some("namespace1"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 10))),
            )
            .add_summary(
                "summary2",
                Kind::Incremental,
                SummaryParams {
                    min: 1,
                    max: 100000,
                    count: 1,
                },
                Some("namespace2"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 100))),
            )
            .add_summary(
                "summary3",
                Kind::Incremental,
                SummaryParams {
                    min: 1,
                    max: 1000000,
                    count: 1,
                },
                Some("namespace3"),
                Some(Box::new(RandomTagGenerator::new(0, 5, 1000))),
            )
            .build()
    }
}

pub trait TagGenerator {
    fn update(&mut self);

    fn generate_tags(&self) -> BTreeMap<String, String>;
}

pub struct RoundRobinTagGenerator {
    index: usize,
    tags: Vec<BTreeMap<String, String>>,
}

impl RoundRobinTagGenerator {
    pub fn new(tags: Vec<BTreeMap<String, String>>) -> Self {
        RoundRobinTagGenerator { index: 0, tags }
    }
}

impl TagGenerator for RoundRobinTagGenerator {
    fn update(&mut self) {
        self.index = (self.index + 1) % self.tags.len();
    }

    fn generate_tags(&self) -> BTreeMap<String, String> {
        self.tags[self.index].clone()
    }
}

pub struct RandomTagGenerator {
    min_fields: u64,
    max_fields: u64,
    cardinality: u64,
}

impl RandomTagGenerator {
    pub fn new(min_fields: u64, max_fields: u64, cardinality: u64) -> Self {
        RandomTagGenerator {
            min_fields,
            max_fields,
            cardinality,
        }
    }
}

impl TagGenerator for RandomTagGenerator {
    fn update(&mut self) {}

    fn generate_tags(&self) -> BTreeMap<String, String> {
        let num_fields = random_in_range(self.min_fields, self.max_fields - 1);
        let mut tags = BTreeMap::new();
        for i in 0..num_fields {
            tags.insert(
                format!("field{}", i + 1),
                format!("value{}", random_in_range(0, self.cardinality) + 1),
            );
        }
        tags
    }
}

impl Generator {
    pub fn generate_next(&mut self) -> String {
        let len = self.generators.len();
        if let Some(metric) = self.generators.get_mut(self.index) {
            self.index = (self.index + 1) % len;
            let generated =
                serde_json::to_string(metric).expect("metric serialization should not fail");
            metric.update();
            generated
        } else {
            panic!("there should be at least one metric")
        }
    }
}

struct Metric {
    name: String,
    namespace: Option<String>,
    kind: Kind,
    tag_generator: Option<Box<dyn TagGenerator + Send>>,
    value: Value,
}

impl Metric {
    fn update(&mut self) {
        if let Some(tag_generator) = &mut self.tag_generator {
            tag_generator.update();
        }
        self.value.update();
    }
}

pub enum Kind {
    Incremental,
    Absolute,
}

impl Serialize for Metric {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut metric = serializer.serialize_map(Some(5))?;

        metric.serialize_entry("name", &self.name)?;

        match self.kind {
            Kind::Incremental => {
                metric.serialize_entry("kind", "incremental")?;
            }
            Kind::Absolute => {
                metric.serialize_entry("kind", "absolute")?;
            }
        }

        if let Some(namespace) = &self.namespace {
            metric.serialize_entry("namespace", namespace)?;
        }

        if let Some(tag_generator) = &self.tag_generator {
            metric.serialize_entry("tags", &tag_generator.generate_tags())?;
        }

        metric.serialize_entry("value", &self.value)?;

        metric.end()
    }
}

pub struct CounterParams {
    pub start: u64,
    pub min_increment: u64,
    pub max_increment: u64,
}

pub struct GaugeParams {
    pub lower_bound: u64,
    pub upper_bound: u64,
}

pub struct HistogramParams {
    pub min: u64,
    pub max: u64,
    pub count: u32,
}

pub struct SummaryParams {
    pub min: u64,
    pub max: u64,
    pub count: u32,
}

enum Value {
    Counter {
        value: u64,
        params: CounterParams,
    },
    Gauge {
        value: u64,
        params: GaugeParams,
    },
    Histogram {
        value: Histogram,
        params: HistogramParams,
    },
    Summary {
        value: Summary,
        params: SummaryParams,
    },
}

impl Value {
    fn update(&mut self) {
        match self {
            Value::Counter { value, params } => {
                *value += if params.min_increment == params.max_increment {
                    params.min_increment
                } else {
                    random_in_range(params.min_increment, params.max_increment)
                };
            }
            Value::Gauge { value, params } => {
                *value = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| {
                    (params.lower_bound as i64
                        + ((d.as_secs() as f64).sin()
                            * (params.upper_bound - params.lower_bound) as f64)
                            as i64)
                        .clamp(0, i64::MAX) as u64
                });
            }
            Value::Histogram { value, params } => {
                let _ = value
                    .histogram
                    .increment(random_in_range(params.min, params.max - 1), params.count);
            }
            Value::Summary { value, params } => {
                let _ = value
                    .histogram
                    .increment(random_in_range(params.min, params.max - 1), params.count);
            }
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut metric_value = serializer.serialize_map(Some(2))?;

        match self {
            Self::Counter { value, params: _ } => {
                metric_value.serialize_entry("type", "counter")?;
                metric_value.serialize_entry("value", &(*value as f64))?;
            }
            Self::Gauge { value, params: _ } => {
                metric_value.serialize_entry("type", "gauge")?;
                metric_value.serialize_entry("value", &(*value as f64))?;
            }
            Self::Histogram { value, params: _ } => {
                metric_value.serialize_entry("type", "histogram")?;
                metric_value.serialize_entry("value", value)?;
            }
            Self::Summary { value, params: _ } => {
                metric_value.serialize_entry("type", "summary")?;
                metric_value.serialize_entry("value", value)?;
            }
        }

        metric_value.end()
    }
}

struct Histogram {
    histogram: histogram::Histogram,
    sum: u64,
}

impl Histogram {
    fn new(params: &HistogramParams) -> Self {
        let histogram = histogram::Histogram::new(0, 4, (params.max.log2() as u32 + 1).max(4))
            .expect("invalid histogram parameters");
        Histogram { histogram, sum: 0 }
    }
}

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut histogram = serializer.serialize_map(Some(3))?;
        let count = self.histogram.into_iter().fold(0, |acc, b| acc + b.count());
        histogram.serialize_entry(
            "buckets",
            &Buckets {
                histogram: &self.histogram,
            },
        )?;
        histogram.serialize_entry("sum", &(self.sum as f64))?;
        histogram.serialize_entry("count", &count)?;
        histogram.end()
    }
}

struct Buckets<'a> {
    histogram: &'a histogram::Histogram,
}

impl Serialize for Buckets<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let histogram = self.histogram;
        let mut buckets = serializer.serialize_seq(Some(histogram.buckets()))?;
        for bucket in histogram {
            buckets.serialize_element(&Bucket {
                upper_limit: bucket.high(),
                count: bucket.count(),
            })?;
        }
        buckets.end()
    }
}
struct Bucket {
    upper_limit: u64,
    count: u32,
}

impl Serialize for Bucket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bucket = serializer.serialize_map(Some(2))?;
        bucket.serialize_entry("upper_limit", &self.upper_limit)?;
        bucket.serialize_entry("count", &self.count)?;
        bucket.end()
    }
}

struct Summary {
    histogram: histogram::Histogram,
    sum: u64,
}

impl Summary {
    fn new(params: &SummaryParams) -> Self {
        let histogram = histogram::Histogram::new(0, 4, (params.max.log2() as u32 + 1).max(4))
            .expect("invalid histogram parameters");
        Summary { histogram, sum: 0 }
    }
}

const PERCENTILE_VALUES: [f64; 13] = [
    10.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 70.0, 75.0, 90.0, 95.0, 99.0, 99.9,
];

impl Serialize for Summary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut summary = serializer.serialize_map(Some(3))?;
        summary.serialize_entry(
            "quantiles",
            &Quantiles {
                histogram: &self.histogram,
            },
        )?;
        let count = self.histogram.into_iter().fold(0, |acc, b| acc + b.count());
        summary.serialize_entry("sum", &(self.sum as f64))?;
        summary.serialize_entry("count", &count)?;
        summary.end()
    }
}

struct Quantiles<'a> {
    histogram: &'a histogram::Histogram,
}

impl Serialize for Quantiles<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let histogram = self.histogram;
        let mut quantiles = serializer.serialize_seq(Some(histogram.buckets()))?;

        if let Ok(percentiles) = histogram.percentiles(&PERCENTILE_VALUES) {
            for percentile in percentiles {
                quantiles.serialize_element(&Quantile {
                    quantile: percentile.percentile() / 100.0,
                    value: percentile.bucket().low(),
                })?;
            }
        }

        quantiles.end()
    }
}

struct Quantile {
    quantile: f64,
    value: u64,
}

impl Serialize for Quantile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut quantile = serializer.serialize_map(Some(2))?;
        quantile.serialize_entry("quantile", &self.quantile)?;
        quantile.serialize_entry("value", &self.value)?;
        quantile.end()
    }
}

fn random_in_range(min: u64, max: u64) -> u64 {
    thread_rng().gen_range(min..max)
}
