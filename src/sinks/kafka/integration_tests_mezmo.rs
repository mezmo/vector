#![allow(clippy::print_stdout)] // tests
#![cfg(all(test, feature = "kafka-integration-tests"))]

use assay::assay;
use serde_json;
use std::{collections::HashMap, future::ready, thread, time::Duration};

use crate::{
    config::SinkContext,
    kafka::{KafkaAuthConfig, KafkaCompression},
    mezmo::reshape_log_event_by_message,
    sinks::{
        kafka::{config::KafkaSinkConfig, sink::KafkaSink},
        util::BatchConfig,
        VectorSink,
    },
    template::Template,
    test_util::{
        components::{assert_sink_compliance, SINK_TAGS},
        random_message_object_events_with_stream, random_string, wait_for,
    },
};
use rdkafka::{
    consumer::{BaseConsumer, Consumer},
    Message, Offset, TopicPartitionList,
};
use vector_lib::codecs::{
    encoding::format::JsonSerializerOptions, JsonSerializerConfig, MetricTagValues,
};
use vector_lib::event::{BatchNotifier, BatchStatus};
use vector_lib::lookup::lookup_v2::ConfigTargetPath;

fn kafka_host() -> String {
    std::env::var("KAFKA_HOST").unwrap_or_else(|_| "localhost".into())
}

fn kafka_address(port: u16) -> String {
    format!("{}:{}", kafka_host(), port)
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn kafka_mezmo_does_not_reshape_messages() {
    crate::test_util::trace_init();

    let server = kafka_address(9091);

    let topic = format!("test-{}", random_string(10));
    let kafka_auth = KafkaAuthConfig {
        sasl: None,
        tls: None,
    };
    let config = KafkaSinkConfig {
        bootstrap_servers: server.clone(),
        topic: Template::try_from(format!("{}-%Y%m%d", topic)).unwrap(),
        healthcheck_topic: None,
        key_field: None,
        encoding: JsonSerializerConfig::new(
            MetricTagValues::Single,
            JsonSerializerOptions::default(),
        )
        .into(),
        batch: BatchConfig::default(),
        compression: KafkaCompression::None,
        auth: kafka_auth.clone(),
        socket_timeout_ms: Duration::from_millis(60000),
        message_timeout_ms: Duration::from_millis(300000),
        librdkafka_options: HashMap::new(),
        headers_key: Some(ConfigTargetPath::try_from("headers_key".to_owned()).unwrap()),
        acknowledgements: Default::default(),
    };
    let topic = format!("{}-{}", topic, chrono::Utc::now().format("%Y%m%d"));
    println!("Topic name generated in test: {:?}", topic);

    let num_events = 3;
    let (batch, mut receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = random_message_object_events_with_stream(100, num_events, Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let sink = KafkaSink::new(config, SinkContext::default()).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .expect("Running sink failed");
    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // read back everything from the beginning
    let mut client_config = rdkafka::ClientConfig::new();
    client_config.set("bootstrap.servers", server.as_str());
    client_config.set("group.id", random_string(10));
    client_config.set("enable.partition.eof", "true");
    kafka_auth.apply(&mut client_config).unwrap();

    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(&topic, 0)
        .set_offset(Offset::Beginning)
        .unwrap();

    let consumer: BaseConsumer = client_config.create().unwrap();
    consumer.assign(&tpl).unwrap();

    // wait for messages to show up
    wait_for(
        || match consumer.fetch_watermarks(&topic, 0, Duration::from_secs(3)) {
            Ok((_low, high)) => ready(high > 0),
            Err(err) => {
                println!("retrying due to error fetching watermarks: {}", err);
                ready(false)
            }
        },
    )
    .await;

    // check we have the expected number of messages in the topic
    let (low, high) = consumer
        .fetch_watermarks(&topic, 0, Duration::from_secs(3))
        .unwrap();
    assert_eq!((0, num_events as i64), (low, high));

    // loop instead of iter so we can set a timeout
    let mut failures = 0;
    let mut out = Vec::new();
    while failures < 100 {
        match consumer.poll(Duration::from_secs(3)) {
            Some(Ok(msg)) => {
                let s: &str = msg.payload_view().unwrap().unwrap();
                out.push(s.to_owned());
            }
            None if out.len() >= events.len() => break,
            _ => {
                failures += 1;
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let expected = events
        .iter()
        .map(|e| serde_json::to_value(e.as_log()).unwrap())
        .collect::<Vec<_>>();

    let found = out
        .iter()
        .map(|text_event| serde_json::from_str::<serde_json::Value>(text_event).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(found, expected, "As expected, messages were NOT reshaped");
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn kafka_mezmo_reshapes_messages() {
    crate::test_util::trace_init();

    let server = kafka_address(9091);

    let topic = format!("test-{}", random_string(10));
    let kafka_auth = KafkaAuthConfig {
        sasl: None,
        tls: None,
    };
    let config = KafkaSinkConfig {
        bootstrap_servers: server.clone(),
        topic: Template::try_from(format!("{}-%Y%m%d", topic)).unwrap(),
        healthcheck_topic: None,
        key_field: None,
        encoding: JsonSerializerConfig::new(
            MetricTagValues::Single,
            JsonSerializerOptions::default(),
        )
        .into(),
        batch: BatchConfig::default(),
        compression: KafkaCompression::None,
        auth: kafka_auth.clone(),
        socket_timeout_ms: Duration::from_millis(60000),
        message_timeout_ms: Duration::from_millis(300000),
        librdkafka_options: HashMap::new(),
        headers_key: Some(ConfigTargetPath::try_from("headers_key".to_owned()).unwrap()),
        acknowledgements: Default::default(),
    };
    let topic = format!("{}-{}", topic, chrono::Utc::now().format("%Y%m%d"));
    println!("Topic name generated in test: {:?}", topic);

    let num_events = 3;
    let (batch, mut receiver) = BatchNotifier::new_with_receiver();
    let (mut events, stream) =
        random_message_object_events_with_stream(100, num_events, Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let sink = KafkaSink::new(config, SinkContext::default()).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .expect("Running sink failed");
    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // read back everything from the beginning
    let mut client_config = rdkafka::ClientConfig::new();
    client_config.set("bootstrap.servers", server.as_str());
    client_config.set("group.id", random_string(10));
    client_config.set("enable.partition.eof", "true");
    kafka_auth.apply(&mut client_config).unwrap();

    let mut tpl = TopicPartitionList::new();
    tpl.add_partition(&topic, 0)
        .set_offset(Offset::Beginning)
        .unwrap();

    let consumer: BaseConsumer = client_config.create().unwrap();
    consumer.assign(&tpl).unwrap();

    // wait for messages to show up
    wait_for(
        || match consumer.fetch_watermarks(&topic, 0, Duration::from_secs(3)) {
            Ok((_low, high)) => ready(high > 0),
            Err(err) => {
                println!("retrying due to error fetching watermarks: {}", err);
                ready(false)
            }
        },
    )
    .await;

    // check we have the expected number of messages in the topic
    let (low, high) = consumer
        .fetch_watermarks(&topic, 0, Duration::from_secs(3))
        .unwrap();
    assert_eq!((0, num_events as i64), (low, high));

    // loop instead of iter so we can set a timeout
    let mut failures = 0;
    let mut out = Vec::new();
    while failures < 100 {
        match consumer.poll(Duration::from_secs(3)) {
            Some(Ok(msg)) => {
                let s: &str = msg.payload_view().unwrap().unwrap();
                out.push(s.to_owned());
            }
            None if out.len() >= events.len() => break,
            _ => {
                failures += 1;
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let expected = events
        .iter_mut()
        .map(|e| {
            reshape_log_event_by_message(e.as_mut_log());
            serde_json::to_value(e.as_log()).unwrap()
        })
        .collect::<Vec<_>>();

    let found = out
        .iter()
        .map(|text_event| serde_json::from_str::<serde_json::Value>(text_event).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(found, expected, "messages were correctly reshaped");
}
