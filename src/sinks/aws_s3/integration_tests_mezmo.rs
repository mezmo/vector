#![cfg(all(test, feature = "aws-s3-integration-tests"))]

use crate::mezmo::reshape_log_event_by_message;
use crate::tls::TlsConfig;
use assay::assay;
use bytes::{Bytes, BytesMut};
use futures::Stream;
use regex::Regex;
use similar_asserts::assert_eq;
use tokio_stream::StreamExt;
use vector_lib::codecs::{
    decoding::format::{Deserializer, JsonDeserializerConfig, JsonDeserializerOptions},
    encoding::FramingConfig,
    JsonSerializerConfig, MetricTagValues,
};
use vector_lib::config::proxy::ProxyConfig;
use vector_lib::config::LogNamespace;
use vector_lib::event::{BatchNotifier, BatchStatus, BatchStatusReceiver, Event, EventArray};

use super::S3SinkConfig;
use crate::{
    aws::{AwsAuthentication, RegionOrEndpoint},
    config::SinkContext,
    sinks::{
        s3_common::config::S3Options,
        util::{BatchConfig, Compression, TowerRequestConfig},
    },
    test_util::{
        components::{run_and_assert_sink_compliance, AWS_SINK_TAGS},
        random_message_object_events_with_stream, random_string,
    },
};

use super::file_consolidation_processor::{get_files_to_consolidate, FileConsolidationProcessor};
use super::file_consolidator_async::{FileConsolidationConfig, FileConsolidatorAsync};
use super::integration_tests::{
    client, create_bucket, get_keys, get_lines, get_object, s3_address,
};
use aws_sdk_s3::types::ByteStream;
use flate2::read::GzEncoder;
use std::io::Read;
use std::{thread, time};

fn is_merged_file(file: &str) -> bool {
    let merged_filename_regex: Regex = Regex::new(r"^.*merged_\d+.log$").unwrap();
    merged_filename_regex.is_match(file)
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "1"),
    ]
  )]
async fn s3_message_objects_are_reshaped() {
    let cx = SinkContext::new_test();

    let bucket = uuid::Uuid::new_v4().to_string();

    create_bucket(&bucket, false).await;

    let mut config = json_config(&bucket, 1000000);
    config.key_prefix = "test-prefix".to_string();
    let prefix = config.key_prefix.clone();
    let service = config.create_service(&cx.globals.proxy).await.unwrap();
    let sink = config.build_processor(service, cx).unwrap();

    let (mut generated_events, stream, receiver) = make_object_events_batch(100, 10);

    run_and_assert_sink_compliance(sink, stream, &AWS_SINK_TAGS).await;
    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = get_keys(&bucket, prefix).await;
    assert_eq!(keys.len(), 1);

    let key = keys[0].clone();
    let key_parts = key.split('/');
    assert!(key_parts.count() == 1);
    assert!(key.starts_with("test-prefix"));
    assert!(key.ends_with(".log"));

    let obj = get_object(&bucket, key).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));

    let response_lines = get_lines(obj).await;
    let input = Bytes::from(response_lines[0].clone());
    let deserializer = JsonDeserializerConfig::new(JsonDeserializerOptions::default()).build();
    let got_events = deserializer.parse(input, LogNamespace::Vector).unwrap();

    // Loop to assert results for 2 reasons:
    // 1) SmallVec (AWS results) cannot be `assert_eq!` compared with Vec (generated events)
    // 2) We need to reshape the generated events for comparison

    for (idx, event) in got_events.iter().enumerate() {
        reshape_log_event_by_message(generated_events[idx].as_mut_log());
        assert_eq!(
            event, &generated_events[idx],
            "Event at position {} is correct",
            idx
        );
    }
}

#[assay(
    env = [
      ("MEZMO_RESHAPE_MESSAGE", "0"),
    ]
  )]
async fn s3_message_objects_not_reshaped_because_of_env() {
    let cx = SinkContext::new_test();

    let bucket = uuid::Uuid::new_v4().to_string();

    create_bucket(&bucket, false).await;

    let mut config = json_config(&bucket, 1000000);
    config.key_prefix = "test-prefix".to_string();
    let prefix = config.key_prefix.clone();
    let service = config.create_service(&cx.globals.proxy).await.unwrap();
    let sink = config.build_processor(service, cx).unwrap();

    let (generated_events, stream, receiver) = make_object_events_batch(100, 3);

    run_and_assert_sink_compliance(sink, stream, &AWS_SINK_TAGS).await;
    assert_eq!(receiver.await, BatchStatus::Delivered);

    let keys = get_keys(&bucket, prefix).await;
    assert_eq!(keys.len(), 1);

    let key = keys[0].clone();
    let key_parts = key.split('/');
    assert!(key_parts.count() == 1);
    assert!(key.starts_with("test-prefix"));
    assert!(key.ends_with(".log"));

    let obj = get_object(&bucket, key).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));

    let response_lines = get_lines(obj).await;
    let input = Bytes::from(response_lines[0].clone());
    let deserializer = JsonDeserializerConfig::new(JsonDeserializerOptions::default()).build();
    let got_events = deserializer.parse(input, LogNamespace::Vector).unwrap();

    // The `message` property should still exist

    for (idx, event) in got_events.iter().enumerate() {
        assert_eq!(
            event, &generated_events[idx],
            "Event at position {} is correct",
            idx
        );
    }
}

#[tokio::test]
async fn s3_file_consolidator_enabled_run() {
    let _cx = SinkContext::new_test();
    let bucket = uuid::Uuid::new_v4().to_string();

    create_bucket(&bucket, false).await;

    let auth = AwsAuthentication::test_auth();
    let region = RegionOrEndpoint::with_both("minio", s3_address());
    let proxy = ProxyConfig::default();
    let tls_options: Option<TlsConfig> = None;

    let mut fc = FileConsolidatorAsync::new(
        auth,
        region.region(),
        region.endpoint(),
        proxy,
        tls_options,
        FileConsolidationConfig {
            enabled: true,
            process_every_ms: 10,
            requested_size_bytes: 512000000,
            base_path: None,
            output_format: None,
        },
        bucket.clone(),
    );

    let started = fc.start();
    assert_eq!(started, true, "started true");

    thread::sleep(time::Duration::from_millis(1000));

    let stopped = fc.stop();
    assert_eq!(stopped, true, "stopped true");
}

#[tokio::test]
async fn s3_file_consolidator_disabled_run() {
    let _cx = SinkContext::new_test();

    // testing the default scenario where the consolidator is disabled
    let mut fc: FileConsolidatorAsync = Default::default();

    let started = fc.start();
    assert!(!started, "started false");

    thread::sleep(time::Duration::from_millis(1000));

    let stopped = fc.stop();
    assert!(!stopped, "stopped false");
}

#[tokio::test]
async fn s3_file_consolidation_process_no_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_owned();
    let requested_size_bytes: i64 = 10 * 1024 * 1024;
    let output_format = "ndjson".to_owned();

    create_bucket(&bucket, false).await;

    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket,
        requested_size_bytes,
        key_prefix,
        output_format,
    );
    fcp.run().await;
}

#[tokio::test]
async fn s3_file_consolidation_process_no_tagged_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_string();
    let requested_size_bytes: i64 = 10 * 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "ndjson".to_owned();

    create_bucket(&bucket, false).await;
    put_file(
        "file_no_tags.log".to_string(),
        Bytes::from("unit test".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        None,
    )
    .await;

    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);

    // runs without errors
    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format,
    );
    fcp.run().await;

    let keys = get_keys(&bucket, key_prefix).await;
    assert_eq!(keys.len(), 1);
}

#[tokio::test]
async fn s3_file_consolidation_process_tag_filters() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_string();
    let requested_size_bytes: i64 = 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "ndjson".to_owned();

    create_bucket(&bucket, false).await;

    let pipeline_merged_tags =
        generate_tags("mezmo_pipeline_merged".to_string(), "true".to_string());
    let pipeline_custom_tags = generate_tags("random_tag".to_string(), "true".to_string());
    let mezmo_pipeline_s3_type_ndjson_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), "ndjson".to_string());
    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), "text".to_string());
    let mezmo_pipeline_s3_type_json_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), "json".to_string());
    let mezmo_pipeline_s3_type_unknown_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), "".to_string());

    put_file(
        "previous_merge.log".to_string(),
        Bytes::from("file from previous merge".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        pipeline_merged_tags,
    )
    .await;
    put_file(
        "s3_sink.log".to_string(),
        Bytes::from("file from s3 sink".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        pipeline_custom_tags,
    )
    .await;
    put_file(
        "s3_type_ndjson.log".to_string(),
        Bytes::from("file with ndjson".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        mezmo_pipeline_s3_type_ndjson_tags,
    )
    .await;
    put_file(
        "s3_type_text.log".to_string(),
        Bytes::from("file with text".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        mezmo_pipeline_s3_type_text_tags,
    )
    .await;
    put_file(
        "s3_type_json.log".to_string(),
        Bytes::from("file with json".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        mezmo_pipeline_s3_type_json_tags,
    )
    .await;
    put_file(
        "s3_type_unknown.log".to_string(),
        Bytes::from("file with who knows what".as_bytes()),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        mezmo_pipeline_s3_type_unknown_tags,
    )
    .await;

    // 6 keys should be persisted
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 6);

    //validate the filter works for the requested types
    let filtered_text = get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        "text".to_owned(),
    )
    .await
    .unwrap();
    assert_eq!(filtered_text.len(), 1);
    assert_eq!(filtered_text[0].key, "/s3_type_text.log");

    let filtered_json = get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        "json".to_owned(),
    )
    .await
    .unwrap();
    assert_eq!(filtered_json.len(), 1);
    assert_eq!(filtered_json[0].key, "/s3_type_json.log");

    let filtered_ndjson = get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        "ndjson".to_owned(),
    )
    .await
    .unwrap();
    assert_eq!(filtered_ndjson.len(), 1);
    assert_eq!(filtered_ndjson[0].key, "/s3_type_ndjson.log");

    // run the consolidator and
    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // no changes made since there was only a single of each type
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 6);
    assert!(keys.contains(&"/s3_sink.log".to_string()));
    assert!(keys.contains(&"/s3_type_text.log".to_string()));
    assert!(keys.contains(&"/s3_type_json.log".to_string()));
    assert!(keys.contains(&"/s3_type_ndjson.log".to_string()));
    assert!(keys.contains(&"/s3_type_unknown.log".to_string()));
    assert!(keys.contains(&"/previous_merge.log".to_string()));
}

#[tokio::test]
async fn s3_file_consolidation_process_text_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_string();
    let requested_size_bytes: i64 = 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "text".to_owned();

    create_bucket(&bucket, false).await;

    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());

    for i in 0..3 {
        let filename = format!("file_{}.log", i + 1);

        let data = Bytes::from(format!("this is from file {}", i + 1));
        put_file(
            filename,
            data,
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_text_tags.clone(),
        )
        .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    // 3 keys should be persisted
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 3);

    // run the consolidator and
    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // the files are properly consolidated into 1 file
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);
    assert!(is_merged_file(&keys[0]));

    let obj = get_object(&bucket, keys[0].clone()).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));
    assert_eq!(obj.content_type, Some("text/x-log".to_string()));
    assert_eq!(obj.content_length, 59); // line length plus newlines

    let response_lines = get_lines(obj).await;
    assert_eq!(response_lines.len(), 3);

    assert_eq!(response_lines[0], "this is from file 1");
    assert_eq!(response_lines[1], "this is from file 2");
    assert_eq!(response_lines[2], "this is from file 3");
}

#[tokio::test]
async fn s3_file_consolidation_process_json_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_string();
    let requested_size_bytes: i64 = 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "json".to_owned();

    create_bucket(&bucket, false).await;

    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());

    for i in 0..3 {
        let filename = format!("file_{}.log", i + 1);
        let data = Bytes::from(format!("[\"this is from file {}\"]", i + 1));
        put_file(
            filename,
            data,
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_text_tags.clone(),
        )
        .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    // 3 keys should be persisted
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 3);

    // run the consolidator and
    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // the files are properly consolidated into 1 file
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);
    assert!(is_merged_file(&keys[0]));

    let obj = get_object(&bucket, keys[0].clone()).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));
    assert_eq!(obj.content_type, Some("text/x-log".to_string()));
    assert_eq!(obj.content_length, 67); //text with comma separators

    let response_lines = get_lines(obj).await;
    assert_eq!(response_lines.len(), 1);
    assert_eq!(
        response_lines[0],
        "[\"this is from file 1\",\"this is from file 2\",\"this is from file 3\"]"
    );
}

#[tokio::test]
async fn s3_file_consolidation_process_ndjson_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/".to_string();
    let requested_size_bytes: i64 = 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "ndjson".to_owned();

    create_bucket(&bucket, false).await;

    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());

    for i in 0..3 {
        let filename = format!("file_{}.log", i + 1);
        let data = Bytes::from(format!(
            "{{ \"message\": \"this is from file {}\" }}",
            i + 1
        ));
        put_file(
            filename,
            data,
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_text_tags.clone(),
        )
        .await;

        // forcing a sleep so the file sorting is tested correctly as the
        // emulator only keeps time to the second
        thread::sleep(time::Duration::from_millis(1000));
    }

    // 3 keys should be persisted
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 3);

    // run the consolidator and
    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // the files are properly consolidated into 1 file
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);
    assert!(is_merged_file(&keys[0]));

    let obj = get_object(&bucket, keys[0].clone()).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));
    assert_eq!(obj.content_type, Some("text/x-log".to_string()));
    assert_eq!(obj.content_length, 110); // line length plus newlines

    let response_lines = get_lines(obj).await;
    assert_eq!(response_lines.len(), 3);

    assert_eq!(
        response_lines[0],
        "{ \"message\": \"this is from file 1\" }"
    );
    assert_eq!(
        response_lines[1],
        "{ \"message\": \"this is from file 2\" }"
    );
    assert_eq!(
        response_lines[2],
        "{ \"message\": \"this is from file 3\" }"
    );
}

#[tokio::test]
async fn s3_file_consolidation_compressed_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "/compressed-files/".to_string();
    let requested_size_bytes: i64 = 20 * 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "text".to_owned();

    create_bucket(&bucket, false).await;

    // create some text lines and compress them
    let text = "ozsggnwocqbrtuzwzudhakpibrkfnewnnuoeyopbmshpgcjicrmgasucmizjqycsvjladptmhtygwwystocxsphnyckeijpyfbvy".to_owned();
    let compressed_text = compress_text(&text);

    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());

    for i in 0..3 {
        let filename = format!("file_{}.log", i + 1);
        put_file(
            filename,
            compressed_text.clone(),
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            Some("gzip".to_string()),
            mezmo_pipeline_s3_type_text_tags.clone(),
        )
        .await;
    }

    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 3);

    // only s3 created files with ndjson and text will be merged
    match get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        output_format.clone(),
    )
    .await
    {
        Ok(files) => {
            assert_eq!(files.len(), 3);
            assert_eq!(files[0].size, 85);
            assert_eq!(files[0].key, "/compressed-files/file_1.log");

            assert_eq!(files[1].size, 85);
            assert_eq!(files[1].key, "/compressed-files/file_2.log");

            assert_eq!(files[2].size, 85);
            assert_eq!(files[2].key, "/compressed-files/file_3.log");
        }
        Err(err) => panic!("Retrieving files should not error: {}", err),
    };

    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // validate we're down to 1 files now since 2 of them were merged
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);
    assert!(is_merged_file(&keys[0]));

    let obj = get_object(&bucket, keys[0].clone()).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));
    assert_eq!(obj.content_type, Some("text/x-log".to_string()));
    assert_eq!(obj.content_length, 302); // decompressed plus newlines

    let response_lines = get_lines(obj).await;
    assert_eq!(response_lines.len(), 3);
    assert_eq!(response_lines[0], text);
    assert_eq!(response_lines[1], text);
    assert_eq!(response_lines[2], text);
}

#[tokio::test]
async fn s3_file_consolidation_multiple_consolidated_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "".to_string();
    let requested_size_bytes: i64 = 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_type = "ndjson".to_string();

    create_bucket(&bucket, false).await;

    let hundred_bytes_1 = "{\"property\":\"fkcurxdqnnybrcutaogcvzvdttjzlcavsonfhuianreijaqfpaojjmolsibjzjvcphrjxzorjtvlbphepgfzy\"}";
    for i in 0..5 {
        let mut five_hundred_kb = BytesMut::new();
        for _x in 0..5000 {
            five_hundred_kb.extend_from_slice(hundred_bytes_1.as_bytes());
        }

        let mezmo_pipeline_s3_type_ndjson_tags =
            generate_tags("mezmo_pipeline_s3_type".to_string(), output_type.clone());
        let filename = format!("{}_generated.log", i);
        put_file(
            filename,
            Bytes::from(five_hundred_kb.to_vec()),
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_ndjson_tags.clone(),
        )
        .await;
    }

    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 5);

    match get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        output_type.clone(),
    )
    .await
    {
        Ok(files) => {
            assert_eq!(files.len(), 5);
            for file in files.iter() {
                assert_eq!(file.size, 500000);
            }
        }
        Err(err) => panic!("Retrieving files should not error: {}", err),
    };

    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_type.clone(),
    );
    fcp.run().await;

    // validate we're down to 2 files now
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 2);

    let mut i = 0;
    for k in keys.iter() {
        assert!(is_merged_file(k));

        let obj = get_object(&bucket, k.to_string()).await;
        assert_eq!(obj.content_encoding, Some("identity".to_string()));
        assert_eq!(obj.content_type, Some("text/x-log".to_string()));

        // the file has either 3 or 2 lines
        let response_lines = get_lines(obj).await;
        let lc = response_lines.len();
        assert!(lc == 2 || lc == 3);
        i += lc;
    }

    // all five lines are found between the 2 files
    assert_eq!(i, 5);
}

#[tokio::test]
async fn s3_file_consolidation_large_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "unit-test/".to_string();
    let requested_size_bytes: i64 = 100 * 1024 * 1024;
    let content_type = "text/x-log".to_string();
    let output_format = "text".to_string();

    create_bucket(&bucket, false).await;

    let hundred_bytes = "ozsggnwocqbrtuzwzudhakpibrkfnewnnuoeyopbmshpgcjicrmgasucmizjqycsvjladptmhtygwwystocxsphnyckeijpyfbvy";

    // build about 8 MB worth of small files so we can flush a single part
    for i in 0..15 {
        let mut five_hundred_kb = BytesMut::new();
        for _x in 0..5000 {
            five_hundred_kb.extend_from_slice(hundred_bytes.as_bytes());
            five_hundred_kb.extend_from_slice(b"\n");
        }

        let mezmo_pipeline_s3_type_text_tags =
            generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());
        let filename = format!("{}_generated.log", i);
        put_file(
            filename,
            Bytes::from(five_hundred_kb.to_vec()),
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_text_tags,
        )
        .await;
    }

    // a small compressed file to show decompression during upload parts.
    let compressed_text = {
        let mut compressed = BytesMut::new();
        for _i in 0..10 {
            compressed.extend_from_slice(hundred_bytes.as_bytes());
            compressed.extend_from_slice(b"\n");
        }

        let str = String::from_utf8(compressed.to_vec()).unwrap();
        compress_text(&str)
    };

    // create a 60 MB file to go over the threshold and use the upload_copy_part
    let mut six_megs_uncompressed = BytesMut::new();
    for _i in 0..600000 {
        six_megs_uncompressed.extend_from_slice(hundred_bytes.as_bytes());
        six_megs_uncompressed.extend_from_slice(b"\n");
    }

    let mezmo_pipeline_s3_type_text_tags =
        generate_tags("mezmo_pipeline_s3_type".to_string(), "text".to_string());

    put_file(
        "some_compressed_data.log".to_string(),
        compressed_text,
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        Some("gzip".to_string()),
        mezmo_pipeline_s3_type_text_tags.clone(),
    )
    .await;
    put_file(
        "6MB_uncompressed.log".to_string(),
        Bytes::from(six_megs_uncompressed),
        key_prefix.clone(),
        bucket.clone(),
        content_type.clone(),
        None,
        mezmo_pipeline_s3_type_text_tags.clone(),
    )
    .await;

    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 17);

    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // validate we're down to 1 file
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);
    assert!(is_merged_file(&keys[0]));

    let obj = get_object(&bucket, keys[0].clone()).await;
    assert_eq!(obj.content_encoding, Some("identity".to_string()));
    assert_eq!(obj.content_type, Some("text/x-log".to_string()));

    // 15 files of 5000 lines
    // 1 file of 10 lines
    // 1 file of 60000 lines
    // newlines between each added file
    let response_lines = get_lines(obj).await;
    assert_eq!(response_lines.len(), 675_026);
}

#[tokio::test]
async fn s3_file_consolidation_lots_of_10mb_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "unit-test/".to_string();
    let requested_size_bytes: i64 = 5_000_000_000;
    let content_type = "text/x-log".to_string();
    let output_format = "text".to_owned();

    create_bucket(&bucket, false).await;

    let hundred_bytes = "ozsggnwocqbrtuzwzudhakpibrkfnewnnuoeyopbmshpgcjicrmgasucmizjqycsvjladptmhtygwwystocxsphnyckeijpyfbvy";
    for i in 0..15 {
        let mut ten_megs_uncompressed = BytesMut::new();
        for _i in 0..100_000 {
            ten_megs_uncompressed.extend_from_slice(hundred_bytes.as_bytes());
            ten_megs_uncompressed.extend_from_slice(b"\n");
        }

        let mezmo_pipeline_s3_type_text_tags =
            generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());
        let filename = format!("10MB_{}_generated.log", i);
        put_file(
            filename,
            Bytes::from(ten_megs_uncompressed.to_vec()),
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_text_tags,
        )
        .await;
    }

    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 15);

    let fcp = FileConsolidationProcessor::new(
        &s3_client,
        bucket.clone(),
        requested_size_bytes,
        key_prefix.clone(),
        output_format.clone(),
    );
    fcp.run().await;

    // validate we're down to 1 file
    let keys = get_keys(&bucket, key_prefix.clone()).await;
    assert_eq!(keys.len(), 1);

    // the new file that should contain the text of the docs
    if let Some(k) = keys.into_iter().find(|s| is_merged_file(s)) {
        let obj = get_object(&bucket, k).await;
        assert_eq!(obj.content_encoding, Some("identity".to_string()));
        assert_eq!(obj.content_type, Some("text/x-log".to_string()));
        assert_eq!(obj.content_length, 151_500_014);

        // 15 files of 100_000 lines that are all bashed together
        let response_lines = get_lines(obj).await;
        assert_eq!(response_lines.len(), 1_500_014);
    } else {
        panic!("did not find the merged file as expected");
    }
}

#[tokio::test]
async fn s3_file_consolidation_large_amount_of_files() {
    let _cx = SinkContext::new_test();

    let s3_client = client().await;
    let bucket = uuid::Uuid::new_v4().to_string();
    let key_prefix = "large-amount-of-files/".to_string();
    let content_type = "text/x-log".to_string();
    let output_format = "text".to_owned();

    create_bucket(&bucket, false).await;

    // default is 1000 records, so make sure we go over to test the continuation
    // NOTE: this is an expensive test, takes ~45 seconds locally :/
    for n in 1..1006 {
        let mezmo_pipeline_s3_type_ndjson_tags =
            generate_tags("mezmo_pipeline_s3_type".to_string(), output_format.clone());

        let filename = format!("{}.log", n);
        let data = Bytes::from(format!("This is the content of {}.log", n));

        put_file(
            filename,
            data,
            key_prefix.clone(),
            bucket.clone(),
            content_type.clone(),
            None,
            mezmo_pipeline_s3_type_ndjson_tags,
        )
        .await;
    }

    // only s3 created files with ndjson and text will be merged
    match get_files_to_consolidate(
        &s3_client,
        bucket.clone(),
        key_prefix.clone(),
        output_format.clone(),
    )
    .await
    {
        Ok(files) => {
            assert_eq!(files.len(), 1005);
        }
        Err(err) => panic!("Retrieving files should not error: {}", err),
    };
}

async fn put_file(
    file_name: String,
    content: Bytes,
    prefix: String,
    bucket: String,
    content_type: String,
    content_encoding: Option<String>,
    tags: Option<String>,
) {
    _ = client()
        .await
        .put_object()
        .body(ByteStream::from(content))
        .bucket(bucket.clone())
        .key(format!("{}{}", prefix.clone(), file_name))
        .set_content_type(Some(content_type.clone()))
        .set_content_encoding(content_encoding.clone())
        .set_tagging(tags)
        .send()
        .await;
}

fn compress_text(value: &String) -> Bytes {
    let mut ret_vec = [0; 1000000];
    let mut bytestring = value.as_bytes();
    let mut gz = GzEncoder::new(&mut bytestring, flate2::Compression::fast());
    let count = gz.read(&mut ret_vec).unwrap();
    let vec = ret_vec[0..count].to_vec();

    let mut bytes_mut = BytesMut::with_capacity(0);
    bytes_mut.extend_from_slice(&vec);
    Bytes::from(bytes_mut)
}

fn generate_tags(key: String, value: String) -> Option<String> {
    let tags = {
        let mut tagging = url::form_urlencoded::Serializer::new(String::new());
        tagging.append_pair(&key, &value);
        tagging.append_pair("mezmo_pipeline_s3_sink", "true");
        tagging.finish()
    };

    Some(tags)
}

fn json_config(bucket: &str, batch_size: usize) -> S3SinkConfig {
    let mut batch = BatchConfig::default();
    batch.max_events = Some(batch_size);
    batch.timeout_secs = Some(5.0);

    S3SinkConfig {
        bucket: bucket.to_string(),
        key_prefix: random_string(10) + "/date=%F",
        filename_time_format: "abcd".into(),
        filename_append_uuid: false,
        filename_extension: None,
        options: S3Options::default(),
        region: RegionOrEndpoint::with_both("minio", s3_address()),
        encoding: (
            None::<FramingConfig>,
            JsonSerializerConfig::new(MetricTagValues::Single),
        )
            .into(),
        compression: Compression::None,
        batch,
        request: TowerRequestConfig::default(),
        tls: Default::default(),
        auth: Default::default(),
        acknowledgements: Default::default(),
        file_consolidation_config: Default::default(),
        timezone: Default::default(),
    }
}

fn make_object_events_batch(
    len: usize,
    count: usize,
) -> (
    Vec<Event>,
    impl Stream<Item = EventArray>,
    BatchStatusReceiver,
) {
    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let (events, stream) = random_message_object_events_with_stream(len, count, Some(batch));

    (events, stream.map(Into::into), receiver)
}
