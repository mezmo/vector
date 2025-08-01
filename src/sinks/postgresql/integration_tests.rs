#![allow(clippy::print_stdout, unused)]
#![cfg(all(test, feature = "postgresql-integration-tests"))]

use crate::sinks::postgresql::config::{
    PostgreSQLConflictsConfig, PostgreSQLFieldConfig, PostgreSQLSchemaConfig, PostgreSQLSinkConfig,
};
use crate::sinks::postgresql::sink::PostgreSQLSink;
use crate::test_util::components::{assert_sink_compliance, SINK_TAGS};
use crate::test_util::{
    generate_events_with_stream, map_event_batch_stream, random_string, trace_init,
};
use chrono::{DateTime, SubsecRound, Utc};
use futures_util::{stream, Stream};
use serde_json::json;
use serde_json::Value as SerdeValue;
use std::env;
use tokio_postgres::{Client, NoTls, Statement};
use vector_lib::btreemap;
use vector_lib::config::LogNamespace;
use vector_lib::event::{
    Event, EventArray, LogEvent, Metric, MetricKind, MetricTags, MetricValue, Value,
};
use vector_lib::finalization::BatchNotifier;
use vector_lib::sink::VectorSink;

fn connection_string() -> String {
    env::var("PG_URL").unwrap_or_else(|_| {
        let host = env::var("PG_HOST").unwrap_or_else(|_| "localhost".into());
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "vector".into());
        let pass = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "vector".into());
        let dbname = env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".into());
        format!("postgres://{user}:{pass}@{host}/{dbname}")
    })
}

fn log_table_schema(table_name: &str) -> Vec<String> {
    vec![
        format!(
            r#"CREATE TABLE IF NOT EXISTS {table_name} (
                event_ts TIMESTAMP WITH TIME ZONE,
                message VARCHAR(100),
                one VARCHAR(80),
                two VARCHAR(80),
                UNIQUE(one, two),
                serialized_json_field TEXT,
                json_field JSONB
            )"#
        ),
        format!("TRUNCATE {table_name}"),
    ]
}

fn metric_table_schema(table_name: &str) -> Vec<String> {
    vec![
        format!(
            r#"CREATE TABLE IF NOT EXISTS {table_name} (
                event_ts TIMESTAMP WITH TIME ZONE,
                name VARCHAR(80),
                tag VARCHAR(80),
                value DOUBLE PRECISION
            )"#
        ),
        format!("TRUNCATE {table_name}"),
    ]
}

fn jsonb_table_schema() -> Vec<String> {
    let table_name = "jsonb_testing";
    vec![
        format!(
            r#"CREATE TABLE IF NOT EXISTS {table_name} (
                my_col JSONB
            )"#
        ),
        format!("TRUNCATE {table_name}"),
    ]
}

async fn test_table_init(schema_sql: Vec<String>) -> Client {
    let config: tokio_postgres::Config = connection_string().parse().unwrap();
    let (client, connection) = config.connect(NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            panic!("postgresql connection error: {e:#?}");
        }
    });

    for sql in schema_sql {
        client.execute(&sql, &[]).await.unwrap();
    }

    client
}

fn log_schema_config(table_name: &str) -> PostgreSQLSchemaConfig {
    PostgreSQLSchemaConfig {
        table: table_name.to_owned(),
        fields: vec![
            PostgreSQLFieldConfig {
                name: "event_ts".to_owned(),
                path: ".timestamp".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "message".to_owned(),
                path: ".message".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "one".to_owned(),
                path: ".nested.one".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "two".to_owned(),
                path: ".nested.two".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "serialized_json_field".to_owned(),
                path: ".serialized_json_field".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "json_field".to_owned(),
                path: ".json_field".to_owned(),
            },
        ],
    }
}

fn metric_schema_config(table_name: &str) -> PostgreSQLSchemaConfig {
    PostgreSQLSchemaConfig {
        table: table_name.to_owned(),
        fields: vec![
            PostgreSQLFieldConfig {
                name: "event_ts".to_owned(),
                path: ".timestamp".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "name".to_owned(),
                path: ".name".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "tag".to_owned(),
                path: ".tags.custom_tag".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "value".to_owned(),
                path: ".counter.value".to_owned(),
            },
        ],
    }
}

fn jsonb_schema_config() -> PostgreSQLSchemaConfig {
    let table_name = "jsonb_testing";

    PostgreSQLSchemaConfig {
        table: table_name.to_owned(),
        fields: vec![PostgreSQLFieldConfig {
            name: "my_col".to_owned(),
            path: ".my_col".to_owned(),
        }],
    }
}

async fn check_log_events_match(psql_client: &Client, table_name: &str, expected: &Vec<Event>) {
    let stmt = format!(
        "SELECT event_ts, message, one, two, serialized_json_field, json_field FROM {table_name}"
    );
    let rows = psql_client.query(&stmt, &[]).await.unwrap();
    assert_eq!(rows.len(), expected.len());

    for (actual, expected) in std::iter::zip(rows, expected) {
        let expected = expected.as_log();
        assert_eq!(
            actual.get::<usize, DateTime<Utc>>(0).round_subsecs(5),
            expected
                .get(".timestamp")
                .unwrap()
                .as_timestamp()
                .unwrap()
                .round_subsecs(5)
        );
        assert_eq!(
            actual.get::<usize, String>(1),
            expected.get(".message").unwrap().as_str().unwrap()
        );
        assert_eq!(
            actual.get::<usize, String>(2),
            expected.get(".nested.one").unwrap().as_str().unwrap()
        );
        assert_eq!(
            actual.get::<usize, String>(3),
            expected.get(".nested.two").unwrap().as_str().unwrap()
        );

        // Postgres should have coerced the JSON field into a text string for storage
        let serialized_json_field = actual.get::<usize, String>(4);
        let expected_serialized_json = serde_json::to_string(&json!(expected
            .value()
            .get(".serialized_json_field")
            .expect(".serialized_json_field should exist in the event")))
        .expect(".serialized_json_field should be serialized JSON");

        assert_eq!(serialized_json_field, expected_serialized_json);

        // Postgres should return the data as a serde Value to be compared to the `Object` value of `.json_field`
        let json_field: SerdeValue = actual.get::<usize, SerdeValue>(5);
        let expected_json_value: SerdeValue = json!(expected
            .value()
            .get(".json_field")
            .expect(".json_field should exist in the event"));

        assert_eq!(json_field, expected_json_value);
    }
}

async fn check_metric_events_match(psql_client: &Client, table_name: &str, expected: &Vec<Event>) {
    let stmt = format!("SELECT event_ts, name, tag, value FROM {table_name}");
    let rows = psql_client.query(&stmt, &[]).await.unwrap();
    assert_eq!(rows.len(), expected.len());

    for (actual, expected) in std::iter::zip(rows, expected) {
        let expected = expected.as_metric();
        assert_eq!(
            actual.get::<usize, DateTime<Utc>>(0).round_subsecs(5),
            expected.timestamp().unwrap().round_subsecs(5)
        );
        assert_eq!(&actual.get::<usize, String>(1), expected.name());
        assert_eq!(
            &actual.get::<usize, String>(2),
            expected.tags().unwrap().get("custom_tag").unwrap()
        );
        match expected.value() {
            MetricValue::Counter { value } => {
                assert_eq!(actual.get::<usize, f64>(3), *value)
            }
            _ => panic!("metric contained an unexpected MetricValue instance"),
        }
    }
}

#[tokio::test]
async fn log_events() {
    let test_table_name = "log_events";

    trace_init();
    let psql_client = test_table_init(log_table_schema(test_table_name)).await;

    let generator = |idx| {
        let mut log_event = LogEvent::from(format!("message {idx}"));
        log_event.insert(".nested.one", random_string(80));
        log_event.insert(".nested.two", random_string(80));
        // Test postgres coercion from JSON to TEXT
        log_event.insert(
            ".serialized_json_field",
            Value::from(btreemap! {
                "hello" => true,
                "num" => 123,
                "serialized_null_is_allowed" => "here's a null \u{0000} character",
                "nested_object" => {
                    btreemap! {
                        "one" => 1
                    }
                }
            }),
        );
        log_event.insert(
            ".json_field",
            Value::from(btreemap! {
                "my_key" => "this is a json object",
                "unicode" => "unicode �� charactres are ok in jsonb, just not nulls",
            }),
        );
        Event::Log(log_event)
    };

    let num_events = 3;
    let (batch, mut _receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = generate_events_with_stream(generator, num_events, Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let config = PostgreSQLSinkConfig {
            connection: connection_string(),
            schema: log_schema_config(test_table_name),
            ..Default::default()
        };
        let sink = PostgreSQLSink::new(config).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .unwrap();

    check_log_events_match(&psql_client, test_table_name, &events).await;
}

#[tokio::test]
async fn log_events_skip_on_conflicts() {
    let test_table_name = "log_events_skip_on_conflicts";

    trace_init();
    let psql_client = test_table_init(log_table_schema(test_table_name)).await;

    let generator = |idx| {
        let mut log_event = LogEvent::from(format!("message {idx}"));
        log_event.insert(".nested.one", "a one");
        log_event.insert(".nested.two", "a two");
        log_event.insert(
            ".serialized_json_field",
            Value::from(btreemap! {
                "hello" => true,
                "num" => 123,
                "nested_object" => {
                    btreemap! {
                        "one" => 1
                    }
                }
            }),
        );
        log_event.insert(
            ".json_field",
            Value::from(btreemap! {
                "my_key" => "this is a json object",
            }),
        );
        Event::Log(log_event)
    };

    let num_events = 3;
    let (batch, mut _receiver) = BatchNotifier::new_with_receiver();
    let (mut events, stream) = generate_events_with_stream(generator, num_events, Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let config = PostgreSQLSinkConfig {
            connection: connection_string(),
            schema: log_schema_config(test_table_name),
            conflicts: Some(PostgreSQLConflictsConfig::Nothing {
                target: vec!["one".to_owned(), "two".to_owned()],
            }),
            ..Default::default()
        };
        let sink = PostgreSQLSink::new(config).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .unwrap();

    events.truncate(1);
    check_log_events_match(&psql_client, test_table_name, &events).await;
}

#[tokio::test]
async fn log_events_update_on_conflicts() {
    let test_table_name = "log_events_update_on_conflicts";

    trace_init();
    let psql_client = test_table_init(log_table_schema(test_table_name)).await;

    let generator = |idx| {
        let mut log_event = LogEvent::from(format!("message {idx}"));
        log_event.insert(".nested.one", "a one");
        log_event.insert(".nested.two", "a two");
        log_event.insert(
            ".serialized_json_field",
            Value::from(btreemap! {
                "hello" => true,
                "num" => 123,
                "nested_object" => {
                    btreemap! {
                        "one" => 1
                    }
                }
            }),
        );
        log_event.insert(
            ".json_field",
            Value::from(btreemap! {
                "my_key" => "this is a json object",
            }),
        );
        Event::Log(log_event)
    };

    let num_events = 3;
    let (batch, mut _receiver) = BatchNotifier::new_with_receiver();
    let (mut events, stream) = generate_events_with_stream(generator, num_events, Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let config = PostgreSQLSinkConfig {
            connection: connection_string(),
            schema: log_schema_config(test_table_name),
            conflicts: Some(PostgreSQLConflictsConfig::Update {
                target: vec!["one".to_owned(), "two".to_owned()],
                fields: vec!["event_ts".to_owned()],
            }),
            ..Default::default()
        };
        let sink = PostgreSQLSink::new(config).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .unwrap();

    events.drain(0..(num_events - 1));
    events
        .first_mut()
        .unwrap()
        .as_mut_log()
        .insert(".message", "message 0");
    check_log_events_match(&psql_client, test_table_name, &events).await;
}

#[tokio::test]
async fn metrics() {
    let test_table_name = "metrics";

    trace_init();
    let psql_client = test_table_init(metric_table_schema(test_table_name)).await;

    let num_events = 5;
    let (batch, mut _receiver) = BatchNotifier::new_with_receiver();
    let events: Vec<_> = (0..num_events)
        .map(|index| {
            let mut metric = Metric::new(
                format!("counter_{}", index),
                MetricKind::Absolute,
                MetricValue::Counter {
                    value: index as f64,
                },
            );
            let tags = MetricTags::from([("custom_tag".to_owned(), format!("one_{index}"))]);
            let metric = metric
                .with_timestamp(Some(Utc::now()))
                .with_tags(Some(tags));
            Event::Metric(metric)
        })
        .collect();
    let stream = map_event_batch_stream(stream::iter(events.clone()), Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let config = PostgreSQLSinkConfig {
            connection: connection_string(),
            schema: metric_schema_config(test_table_name),
            ..Default::default()
        };
        let sink = PostgreSQLSink::new(config).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .unwrap();

    check_metric_events_match(&psql_client, test_table_name, &events).await;
}

#[tokio::test]
async fn jsonb_value_storage() {
    let events: Vec<Event> = vec![
        Event::from_json_value(
            json! ({
                "name": "regular string",
                "my_col": json!({"my_key": "a regular string"}),
                "expected": "a regular string"
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name": "string with unicode characters",
                "my_col": json!({"my_key": "normal unicode string ��"}),
                "expected": "normal unicode string ��"
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name": "string with unicode nulls",
                "my_col": json!({"my_key": "a regular string \u{0000}with\u{0000} nulls"}),
                "expected": "a regular string with nulls"
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "integer",
                "my_col": json!({"my_key": 3587}),
                "expected": 3587
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "boolean",
                "my_col": json!({"my_key": true}),
                "expected": true
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "float",
                "my_col": json!({"my_key": 123.456}),
                "expected": 123.456
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "nested object with string keys and no bad values",
                "my_col": json!({"my_key": json!({"hello": true, "num": 123})}),
                "expected": json!({"hello": true, "num": 123})
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "nested object with unicode keys and no bad values",
                "my_col": json!({"my_key": json!({"hello �": true, "num": 123})}),
                "expected": json!({"hello �": true, "num": 123})
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "nested object with unicode nulls in the values",
                "my_col": json!({"my_key": json!({"hello": "goodbye null\u{0000}, nice knowing\u{0000} you"})}),
                "expected": json!({"hello": "goodbye null, nice knowing you"})
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "nested array with unicode nulls",
                "my_col": json!({"my_key": json!({"arr": ["goodbye �� null\u{0000}, nice knowing\u{0000} you"]})}),
                "expected": json!({"arr": ["goodbye �� null, nice knowing you"]})
            }),
            LogNamespace::default(),
        )
        .expect("event"),
        Event::from_json_value(
            json! ({
                "name" : "array with unicode nulls",
                "my_col": json!({"my_key": json!(["goodbye �� null\u{0000}, nice knowing\u{0000} you"])}),
                "expected": json!(["goodbye �� null, nice knowing you"])
            }),
            LogNamespace::default(),
        )
        .expect("event"),
    ];

    trace_init();
    let psql_client = &test_table_init(jsonb_table_schema()).await;

    let (batch, mut _receiver) = BatchNotifier::new_with_receiver();
    let stream = map_event_batch_stream(stream::iter(events.clone()), Some(batch));

    assert_sink_compliance(&SINK_TAGS, async move {
        let config = PostgreSQLSinkConfig {
            connection: connection_string(),
            schema: jsonb_schema_config(),
            ..Default::default()
        };
        let sink = PostgreSQLSink::new(config).unwrap();
        let sink = VectorSink::from_event_streamsink(sink);
        sink.run(stream).await
    })
    .await
    .unwrap();

    let stmt = format!("SELECT my_col FROM jsonb_testing");
    let rows = psql_client.query(&stmt, &[]).await.unwrap();

    // Grab all values AS JSON from the DB so that we can retain the types for comparison.
    // Use SerdeValue to handle typing for retrieving the column's value.
    for (row, event) in std::iter::zip(rows, events) {
        let log = event.as_log();
        let value = log.get(".expected").unwrap();
        let expected: SerdeValue = json!({"my_key": value});
        assert_eq!(
            row.get::<usize, SerdeValue>(0),
            expected,
            "{}",
            format!(
                "Test case not found in the DB: {:?}",
                log.get(".name").unwrap().to_string_lossy()
            )
        );
    }
}
