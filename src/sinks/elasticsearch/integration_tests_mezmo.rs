// These tests have been added by the Mezmo team to test specifics with reshaping the body
// of the event to be based on the `message` property.
#![cfg(all(test, feature = "es-integration-tests"))]

use assay::assay;
use futures::{future::ready, stream};
use serde_json::{Value, json};
use vector_lib::event::{BatchNotifier, BatchStatus, Event, LogEvent};

// Shared integration testing structures (made `pub` by Mezmo devs)
use super::integration_tests::{batch_settings, flush, gen_index, http_server};
// Certain impls and structs at the top level
use super::{BulkConfig, ElasticsearchCommon, ElasticsearchConfig};

use crate::{
    config::{SinkConfig, SinkContext},
    sinks::util::Compression,
    test_util::components::{HTTP_SINK_TAGS, run_and_assert_sink_compliance},
};

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn elasticsearch_mezmo_message_reshaping_happens() {
    let index = gen_index();
    let config = ElasticsearchConfig {
        endpoints: vec![http_server()],
        bulk: BulkConfig {
            index: index.clone(),
            ..Default::default()
        },
        doc_type: "log_lines".into(),
        id_key: Some("my_id".into()),
        compression: Compression::None,
        batch: batch_settings(),
        ..Default::default()
    };
    let common = ElasticsearchCommon::parse_single(&config)
        .await
        .expect("Config error");
    let base_url = common.base_url.clone();

    let cx = SinkContext::default();
    let (sink, _hc) = config.build(cx.clone()).await.unwrap();

    let (batch, mut receiver) = BatchNotifier::new_with_receiver();
    let mut input_event = LogEvent::from("raw log line").with_batch_notifier(&batch);
    input_event.insert("my_id", "42"); // This `id` will be stored prior to trashing the propery
    input_event.insert("foo", "bar"); // Will be trashed
    input_event.insert("message.one", "hello");
    input_event.insert("message.two", 2);
    input_event.insert("message.three.embed", true);
    drop(batch);

    run_and_assert_sink_compliance(
        sink,
        stream::once(ready(Event::from(input_event))),
        &HTTP_SINK_TAGS,
    )
    .await;

    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // make sure writes all all visible
    flush(common).await.unwrap();

    let response = reqwest::Client::new()
        .get(format!("{base_url}/{index}/_search"))
        .json(&json!({
            "query": { "query_string": { "query": "*" } }
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let total = response["hits"]["total"]
        .as_u64()
        .or_else(|| response["hits"]["total"]["value"].as_u64())
        .expect("Elasticsearch response does not include hits->total nor hits->total->value");
    assert_eq!(1, total);

    let hits = response["hits"]["hits"]
        .as_array()
        .expect("Elasticsearch response does not include hits->hits");

    let hit = hits.iter().next().unwrap();
    assert_eq!("42", hit["_id"]);

    let value = hit
        .get("_source")
        .expect("Elasticsearch hit missing _source");
    assert_eq!(None, value["my_id"].as_str());

    let expected = json!({
        "one": "hello",
        "two": 2,
        "three": {"embed": true},
    });
    assert_eq!(&expected, value, "Message reshaping was done correctly");
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn elasticsearch_mezmo_message_reshaping_does_not_happen() {
    let index = gen_index();
    let config = ElasticsearchConfig {
        endpoints: vec![http_server()],
        bulk: BulkConfig {
            index: index.clone(),
            ..Default::default()
        },
        doc_type: "log_lines".into(),
        id_key: Some("my_id".into()),
        compression: Compression::None,
        batch: batch_settings(),
        ..Default::default()
    };
    let common = ElasticsearchCommon::parse_single(&config)
        .await
        .expect("Config error");
    let base_url = common.base_url.clone();

    let cx = SinkContext::default();
    let (sink, _hc) = config.build(cx.clone()).await.unwrap();

    let (batch, mut receiver) = BatchNotifier::new_with_receiver();
    let mut input_event = LogEvent::default().with_batch_notifier(&batch);
    input_event.insert("my_id", "42");
    input_event.insert("foo", "bar");
    input_event.insert("message.one", "hello");
    drop(batch);

    run_and_assert_sink_compliance(
        sink,
        stream::once(ready(Event::from(input_event))),
        &HTTP_SINK_TAGS,
    )
    .await;

    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // make sure writes all all visible
    flush(common).await.unwrap();

    let response = reqwest::Client::new()
        .get(format!("{base_url}/{index}/_search"))
        .json(&json!({
            "query": { "query_string": { "query": "*" } }
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let total = response["hits"]["total"]
        .as_u64()
        .or_else(|| response["hits"]["total"]["value"].as_u64())
        .expect("Elasticsearch response does not include hits->total nor hits->total->value");
    assert_eq!(1, total);

    let hits = response["hits"]["hits"]
        .as_array()
        .expect("Elasticsearch response does not include hits->hits");

    let hit = hits.iter().next().unwrap();
    assert_eq!("42", hit["_id"]);

    let value = hit
        .get("_source")
        .expect("Elasticsearch hit missing _source");
    assert_eq!(None, value["my_id"].as_str());

    let expected = json!({
        "foo": "bar",
        "message": {
            "one": "hello",
        }
    });
    assert_eq!(
        &expected, value,
        "Message reshaping was skipped because env is not set"
    );
}
