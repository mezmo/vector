use chrono::{NaiveDateTime, TimeZone, Utc};
use indoc::indoc;
use std::collections::BTreeMap;

use vector_lib::{
    event::{Event, LogEvent},
    finalization::{BatchNotifier, BatchStatus},
};

use crate::{
    config::SinkConfig,
    event::Value,
    sinks::{opentelemetry::config::OpentelemetrySinkConfig, util::test::load_sink},
    test_util::{
        components::{run_and_assert_sink_compliance, SINK_TAGS},
        generate_events_with_stream,
    },
};

fn line_generator(index: usize) -> String {
    format!("opentelemetry test log index {}", index)
}

fn event_generator(index: usize) -> Event {
    Event::Log(LogEvent::from(line_generator(index)))
}

#[cfg(feature = "opentelemetry-sink-integration-tests")]
#[tokio::test]
async fn test_opentelemetry_sink_endpoint() {
    let config = indoc! {r#"
        endpoint = "opentelemetry-endpoint"
        compression = "gzip"
        [request.headers]
        Auth = "token:thing_and-stuff"
        X-My-Custom-Header = "_%_{}_-_&_._`_|_~_!_#_&_$_"
    "#};

    let endpoint = std::env::var("TEST_OPENTELEMETRY_ENDPOINT")
        .expect("test endpoint environment variable not set");

    assert!(
        !endpoint.is_empty(),
        "$TEST_OPENTELEMETRY_ENDPOINT required"
    );

    let config = config.replace("opentelemetry-endpoint", &endpoint);
    let (config, cx) = load_sink::<OpentelemetrySinkConfig>(config.as_str()).unwrap();

    let (sink, _) = config.build(cx).await.unwrap();

    let trace_id = [
        95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
    ];
    let span_id = [76, 114, 27, 243, 62, 60, 175, 143];
    let expected_resource_attribute = Value::Object(BTreeMap::from([
        ("str".into(), "bar".into()),
        ("int".into(), Value::from(100)),
        ("flt".into(), Value::from(100.123_f64)),
        ("bool".into(), Value::from(false)),
        ("empty".into(), Value::Null),
        (
            "list".into(),
            Value::Array(vec![
                "bar".into(),
                Value::from(100),
                Value::from(100.123_f64),
                Value::from(false),
                Value::Null,
            ]),
        ),
    ]));
    let expected_scope_attributes = expected_resource_attribute.clone();
    let expected_log_attributes = Value::Object(BTreeMap::from([
        ("str".into(), "bar".into()),
        ("int".into(), Value::from(100)),
        ("flt".into(), Value::from(100.123_f64)),
        ("bool".into(), Value::from(false)),
        ("empty".into(), Value::Null),
        (
            "attributes".into(),
            Value::Object(BTreeMap::from([
                ("str".into(), "bar".into()),
                ("int".into(), Value::from(100)),
                ("flt".into(), Value::from(100.123_f64)),
                ("bool".into(), Value::from(false)),
                ("empty".into(), Value::Null),
            ])),
        ),
    ]));

    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let generator = |idx| {
        let mut event = event_generator(idx);
        let log = event.as_mut_log();

        log.insert(
            "metadata",
            Value::Object(BTreeMap::from([
                ("attributes".into(), expected_log_attributes.clone()),
                ("flags".into(), Value::from(1)),
                (
                    "observed_timestamp".into(),
                    Value::from(
                        Utc.from_utc_datetime(
                            &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 0o11_u32)
                                .expect("timestamp should be a valid timestamp"),
                        ),
                    ),
                ),
                (
                    "time".into(),
                    Value::from(
                        Utc.from_utc_datetime(
                            &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 0o11_u32)
                                .expect("timestamp should be a valid timestamp"),
                        ),
                    ),
                ),
                ("severity_number".into(), 17.into()),
                ("severity_text".into(), "ERROR".into()),
                ("level".into(), "ERROR".into()),
                (
                    "trace_id".into(),
                    Value::from(faster_hex::hex_string(&trace_id)),
                ),
                (
                    "span_id".into(),
                    Value::from(faster_hex::hex_string(&span_id)),
                ),
                ("resource".into(), expected_resource_attribute.clone()),
                (
                    "scope".into(),
                    Value::Object(BTreeMap::from([
                        ("attributes".into(), expected_scope_attributes.clone()),
                        ("name".into(), "sone_scope_name".into()),
                        ("version".into(), "1.0.0".into()),
                    ])),
                ),
            ])),
        );

        event
    };
    let (messages, events) = generate_events_with_stream(generator, 5, Some(batch));

    for (index, message) in messages.iter().enumerate() {
        assert_eq!(
            Value::from(format!("opentelemetry test log index {}", index)),
            message.clone().into_log().get_message().unwrap().to_owned()
        );
    }

    run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;

    assert_eq!(receiver.await, BatchStatus::Delivered);
}
