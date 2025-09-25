//! Tests for custom reshaping of the event to manipulate `message`

use std::io::{BufRead, BufReader};

use assay::assay;
use bytes::Buf;
use flate2::read::MultiGzDecoder;
use futures::stream;

use vector_lib::event::{BatchNotifier, BatchStatus};

use crate::{
    mezmo::reshape_log_event_by_message,
    sinks::{prelude::*, util::test::build_test_server},
    test_util::{next_addr, random_lines_with_stream, random_message_object_events_with_stream},
};

use super::config::HttpSinkConfig;

async fn build_sink(extra_config: &str) -> (std::net::SocketAddr, crate::sinks::VectorSink) {
    let in_addr = next_addr();

    let config = format!(
        r#"
                uri = "http://{in_addr}/frames"
                compression = "gzip"
                framing.method = "newline_delimited"
                encoding.codec = "json"
                {extra_config}
            "#,
    );
    let config: HttpSinkConfig = toml::from_str(&config).unwrap();

    let cx = SinkContext::default();

    let (sink, _) = config.build(cx).await.unwrap();
    (in_addr, sink)
}

async fn build_text_encoding_sink(
    extra_config: &str,
) -> (std::net::SocketAddr, crate::sinks::VectorSink) {
    let in_addr = next_addr();
    let config = format!(
        r#"
            uri = "http://{in_addr}/frames"
            compression = "gzip"
            framing.method = "newline_delimited"
            encoding.codec = "text"
            {extra_config}
        "#,
    );
    let config: HttpSinkConfig = toml::from_str(&config).unwrap();
    let cx = SinkContext::default();

    let (sink, _) = config.build(cx).await.unwrap();
    (in_addr, sink)
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn http_mezmo_does_not_reshape_message() {
    let (in_addr, sink) = build_sink("").await;

    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = random_message_object_events_with_stream(100, 3, Some(batch));

    let (rx, trigger, server) = build_test_server(in_addr);
    tokio::spawn(server);

    sink.run(stream).await.unwrap();
    assert_eq!(receiver.await, BatchStatus::Delivered);

    drop(trigger);

    let found = rx
        .flat_map(|(_, body)| {
            stream::iter(BufReader::new(MultiGzDecoder::new(body.reader())).lines())
        })
        .map(Result::unwrap)
        .map(|line| serde_json::from_str::<serde_json::Value>(&line).unwrap())
        .collect::<Vec<_>>()
        .await;

    let expected = events
        .iter()
        .map(|e| serde_json::to_value(e.as_log()).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(found, expected, "Messages were not reshaped");
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn http_mezmo_correctly_reshapes_message() {
    let (in_addr, sink) = build_sink("").await;

    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (mut events, stream) = random_message_object_events_with_stream(100, 3, Some(batch));

    let (rx, trigger, server) = build_test_server(in_addr);
    tokio::spawn(server);

    sink.run(stream).await.unwrap();
    assert_eq!(receiver.await, BatchStatus::Delivered);

    drop(trigger);

    let found = rx
        .flat_map(|(_, body)| {
            stream::iter(BufReader::new(MultiGzDecoder::new(body.reader())).lines())
        })
        .map(Result::unwrap)
        .map(|line| serde_json::from_str::<serde_json::Value>(&line).unwrap())
        .collect::<Vec<_>>()
        .await;

    let expected = events
        .iter_mut()
        .map(|e| {
            reshape_log_event_by_message(e.as_mut_log());
            serde_json::to_value(e.as_log()).unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(found, expected, "Messages were not reshaped");
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn http_mezmo_does_not_reshape_text_encoding_messages() {
    let (in_addr, sink) = build_text_encoding_sink("").await;
    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = random_lines_with_stream(100, 3, Some(batch));

    let (rx, trigger, server) = build_test_server(in_addr);
    tokio::spawn(server);

    sink.run(stream).await.unwrap();
    assert_eq!(receiver.await, BatchStatus::Delivered);

    drop(trigger);

    let found = rx
        .flat_map(|(_, body)| {
            stream::iter(BufReader::new(MultiGzDecoder::new(body.reader())).lines())
        })
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .await;

    assert_eq!(found, events, "Messages were not reshaped");
}
