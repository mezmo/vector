// These tests have been added by the Mezmo team to test specifics with reshaping the body
// of the event to be based on the `message` property.
#![cfg(all(test, feature = "azure-blob-integration-tests"))]

use crate::template::Template;
use crate::test_util::{random_message_object_events_with_stream, random_string};
use assay::assay;
use vector_lib::codecs::{JsonSerializerConfig, MetricTagValues, NewlineDelimitedEncoderConfig};

// Use THEIR implementation - reduces code copying
use super::integration_tests::AzureBlobSinkConfig;
use crate::mezmo::reshape_log_event_by_message;

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn azure_blob_mezmo_message_reshaping_does_not_happen() {
    let blob_prefix = format!("json/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: Template::try_from(blob_prefix.clone()).unwrap(),
        encoding: (
            Some(NewlineDelimitedEncoderConfig::new()),
            JsonSerializerConfig::new(MetricTagValues::Single),
        )
            .into(),
        ..config
    };
    let (events, stream) = random_message_object_events_with_stream(100, 10, None);

    config.run_assert(stream).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(blob.properties.content_type, String::from("text/plain"));
    let expected = events
        .iter()
        .map(|event| serde_json::to_string(&event.as_log()).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        expected, blob_lines,
        "Events were not reshaped because the env is not set correctly"
    );
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn azure_blob_mezmo_message_reshaping_happens() {
    let blob_prefix = format!("json/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: Template::try_from(blob_prefix.clone()).unwrap(),
        encoding: (
            Some(NewlineDelimitedEncoderConfig::new()),
            JsonSerializerConfig::new(MetricTagValues::Single),
        )
            .into(),
        ..config
    };
    let (mut events, stream) = random_message_object_events_with_stream(100, 3, None);

    config.run_assert(stream).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(blob.properties.content_type, String::from("text/plain"));
    let expected = events
        .iter_mut()
        .map(|event| {
            reshape_log_event_by_message(event.as_mut_log());
            serde_json::to_string(&event.as_log()).unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(expected, blob_lines, "Events were properly reshaped");
}
