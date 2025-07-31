// These tests have been added by the Mezmo team to test specifics with reshaping the body
// of the event to be based on the `message` property.
#![cfg(all(test, feature = "azure-blob-integration-tests"))]

use bytes::{Bytes, BytesMut};
use flate2::read::GzEncoder;
use flate2::Compression;
use std::io::Read;

use crate::template::Template;
use crate::test_util::{random_message_object_events_with_stream, random_string};
use assay::assay;
use std::{collections::BTreeMap, thread, time};
use vector_lib::codecs::{
    encoding::format::JsonSerializerOptions, JsonSerializerConfig, MetricTagValues,
    NewlineDelimitedEncoderConfig,
};

// Use THEIR implementation - reduces code copying
use super::integration_tests::AzureBlobSinkConfig;
use crate::mezmo::reshape_log_event_by_message;

use super::file_consolidation_processor::{get_files_to_consolidate, FileConsolidationProcessor};
use super::file_consolidator_async::{FileConsolidationConfig, FileConsolidatorAsync};

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn azure_blob_mezmo_message_reshaping_does_not_happen() {
    let config = get_test_config(None, None).await;
    let (events, stream) = random_message_object_events_with_stream(100, 10, None);

    config.run_assert(stream).await;

    let blobs = config.list_blobs(config.blob_prefix.to_string()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(
        blob.properties.content_type,
        String::from("application/x-ndjson")
    );
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
    let config = get_test_config(None, None).await;
    let (mut events, stream) = random_message_object_events_with_stream(100, 3, None);

    config.run_assert(stream).await;

    let blobs = config.list_blobs(config.blob_prefix.to_string()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(
        blob.properties.content_type,
        String::from("application/x-ndjson")
    );
    let expected = events
        .iter_mut()
        .map(|event| {
            reshape_log_event_by_message(event.as_mut_log());
            serde_json::to_string(&event.as_log()).unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(expected, blob_lines, "Events were properly reshaped");
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "2"),
    ]
  )]
async fn azure_blob_mezmo_tags_are_added() {
    let mut tags = BTreeMap::new();
    tags.insert("tag1".to_string(), "value of tag1".to_string());
    tags.insert("tag2".to_string(), "value of tag2".to_string());

    let config = get_test_config(None, Some(tags)).await;
    let (_, stream) = random_message_object_events_with_stream(100, 3, None);

    config.run_assert(stream).await;

    let blobs = config.list_blobs(config.blob_prefix.to_string()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));

    let tags = config.get_tags(blobs[0].clone()).await;
    assert_eq!(tags.tag_set.tags.len(), 2);

    assert_eq!(
        vec!["tag1".to_owned(), "tag2".to_owned()],
        tags.tag_set
            .tags
            .iter()
            .map(|t| t.key.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec!["value of tag1".to_owned(), "value of tag2".to_owned()],
        tags.tag_set
            .tags
            .iter()
            .map(|t| t.value.clone())
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn azure_file_consolidator_disabled_run() {
    // testing the default scenario where the consolidator is disabled
    let mut fc: FileConsolidatorAsync = Default::default();

    let started = fc.start();
    assert!(!started, "started false");

    thread::sleep(time::Duration::from_millis(500));

    let stopped = fc.stop();
    assert!(!stopped, "stopped false");
}

#[tokio::test]
async fn azure_file_consolidator_enabled_run() {
    let config = get_test_config(None, None).await;

    let mut fc = FileConsolidatorAsync::new(
        config.connection_string,
        config.container_name,
        FileConsolidationConfig {
            enabled: true,
            process_every_ms: 10,
            requested_size_bytes: 512000000,
            base_path: "".to_owned(),
            output_format: "ndjson".to_owned(),
        },
    );

    let started = fc.start();
    assert!(started, "started true");

    thread::sleep(time::Duration::from_millis(500));

    let stopped = fc.stop();
    assert!(stopped, "stopped true");
}

#[tokio::test]
async fn azure_file_consolidation_process_no_files() {
    let config = get_test_config(None, None).await;
    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = config.blob_prefix.clone().to_string();
    let requested_size_bytes = 512000000;
    let output_format = "ndjson".to_owned();

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name,
        base_path,
        requested_size_bytes,
        output_format,
    );
    fcp.run().await;
}

#[tokio::test]
async fn azure_file_consolidation_process_files_no_tags() {
    let config = get_test_config(None, None).await;
    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = config.blob_prefix.clone().to_string();
    let requested_size_bytes = 512000000;
    let output_format = "ndjson".to_owned();

    for _ in 0..3 {
        let (_, stream) = random_message_object_events_with_stream(100, 3, None);
        config.run_assert(stream).await;
    }

    // three files
    let blobs = config.list_blobs(config.blob_prefix.to_string()).await;
    assert_eq!(blobs.len(), 3);

    // no files found for consolidation
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        config.blob_prefix.clone().to_string(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name,
        base_path.clone(),
        requested_size_bytes,
        output_format,
    );
    fcp.run().await;

    // the files haven't changed
    let blobs = config.list_blobs(config.blob_prefix.to_string()).await;
    assert_eq!(blobs.len(), 3);
}

#[tokio::test]
async fn azure_file_consolidation_process_tag_filters() {
    let config = get_test_config(None, None).await;
    let client = config.get_client().await;
    let container_name = config.container_name;
    let prefix = format!("json/into/blob/{}/", random_string(10));

    let mut formats: BTreeMap<String, usize> = BTreeMap::new();
    formats.insert("ndjson".to_owned(), 5);
    formats.insert("json".to_owned(), 7);
    formats.insert("text".to_owned(), 3);
    formats.insert("something".to_owned(), 1);

    // create 3 files of each type
    for f in formats.keys() {
        let mut tags_data: BTreeMap<String, String> = BTreeMap::new();
        tags_data.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags_data.insert("mezmo_pipeline_azure_type".to_owned(), f.to_string());
        let tags = Some(tags_data);

        let config = get_test_config(Some(prefix.clone()), tags).await;
        for _ in 0..formats[f] {
            let (_, stream) = random_message_object_events_with_stream(100, 1, None);
            config.run_assert(stream).await;
        }
    }

    for f in formats.keys() {
        let files =
            get_files_to_consolidate(&client, container_name.clone(), prefix.clone(), f.clone())
                .await
                .unwrap();
        assert_eq!(files.len(), formats[f]);
    }
}

#[tokio::test]
async fn azure_file_consolidation_process_text_files() {
    let prefix = format!("json/into/blob/{}/text-files/", random_string(10));
    let config = get_test_config(Some(prefix.clone()), None).await;
    let output_format = "text".to_owned();

    let lines = vec![
        "this is some plaintext",
        "written to individual files",
        "for the purpose",
        "of unit testing",
        "successful processes",
    ];

    for line in lines.iter() {
        let mut tags: BTreeMap<String, String> = BTreeMap::new();
        tags.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags.insert(
            "mezmo_pipeline_azure_type".to_owned(),
            output_format.clone(),
        );

        let filename = format!("{}{}.log", prefix.clone(), random_string(10));
        let data = line.as_bytes();
        config
            .put_blob(filename, "text/plain", "None", Some(tags), data.into())
            .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = prefix.clone();
    let requested_size_bytes = 512000000;

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 5);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name.clone(),
        base_path.clone(),
        requested_size_bytes,
        output_format.clone(),
    );
    fcp.run().await;

    // no more files should be found
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        prefix.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    // list the files
    let blobs = config.list_blobs(prefix.clone()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    assert!(blobs[0].clone().contains("merged"));

    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(blob.properties.content_type, String::from("text/plain"));
    assert_eq!(blob_lines.len(), lines.len());
    assert_eq!(blob_lines, lines);
}

#[tokio::test]
async fn azure_file_consolidation_process_json_files() {
    let prefix = format!("json/into/blob/{}/json-files/", random_string(10));
    let config = get_test_config(Some(prefix.clone()), None).await;

    let lines = vec![
        "[{ \"message\": \"this is some json\" }]",
        "[{ \"message\": \"written to individual files\" }]",
        "[{ \"message\": \"for the purpose\" }]",
        "[{ \"message\": \"of unit testing\" }]",
        "[{ \"message\": \"successful processes\" }]",
    ];

    for line in lines.iter() {
        let mut tags: BTreeMap<String, String> = BTreeMap::new();
        tags.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags.insert("mezmo_pipeline_azure_type".to_owned(), "json".to_string());

        let filename = format!("{}{}.log", prefix.clone(), random_string(10));
        let data = line.as_bytes();
        config
            .put_blob(
                filename,
                "application/json",
                "None",
                Some(tags),
                data.into(),
            )
            .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = prefix.clone();
    let requested_size_bytes = 512000000;
    let output_format = "json".to_owned();

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 5);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name.clone(),
        base_path.clone(),
        requested_size_bytes,
        output_format.clone(),
    );
    fcp.run().await;

    // no more files should be found
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        prefix.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    // list the files
    let blobs = config.list_blobs(prefix.clone()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    assert!(blobs[0].clone().contains("merged"));

    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(
        blob.properties.content_type,
        String::from("application/json")
    );

    // all the json files concatinated together as a new json string
    assert_eq!(blob_lines.len(), 1);
    assert_eq!(
        blob_lines[0],
        "[{ \"message\": \"this is some json\" },{ \"message\": \"written to individual files\" },{ \"message\": \"for the purpose\" },{ \"message\": \"of unit testing\" },{ \"message\": \"successful processes\" }]",
    );
}

#[tokio::test]
async fn azure_file_consolidation_process_ndjson_files() {
    let prefix = format!("json/into/blob/{}/ndjson-files/", random_string(10));
    let config = get_test_config(Some(prefix.clone()), None).await;

    let lines = vec![
        "{\"message\": \"this is some json\"}",
        "{\"message\": \"that is building\"}",
        "{\"message\": \"individual files\"}",
        "{\"message\": \"that may have some data\"}",
        "{\"message\": \"for testing purposes\"}",
    ];

    for line in lines.iter() {
        let mut tags: BTreeMap<String, String> = BTreeMap::new();
        tags.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags.insert("mezmo_pipeline_azure_type".to_owned(), "ndjson".to_string());

        let filename = format!("{}{}.log", prefix.clone(), random_string(10));
        let data = line.as_bytes();
        config
            .put_blob(
                filename,
                "application/x-ndjson",
                "None",
                Some(tags),
                data.into(),
            )
            .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = prefix.clone();
    let requested_size_bytes = 512000000;
    let output_format = "ndjson".to_owned();

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 5);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name.clone(),
        base_path.clone(),
        requested_size_bytes,
        output_format.clone(),
    );
    fcp.run().await;

    // no more files should be found
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        prefix.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    // list the files
    let blobs = config.list_blobs(prefix.clone()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    assert!(blobs[0].clone().contains("merged"));

    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(
        blob.properties.content_type,
        String::from("application/x-ndjson")
    );

    // all the json lines concatinated together as newlines
    assert_eq!(blob_lines.len(), 5);
    assert_eq!(blob_lines, lines);
}

#[tokio::test]
async fn azure_file_consolidation_process_compressed_files() {
    let prefix = format!("json/into/blob/{}/compressed-files/", random_string(10));
    let config = get_test_config(Some(prefix.clone()), None).await;
    let output_format = "text".to_owned();

    let lines = vec![
        "this is some text",
        "that will be compressed",
        "so we prove that gzip",
        "files are handled appropriately",
        "when merged together",
    ];

    for line in lines.iter() {
        let mut tags: BTreeMap<String, String> = BTreeMap::new();
        tags.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags.insert(
            "mezmo_pipeline_azure_type".to_owned(),
            output_format.clone(),
        );

        let filename = format!("{}{}.log.gz", prefix.clone(), random_string(10));

        // compress the data
        let mut ret_vec = [0; 100];
        let mut bytestring = line.as_bytes();
        let mut gz = GzEncoder::new(&mut bytestring, Compression::fast());
        let count = gz.read(&mut ret_vec).unwrap();
        let vec = ret_vec[0..count].to_vec();

        let mut bytes_mut = BytesMut::with_capacity(0);
        bytes_mut.extend_from_slice(&vec);
        let data = Bytes::from(bytes_mut);

        config
            .put_blob(filename, "text/plain", "gzip", Some(tags), data.into())
            .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = prefix.clone();
    let requested_size_bytes = 512000000;

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 5);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name.clone(),
        base_path.clone(),
        requested_size_bytes,
        output_format.clone(),
    );
    fcp.run().await;

    // no more files should be found
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        prefix.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    // list the files
    let blobs = config.list_blobs(prefix.clone()).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    assert!(blobs[0].clone().contains("merged"));

    let (blob, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(blob.properties.content_type, String::from("text/plain"));

    // all the json lines concatinated together as newlines
    assert_eq!(blob_lines.len(), 5);
    assert_eq!(blob_lines, lines);
}

#[tokio::test]
async fn azure_file_consolidation_process_file_size_limits() {
    let prefix = format!("json/into/blob/{}/max-file-sizes/", random_string(10));
    let config = get_test_config(Some(prefix.clone()), None).await;
    let output_format = "text".to_owned();

    // create 10x 10 KB files
    for _ in 0..10 {
        let hundred_bytes = random_string(100);

        let mut five_mb = BytesMut::new();
        for _ in 0..100 {
            five_mb.extend_from_slice(hundred_bytes.as_bytes());
            five_mb.extend_from_slice(b"\n");
        }

        let data = Bytes::from(five_mb.to_vec());

        let mut tags: BTreeMap<String, String> = BTreeMap::new();
        tags.insert("mezmo_pipeline_azure_sink".to_owned(), "true".to_owned());
        tags.insert(
            "mezmo_pipeline_azure_type".to_owned(),
            output_format.clone(),
        );

        let filename = format!("{}{}.log", prefix.clone(), random_string(10));
        config
            .put_blob(filename, "text/plain", "None", Some(tags), data.into())
            .await;
    }

    let client = config.get_client().await;
    let container_name = config.container_name.clone();
    let base_path = prefix.clone();
    let requested_size_bytes = 40000; // 40 KB

    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        base_path.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 10);

    let fcp = FileConsolidationProcessor::new(
        &client,
        container_name.clone(),
        base_path.clone(),
        requested_size_bytes,
        output_format.clone(),
    );
    fcp.run().await;

    // no more files should be found
    let files = get_files_to_consolidate(
        &client,
        container_name.clone(),
        prefix.clone(),
        output_format.clone(),
    )
    .await
    .unwrap();
    assert_eq!(files.len(), 0);

    // list the files
    let blobs = config.list_blobs(prefix.clone()).await;
    assert_eq!(blobs.len(), 3);

    let mut lines_count = 0;
    for b in blobs {
        assert!(b.clone().ends_with(".log"));
        assert!(b.clone().contains("merged"));

        let (blob, blob_lines) = config.get_blob(b.clone()).await;
        assert_eq!(blob.properties.content_type, String::from("text/plain"));

        lines_count += blob_lines.len();
    }

    // each file plus newlines between
    assert_eq!(lines_count, 1007);
}

async fn get_test_config(
    prefix: Option<String>,
    tags: Option<BTreeMap<String, String>>,
) -> AzureBlobSinkConfig {
    let blob_prefix = if let Some(..) = prefix {
        prefix.unwrap()
    } else {
        format!("unittest/{}/", random_string(10))
    };
    let config = AzureBlobSinkConfig::new_emulator().await;
    AzureBlobSinkConfig {
        blob_prefix: Template::try_from(blob_prefix.clone()).unwrap(),
        encoding: (
            Some(NewlineDelimitedEncoderConfig::new()),
            JsonSerializerConfig::new(MetricTagValues::Single, JsonSerializerOptions::default()),
        )
            .into(),
        tags,
        ..config
    }
}
