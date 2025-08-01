use crate::{
    codecs::EncodingConfig,
    config::{SinkConfig, SinkContext},
    mezmo::reshape_log_event_by_message,
    sinks::{
        splunk_hec::{
            common::{
                integration_test_helpers::{get_token, splunk_api_address, splunk_hec_address},
                EndpointTarget,
            },
            logs::config::HecLogsSinkConfig,
        },
        util::{BatchConfig, Compression, TowerRequestConfig},
    },
    test_util::{
        components::{run_and_assert_sink_compliance, HTTP_SINK_TAGS},
        random_message_object_events_with_stream,
    },
};
use assay::assay;
use serde_json::Value as JsonValue;
use tokio::time::{sleep, Duration};
use vector_lib::codecs::{
    encoding::format::JsonSerializerOptions, JsonSerializerConfig, MetricTagValues,
};
use vector_lib::lookup::lookup_v2::ConfigValuePath;

const USERNAME: &str = "admin";
const PASSWORD: &str = "password";

async fn recent_entries(index: Option<&str>) -> Vec<JsonValue> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // https://docs.splunk.com/Documentation/Splunk/7.2.1/RESTREF/RESTsearch#search.2Fjobs
    let search_query = match index {
        Some(index) => format!("search index={}", index),
        None => "search index=*".into(),
    };
    let res = client
        .post(format!(
            "{}/services/search/jobs?output_mode=json",
            splunk_api_address()
        ))
        .form(&vec![
            ("search", &search_query[..]),
            ("exec_mode", "oneshot"),
            ("f", "*"),
        ])
        .basic_auth(USERNAME, Some(PASSWORD))
        .send()
        .await
        .unwrap();
    let json: JsonValue = res.json().await.unwrap();

    json["results"].as_array().unwrap().clone()
}

// It usually takes ~1 second for the event to show up in search, so poll until
// we see it.
async fn find_entries(events: &[JsonValue]) -> bool {
    let mut found_all = false;
    for _ in 0..20 {
        let entries = recent_entries(None).await;
        found_all = events.iter().all(|message| {
            entries.iter().any(|entry| {
                let event_json =
                    serde_json::from_str::<serde_json::Value>(entry["_raw"].as_str().unwrap())
                        .unwrap();
                &event_json == message
            })
        });

        if found_all {
            break;
        }

        sleep(Duration::from_millis(200)).await;
    }
    found_all
}

async fn config(
    encoding: EncodingConfig,
    indexed_fields: Vec<ConfigValuePath>,
) -> HecLogsSinkConfig {
    let mut batch = BatchConfig::default();
    batch.max_events = Some(5);

    HecLogsSinkConfig {
        default_token: get_token().await.into(),
        endpoint: splunk_hec_address(),
        host_key: None,
        indexed_fields,
        index: None,
        sourcetype: None,
        source: None,
        encoding,
        compression: Compression::None,
        batch,
        request: TowerRequestConfig::default(),
        tls: None,
        acknowledgements: Default::default(),
        timestamp_nanos_key: None,
        timestamp_key: Default::default(),
        auto_extract_timestamp: None,
        endpoint_target: EndpointTarget::Event,
    }
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn splunk_mezmo_does_not_reshape_messages() {
    let cx = SinkContext::new_test();

    let config = config(
        JsonSerializerConfig::new(MetricTagValues::Single, JsonSerializerOptions::default()).into(),
        vec![],
    )
    .await;
    let (sink, _) = config.build(cx).await.unwrap();

    let (events, stream) = random_message_object_events_with_stream(100, 3, None);
    run_and_assert_sink_compliance(sink, stream, &HTTP_SINK_TAGS).await;

    let expected = events
        .iter()
        .map(|e| serde_json::to_value(e.as_log()).unwrap())
        .collect::<Vec<_>>();

    assert!(
        find_entries(expected.as_slice()).await,
        "As expected, events were NOT reshaped"
    );
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn splunk_mezmo_should_reshape_messages() {
    let cx = SinkContext::new_test();

    let config = config(
        JsonSerializerConfig::new(MetricTagValues::Single, JsonSerializerOptions::default()).into(),
        vec![],
    )
    .await;
    let (sink, _) = config.build(cx).await.unwrap();

    let (mut events, stream) = random_message_object_events_with_stream(100, 3, None);
    run_and_assert_sink_compliance(sink, stream, &HTTP_SINK_TAGS).await;

    let expected = events
        .iter_mut()
        .map(|e| {
            reshape_log_event_by_message(e.as_mut_log());
            serde_json::to_value(e.as_log()).unwrap()
        })
        .collect::<Vec<_>>();

    assert!(
        find_entries(expected.as_slice()).await,
        "As expected, events were NOT reshaped"
    );
}
