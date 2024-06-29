#[cfg(feature = "opentelemetry-sink-integration-tests")]
#[cfg(test)]
mod test {
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

    fn trace_event_generator(_index: usize) -> Event {
        let mut event_data = BTreeMap::<String, Value>::new();

        let trace_id_hex = faster_hex::hex_string(&[
            95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ]);
        let span_id_hex = faster_hex::hex_string(&[76, 114, 27, 243, 62, 60, 175, 143]);
        let parent_span_id_hex = faster_hex::hex_string(&[79, 114, 27, 243, 61, 60, 175, 143]);
        let link_1_trace_id_hex = faster_hex::hex_string(&[
            96, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142,
        ]);
        let link_1_span_id_hex = faster_hex::hex_string(&[77, 114, 27, 243, 62, 60, 175, 143]);

        let message = btreemap! {
            "name" => "test_span_name",
            "trace_id" => Value::from(trace_id_hex.clone()),
            "trace_state" => "foo=,apple=banana",
            "span_id" => Value::from(span_id_hex.clone()),
            "parent_span_id" => Value::from(parent_span_id_hex.clone()),
            // LOG-19724: this field is not currently captured/defined in our source impl
            "flags" => 1,
            "start_timestamp" => Utc.from_utc_datetime(
                &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                    .expect("timestamp should be a valid timestamp"),
            ),
            "dropped_attributes_count" => 1,
            "dropped_events_count" => 2,
            "dropped_links_count" => 3,
            "end_timestamp" => Utc.from_utc_datetime(
                &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 12_u32)
                    .expect("timestamp should be a valid timestamp"),
            ),
            "events" => vec![
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_1_attr",
                    },
                    "dropped_attributes_count" => 4,
                    "name" => "test_name_1",
                    "timestamp" => Utc.from_utc_datetime(
                        &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 13_u32)
                            .expect("timestamp should be a valid timestamp"),
                    ),
                },
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_event_2_attr",
                    },
                    "dropped_attributes_count" => 5,
                    "name" => "test_name_2",
                    "timestamp" => Utc.from_utc_datetime(
                        &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 14_u32)
                            .expect("timestamp should be a valid timestamp"),
                    ),
                }
            ],
            "hostname" => Value::Null,
            "kind" => 2,
            "links" => vec![
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_link_1_attr",
                    },
                    "dropped_attributes_count" => 6,
                    "span_id" => Value::from(link_1_span_id_hex.clone()),
                    "trace_id" => Value::from(link_1_trace_id_hex.clone()),
                    "trace_state" => "bar=,carrot=broccoli",
                },
                btreemap!{
                    "attributes" => btreemap!{
                        "test" => "test_link_2_attr",
                    },
                    "dropped_attributes_count" => 7,
                    "span_id" => Value::from("invalid"),
                    "trace_id" => Value::from("invalid"),
                    "trace_state" => "invalid",
                }
            ],
            "status" => btreemap!{
                "code" => 2,
                "message" => "test_status_message",
            },
        };

        event_data.insert("message".to_owned(), Value::Object(message));

        Event::Log(LogEvent::from(event_data))
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

    #[tokio::test]
    async fn test_opentelemetry_sink_trace_endpoint() {
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

        let uniq_id: [u8; 8] = [76, 114, 27, 243, 62, 60, 175, 143];
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
        let expected_trace_attributes = Value::Object(BTreeMap::from([
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
            let mut event = trace_event_generator(idx);
            let trace = event.as_mut_log();

            trace.insert(
                "metadata",
                btreemap! {
                    "resource" => btreemap!{
                        "attributes" => expected_resource_attribute.clone(),
                        "dropped_attributes_count" => 5,
                        "schema_url" => "https://resource.example.com",
                    },
                    "scope" => btreemap!{
                        "name" => "test_scope_name",
                        "schema_url" => "https://scope.example.com",
                        "version" => "1.2.3",
                        "attributes" => expected_scope_attributes.clone(),
                    },
                    "attributes" => expected_trace_attributes.clone(),
                    "level" => "trace",
                    "span_uniq_id" => uniq_id,
                },
            );

            event
        };
        let (_messages, events) = generate_events_with_stream(generator, 5, Some(batch));

        run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;

        assert_eq!(receiver.await, BatchStatus::Delivered);
    }
}
