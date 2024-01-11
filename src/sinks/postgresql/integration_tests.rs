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
use std::env;
use tokio_postgres::{Client, NoTls};
use vector_common::finalization::BatchNotifier;
use vector_lib::event::{Event, EventArray, LogEvent, Metric, MetricKind, MetricTags, MetricValue};
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
                UNIQUE(one, two)
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

async fn check_log_events_match(psql_client: &Client, table_name: &str, expected: &Vec<Event>) {
    let stmt = format!("SELECT event_ts, message, one, two FROM {table_name}");
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
