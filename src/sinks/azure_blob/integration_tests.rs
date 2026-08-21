use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    sync::Arc,
};

use azure_core::http::{RequestContent, StatusCode};
use azure_storage_blob::{
    BlobContainerClient,
    models::{BlobTags, BlockBlobClientUploadOptions},
};

use bytes::{Buf, Bytes, BytesMut};
use flate2::read::GzDecoder;
use futures::{Stream, StreamExt, stream};
use vector_lib::{
    ByteSizeOf,
    codecs::{
        JsonSerializerConfig, NewlineDelimitedEncoderConfig, TextSerializerConfig,
        encoding::FramingConfig,
    },
};

pub use super::config::AzureBlobSinkConfig;
use crate::{
    config::SinkContext,
    event::{Event, EventArray, LogEvent},
    sinks::{
        VectorSink, azure_common,
        util::{Compression, TowerRequestConfig},
    },
    test_util::{
        components::{SINK_TAGS, assert_sink_compliance},
        random_events_with_stream, random_lines, random_lines_with_stream, random_string,
    },
};

#[tokio::test]
async fn azure_blob_healthcheck_passed() {
    let config = AzureBlobSinkConfig::new_emulator().await;
    let client = azure_common::config::build_client(
        config.connection_string.clone().into(),
        config.container_name.clone(),
        &crate::config::ProxyConfig::default(),
    )
    .expect("Failed to create client");

    azure_common::config::build_healthcheck(
        config.container_name,
        Some(client),
        SinkContext::default(),
    )
    .expect("Failed to build healthcheck")
    .await
    .expect("Failed to pass healthcheck");
}

#[tokio::test]
async fn azure_blob_healthcheck_unknown_container() {
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        container_name: String::from("other-container-name"),
        ..config
    };
    let client = azure_common::config::build_client(
        config.connection_string.clone().into(),
        config.container_name.clone(),
        &crate::config::ProxyConfig::default(),
    )
    .expect("Failed to create client");

    assert_eq!(
        azure_common::config::build_healthcheck(
            config.container_name,
            Some(client),
            SinkContext::default()
        )
        .unwrap()
        .await
        .unwrap_err()
        .to_string(),
        "Container: \"other-container-name\" not found"
    );
}

#[tokio::test]
async fn azure_blob_insert_lines_into_blob() {
    let blob_prefix = format!("lines/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: blob_prefix.clone().try_into().unwrap(),
        ..config
    };
    let (lines, input) = random_lines_with_stream(100, 10, None);

    config.run_assert(input).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (content_type, content_encoding, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(content_type, Some(String::from("text/plain")));
    assert_eq!(content_encoding, None);
    assert_eq!(lines, blob_lines);
}

#[tokio::test]
async fn azure_blob_insert_json_into_blob() {
    let blob_prefix = format!("json/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: blob_prefix.clone().try_into().unwrap(),
        encoding: (
            Some(NewlineDelimitedEncoderConfig::new()),
            JsonSerializerConfig::default(),
        )
            .into(),
        ..config
    };
    let (events, input) = random_events_with_stream(100, 10, None);

    config.run_assert(input).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log"));
    let (content_type, content_encoding, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(content_encoding, None);
    assert_eq!(content_type, Some(String::from("application/x-ndjson")));
    let expected = events
        .iter()
        .map(|event| serde_json::to_string(&event.as_log().all_event_fields().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(expected, blob_lines);
}

#[ignore]
#[tokio::test]
// This test fails to get the posted blob with "header not found content-length".
// However, we inspected that the sink writes the expected contents to Azure thus this is a retrieval/test issue.
// Additional context: https://github.com/Azure/Azurite/issues/629
async fn azure_blob_insert_lines_into_blob_gzip() {
    let blob_prefix = format!("lines-gzip/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: blob_prefix.clone().try_into().unwrap(),
        compression: Compression::gzip_default(),
        ..config
    };
    let (lines, events) = random_lines_with_stream(100, 10, None);

    config.run_assert(events).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log.gz"));
    let (content_type, content_encoding, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(content_encoding, Some(String::from("gzip")));
    assert_eq!(content_type, Some(String::from("text/plain")));
    assert_eq!(lines, blob_lines);
}

#[ignore]
#[tokio::test]
// This test will fail with Azurite blob emulator because of this issue:
// https://github.com/Azure/Azurite/issues/629
async fn azure_blob_insert_json_into_blob_gzip() {
    let blob_prefix = format!("json-gzip/into/blob/{}", random_string(10));
    let config = AzureBlobSinkConfig::new_emulator().await;
    let config = AzureBlobSinkConfig {
        blob_prefix: blob_prefix.clone().try_into().unwrap(),
        encoding: (
            Some(NewlineDelimitedEncoderConfig::new()),
            JsonSerializerConfig::default(),
        )
            .into(),
        compression: Compression::gzip_default(),
        ..config
    };
    let (events, input) = random_events_with_stream(100, 10, None);

    config.run_assert(input).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 1);
    assert!(blobs[0].clone().ends_with(".log.gz"));
    let (content_type, content_encoding, blob_lines) = config.get_blob(blobs[0].clone()).await;
    assert_eq!(content_encoding, Some(String::from("gzip")));
    assert_eq!(content_type, Some(String::from("application/x-ndjson")));
    let expected = events
        .iter()
        .map(|event| serde_json::to_string(&event.as_log().all_event_fields().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(expected, blob_lines);
}

#[tokio::test]
async fn azure_blob_rotate_files_after_the_buffer_size_is_reached() {
    let groups = 3;
    let (lines, size, input) = random_lines_with_stream_with_group_key(100, 30, groups);
    let size_per_group = (size / groups) + 10;

    let blob_prefix = format!("lines-rotate/into/blob/{}", random_string(10));
    let mut config = AzureBlobSinkConfig::new_emulator().await;
    config.batch.max_bytes = Some(size_per_group);

    let config = AzureBlobSinkConfig {
        blob_prefix: (blob_prefix.clone() + "{{key}}").try_into().unwrap(),
        blob_append_uuid: Some(false),
        batch: config.batch,
        ..config
    };

    config.run_assert(input).await;

    let blobs = config.list_blobs(blob_prefix).await;
    assert_eq!(blobs.len(), 3);
    let response = stream::iter(blobs)
        .fold(Vec::new(), |mut acc, blob| async {
            let (_, _, lines) = config.get_blob(blob).await;
            acc.push(lines);
            acc
        })
        .await;

    for i in 0..groups {
        assert_eq!(&lines[(i * 10)..((i + 1) * 10)], response[i].as_slice());
    }
}

impl AzureBlobSinkConfig {
    pub async fn new_emulator() -> AzureBlobSinkConfig {
        let address = std::env::var("AZURE_ADDRESS").unwrap_or_else(|_| "localhost".into());
        // MEZMO: allow pointing the suite at a real storage account. Azurite does not
        // reproduce the service's response shapes or URL handling faithfully enough to
        // catch every regression in the consolidation code.
        let connection_string = std::env::var("AZURE_STORAGE_CONNECTION_STRING").unwrap_or_else(|_| {
            format!("UseDevelopmentStorage=true;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://{address}:10000/devstoreaccount1;QueueEndpoint=http://{address}:10001/devstoreaccount1;TableEndpoint=http://{address}:10002/devstoreaccount1;")
        });
        let container_name =
            std::env::var("AZURE_STORAGE_CONTAINER").unwrap_or_else(|_| "logs".to_string());
        let config = AzureBlobSinkConfig {
            connection_string: connection_string.into(),
            container_name,
            blob_prefix: Default::default(),
            blob_time_format: None,
            blob_append_uuid: None,
            encoding: (None::<FramingConfig>, TextSerializerConfig::default()).into(),
            compression: Compression::None,
            batch: Default::default(),
            request: TowerRequestConfig::default(),
            acknowledgements: Default::default(),
            file_consolidation_config: Default::default(),
            tags: Default::default(),
        };

        config.ensure_container().await;

        config
    }

    fn to_sink(&self) -> VectorSink {
        let client = azure_common::config::build_client(
            self.connection_string.clone().into(),
            self.container_name.clone(),
            &crate::config::ProxyConfig::default(),
        )
        .expect("Failed to create client");

        self.build_processor(Some(client), SinkContext::default())
            .expect("Failed to create sink")
    }

    pub async fn run_assert(&self, input: impl Stream<Item = EventArray> + Send) {
        // `to_sink` needs to be inside the assertion check
        assert_sink_compliance(&SINK_TAGS, async move { self.to_sink().run(input).await })
            .await
            .expect("Running sink failed");
    }

    pub async fn list_blobs(&self, prefix: String) -> Vec<String> {
        let client = azure_common::config::build_client(
            self.connection_string.clone().into(),
            self.container_name.clone(),
            &crate::config::ProxyConfig::default(),
        )
        .unwrap();

        // Iterate pager results and collect blob names. Filter by prefix server-side.
        let mut pager = client
            .list_blobs(None)
            .expect("Failed to start list blobs pager");
        let mut names = Vec::new();
        while let Some(result) = pager.next().await {
            let item = result.expect("Failed to fetch blobs");
            if let Some(name) = item.name.and_then(|bn| bn.content)
                && name.starts_with(&prefix)
            {
                names.push(name);
            }
        }

        names
    }

    // MEZMO: helper to build a raw container client for file-consolidation tests.
    pub async fn get_client(&self) -> Arc<BlobContainerClient> {
        azure_common::config::build_client(
            self.connection_string.clone().into(),
            self.container_name.clone(),
            &crate::config::ProxyConfig::default(),
        )
        .unwrap()
    }

    // MEZMO: helper to upload a blob with a content type, encoding and index tags.
    pub async fn put_blob(
        &self,
        filename: String,
        content_type: &str,
        encoding: &str,
        file_tags: Option<BTreeMap<String, String>>,
        data: Bytes,
    ) {
        let client = self.get_client().await;

        // Azure blob index tags are sent as a URL-encoded `key=value&...` string.
        let blob_tags_string = file_tags.as_ref().map(|tags| {
            let mut ser = url::form_urlencoded::Serializer::new(String::new());
            for (key, value) in tags {
                ser.append_pair(key, value);
            }
            ser.finish()
        });

        let byte_size = data.len() as u64;
        let options = BlockBlobClientUploadOptions {
            blob_content_type: Some(content_type.to_string()),
            blob_content_encoding: Some(encoding.to_string()),
            blob_tags_string,
            ..Default::default()
        };

        client
            .blob_client(filename.as_str())
            .upload(
                RequestContent::from(data.to_vec()),
                true,
                byte_size,
                Some(options),
            )
            .await
            .unwrap();
    }

    // MEZMO: helper to read the index tags of a blob.
    pub async fn get_tags(&self, blob: String) -> BlobTags {
        let client = self.get_client().await;
        client
            .blob_client(blob.as_str())
            .get_tags(None)
            .await
            .unwrap()
            .into_model()
            .unwrap()
    }

    pub async fn get_blob(&self, blob: String) -> (Option<String>, Option<String>, Vec<String>) {
        let client = azure_common::config::build_client(
            self.connection_string.clone().into(),
            self.container_name.clone(),
            &crate::config::ProxyConfig::default(),
        )
        .unwrap();

        let blob_client = client.blob_client(&blob);

        // Fetch properties to obtain content-type and content-encoding
        let props_resp = blob_client
            .get_properties(None)
            .await
            .expect("Failed to get blob properties");
        let headers = props_resp.headers();
        let content_type = headers.iter().find_map(|(name, value)| {
            let key = name.as_str();
            if key.eq_ignore_ascii_case("content-type") {
                Some(value.as_str().to_string())
            } else {
                None
            }
        });
        let content_encoding = headers.iter().find_map(|(name, value)| {
            let key = name.as_str();
            if key.eq_ignore_ascii_case("content-encoding") {
                Some(value.as_str().to_string())
            } else {
                None
            }
        });

        // Download blob content (full or first MB as needed)
        let downloaded = blob_client
            .download(None)
            .await
            .expect("Failed to download blob");
        let body_bytes = downloaded
            .into_body()
            .collect()
            .await
            .expect("Failed to read blob body");
        let data = body_bytes.to_vec();

        (content_type, content_encoding, self.get_blob_content(data))
    }

    fn get_blob_content(&self, data: Vec<u8>) -> Vec<String> {
        let body = BytesMut::from(data.as_slice()).freeze().reader();

        if self.compression == Compression::None {
            BufReader::new(body).lines().map(|l| l.unwrap()).collect()
        } else {
            BufReader::new(GzDecoder::new(body))
                .lines()
                .map(|l| l.unwrap())
                .collect()
        }
    }

    async fn ensure_container(&self) {
        let client = azure_common::config::build_client(
            self.connection_string.clone().into(),
            self.container_name.clone(),
            &crate::config::ProxyConfig::default(),
        )
        .unwrap();
        let result = client.create_container(None).await;

        let response = match result {
            Ok(_) => Ok(()),
            Err(error) => match error.http_status() {
                Some(StatusCode::Conflict) => Ok(()),
                _ => Err(error),
            },
        };

        response.expect("Failed to create container")
    }
}

fn random_lines_with_stream_with_group_key(
    len: usize,
    count: usize,
    groups: usize,
) -> (Vec<String>, usize, impl Stream<Item = EventArray>) {
    let key = count / groups;
    let lines = random_lines(len).take(count).collect::<Vec<_>>();
    let (size, events) = lines
        .clone()
        .into_iter()
        .enumerate()
        .map(move |(i, line)| {
            let mut log = LogEvent::from(line);
            let i = ((i / key) + 1) as i32;
            log.insert("key", i);
            Event::from(log)
        })
        .fold((0, Vec::new()), |(mut size, mut events), event| {
            size += event.size_of();
            events.push(event.into());
            (size, events)
        });

    (lines, size, stream::iter(events))
}
