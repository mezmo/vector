//! `Pulsar` source.
//! Accepts log events streamed from [`Apache Pulsar`][pulsar].
//!
//! [pulsar]: https://pulsar.apache.org/
use std::path::Path;

use bytes::Bytes;
use chrono::TimeZone;
use futures_util::StreamExt;
use pulsar::{
    Authentication, ConnectionRetryOptions, Consumer, Pulsar, SubType, TokioExecutor,
    authentication::oauth2::{OAuth2Authentication, OAuth2Params},
    consumer::{InitialPosition, Message},
    message::proto::MessageIdData,
};
use regex::Regex;
use std::time::Duration;
use tokio_util::codec::FramedRead;
use vector_lib::{
    EstimatedJsonEncodedSizeOf,
    codecs::{
        StreamDecodingError,
        decoding::{DeserializerConfig, FramingConfig},
    },
    config::{LegacyKey, LogNamespace, SourceAcknowledgementsConfig, SourceOutput},
    configurable::configurable_component,
    event::Event,
    finalization::BatchStatus,
    finalizer::OrderedFinalizer,
    internal_event::{
        ByteSize, BytesReceived, CountByteSize, EventsReceived, InternalEventHandle, Protocol,
        Registered,
    },
    sensitive_string::SensitiveString,
    shutdown::ShutdownSignal,
};
use vrl::value::ObjectMap;
use vrl::{owned_value_path, path, value::Kind};

use crate::{
    SourceSender,
    codecs::{Decoder, DecodingConfig},
    config::{SourceConfig, SourceContext},
    event::BatchNotifier,
    event::Value,
    internal_events::{
        PulsarErrorEvent, PulsarErrorEventData, PulsarErrorEventType, StreamClosedError,
    },
    mezmo_env_config,
    serde::default_false,
    serde::{bool_or_struct, default_decoding, default_framing_message_based},
};
use std::sync::LazyLock;

/// Configuration for the `pulsar` source.
#[configurable_component(source("pulsar", "Collect logs from Apache Pulsar."))]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
#[serde(deny_unknown_fields)]
pub struct PulsarSourceConfig {
    /// The endpoint to which the Pulsar client should connect to.
    #[configurable(metadata(docs::examples = "pulsar://127.0.0.1:6650"))]
    #[serde(alias = "address")]
    endpoint: String,

    /// The Pulsar topic names to read events from.
    #[configurable(metadata(docs::examples = "[persistent://public/default/my-topic]"))]
    topics: Vec<String>,

    /// The Pulsar consumer name.
    #[configurable(metadata(docs::examples = "consumer-name"))]
    consumer_name: Option<String>,

    /// The Pulsar subscription name.
    #[configurable(metadata(docs::examples = "subscription_name"))]
    subscription_name: Option<String>,

    /// The consumer's priority level.
    ///
    /// The broker follows descending priorities. For example, 0=max-priority, 1, 2,...
    ///
    /// In Shared subscription type, the broker first dispatches messages to the max priority level consumers if they have permits. Otherwise, the broker considers next priority level consumers.
    priority_level: Option<i32>,

    /// Max count of messages in a batch.
    batch_size: Option<u32>,

    #[configurable(derived)]
    auth: Option<AuthConfig>,

    #[configurable(derived)]
    dead_letter_queue_policy: Option<DeadLetterQueuePolicy>,

    /// The subscription type to use.
    #[configurable(derived)]
    #[configurable(metadata(docs::examples = "Exclusive"))]
    #[configurable(metadata(docs::examples = "Shared"))]
    #[configurable(metadata(docs::examples = "Failover"))]
    #[configurable(metadata(docs::examples = "KeyShared"))]
    #[serde(default)]
    subscription_type: SubscriptionType,

    /// The read position that the consumer should start from.
    #[configurable(derived)]
    #[configurable(metadata(docs::examples = "Earliest"))]
    #[configurable(metadata(docs::examples = "Latest"))]
    #[serde(default)]
    consumer_position: ConsumerPosition,

    #[configurable(derived)]
    #[serde(default = "default_framing_message_based")]
    #[derivative(Default(value = "default_framing_message_based()"))]
    framing: FramingConfig,

    #[configurable(derived)]
    #[serde(default = "default_decoding")]
    #[derivative(Default(value = "default_decoding()"))]
    decoding: DeserializerConfig,

    #[configurable(derived)]
    #[serde(default, deserialize_with = "bool_or_struct")]
    acknowledgements: SourceAcknowledgementsConfig,

    /// The namespace to use for logs. This overrides the global setting.
    #[configurable(metadata(docs::hidden))]
    #[serde(default)]
    log_namespace: Option<bool>,

    /// If event batch delivery fails (from a bad sink?), Pulsar messages can be nacked using this setting,
    /// thus causing redelivery from the broker.
    ///
    /// When `false`, failed event delivery will retry the failed events based on the sink's builtin retry mechanism.
    /// If sink retries are exhausted, the message will be acked (dropped) for `BatchStatus::Errored` (500s) or `BatchStatus::Rejected` (400s).
    /// Since the user has no ability to skip failed messages, the system must make a best effort to deliver, but move on afterwards.
    /// The consumer can still restart from `Earliest`, manually reset the cursor, or use "replay" to get all missed messages.
    /// Therefore, data loss is really just perceptual.
    ///
    /// When `true`, failed event delivery nacks the original Pulsar message, and it will constantly be re-consumed by this source until successful.
    /// Such events will travel through downstream components, which may skew aggregations and other stateful processing, but
    /// it guarantees no message loss even during downstream outages. Use of this feature would be on a per use case basis.
    ///
    /// NOTE: When `false`, this mimics Kafka's functionality in a roundabout way. In Kafka, the offset is not moved upon failure.
    /// However, the first message to succeed will move the offset and implicitly ack everything that came before it. Pulsar tracks
    /// acking per-message, so avoiding acks creates "ack holes" which become very resource-intensive to recover from if they are numerous.
    /// For this reason, `broker_redelivery_enabled` is set to `false` by default, and failures are proactively acked to avoid holes.
    #[configurable(derived)]
    #[serde(default = "default_false")]
    broker_redelivery_enabled: bool,

    #[configurable(derived)]
    #[serde(default)]
    tls: Option<TlsOptions>,

    /// If using partitioned topics, this enables auto-detection of newly-added partitions
    /// in the consumer. That functionality requires that only 1 topic to be passed into this config.
    #[configurable(derived)]
    #[serde(default = "default_false")]
    partitioned_topic_auto_discovery: bool,

    /// How often to make sure the consumer has all topics (partitioned or regex).
    #[configurable(metadata(docs::examples = "10"))]
    #[serde(default = "default_topic_refresh_secs")]
    topic_refresh_secs: u8,

    #[configurable(derived)]
    #[serde(default)]
    connection_retry_options: Option<PulsarConnectionRetryOptions>,
}

fn default_topic_refresh_secs() -> u8 {
    mezmo_env_config!("MEZMO_PULSAR_TOPIC_REFRESH_SECS", 30)
}

/// Specify the subscription type for the consumer
#[configurable_component]
#[derive(Clone, Copy, Debug, Default)]
pub enum SubscriptionType {
    /// Exclusive subscription type.
    Exclusive,
    /// Shared subscription type.
    #[default]
    Shared,
    /// Failover subscription type.
    Failover,
    /// Key_Shared subscription type.
    KeyShared,
}

impl From<SubscriptionType> for SubType {
    fn from(val: SubscriptionType) -> SubType {
        match val {
            SubscriptionType::Exclusive => SubType::Exclusive,
            SubscriptionType::Shared => SubType::Shared,
            SubscriptionType::Failover => SubType::Failover,
            SubscriptionType::KeyShared => SubType::KeyShared,
        }
    }
}

/// Spcify the position from which the consumer should start reading.
#[configurable_component]
#[derive(Clone, Copy, Debug, Default)]
pub enum ConsumerPosition {
    /// Read from the beginning
    Earliest,
    /// Read from the last-known message
    #[default]
    Latest,
}

impl From<ConsumerPosition> for InitialPosition {
    fn from(val: ConsumerPosition) -> InitialPosition {
        match val {
            ConsumerPosition::Earliest => InitialPosition::Earliest,
            ConsumerPosition::Latest => InitialPosition::Latest,
        }
    }
}

/// Authentication configuration.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
enum AuthConfig {
    /// Basic authentication.
    Basic {
        /// Basic authentication name/username.
        ///
        /// This can be used either for basic authentication (username/password) or JWT authentication.
        /// When used for JWT, the value should be `token`.
        #[configurable(metadata(docs::examples = "${PULSAR_NAME}"))]
        #[configurable(metadata(docs::examples = "name123"))]
        name: String,

        /// Basic authentication password/token.
        ///
        /// This can be used either for basic authentication (username/password) or JWT authentication.
        /// When used for JWT, the value should be the signed JWT, in the compact representation.
        #[configurable(metadata(docs::examples = "${PULSAR_TOKEN}"))]
        #[configurable(metadata(docs::examples = "123456789"))]
        token: SensitiveString,
    },

    /// OAuth authentication.
    OAuth {
        #[configurable(derived)]
        oauth2: OAuth2Config,
    },
}

/// OAuth2-specific authentication configuration.
#[configurable_component]
#[derive(Clone, Debug)]
pub struct OAuth2Config {
    /// The issuer URL.
    #[configurable(metadata(docs::examples = "${OAUTH2_ISSUER_URL}"))]
    #[configurable(metadata(docs::examples = "https://oauth2.issuer"))]
    issuer_url: String,

    /// The credentials URL.
    ///
    /// A data URL is also supported.
    #[configurable(metadata(docs::examples = "${OAUTH2_CREDENTIALS_URL}"))]
    #[configurable(metadata(docs::examples = "file:///oauth2_credentials"))]
    #[configurable(metadata(docs::examples = "data:application/json;base64,cHVsc2FyCg=="))]
    credentials_url: String,

    /// The OAuth2 audience.
    #[configurable(metadata(docs::examples = "${OAUTH2_AUDIENCE}"))]
    #[configurable(metadata(docs::examples = "pulsar"))]
    audience: Option<String>,

    /// The OAuth2 scope.
    #[configurable(metadata(docs::examples = "${OAUTH2_SCOPE}"))]
    #[configurable(metadata(docs::examples = "admin"))]
    scope: Option<String>,
}

/// Dead Letter Queue policy configuration.
#[configurable_component]
#[derive(Clone, Debug)]
struct DeadLetterQueuePolicy {
    /// Maximum number of times that a message will be redelivered before being sent to the dead letter queue.
    pub max_redeliver_count: usize,

    /// Name of the dead letter topic where the failing messages will be sent.
    pub dead_letter_topic: String,
}

#[configurable_component]
#[configurable(description = "TLS options configuration for the Pulsar client.")]
#[derive(Clone, Debug)]
pub struct TlsOptions {
    /// File path containing a list of PEM encoded certificates
    #[configurable(metadata(docs::examples = "/etc/certs/chain.pem"))]
    pub ca_file: String,

    /// Enables certificate verification.
    ///
    /// Do NOT set this to `false` unless you understand the risks of not verifying the validity of certificates.
    pub verify_certificate: Option<bool>,

    /// Whether hostname verification is enabled when verify_certificate is false
    ///
    /// Set to true if not specified.
    pub verify_hostname: Option<bool>,
}

#[configurable_component]
#[configurable(description = "Connection retry options for the pulsar client.")]
#[derive(Clone, Debug)]
pub struct PulsarConnectionRetryOptions {
    /// Minimum delay between connection retries
    #[configurable(metadata(docs::type_unit = "milliseconds"))]
    #[configurable(metadata(docs::examples = 10))]
    pub min_backoff_ms: Option<u64>,
    /// Maximum delay between reconnection retries
    #[configurable(metadata(docs::type_unit = "seconds"))]
    #[configurable(metadata(docs::examples = 30))]
    pub max_backoff_secs: Option<u64>,
    /// Maximum number of connection retries
    #[configurable(metadata(docs::examples = 12))]
    pub max_retries: Option<u32>,
    /// Time limit to establish a connection
    #[configurable(metadata(docs::type_unit = "seconds"))]
    #[configurable(metadata(docs::examples = 30))]
    pub connection_timeout_secs: Option<u64>,
    /// Keep-alive interval for each broker connection
    #[configurable(metadata(docs::type_unit = "seconds"))]
    #[configurable(metadata(docs::examples = 60))]
    pub keep_alive_secs: Option<u64>,
    /// Maximum idle time before a connection is eligible for cleanup
    #[configurable(metadata(docs::type_unit = "seconds"))]
    #[configurable(metadata(docs::examples = 120))]
    pub connection_max_idle: Option<u64>,
}

#[derive(Debug)]
struct FinalizerEntry {
    topic: String,
    message_id: MessageIdData,
}

impl_generate_config_from_default!(PulsarSourceConfig);

static TOPIC_PARSE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\w+://)?(?P<tenant>[^/]+)/(?P<namespace>[^/]+)/(?P<topic>.+)").unwrap()
});

#[async_trait::async_trait]
#[typetag::serde(name = "pulsar")]
impl SourceConfig for PulsarSourceConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let log_namespace = cx.log_namespace(self.log_namespace);

        let consumer = self.create_consumer().await?;
        let decoder =
            DecodingConfig::new(self.framing.clone(), self.decoding.clone(), log_namespace)
                .build()?;
        let acknowledgements = cx.do_acknowledgements(self.acknowledgements);

        Ok(Box::pin(pulsar_source(
            consumer,
            decoder,
            cx.shutdown,
            cx.out,
            acknowledgements,
            log_namespace,
            self.broker_redelivery_enabled,
        )))
    }

    fn outputs(&self, global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        let log_namespace = global_log_namespace.merge(self.log_namespace);

        let schema_definition = self
            .decoding
            .schema_definition(log_namespace)
            .with_standard_vector_source_metadata()
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!("publish_time"))),
                &owned_value_path!("publish_time"),
                Kind::timestamp(),
                Some("publish_time"),
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!("topic"))),
                &owned_value_path!("topic"),
                Kind::bytes(),
                Some("topic"),
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!("producer_name"))),
                &owned_value_path!("producer_name"),
                Kind::bytes(),
                Some("producer_name"),
            );
        vec![SourceOutput::new_maybe_logs(
            self.decoding.output_type(),
            schema_definition,
        )]
    }

    fn can_acknowledge(&self) -> bool {
        true
    }
}

impl PulsarSourceConfig {
    async fn create_consumer(
        &self,
    ) -> crate::Result<pulsar::consumer::Consumer<String, TokioExecutor>> {
        let mut builder = Pulsar::builder(&self.endpoint, TokioExecutor);

        let mut retry_options = ConnectionRetryOptions {
            // Use an infinite number of retries by default so that Vector doesn't
            // stop processing this source.
            max_retries: u32::MAX,
            // Broker-side keepalives are 30-60s. Make this shorter to ensure it stays alive.
            keep_alive: Duration::from_secs(20),
            ..ConnectionRetryOptions::default()
        };

        if let Some(opts) = &self.connection_retry_options {
            if let Some(ms) = opts.min_backoff_ms {
                retry_options.min_backoff = Duration::from_millis(ms);
            }
            if let Some(secs) = opts.max_backoff_secs {
                retry_options.max_backoff = Duration::from_secs(secs);
            }
            if let Some(retries) = opts.max_retries {
                retry_options.max_retries = retries;
            }
            if let Some(secs) = opts.connection_timeout_secs {
                retry_options.connection_timeout = Duration::from_secs(secs);
            }
            if let Some(secs) = opts.keep_alive_secs {
                retry_options.keep_alive = Duration::from_secs(secs);
            }
            if let Some(secs) = opts.connection_max_idle {
                retry_options.connection_max_idle = Duration::from_secs(secs);
            }
        }

        debug!("Creating pulsar consumer with options: {:?}", retry_options);

        builder = builder.with_connection_retry_options(retry_options);

        if let Some(auth) = &self.auth {
            builder = match auth {
                AuthConfig::Basic { name, token } => builder.with_auth(Authentication {
                    name: name.clone(),
                    data: token.inner().as_bytes().to_vec(),
                }),
                AuthConfig::OAuth { oauth2 } => builder.with_auth_provider(
                    OAuth2Authentication::client_credentials(OAuth2Params {
                        issuer_url: oauth2.issuer_url.clone(),
                        credentials_url: oauth2.credentials_url.clone(),
                        audience: oauth2.audience.clone(),
                        scope: oauth2.scope.clone(),
                    }),
                ),
            };
        }
        if let Some(options) = &self.tls {
            builder = builder.with_certificate_chain_file(Path::new(&options.ca_file))?;
            builder =
                builder.with_allow_insecure_connection(!options.verify_certificate.unwrap_or(true));
            builder = builder
                .with_tls_hostname_verification_enabled(options.verify_hostname.unwrap_or(true));
        }

        let pulsar = builder.build().await?;

        let mut consumer_builder: pulsar::ConsumerBuilder<TokioExecutor> = pulsar
            .consumer()
            .with_subscription_type(self.subscription_type.into())
            .with_options(pulsar::consumer::ConsumerOptions {
                priority_level: self.priority_level,
                initial_position: self.consumer_position.into(),
                ..Default::default()
            })
            .with_topic_refresh(std::time::Duration::from_secs(
                self.topic_refresh_secs.into(),
            ));

        if let Some(dead_letter_queue_policy) = &self.dead_letter_queue_policy {
            consumer_builder =
                consumer_builder.with_dead_letter_policy(pulsar::consumer::DeadLetterPolicy {
                    max_redeliver_count: dead_letter_queue_policy.max_redeliver_count,
                    dead_letter_topic: dead_letter_queue_policy.dead_letter_topic.clone(),
                });
        }

        if let Some(batch_size) = self.batch_size {
            consumer_builder = consumer_builder.with_batch_size(batch_size);
        }
        if let Some(consumer_name) = &self.consumer_name {
            consumer_builder = consumer_builder.with_consumer_name(consumer_name);
        }
        if let Some(subscription_name) = &self.subscription_name {
            consumer_builder = consumer_builder.with_subscription(subscription_name);
        }

        // Mezmo: Use regex subscription so that newly-added topic partitions are automatically detected.
        // If multiple topics are specified, regex cannot be used, so fall back to a static topic list.
        if self.partitioned_topic_auto_discovery && self.topics.len() == 1 {
            let topic = &self.topics[0];
            let mut producer_builder = pulsar.producer().with_topic(topic.clone());

            if let Some(consumer_name) = &self.consumer_name {
                producer_builder = producer_builder.with_name(format!("prime-{consumer_name}"));
            }

            // If auto-topic-creation is on, this will make all partitions such that the regex
            // subscription will immediately see them rather than have to wait for another publisher
            // to create them. In that time, data can be lost since we're starting at `Latest`.
            match producer_builder.build().await {
                Ok(_) => {
                    debug!("Successfully created producer to prime {topic}.");
                }
                Err(err) => {
                    error!("Failed to create producer to prime {topic}: {err}");
                }
            }

            let captures = TOPIC_PARSE_REGEX.captures(topic).ok_or_else(|| {
                format!(
                    "Topic must be in the format [protocol://]tenant/namespace/topic: '{topic}'"
                )
            })?;
            let tenant = captures.name("tenant").unwrap().as_str();
            let namespace = captures.name("namespace").unwrap().as_str();

            let topic_regex_str = format!("{}.*", regex::escape(topic));
            let consumer_topic_regex = Regex::new(&topic_regex_str)
                .map_err(|err| format!("Invalid topic regex '{topic_regex_str}': {err}"))?;

            consumer_builder = consumer_builder
                .with_lookup_namespace(format!("{tenant}/{namespace}"))
                .with_topic_regex(consumer_topic_regex);
            debug!(
                "Using topic regex '{}' with refresh every {} seconds",
                topic, self.topic_refresh_secs
            );
        } else {
            debug!(
                "Using multiple topics subscription (refreshed every {} secs) with no partition auto-discovery: {:?}",
                self.topic_refresh_secs, self.topics
            );
            consumer_builder = consumer_builder.with_topics(&self.topics);
        }

        let consumer = consumer_builder.build::<String>().await?;

        Ok(consumer)
    }
}

async fn pulsar_source(
    mut consumer: Consumer<String, TokioExecutor>,
    decoder: Decoder,
    mut shutdown: ShutdownSignal,
    mut out: SourceSender,
    acknowledgements: bool,
    log_namespace: LogNamespace,
    broker_redelivery_enabled: bool,
) -> Result<(), ()> {
    let (finalizer, mut ack_stream) =
        OrderedFinalizer::<FinalizerEntry>::maybe_new(acknowledgements, Some(shutdown.clone()));

    let bytes_received = register!(BytesReceived::from(Protocol::TCP));
    let events_received = register!(EventsReceived);
    let pulsar_error_events = register!(PulsarErrorEvent);

    loop {
        let msg_result = tokio::select! {
            _ = &mut shutdown => break,
            entry = ack_stream.next() => {
                if let Some((status, entry)) = entry {
                    handle_ack(&mut consumer, status, entry, &pulsar_error_events, broker_redelivery_enabled).await;
                }
                continue;
            },
            msg_result = consumer.next() => {
                msg_result
            },
        };

        match msg_result {
            Some(Ok(msg)) => {
                bytes_received.emit(ByteSize(msg.payload.data.len()));
                parse_message(
                    msg,
                    &decoder,
                    &finalizer,
                    &mut out,
                    &mut consumer,
                    log_namespace,
                    &events_received,
                    &pulsar_error_events,
                )
                .await;
            }

            Some(Err(e)) => {
                pulsar_error_events.emit(PulsarErrorEventData {
                    msg: e.to_string(),
                    error_type: PulsarErrorEventType::Read,
                });
                error!("Error reading message from pulsar: {e}");
            }

            None => {
                debug!("Pulsar consumer stream ended");
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn parse_message(
    msg: Message<String>,
    decoder: &Decoder,
    finalizer: &Option<OrderedFinalizer<FinalizerEntry>>,
    out: &mut SourceSender,
    consumer: &mut Consumer<String, TokioExecutor>,
    log_namespace: LogNamespace,
    events_received: &Registered<EventsReceived>,
    pulsar_error_events: &Registered<PulsarErrorEvent>,
) {
    let publish_time = i64::try_from(msg.payload.metadata.publish_time)
        .ok()
        .and_then(|millis| chrono::Utc.timestamp_millis_opt(millis).latest());
    let topic = msg.topic.clone();
    let producer_name = msg.payload.metadata.producer_name.clone();

    let sequence_id = msg.payload.metadata.sequence_id;
    let message_ledger_id = msg.message_id.id.ledger_id;
    let message_entry_id = msg.message_id.id.entry_id;

    let mut headers_map = ObjectMap::new();
    for kv in &msg.metadata().properties {
        headers_map.insert(
            kv.key.to_owned().into(),
            Value::from(Bytes::from(kv.value.to_owned())),
        );
    }

    let mut stream = FramedRead::new(msg.payload.data.as_ref(), decoder.clone());
    let stream = async_stream::stream! {
        while let Some(next) = stream.next().await {
            match next {
                Ok((events, _byte_size)) => {
                    events_received.emit(CountByteSize(
                        events.len(),
                        events.estimated_json_encoded_size_of(),
                    ));

                    let now = chrono::Utc::now();

                    let events = events.into_iter().map(|mut event| {
                        if let Event::Log(ref mut log) = event {
                            log_namespace.insert_standard_vector_source_metadata(
                                log,
                                PulsarSourceConfig::NAME,
                                now,
                            );

                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("publish_time"))),
                                path!("publish_time"),
                                publish_time,
                            );

                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("topic"))),
                                path!("topic"),
                                topic.clone(),
                            );

                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("producer_name"))),
                                path!("producer_name"),
                                producer_name.clone(),
                            );

                            log.insert("headers", headers_map.clone());

                            // mezmo additions
                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("sequence_id"))),
                                path!("sequence_id"),
                                sequence_id,
                            );

                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("message_ledger_id"))),
                                path!("message_ledger_id"),
                                message_ledger_id,
                            );

                            log_namespace.insert_source_metadata(
                                PulsarSourceConfig::NAME,
                                log,
                                Some(LegacyKey::InsertIfEmpty(path!("message_entry_id"))),
                                path!("message_entry_id"),
                                message_entry_id,
                            );
                        }
                        event
                    });

                    for event in events {
                        yield event;
                    }
                }
                Err(error) => {
                    // Error is logged by `crate::codecs`, no further
                    // handling is needed here.
                    if !error.can_continue() {
                        break;
                    }
                }
            }
        }
    }
    .boxed();

    finalize_event_stream(
        consumer,
        finalizer,
        out,
        stream,
        msg.topic.clone(),
        msg.message_id().clone(),
        pulsar_error_events,
    )
    .await;
}

/// Send the event stream created by the framed read to the `out` stream.
async fn finalize_event_stream(
    consumer: &mut Consumer<String, TokioExecutor>,
    finalizer: &Option<OrderedFinalizer<FinalizerEntry>>,
    out: &mut SourceSender,
    mut stream: std::pin::Pin<Box<dyn futures_util::Stream<Item = Event> + Send + '_>>,
    topic: String,
    message_id: MessageIdData,
    pulsar_error_events: &Registered<PulsarErrorEvent>,
) {
    match finalizer {
        Some(finalizer) => {
            let (batch, receiver) = BatchNotifier::new_with_receiver();
            let mut stream = stream.map(|event| event.with_batch_notifier(&batch));

            match out.send_event_stream(&mut stream).await {
                Err(_error) => {
                    emit!(StreamClosedError { count: 1 });
                }
                Ok(_) => {
                    finalizer.add(FinalizerEntry { topic, message_id }, receiver);
                }
            }
        }
        None => match out.send_event_stream(&mut stream).await {
            Err(_error) => {
                emit!(StreamClosedError { count: 1 });
            }
            Ok(_) => {
                if let Err(error) = consumer.ack_with_id(topic.as_str(), message_id).await {
                    pulsar_error_events.emit(PulsarErrorEventData {
                        msg: error.to_string(),
                        error_type: PulsarErrorEventType::Ack,
                    });
                }
            }
        },
    }
}

async fn handle_ack(
    consumer: &mut Consumer<String, TokioExecutor>,
    status: BatchStatus,
    entry: FinalizerEntry,
    pulsar_error_events: &Registered<PulsarErrorEvent>,
    broker_redelivery_enabled: bool,
) {
    match status {
        BatchStatus::Delivered => {
            if let Err(error) = consumer
                .ack_with_id(entry.topic.as_str(), entry.message_id)
                .await
            {
                pulsar_error_events.emit(PulsarErrorEventData {
                    msg: error.to_string(),
                    error_type: PulsarErrorEventType::Ack,
                });
            }
        }
        BatchStatus::Errored | BatchStatus::Rejected => {
            if broker_redelivery_enabled {
                // Nack the message, and the broker will redeliver it.
                if let Err(error) = consumer
                    .nack_with_id(entry.topic.as_str(), entry.message_id)
                    .await
                {
                    pulsar_error_events.emit(PulsarErrorEventData {
                        msg: error.to_string(),
                        error_type: PulsarErrorEventType::NAck,
                    });
                }
            } else {
                // Ack the message so that it doesn't remain "unacked" and create ack holes. We don't want redelivery.
                // Kafka does this implicitly--although it doesn't move the cursor on failure, the first successful message
                // after a bunch of failures *will* move the cursor, and implicitly ack every message before that point.
                if let Err(error) = consumer
                    .ack_with_id(entry.topic.as_str(), entry.message_id)
                    .await
                {
                    pulsar_error_events.emit(PulsarErrorEventData {
                        msg: error.to_string(),
                        error_type: PulsarErrorEventType::Ack,
                    });
                }
                debug!({
                    topic = entry.topic,
                }, "Cannot deliver to destination: {:?}", status);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::sources::pulsar::PulsarSourceConfig;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<PulsarSourceConfig>();
    }

    fn config_from_str(config_str: &str) -> PulsarSourceConfig {
        let full_config = format!(
            r#"
            endpoint = "pulsar://127.0.0.1:6650"
            subscription_name = "test-subscription"
            topics = ["test-topic"]
            {config_str}
        "#
        );
        toml::from_str(&full_config).unwrap()
    }

    #[test]
    fn parse_connection_retry_options_full() {
        let config = config_from_str(
            r#"
            [connection_retry_options]
            min_backoff_ms = 20
            max_backoff_secs = 25
            max_retries = 50
            connection_timeout_secs = 59
            keep_alive_secs = 58
            connection_max_idle = 55
        "#,
        );

        let opts = config.connection_retry_options.unwrap();
        assert_eq!(opts.min_backoff_ms, Some(20));
        assert_eq!(opts.max_backoff_secs, Some(25));
        assert_eq!(opts.max_retries, Some(50));
        assert_eq!(opts.connection_timeout_secs, Some(59));
        assert_eq!(opts.keep_alive_secs, Some(58));
        assert_eq!(opts.connection_max_idle, Some(55));
    }

    #[test]
    fn parse_connection_retry_options_partial() {
        let config = config_from_str(
            r#"
            [connection_retry_options]
            max_retries = 5
            connection_timeout_secs = 55
        "#,
        );

        let opts = config.connection_retry_options.unwrap();
        assert_eq!(opts.min_backoff_ms, None);
        assert_eq!(opts.max_backoff_secs, None);
        assert_eq!(opts.max_retries, Some(5));
        assert_eq!(opts.connection_timeout_secs, Some(55));
        assert_eq!(opts.keep_alive_secs, None);
    }

    #[test]
    fn parse_connection_retry_options_empty() {
        let config = config_from_str(
            r#"
            [connection_retry_options]
        "#,
        );

        let opts = config.connection_retry_options.unwrap();
        assert_eq!(
            opts.min_backoff_ms, None,
            "Expected min_backoff_ms to be None"
        );
        assert_eq!(
            opts.max_backoff_secs, None,
            "Expected max_backoff_secs to be None"
        );
        assert_eq!(opts.max_retries, None, "Expected max_retries to be None");
        assert_eq!(
            opts.connection_timeout_secs, None,
            "Expected connection_timeout_secs to be None"
        );
        assert_eq!(
            opts.keep_alive_secs, None,
            "Expected keep_alive_secs to be None"
        );
        assert_eq!(
            opts.connection_max_idle, None,
            "Expected connection_max_idle to be None"
        );
    }

    #[test]
    fn no_connection_retry_options_is_none() {
        let config = config_from_str("");
        assert!(
            config.connection_retry_options.is_none(),
            "Expected connection_retry_options to be None when not in config"
        );
    }
}

#[cfg(feature = "pulsar-integration-tests")]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{
        config::log_schema,
        test_util::{
            collect_n,
            components::{SOURCE_TAGS, assert_source_compliance},
            random_string, trace_init,
        },
        tls::TEST_PEM_INTERMEDIATE_CA_PATH,
    };
    use pulsar::producer;
    use std::collections::HashMap;

    fn pulsar_host() -> String {
        std::env::var("PULSAR_HOST").unwrap_or_else(|_| "127.0.0.1".into())
    }

    fn pulsar_address(scheme: &str, port: u16) -> String {
        format!("{}://{}:{}", scheme, pulsar_host(), port)
    }
    #[tokio::test]
    async fn consumes_event_with_acknowledgements() {
        pulsar_send_receive(
            &pulsar_address("pulsar", 6650),
            true,
            LogNamespace::Legacy,
            None,
        )
        .await;
    }

    #[tokio::test]
    async fn consumes_event_with_acknowledgements_vector_namespace() {
        pulsar_send_receive(
            &pulsar_address("pulsar", 6650),
            true,
            LogNamespace::Vector,
            None,
        )
        .await;
    }

    #[tokio::test]
    async fn consumes_event_without_acknowledgements() {
        pulsar_send_receive(
            &pulsar_address("pulsar", 6650),
            false,
            LogNamespace::Legacy,
            None,
        )
        .await;
    }

    #[tokio::test]
    async fn consumes_event_without_acknowledgements_vector_namespace() {
        pulsar_send_receive(
            &pulsar_address("pulsar", 6650),
            false,
            LogNamespace::Vector,
            None,
        )
        .await;
    }

    #[tokio::test]
    async fn consumes_event_with_tls() {
        pulsar_send_receive(
            &pulsar_address("pulsar+ssl", 6651),
            false,
            LogNamespace::Vector,
            Some(TlsOptions {
                ca_file: TEST_PEM_INTERMEDIATE_CA_PATH.into(),
                verify_certificate: None,
                verify_hostname: None,
            }),
        )
        .await;
    }

    async fn pulsar_send_receive(
        endpoint: &str,
        acknowledgements: bool,
        log_namespace: LogNamespace,
        tls: Option<TlsOptions>,
    ) {
        trace_init();

        let topic = format!("test-{}", random_string(10));
        let cnf = PulsarSourceConfig {
            endpoint: endpoint.into(),
            topics: vec![topic.clone()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default().into(),
            consumer_position: ConsumerPosition::default().into(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: acknowledgements.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: tls.clone(),
            partitioned_topic_auto_discovery: false,
            topic_refresh_secs: 30,
            connection_retry_options: None,
        };
        let mut builder = Pulsar::<TokioExecutor>::builder(&cnf.endpoint, TokioExecutor);
        if let Some(options) = &tls {
            builder = builder
                .with_certificate_chain_file(Path::new(&options.ca_file))
                .unwrap();
            builder =
                builder.with_allow_insecure_connection(!options.verify_certificate.unwrap_or(true));
            builder = builder
                .with_tls_hostname_verification_enabled(options.verify_hostname.unwrap_or(true));
        }

        let pulsar = builder.build().await.unwrap();

        let consumer = cnf.create_consumer().await.unwrap();
        let decoder = DecodingConfig::new(
            cnf.framing.clone(),
            cnf.decoding.clone(),
            LogNamespace::Legacy,
        )
        .build()
        .unwrap();

        let mut producer = pulsar.producer().with_topic(topic).build().await.unwrap();

        let msg = "test message";
        let mut headers = HashMap::new();
        headers.insert("my_header1".to_string(), "somevalue".to_string());
        headers.insert("my_header2".to_string(), "anothervalue".to_string());
        let properties = headers.clone();

        let events = assert_source_compliance(&SOURCE_TAGS, async move {
            let (tx, rx) = SourceSender::new_test();
            tokio::spawn(pulsar_source(
                consumer,
                decoder,
                ShutdownSignal::noop(),
                tx,
                acknowledgements,
                log_namespace,
                true,
            ));

            let message = producer::Message {
                payload: msg.into(),
                properties,
                ..Default::default()
            };
            producer.send_non_blocking(message).await.unwrap();

            collect_n(rx, 1).await
        })
        .await;

        assert_eq!(
            events[0].as_log()[log_schema().message_key().unwrap().to_string()],
            msg.into()
        );

        let mut expected_headers = ObjectMap::new();
        for (key, value) in headers {
            let key_str: &str = key.as_str();
            let value_str: &str = value.as_str();
            expected_headers.insert(key_str.into(), Value::from(value_str));
        }
        assert_eq!(events[0].as_log()["headers"], Value::from(expected_headers));
    }

    #[tokio::test]
    async fn test_partitioned_topic_auto_discovery_enabled_single_topic() {
        trace_init();

        let topic = format!("persistent://public/default/test-{}", random_string(10));
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![topic.clone()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: true,
            topic_refresh_secs: 10,
            connection_retry_options: None,
        };

        // Should succeed with valid topic format
        let consumer = cnf.create_consumer().await;
        assert!(
            consumer.is_ok(),
            "Consumer creation should succeed with valid topic format"
        );
    }

    #[tokio::test]
    async fn test_partitioned_topic_auto_discovery_enabled_but_multiple_topics() {
        trace_init();

        let topic1 = format!("persistent://public/default/test-{}", random_string(10));
        let topic2 = format!("persistent://public/default/test-{}", random_string(10));
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![topic1, topic2],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: true,
            topic_refresh_secs: 15,
            connection_retry_options: None,
        };

        // Should succeed but fall back to static topic list (no regex)
        let consumer = cnf.create_consumer().await;
        assert!(
            consumer.is_ok(),
            "Consumer creation should succeed with multiple topics"
        );
    }

    #[tokio::test]
    async fn test_partitioned_topic_auto_discovery_disabled() {
        trace_init();

        let topic = format!("persistent://public/default/test-{}", random_string(10));
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![topic.clone()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: false,
            topic_refresh_secs: 20,
            connection_retry_options: None,
        };

        // Should succeed using static topic list
        let consumer = cnf.create_consumer().await;
        assert!(
            consumer.is_ok(),
            "Consumer creation should succeed with auto-discovery disabled"
        );
    }

    #[tokio::test]
    async fn test_topic_parsing_error_invalid_format() {
        trace_init();

        let invalid_topic = "invalid-topic-format";
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![invalid_topic.to_string()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: true,
            topic_refresh_secs: 30,
            connection_retry_options: None,
        };

        let consumer_result = cnf.create_consumer().await;
        assert!(
            consumer_result.is_err(),
            "Consumer creation should fail with invalid topic format"
        );

        match consumer_result {
            Err(error) => {
                let error_msg = format!("{error}");
                assert_eq!(
                    error_msg,
                    "Topic must be in the format [protocol://]tenant/namespace/topic: 'invalid-topic-format'",
                    "Error should mention topic parsing failure: {error_msg}",
                );
            }
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_topic_parsing_error_empty_tenant() {
        trace_init();

        let invalid_topic = "persistent:///default/topic";
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![invalid_topic.to_string()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: true,
            topic_refresh_secs: 30,
            connection_retry_options: None,
        };

        let consumer_result = cnf.create_consumer().await;
        assert!(
            consumer_result.is_err(),
            "Consumer creation should fail with a tenant error"
        );

        match consumer_result {
            Err(error) => {
                let error_msg = format!("{error}");
                assert_eq!(
                    error_msg,
                    "Topic must be in the format [protocol://]tenant/namespace/topic: 'persistent:///default/topic'",
                    "Error should mention topic parsing failure: {error_msg}",
                );
            }
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_topic_parsing_error_empty_namespace() {
        trace_init();

        let invalid_topic = "persistent://public//topic";
        let cnf = PulsarSourceConfig {
            endpoint: pulsar_address("pulsar", 6650),
            topics: vec![invalid_topic.to_string()],
            consumer_name: None,
            subscription_name: None,
            priority_level: None,
            batch_size: None,
            auth: None,
            dead_letter_queue_policy: None,
            subscription_type: SubscriptionType::default(),
            consumer_position: ConsumerPosition::default(),
            framing: FramingConfig::Bytes,
            decoding: DeserializerConfig::Bytes,
            acknowledgements: false.into(),
            log_namespace: None,
            broker_redelivery_enabled: true,
            tls: None,
            partitioned_topic_auto_discovery: true,
            topic_refresh_secs: 30,
            connection_retry_options: None,
        };

        let consumer_result = cnf.create_consumer().await;
        assert!(
            consumer_result.is_err(),
            "Consumer creation should fail with a namespace error"
        );

        match consumer_result {
            Err(error) => {
                let error_msg = format!("{error}");
                assert_eq!(
                    error_msg,
                    "Topic must be in the format [protocol://]tenant/namespace/topic: 'persistent://public//topic'",
                    "Error should mention topic parsing failure: {error_msg}",
                );
            }
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_topic_refresh_secs_variations() {
        trace_init();

        let topic = format!("persistent://public/default/test-{}", random_string(10));

        // Test different topic_refresh_secs values
        for refresh_secs in [1, 5, 30, 60, 255] {
            let cnf = PulsarSourceConfig {
                endpoint: pulsar_address("pulsar", 6650),
                topics: vec![topic.clone()],
                consumer_name: None,
                subscription_name: None,
                priority_level: None,
                batch_size: None,
                auth: None,
                dead_letter_queue_policy: None,
                subscription_type: SubscriptionType::default(),
                consumer_position: ConsumerPosition::default(),
                framing: FramingConfig::Bytes,
                decoding: DeserializerConfig::Bytes,
                acknowledgements: false.into(),
                log_namespace: None,
                broker_redelivery_enabled: true,
                tls: None,
                partitioned_topic_auto_discovery: true,
                topic_refresh_secs: refresh_secs,
                connection_retry_options: None,
            };

            let consumer = cnf.create_consumer().await;
            assert!(
                consumer.is_ok(),
                "Consumer creation should succeed with topic_refresh_secs={refresh_secs}",
            );
        }
    }

    #[tokio::test]
    async fn test_topic_formats_with_auto_discovery() {
        trace_init();

        // Test various valid topic formats
        let test_cases = vec![
            "persistent://public/default/test-persistent-topic",
            "non-persistent://public/default/non-persistent-topic",
            "public/default/simple-topic", // without protocol
        ];

        for topic in test_cases {
            let cnf = PulsarSourceConfig {
                endpoint: pulsar_address("pulsar", 6650),
                topics: vec![topic.to_string()],
                consumer_name: None,
                subscription_name: None,
                priority_level: None,
                batch_size: None,
                auth: None,
                dead_letter_queue_policy: None,
                subscription_type: SubscriptionType::default(),
                consumer_position: ConsumerPosition::default(),
                framing: FramingConfig::Bytes,
                decoding: DeserializerConfig::Bytes,
                acknowledgements: false.into(),
                log_namespace: None,
                broker_redelivery_enabled: true,
                tls: None,
                partitioned_topic_auto_discovery: true,
                topic_refresh_secs: 30,
                connection_retry_options: None,
            };

            let consumer = cnf.create_consumer().await;
            assert!(
                consumer.is_ok(),
                "Consumer creation should succeed with topic format: {topic}",
            );
        }
    }
}
