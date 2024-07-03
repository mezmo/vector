#[cfg(feature = "opentelemetry-sink-integration-tests")]
#[cfg(test)]
mod test {
    use chrono::{NaiveDateTime, TimeZone, Utc};
    use indoc::indoc;
    use std::collections::BTreeMap;

    use vector_lib::{
        config::log_schema,
        event::{
            metric::mezmo::{from_f64_or_zero, to_metric},
            Event, LogEvent,
        },
        finalization::{BatchNotifier, BatchStatus},
        lookup::PathPrefix,
    };

    use crate::{
        config::SinkConfig,
        event::Value,
        sinks::{opentelemetry::config::OpentelemetrySinkConfig, util::test::load_sink},
        test_util::{
            components::{run_and_assert_sink_compliance, SINK_TAGS},
            generate_events_with_stream, generate_metrics_with_stream,
        },
    };

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
        ) -> BTreeMap<String, Value> {
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
                        "type" => Value::from("gauge"),
                        "value" => Value::from(index as f64 * 11.0),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            "name" => Value::from("system.filesystem.usage"),
                            "description" => Value::from("test_description"),
                            "unit" => Value::from("GiBy/s"),
                            "exemplars" => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    "filtered_attributes" => btreemap! {"foo" => Value::from("bar")},
                                    "span_id" => span_id,
                                    "trace_id" => trace_id,
                                    "time_unix" => Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp"),
                                        )
                                    ),
                                    "value" => Value::Integer(10),
                                }

                            )])),
                            "flags" => Value::Integer(1),
                            "start_time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                        };

                        if original_otlp_type.unwrap() == "sum" {
                            arbitrary
                                .insert("aggregation_temporality".to_string(), Value::Integer(0));
                            arbitrary.insert("is_monotonic".to_string(), Value::Boolean(false));
                        }

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::Counter => {
                    let mut value = btreemap! {
                        "type" => Value::from("counter"),
                        "value" => Value::from(index as f64 * 11.0),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            "name" => Value::from("system.filesystem.usage"),
                            "description" => Value::from("test_description"),
                            "unit" => Value::from("GiBy/s"),
                            "exemplars" => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    "filtered_attributes" => btreemap! {"foo" => Value::from("bar")},
                                    "span_id" => span_id,
                                    "trace_id" => trace_id,
                                    "time_unix" => Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp"),
                                        )
                                    ),
                                    "value" => Value::Integer(10),
                                }

                            )])),
                            "flags" => Value::Integer(1),
                            "start_time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "aggregation_temporality" => Value::Integer(2),
                            "is_monotonic" => Value::Boolean(true),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::Set => {
                    let mut values = vec![];
                    for value in 0..index {
                        values.push(Value::from(String::from((value as i32).to_string())));
                    }
                    btreemap! {
                        "type" => Value::from("set"),
                        "value" => Value::Object(btreemap! {
                            "values" => Value::from(values),
                        }),
                    }
                }
                Self::AggregatedHistogram => {
                    let mut value = btreemap! {
                        "type" => Value::from("histogram"),
                        "value" => Value::Object(btreemap! {
                            "count" => Value::Integer(10),
                            "sum" => from_f64_or_zero(3.7),
                            "buckets" => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.005,
                                    "count" => 214,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.01,
                                    "count" => 6,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.025,
                                    "count" => 1,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.05,
                                    "count" => 1,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.075,
                                    "count" => 2,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.1,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.25,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.5,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 0.75,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 1.0,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 2.5,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 5.0,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 7.5,
                                    "count" => 0,
                                }),
                                Value::Object(btreemap! {
                                    "upper_limit" => 10.0,
                                    "count" => 0,
                                }),
                            ]))
                        }),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            "name" => Value::from("system.filesystem.usage"),
                            "description" => Value::from("test_description"),
                            "unit" => Value::from("GiBy/s"),
                            "exemplars" => Value::Array(Vec::from([Value::Object(
                                btreemap! {
                                    "filtered_attributes" => btreemap! {"foo" => Value::from("bar")},
                                    "span_id" => span_id,
                                    "trace_id" => trace_id,
                                    "time_unix" => Value::from(
                                        Utc.from_utc_datetime(
                                            &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                                .expect("timestamp should be a valid timestamp"),
                                        )
                                    ),
                                    "value" => Value::Integer(10),
                                }

                            )])),
                            "flags" => Value::Integer(1),
                            "bucket_counts" => Value::Array(Vec::from([
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

                            "explicit_bounds" => Value::Array(Vec::from([
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
                            "max" => from_f64_or_zero(9.9),
                            "min" => from_f64_or_zero(0.1),
                            "start_time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "aggregation_temporality" => Value::Integer(2),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::DistributionHistogram => {
                    btreemap! {
                        "type" => Value::from("distribution"),
                        "value" => Value::Object(btreemap! {
                            "samples" => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    "value" => 1.0,
                                    "rate" => 3,
                                }),
                                Value::Object(btreemap! {
                                    "value" => 2.0,
                                    "rate" => 3,
                                }),
                                Value::Object(btreemap! {
                                    "value" => 3.0,
                                    "rate" => 2,
                                })
                            ])),
                            "statistic" => Value::from("histogram"),
                        })
                    }
                }
                Self::AggregatedSummary => {
                    let mut value = btreemap! {
                        "type" => Value::from("summary"),
                        "value" => Value::Object(btreemap! {
                            "count" => Value::Integer(10),
                            "sum" => from_f64_or_zero(3.7),
                            "quantiles" => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    "quantile" => 0.005,
                                    "value" => 10,
                                })
                            ]))
                        }),
                    };

                    // Event generate out of otlp metric which came through the OTLP Source
                    // The value must contain arbitrary fields (otlp specific)
                    if otlp_event {
                        let mut arbitrary = btreemap! {
                            "name" => Value::from("system.filesystem.usage"),
                            "description" => Value::from("test_description"),
                            "unit" => Value::from("GiBy/s"),
                            "count" => Value::Integer(10),
                            "sum" => from_f64_or_zero(10.0),
                            "flags" => Value::Integer(1),
                            "quantile_values" => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    "quantile" => 0.005,
                                    "value" => 10,
                                })
                            ])),
                            "start_time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                            "time_unix" => Value::from(
                                Utc.from_utc_datetime(
                                    &NaiveDateTime::from_timestamp_opt(1_579_134_612_i64, 11_u32)
                                        .expect("timestamp should be a valid timestamp"),
                                )
                            ),
                        };

                        value.append(&mut arbitrary);
                    }

                    value
                }
                Self::DistributionSummary => {
                    btreemap! {
                        "type" => Value::from("distribution"),
                        "value" => Value::Object(btreemap! {
                            "samples" => Value::Array(Vec::from([
                                Value::Object(btreemap! {
                                    "value" => 1.0,
                                    "rate" => 3,
                                }),
                                Value::Object(btreemap! {
                                    "value" => 2.0,
                                    "rate" => 3,
                                }),
                                Value::Object(btreemap! {
                                    "value" => 3.0,
                                    "rate" => 2,
                                })
                            ])),
                            "statistic" => Value::from("summary"),
                        })
                    }
                }
            };

            if otlp_event {
                value.insert(
                    log_schema().user_metadata_key().to_string(),
                    Value::Object(btreemap! {
                        "original_type" => Value::from(original_otlp_type.unwrap()),
                        "data_provider" => Value::from("otlp"),
                        "resource" => Value::Object(btreemap! {
                            "attributes" => btreemap! {"foo" => Value::from("bar")},
                            "dropped_attributes_count" => Value::Integer(1),
                            "uniq_id" => Value::from(faster_hex::hex_string(&resource_uniq_id.unwrap())),
                        }),
                        "scope" => Value::Object(btreemap! {
                            "attributes" => btreemap! {"foo" => Value::from("bar")},
                            "dropped_attributes_count" => Value::Integer(1),
                            "name" => Value::from("test_name"),
                            "version" => Value::Null,
                        }),
                        "attributes" => btreemap! {"foo" => Value::from("bar")},
                    })
                );
            }

            value
        }
    }

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

    fn metric_event_generator(
        index: usize,
        metric_type: TestMetricGenerator,
        metric_kind: &str,
        otlp_event: bool,
        original_otlp_type: Option<&str>,
        resource_uniq_id: Option<[u8; 8]>,
    ) -> Event {
        let mut event_data = BTreeMap::<String, Value>::new();
        let value =
            metric_type.generate_value(index, otlp_event, original_otlp_type, resource_uniq_id);

        event_data.insert(
            "message".to_owned(),
            Value::Object(btreemap! {
                "kind" => Value::from(metric_kind),
                "name" => Value::from("system_filesystem_usage_gibibytes_per_second"),
                "tags" => btreemap! {"foo" => "bar"},
                "value" => Value::from(value.clone())
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

    #[tokio::test]
    async fn test_opentelemetry_sink_log_endpoint() {
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

    #[tokio::test]
    async fn test_opentelemetry_sink_metric_endpoint_otlp_events() {
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

        let (batch, receiver) = BatchNotifier::new_with_receiver();

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

        let (_, stream) = generate_metrics_with_stream(generator, gen_settings.len(), Some(batch));

        run_and_assert_sink_compliance(sink, stream, &SINK_TAGS).await;

        assert_eq!(receiver.await, BatchStatus::Delivered);
    }

    #[tokio::test]
    async fn test_opentelemetry_sink_metric_endpoint_not_otlp_events() {
        let config = indoc! {r#"
            endpoint = "opentelemetry-endpoint"
            compression = "gzip"
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

        let (batch, receiver) = BatchNotifier::new_with_receiver();

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

        let (_, stream) = generate_metrics_with_stream(generator, gen_settings.len(), Some(batch));

        run_and_assert_sink_compliance(sink, stream, &SINK_TAGS).await;

        assert_eq!(receiver.await, BatchStatus::Delivered);
    }

    #[tokio::test]
    async fn test_opentelemetry_sink_metric_endpoint_mixed_events() {
        let config = indoc! {r#"
            endpoint = "opentelemetry-endpoint"
            compression = "gzip"
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

        let (batch, receiver) = BatchNotifier::new_with_receiver();

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

        let (_, stream) = generate_metrics_with_stream(generator, gen_settings.len(), Some(batch));

        run_and_assert_sink_compliance(sink, stream, &SINK_TAGS).await;

        assert_eq!(receiver.await, BatchStatus::Delivered);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_opentelemetry_sink_metric_endpoint_unsupported_events() {
        let config = indoc! {r#"
            endpoint = "opentelemetry-endpoint"
            compression = "gzip"
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

        let (batch, _) = BatchNotifier::new_with_receiver();

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

        let (_, stream) = generate_metrics_with_stream(generator, gen_settings.len(), Some(batch));

        run_and_assert_sink_compliance(sink, stream, &SINK_TAGS).await;
    }
}
