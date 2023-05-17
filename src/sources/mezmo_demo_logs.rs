use chrono::Utc;
use codecs::{
    decoding::{DeserializerConfig, FramingConfig},
    StreamDecodingError,
};
use fakedata::mezmo::access_log::json_access_log_line;
use fakedata::mezmo::error_log::apache_error_log_line;
use fakedata::mezmo::metrics;
use fakedata::mezmo::{
    access_log::{apache_common_log_line, nginx_access_log_line},
    financial::EventGenerator,
    sensor::SensorMqttMessage,
    syslog::{syslog_3164_log_line, syslog_5424_log_line},
};
use futures::StreamExt;
use rand::seq::SliceRandom;
use snafu::Snafu;
use std::task::Poll;
use tokio::sync::OnceCell;
use tokio::time::{self, Duration};
use tokio_util::codec::FramedRead;
use vector_common::internal_event::{
    ByteSize, BytesReceived, CountByteSize, InternalEventHandle as _, Protocol,
};
use vector_config::configurable_component;
use vector_core::{config::LogNamespace, EstimatedJsonEncodedSizeOf};

use crate::{
    codecs::{Decoder, DecodingConfig},
    config::{Output, SourceConfig, SourceContext},
    internal_events::{DemoLogsEventProcessed, EventsReceived, StreamClosedError},
    serde::{default_decoding, default_framing_message_based},
    shutdown::ShutdownSignal,
    SourceSender,
};

/// Configuration for the `mezmo_demo_logs` source.
#[configurable_component(source("mezmo_demo_logs"))]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct MezmoDemoLogsConfig {
    /// The amount of time, in seconds, to pause between each batch of output lines.
    ///
    /// The default is one batch per second. In order to remove the delay and output batches as quickly as possible, set
    /// `interval` to `0.0`.
    #[serde(alias = "batch_interval")]
    #[derivative(Default(value = "default_interval()"))]
    pub interval: f64,

    /// The total number of lines to output.
    ///
    /// By default, the source continuously prints logs (infinitely).
    #[derivative(Default(value = "default_count()"))]
    pub count: usize,

    #[serde(flatten)]
    pub format: MezmoOutputFormat,

    #[configurable(derived)]
    #[derivative(Default(value = "default_framing_message_based()"))]
    pub framing: FramingConfig,

    #[configurable(derived)]
    #[derivative(Default(value = "default_decoding()"))]
    pub decoding: DeserializerConfig,

    /// The namespace to use for logs. This overrides the global setting
    #[serde(default)]
    #[configurable(metadata(docs::hidden))]
    pub log_namespace: Option<bool>,
}

const fn default_interval() -> f64 {
    1.0
}

const fn default_count() -> usize {
    isize::MAX as usize
}

const fn default_device_count() -> usize {
    3
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum MezmoDemoLogsConfigError {
    #[snafu(display("A non-empty list of lines is required for the shuffle format"))]
    ShuffleDemoLogsItemsEmpty,
}

/// Output format configuration.
#[configurable_component]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
#[serde(tag = "format", rename_all = "snake_case")]
#[configurable(metadata(
    docs::enum_tag_description = "The format of the randomly generated output."
))]
pub enum MezmoOutputFormat {
    /// Lines are chosen at random from the list specified using `lines`.
    Shuffle {
        /// If `true`, each output line starts with an increasing sequence number, beginning with 0.
        #[serde(default)]
        sequence: bool,
        /// The list of lines to output.
        lines: Vec<String>,
    },

    /// Randomly generated logs in [Apache common](\(urls.apache_common)) format.
    ApacheCommon,

    /// Randomly generated logs in [Apache error](\(urls.apache_error)) format.
    ApacheError,

    /// Nginx
    Nginx,

    /// Sensor data
    EnvSensor,

    /// Financial Data
    Financial {
        /// number of devices that should generate records
        #[serde(default = "default_device_count")]
        devices: usize,
    },

    /// Randomly generated logs in Syslog format ([RFC 5424](\(urls.syslog_5424))).
    #[serde(alias = "rfc5424")]
    Syslog,

    /// Randomly generated logs in Syslog format ([RFC 3164](\(urls.syslog_3164))).
    #[serde(alias = "rfc3164")]
    BsdSyslog,

    /// Randomly generated HTTP server logs in [JSON](\(urls.json)) format.
    #[derivative(Default)]
    Json,

    /// HTTP-based metrics in the Mezmo metrics format
    HttpMetrics,

    /// Generic metrics in the Mezmo metrics format
    GenericMetrics,
}

struct State {
    financial_evt_state: OnceCell<EventGenerator>,
    metrics_evt_state: OnceCell<metrics::Generator>,
}

impl State {
    fn new() -> State {
        State {
            financial_evt_state: OnceCell::new(),
            metrics_evt_state: OnceCell::new(),
        }
    }

    #[cfg(any(test, feature = "test"))]
    fn new_with(financial_evt_state: OnceCell<EventGenerator>) -> State {
        State {
            financial_evt_state,
            metrics_evt_state: OnceCell::new(),
        }
    }
}

impl MezmoOutputFormat {
    fn generate_line(&self, n: usize, state: &mut State) -> String {
        emit!(DemoLogsEventProcessed);

        match self {
            Self::Shuffle {
                sequence,
                ref lines,
            } => Self::shuffle_generate(*sequence, lines, n),
            Self::ApacheCommon => apache_common_log_line(),
            Self::ApacheError => apache_error_log_line(),
            Self::Nginx => nginx_access_log_line(),
            Self::EnvSensor => {
                let log = SensorMqttMessage::gen_sensor_message();
                serde_json::to_string(&log).expect("sensor data should always be json encodable")
            }
            Self::Financial { devices } => {
                if !state.financial_evt_state.initialized() {
                    let _ = state.financial_evt_state.set(EventGenerator::new(*devices));
                }
                let gen = state
                    .financial_evt_state
                    .get_mut()
                    .expect("financial event state should be set");
                let log = gen.gen_event();
                serde_json::to_string(&log).expect("financial data should always be json encodable")
            }
            Self::Syslog => syslog_5424_log_line(),
            Self::BsdSyslog => syslog_3164_log_line(),
            Self::Json => {
                let log = json_access_log_line();
                serde_json::to_string(&log).expect("json log event should be json encodable")
            }
            Self::HttpMetrics => {
                if !state.metrics_evt_state.initialized() {
                    let _ = state
                        .metrics_evt_state
                        .set(metrics::GeneratorBuilder::build_http());
                }
                state
                    .metrics_evt_state
                    .get_mut()
                    .expect("http metric event state should be set")
                    .generate_next()
            }
            Self::GenericMetrics => {
                if !state.metrics_evt_state.initialized() {
                    let _ = state
                        .metrics_evt_state
                        .set(metrics::GeneratorBuilder::build_generic());
                }
                state
                    .metrics_evt_state
                    .get_mut()
                    .expect("generic metric event state should be set")
                    .generate_next()
            }
        }
    }

    fn shuffle_generate(sequence: bool, lines: &[String], n: usize) -> String {
        // unwrap can be called here because `lines` can't be empty
        let line = lines.choose(&mut rand::thread_rng()).unwrap();

        if sequence {
            format!("{} {}", n, line)
        } else {
            line.into()
        }
    }

    // Ensures that the `lines` list is non-empty if `Shuffle` is chosen
    pub(self) fn validate(&self) -> Result<(), MezmoDemoLogsConfigError> {
        match self {
            Self::Shuffle { lines, .. } => {
                if lines.is_empty() {
                    Err(MezmoDemoLogsConfigError::ShuffleDemoLogsItemsEmpty)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl MezmoDemoLogsConfig {
    #[cfg(test)]
    pub fn repeat(
        lines: Vec<String>,
        count: usize,
        interval: f64,
        log_namespace: Option<bool>,
    ) -> Self {
        Self {
            count,
            interval,
            format: MezmoOutputFormat::Shuffle {
                lines,
                sequence: false,
            },
            framing: default_framing_message_based(),
            decoding: default_decoding(),
            log_namespace,
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn mezmo_demo_logs_source(
    interval: f64,
    count: usize,
    format: MezmoOutputFormat,
    decoder: Decoder,
    mut state: State,
    mut shutdown: ShutdownSignal,
    mut out: SourceSender,
    log_namespace: LogNamespace,
) -> Result<(), ()> {
    let maybe_interval: Option<f64> = (interval != 0.0).then_some(interval);
    let mut interval = maybe_interval.map(|i| time::interval(Duration::from_secs_f64(i)));

    let bytes_received = register!(BytesReceived::from(Protocol::NONE));
    let events_received = register!(EventsReceived);

    for n in 0..count {
        if matches!(futures::poll!(&mut shutdown), Poll::Ready(_)) {
            break;
        }

        if let Some(interval) = &mut interval {
            interval.tick().await;
        }
        bytes_received.emit(ByteSize(0));

        let line = format.generate_line(n, &mut state);

        let mut stream = FramedRead::new(line.as_bytes(), decoder.clone());
        while let Some(next) = stream.next().await {
            match next {
                Ok((events, _byte_size)) => {
                    let count = events.len();
                    let byte_size = events.estimated_json_encoded_size_of();
                    events_received.emit(CountByteSize(count, byte_size));
                    let now = Utc::now();

                    let events = events.into_iter().map(|mut event| {
                        let log = event.as_mut_log();
                        log_namespace.insert_standard_vector_source_metadata(
                            log,
                            MezmoDemoLogsConfig::NAME,
                            now,
                        );

                        event
                    });
                    out.send_batch(events).await.map_err(|error| {
                        emit!(StreamClosedError { error, count });
                    })?;
                }
                Err(error) => {
                    // Error is logged by `crate::codecs::Decoder`, no further
                    // handling is needed here.
                    if !error.can_continue() {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

impl_generate_config_from_default!(MezmoDemoLogsConfig);

#[async_trait::async_trait]
#[typetag::serde(name = "mezmo_demo_logs")]
impl SourceConfig for MezmoDemoLogsConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let log_namespace = cx.log_namespace(self.log_namespace);

        self.format.validate()?;
        let decoder =
            DecodingConfig::new(self.framing.clone(), self.decoding.clone(), log_namespace).build();
        let state = State::new();
        // let _acknowledgements = cx.do_acknowledgements(self.acknowledgements);
        Ok(Box::pin(mezmo_demo_logs_source(
            self.interval,
            self.count,
            self.format.clone(),
            decoder,
            state,
            cx.shutdown,
            cx.out,
            log_namespace,
        )))
    }

    fn outputs(&self, global_log_namespace: LogNamespace) -> Vec<Output> {
        // There is a global and per-source `log_namespace` config. The source config overrides the global setting,
        // and is merged here.
        let log_namespace = global_log_namespace.merge(self.log_namespace);

        let schema_definition = self
            .decoding
            .schema_definition(log_namespace)
            .with_standard_vector_source_metadata();

        vec![Output::default(self.decoding.output_type()).with_schema_definition(schema_definition)]
    }

    fn can_acknowledge(&self) -> bool {
        // Supporting end to end acknowledgements for an event involves three parts:
        //   1. Replacing the existing event finalizer with a new finalizer tied to a notification
        //      channel (e.g. BatchNotifier).
        //   2. Polling the recv side of the notification channel and perform source specific logic
        //      that needs to happen when the event is processed (e.g. move read cursor forward).
        //   3. Return true from this method for the topology builder so it can do some basic checks
        //      on the resulting pipeline.
        //
        // For the demo logs to fake acknowledgement support, we just return true here to silence
        // the topology checks. Since we wouldn't do anything on data finalization, the source doesn't
        // bother wrapping the finalizers or listening on the notification channel.
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use futures::{poll, Stream, StreamExt};

    use super::*;
    use crate::{
        config::log_schema,
        event::Event,
        shutdown::ShutdownSignal,
        test_util::components::{assert_source_compliance, SOURCE_TAGS},
        SourceSender,
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<MezmoDemoLogsConfig>();
    }

    async fn runit(config: &str) -> impl Stream<Item = Event> {
        assert_source_compliance(&SOURCE_TAGS, async {
            let (tx, rx) = SourceSender::new_test();
            let config: MezmoDemoLogsConfig = toml::from_str(config).unwrap();
            let decoder = DecodingConfig::new(
                default_framing_message_based(),
                default_decoding(),
                LogNamespace::Legacy,
            )
            .build();

            let state = State::new_with(OnceCell::new_with(Some(EventGenerator::new(1))));
            mezmo_demo_logs_source(
                config.interval,
                config.count,
                config.format,
                decoder,
                state,
                ShutdownSignal::noop(),
                tx,
                LogNamespace::Legacy,
            )
            .await
            .unwrap();

            rx
        })
        .await
    }

    #[test]
    fn config_shuffle_lines_not_empty() {
        let empty_lines: Vec<String> = Vec::new();

        let errant_config = MezmoDemoLogsConfig {
            format: MezmoOutputFormat::Shuffle {
                sequence: false,
                lines: empty_lines,
            },
            ..MezmoDemoLogsConfig::default()
        };

        assert_eq!(
            errant_config.format.validate(),
            Err(MezmoDemoLogsConfigError::ShuffleDemoLogsItemsEmpty)
        );
    }

    #[tokio::test]
    async fn shuffle_demo_logs_copies_lines() {
        let message_key = log_schema().message_key();
        let mut rx = runit(
            r#"format = "shuffle"
               lines = ["one", "two", "three", "four"]
               count = 5"#,
        )
        .await;

        let lines = &["one", "two", "three", "four"];

        for _ in 0..5 {
            let event = match poll!(rx.next()) {
                Poll::Ready(event) => event.unwrap(),
                _ => unreachable!(),
            };
            let log = event.as_log();
            let message = log[&message_key].to_string_lossy();
            assert!(lines.contains(&&*message));
        }

        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn shuffle_demo_logs_limits_count() {
        let mut rx = runit(
            r#"format = "shuffle"
               lines = ["one", "two"]
               count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn shuffle_demo_logs_adds_sequence() {
        let message_key = log_schema().message_key();
        let mut rx = runit(
            r#"format = "shuffle"
               lines = ["one", "two"]
               sequence = true
               count = 5"#,
        )
        .await;

        for n in 0..5 {
            let event = match poll!(rx.next()) {
                Poll::Ready(event) => event.unwrap(),
                _ => unreachable!(),
            };
            let log = event.as_log();
            let message = log[&message_key].to_string_lossy();
            assert!(message.starts_with(&n.to_string()));
        }

        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn shuffle_demo_logs_obeys_interval() {
        let start = Instant::now();
        let mut rx = runit(
            r#"format = "shuffle"
               lines = ["one", "two"]
               count = 3
               interval = 1.0"#,
        )
        .await;

        for _ in 0..3 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));

        let duration = start.elapsed();
        assert!(duration >= Duration::from_secs(2));
    }

    #[tokio::test]
    async fn apache_common_format_generates_output() {
        let mut rx = runit(
            r#"format = "apache_common"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn nginx_format_generates_output() {
        let mut rx = runit(
            r#"format = "nginx"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn env_sensor_format_generates_output() {
        let mut rx = runit(
            r#"format = "env_sensor"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn financial_format_generates_output() {
        let mut rx = runit(
            r#"format = "financial"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn syslog_5424_format_generates_output() {
        let mut rx = runit(
            r#"format = "syslog"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn syslog_3164_format_generates_output() {
        let mut rx = runit(
            r#"format = "bsd_syslog"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn http_metrics_format_generates_output() {
        let mut rx = runit(
            r#"format = "http_metrics"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }

    #[tokio::test]
    async fn generic_metrics_format_generates_output() {
        let mut rx = runit(
            r#"format = "generic_metrics"
            count = 5"#,
        )
        .await;

        for _ in 0..5 {
            assert!(poll!(rx.next()).is_ready());
        }
        assert_eq!(poll!(rx.next()), Poll::Ready(None));
    }
}
