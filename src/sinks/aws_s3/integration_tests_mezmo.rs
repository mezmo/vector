#![cfg(all(test, feature = "aws-s3-integration-tests"))]

use crate::mezmo::reshape_log_event_by_message;
use assay::assay;
use bytes::Bytes;
use codecs::decoding::format::Deserializer;
use codecs::decoding::format::JsonDeserializerConfig;
use codecs::{encoding::FramingConfig, JsonSerializerConfig, MetricTagValues};
use futures::Stream;
use similar_asserts::assert_eq;
use tokio_stream::StreamExt;
use vector_core::config::LogNamespace;
use vector_core::event::{BatchNotifier, BatchStatus, BatchStatusReceiver, Event, EventArray};

use super::S3SinkConfig;
use crate::{
    aws::RegionOrEndpoint,
    config::SinkContext,
    sinks::{
        s3_common::config::S3Options,
        util::{BatchConfig, Compression, TowerRequestConfig},
    },
    test_util::{
        components::{run_and_assert_sink_compliance, AWS_SINK_TAGS},
        random_message_object_events_with_stream, random_string,
    },
};

use super::integration_tests::{create_bucket, get_keys, get_lines, get_object, s3_address};

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn s3_message_objects_are_reshaped() {
    let cx = SinkContext::new_test();

    let bucket = uuid::Uuid::new_v4().to_string();

    create_bucket(&bucket, false).await;

    let mut config = json_config(&bucket, 1000000);
    config.key_prefix = "test-prefix".to_string();
    let prefix = config.key_prefix.clone();
    let service = config.create_service(&cx.globals.proxy).await.unwrap();
    let sink = config.build_processor(service, cx).unwrap();

    let (mut generated_events, stream, receiver) = make_object_events_batch(100, 10);

    run_and_assert_sink_compliance(sink, stream, &AWS_SINK_TAGS).await;
    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = get_keys(&bucket, prefix).await;
    assert_eq!(keys.len(), 1);

    let key = keys[0].clone();
    let key_parts = key.split('/');
    assert!(key_parts.count() == 1);
    assert!(key.starts_with("test-prefix"));
    assert!(key.ends_with(".log"));

    let obj = get_object(&bucket, key).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));

    let response_lines = get_lines(obj).await;
    let input = Bytes::from(response_lines[0].clone());
    let deserializer = JsonDeserializerConfig::new().build();
    let got_events = deserializer.parse(input, LogNamespace::Vector).unwrap();

    // Loop to assert results for 2 reasons:
    // 1) SmallVec (AWS results) cannot be `assert_eq!` compared with Vec (generated events)
    // 2) We need to reshape the generated events for comparison

    for (idx, event) in got_events.iter().enumerate() {
        reshape_log_event_by_message(generated_events[idx].as_mut_log());
        assert_eq!(
            event, &generated_events[idx],
            "Event at position {} is correct",
            idx
        );
    }
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn s3_message_objects_not_reshaped_because_of_env() {
    let cx = SinkContext::new_test();

    let bucket = uuid::Uuid::new_v4().to_string();

    create_bucket(&bucket, false).await;

    let mut config = json_config(&bucket, 1000000);
    config.key_prefix = "test-prefix".to_string();
    let prefix = config.key_prefix.clone();
    let service = config.create_service(&cx.globals.proxy).await.unwrap();
    let sink = config.build_processor(service, cx).unwrap();

    let (generated_events, stream, receiver) = make_object_events_batch(100, 3);

    run_and_assert_sink_compliance(sink, stream, &AWS_SINK_TAGS).await;
    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = get_keys(&bucket, prefix).await;
    assert_eq!(keys.len(), 1);

    let key = keys[0].clone();
    let key_parts = key.split('/');
    assert!(key_parts.count() == 1);
    assert!(key.starts_with("test-prefix"));
    assert!(key.ends_with(".log"));

    let obj = get_object(&bucket, key).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));

    let response_lines = get_lines(obj).await;
    let input = Bytes::from(response_lines[0].clone());
    let deserializer = JsonDeserializerConfig::new().build();
    let got_events = deserializer.parse(input, LogNamespace::Vector).unwrap();

    // The `message` property should still exist

    for (idx, event) in got_events.iter().enumerate() {
        assert_eq!(
            event, &generated_events[idx],
            "Event at position {} is correct",
            idx
        );
    }
}

fn json_config(bucket: &str, batch_size: usize) -> S3SinkConfig {
    let mut batch = BatchConfig::default();
    batch.max_events = Some(batch_size);
    batch.timeout_secs = Some(5.0);

    S3SinkConfig {
        bucket: bucket.to_string(),
        key_prefix: random_string(10) + "/date=%F",
        filename_time_format: "abcd".into(),
        filename_append_uuid: false,
        filename_extension: None,
        options: S3Options::default(),
        region: RegionOrEndpoint::with_both("minio", s3_address()),
        encoding: (
            None::<FramingConfig>,
            JsonSerializerConfig::new(MetricTagValues::Single),
        )
            .into(),
        compression: Compression::None,
        batch,
        request: TowerRequestConfig::default(),
        tls: Default::default(),
        auth: Default::default(),
        acknowledgements: Default::default(),
    }
}

fn make_object_events_batch(
    len: usize,
    count: usize,
) -> (
    Vec<Event>,
    impl Stream<Item = EventArray>,
    BatchStatusReceiver,
) {
    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = random_message_object_events_with_stream(len, count, Some(batch));

    (events, stream.map(Into::into), receiver)
}
