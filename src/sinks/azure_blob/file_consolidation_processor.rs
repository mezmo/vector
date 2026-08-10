use azure_core::http::RequestContent;
use azure_storage_blob::BlobContainerClient;
use azure_storage_blob::models::{
    BlobContainerClientListBlobFlatSegmentOptions, BlockBlobClientCommitBlockListOptions,
    BlockLookupList, ListBlobsIncludeItem,
};
use bytes::{Bytes, BytesMut};
use flate2::read::GzDecoder;
use futures::TryStreamExt;

pub use super::config::AzureBlobSinkConfig;

use std::{
    cmp,
    io::Read,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use std::io;
use std::io::Cursor;

use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use base64::{Engine as _, engine::general_purpose};

const ONE_MEGABYTE_USIZE: usize = 1024 * 1024;
const MAX_BLOCKS_IN_PUT_BLOCK: usize = 50_000;

// handles consolidating the small files within AWS into much larger files
// NOTE: no `#[derive(Debug)]` because the new `BlobContainerClient` does not implement Debug.
pub struct FileConsolidationProcessor<'a> {
    container_client: &'a Arc<BlobContainerClient>,
    container_name: String,
    base_path: String,
    requested_size_bytes: u64,
    output_format: String,
}

// handles consolidating all the smaller files generated into larger files of the bucket
// a few assumptions:
// 1. the files are tagged with mezmo keys as found in @get_files_to_consolidate
//    to keep us from accidentally messing with files the customer had in the bucket
// 2. the files themselves aren't huge (currently the sink limits to 10 MB files)
//    so no file size containts are enforced locally for memory issues across the instance
impl<'a> FileConsolidationProcessor<'a> {
    pub const fn new(
        container_client: &'a Arc<BlobContainerClient>,
        container_name: String,
        base_path: String,
        requested_size_bytes: u64,
        output_format: String,
    ) -> Self {
        FileConsolidationProcessor {
            container_client,
            container_name,
            base_path,
            requested_size_bytes,
            output_format,
        }
    }

    pub async fn run(self) {
        let files_to_consolidate: Vec<ConsolidationFile> = match get_files_to_consolidate(
            self.container_client,
            self.container_name.clone(),
            self.base_path.clone(),
            self.output_format.clone(),
        )
        .await
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    ?e,
                    "container={}, base_path={}, Failed to retrieve files to consolidate",
                    self.container_name.clone(),
                    self.base_path.clone(),
                );
                let empty_files: Vec<ConsolidationFile> = Vec::new();
                empty_files
            }
        };

        // customers are specifying a base path to consolidate
        let files_by_dir: HashMap<String, Vec<ConsolidationFile>> =
            group_files_by_directory(files_to_consolidate);

        for dir in files_by_dir.keys() {
            let mut files = files_by_dir.get(dir).unwrap().clone();

            // break the files into groups so we can generate a file of requested file size
            while files.len() > 1 {
                let mut upload_file_parts: Vec<ConsolidationFile> =
                    splice_files_list(self.requested_size_bytes, &mut files);

                // build the new file properties and expiration time
                // make sure the process hasn't ran so fast (really should just unit tests)
                // that we accidentally overwrite a merge file.
                let mut time_since_epoch: u64;
                let mut new_file_key: String;

                loop {
                    time_since_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    // if there's a directory, the trailing slash is not present so add it
                    let newfile_dir = if dir.is_empty() {
                        dir.clone()
                    } else {
                        format!("{}/", dir.clone())
                    };

                    new_file_key = format!("{newfile_dir}merged_{time_since_epoch}.log");

                    info!(
                        "container={}, Determining if merge file already exists, file={}",
                        self.container_name.clone(),
                        new_file_key.clone(),
                    );

                    match self
                        .container_client
                        .blob_client(new_file_key.as_str())
                        .exists()
                        .await
                    {
                        Ok(true) => {
                            info!(
                                "container={}, Merged file already exists, file={}",
                                self.container_name.clone(),
                                new_file_key.clone(),
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                        // the file doesn't exist (or existence can't be confirmed); move on.
                        Ok(false) | Err(_) => break,
                    }
                }
                info!(
                    "container={}, Starting consolidated file={}",
                    self.container_name.clone(),
                    new_file_key.clone(),
                );

                // start grabbing each file and upload it as part of the block
                // don't worry about failures here as blocks will expire after 7 days if not consumed.
                // https://learn.microsoft.com/en-us/rest/api/storageservices/understanding-block-blobs--append-blobs--and-page-blobs

                // keep track of the processed files to delete
                let mut files_to_delete: Vec<String> = Vec::new();
                let is_standard_json_file: bool = self.output_format == "json";

                let mut block_parts: Vec<Vec<u8>> = Vec::new();
                while let Some(file) = upload_file_parts.pop() {
                    let prepend_char: Option<char> =
                        if is_standard_json_file && !files_to_delete.is_empty() {
                            Some(',')
                        } else if !files_to_delete.is_empty() {
                            Some('\n')
                        } else {
                            None
                        };

                    let data = match download_file_as_bytes(
                        self.container_client,
                        self.container_name.clone(),
                        &file,
                        is_standard_json_file && !block_parts.is_empty(), //trim_open_bracket
                        is_standard_json_file && !upload_file_parts.is_empty(), //trim_close_bracket
                        prepend_char,
                    )
                    .await
                    {
                        Ok(d) => d,
                        Err(err) => {
                            error!(
                                ?err,
                                "container={}, Failed to download file={} for merge file={}",
                                self.container_name.clone(),
                                file.key.clone(),
                                new_file_key.clone(),
                            );
                            continue;
                        }
                    };

                    // block id is required, must be unique, the same length for each block,
                    // and <= 64 bytes. A base64-encoded UUID satisfies all of these.
                    let block_id = general_purpose::URL_SAFE_NO_PAD.encode(Uuid::new_v4());
                    let block_id_bytes = block_id.into_bytes();
                    let data_len = data.len() as u64;
                    match self
                        .container_client
                        .blob_client(new_file_key.as_str())
                        .block_blob_client()
                        .stage_block(
                            &block_id_bytes,
                            data_len,
                            RequestContent::from(data.to_vec()),
                            None,
                        )
                        .await
                    {
                        Ok(_response) => {
                            info!(
                                "container={}, Uploaded block file={} for merge file={}",
                                self.container_name.clone(),
                                file.key.clone(),
                                new_file_key.clone(),
                            );
                        }
                        Err(err) => {
                            error!(
                                ?err,
                                "container={}, Failed block file={} for merge file={}",
                                self.container_name.clone(),
                                file.key.clone(),
                                new_file_key.clone(),
                            );
                            continue;
                        }
                    };

                    // keep track of the blobs that have been successfully uploaded
                    // note: they're uncommitted right now as they're just uploaded parts
                    files_to_delete.push(file.key.clone());
                    block_parts.push(block_id_bytes);
                } // end handle individual files

                // complete the file with all the parts
                if !block_parts.is_empty() {
                    // Azure blob index tags are sent as a URL-encoded `key=value&...` string.
                    // Scope the serializer so it is dropped before the `.await` below (it is not
                    // `Send`, and this runs inside a spawned task that requires `Send`).
                    let blob_tags_string = {
                        let mut tag_ser = url::form_urlencoded::Serializer::new(String::new());
                        tag_ser.append_pair("mezmo_pipeline_merged", "true");
                        tag_ser.finish()
                    };

                    let content_type = match self.output_format.as_str() {
                        "json" => "application/json",
                        "ndjson" => "application/x-ndjson",
                        "text" => "text/plain",
                        _ => "application/x-log",
                    };

                    let block_lookup_list = BlockLookupList {
                        committed: Some(Vec::new()),
                        latest: Some(Vec::new()),
                        uncommitted: Some(block_parts),
                    };

                    let blocks = match RequestContent::try_from(block_lookup_list) {
                        Ok(b) => b,
                        Err(err) => {
                            error!(
                                ?err,
                                "container={}, Failed to encode block list for merge file={}",
                                self.container_name.clone(),
                                new_file_key.clone(),
                            );
                            continue;
                        }
                    };

                    let commit_options = BlockBlobClientCommitBlockListOptions {
                        blob_tags_string: Some(blob_tags_string),
                        blob_content_type: Some(content_type.to_string()),
                        ..Default::default()
                    };

                    match self
                        .container_client
                        .blob_client(new_file_key.as_str())
                        .block_blob_client()
                        .commit_block_list(blocks, Some(commit_options))
                        .await
                    {
                        Ok(_response) => {
                            info!(
                                "container={}, Completed merge file={}",
                                self.container_name.clone(),
                                new_file_key.clone(),
                            );
                        }
                        Err(err) => {
                            error!(
                                ?err,
                                "container={}, Failed to complete merge file={}",
                                self.container_name.clone(),
                                new_file_key.clone(),
                            );
                            continue;
                        }
                    };
                }

                // remove all the files from azure that have been merged into the larger file
                for file in files_to_delete {
                    match self
                        .container_client
                        .blob_client(file.as_str())
                        .delete(None)
                        .await
                    {
                        Ok(_) => {
                            info!(
                                message = format!(
                                    "File={} removed from container={} after merge successful file consolidation into {}",
                                    file.clone(),
                                    self.container_name.clone(),
                                    new_file_key.clone()
                                )
                            )
                        }
                        Err(e) => error!(
                            ?e,
                            "container={}, Failed to delete file={} which was merged into {}",
                            self.container_name.clone(),
                            file.clone(),
                            new_file_key.clone()
                        ),
                    };
                }
            } // end else multipart logic
        } // end files to consolidate loop
    } //end run
}

// helper class for the files that we're consolidating into a single file
#[derive(Debug, Clone)]
pub struct ConsolidationFile {
    pub compressed: bool,
    pub size: u64,
    pub key: String,
}

impl ConsolidationFile {
    pub const fn new(compressed: bool, size: u64, key: String) -> ConsolidationFile {
        ConsolidationFile {
            compressed,
            size,
            key,
        }
    }
}

/*
    handles taking in a list of files and grabbing however many
    files which combined is the requested size.
    @requested_size_bytes: the total size of data requested
    @files: the list of files to pick from.
    @@returns: a vector of consolidation files
*/
fn splice_files_list(
    requested_size_bytes: u64,
    files: &mut Vec<ConsolidationFile>,
) -> Vec<ConsolidationFile> {
    let mut total_bytes: u64 = 0;

    // azure only allows so many blocks in a single file,
    // so make sure we don't overshoot that.
    let max_files: usize = cmp::min(files.len(), MAX_BLOCKS_IN_PUT_BLOCK);
    for i in 0..max_files {
        total_bytes += files[i].size;

        // grab files up until the bytes requested
        // and make sure to stop at the max of the api
        if total_bytes >= requested_size_bytes {
            return files.drain(0..i + 1).collect();
        }
    }

    std::mem::take(files)
}

/*
    Handles reading the blob container and evaluating the files
    which can be merged into larger files
    @container_client: the azure blob client
    @base_path: the base path for the files
    @file_type: the type of files to be merged
    @@returns: Vector<ConsolidationFile>, the files which can be merged.
*/
pub async fn get_files_to_consolidate(
    container_client: &Arc<BlobContainerClient>,
    container_name: String,
    base_path: String,
    file_type: String,
) -> Result<Vec<ConsolidationFile>, &'static str> {
    let mut files_to_consolidate: Vec<ConsolidationFile> = Vec::new();

    // the azure API has the ability to list blobs by tag,
    // but its not yet available in the rust version
    let list_options = BlobContainerClientListBlobFlatSegmentOptions {
        prefix: Some(base_path.clone()),
        include: Some(vec![ListBlobsIncludeItem::Tags]),
        ..Default::default()
    };
    let mut pages = match container_client.list_blobs(Some(list_options)) {
        Ok(pager) => pager.into_pages(),
        Err(e) => {
            error!(
                ?e,
                "container={}, base_path={}, Failed to list blobs",
                container_name.clone(),
                base_path.clone(),
            );
            return Ok(files_to_consolidate);
        }
    };

    loop {
        // there's a method on the api to search blobs by particular tags
        // but the results are only for positive results and only include
        // the tags searched and doesn't include the file stats
        let page = match pages.try_next().await {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                error!(
                    ?e,
                    "container={}, base_path={}, Failed to retrieve the next stream of blobs",
                    container_name.clone(),
                    base_path.clone(),
                );
                break;
            }
        };

        let blobs_response = match page.into_model() {
            Ok(b) => b,
            Err(e) => {
                error!(
                    ?e,
                    "container={}, base_path={}, Failed to deserialize the blob listing",
                    container_name.clone(),
                    base_path.clone(),
                );
                continue;
            }
        };

        let mut blobs = blobs_response.segment.blob_items;
        blobs.sort_by(|x, y| {
            let xt = x.properties.as_ref().and_then(|p| p.creation_time);
            let yt = y.properties.as_ref().and_then(|p| p.creation_time);
            yt.cmp(&xt)
        });

        for b in blobs {
            let Some(props) = b.properties.as_ref() else {
                continue;
            };
            let Some(key) = b.name.as_ref().and_then(|n| n.content.clone()) else {
                continue;
            };

            // The 0.7 SDK listing model expects a `BlobTags` element but the service
            // returns `Tags`, so tags never deserialize from the listing response.
            // When the listing reports a nonzero TagCount without a parsed tag set,
            // fetch the tags with a per-blob request instead.
            let tag_set = match b.blob_tags.as_ref().and_then(|t| t.blob_tag_set.clone()) {
                Some(tags) => Some(tags),
                None if props.tag_count.unwrap_or(0) > 0 => {
                    let blob_client = container_client.blob_client(key.as_str());
                    match blob_client.get_tags(None).await {
                        Ok(resp) => match resp.into_model() {
                            Ok(tags) => tags.blob_tag_set,
                            Err(e) => {
                                error!(
                                    ?e,
                                    "container={}, key={}, Failed to deserialize the blob tags",
                                    container_name.clone(),
                                    key.clone(),
                                );
                                None
                            }
                        },
                        Err(e) => {
                            error!(
                                ?e,
                                "container={}, key={}, Failed to retrieve the blob tags",
                                container_name.clone(),
                                key.clone(),
                            );
                            None
                        }
                    }
                }
                None => None,
            };

            if let Some(tag_set) = tag_set.as_ref() {
                // set defaults and resolve via tags
                let mut mezmo_merged_file = false;
                let mut mezmo_produced_file = false;
                let mut can_combine = false;

                for tag in tag_set {
                    match tag.key.as_deref() {
                        Some("mezmo_pipeline_merged") => mezmo_merged_file = true,
                        Some("mezmo_pipeline_azure_sink") => mezmo_produced_file = true,
                        Some("mezmo_pipeline_azure_type") => {
                            can_combine = tag.value.as_deref() == Some(file_type.as_str());
                        }
                        _ => (),
                    }
                }

                if !mezmo_merged_file && mezmo_produced_file && can_combine {
                    let compressed = props.content_encoding.clone().unwrap_or_default() == "gzip";
                    let size = props.content_length.unwrap_or(0);

                    files_to_consolidate.push(ConsolidationFile::new(compressed, size, key));
                }
            }
        }
    }

    Ok(files_to_consolidate)
}

/*
    Handles downloading the byte data from the provided file
    If the file is compressed, handles also decompressing the document
    via gzip compression.
    In an effort to merge json files, allow the truncation of brackets
    so they can be merged directly into a byte buffer representing a new
    json block.
    @container_client: the azure blob client
    @file: the file to download
    @trim_open_bracket: whether to trim leading brackets of json files
    @trim_close_bracket: whether to trim close brackets of json files
    @prepend_char: any character to prepend to the stream
    @@returns: Bytes, the byte data representing the new file
*/
async fn download_file_as_bytes(
    container_client: &Arc<BlobContainerClient>,
    container_name: String,
    file: &ConsolidationFile,
    trim_open_bracket: bool,
    trim_close_bracket: bool,
    prepend_char: Option<char>,
) -> Result<Bytes, &'static str> {
    let b: Bytes =
        download_bytes(container_client, container_name.clone(), file.key.clone()).await?;

    // The transport auto-decompresses gzip bodies when the reqwest `gzip`
    // feature is enabled (it is, via azure_core default features), so a blob
    // whose content-encoding is gzip may arrive already decompressed. Only
    // decompress when the payload actually carries the gzip magic bytes.
    let mut vec: Vec<u8>;
    if file.compressed && b.starts_with(&[0x1f, 0x8b]) {
        vec = decompress_gzip(&b);
    } else {
        vec = b.to_vec();
    }

    if trim_open_bracket && !vec.is_empty() {
        let i: usize = 0;
        while i < vec.len() {
            let c = char::from(vec[i]);
            if c.is_whitespace() {
                vec.remove(0);
                continue;
            }
            if c == '[' {
                vec.remove(0);
            }

            break;
        }
    }

    if trim_close_bracket {
        while !vec.is_empty() {
            let i = vec.len() - 1;
            let c = char::from(vec[i]);
            if c.is_whitespace() {
                vec.remove(i);
                continue;
            }
            if c == ']' {
                vec.remove(i);
                break;
            }

            break;
        }
    }

    if let Some(..) = prepend_char {
        vec.insert(0, prepend_char.unwrap() as u8);
    }

    let mut buf = BytesMut::with_capacity(0);
    buf.extend_from_slice(&vec);
    Ok(buf.freeze())
}

/*
    Handles gzip decompression of the bytes provided.
    @bytes: the byte representation of the file
    @@returns: the vector representing the decompressed bytes
*/
fn decompress_gzip(bytes: &Bytes) -> Vec<u8> {
    //place the bytes into a buffer that'll decode gzip
    let cursor = Cursor::new(bytes);
    let in_gz = GzDecoder::new(cursor);
    let mut in_buf = io::BufReader::with_capacity(ONE_MEGABYTE_USIZE, in_gz);

    // https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/std/io/struct.BufReader.html
    let mut vec: Vec<u8> = Vec::new();
    if let Err(e) = in_buf.read_to_end(&mut vec) {
        error!(?e, "Failed to decompress gzip payload");
    }
    vec
}

/*
    Handles retrieval of the azure blob from storage
    @container_client: the azure blob client
    @key: the file key
    @@returns: the byte data of the file
*/
async fn download_bytes(
    container_client: &Arc<BlobContainerClient>,
    container_name: String,
    key: String,
) -> Result<Bytes, &'static str> {
    static FAILURE: &str = "Failed to download bytes";

    let response = match container_client
        .blob_client(key.as_str())
        .download(None)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!(
                ?e,
                "container={}, key={}, Failed to retrieve bytes for the file",
                container_name.clone(),
                key.clone(),
            );
            return Err(FAILURE);
        }
    };

    let (_status, _headers, body) = response.deconstruct();
    match body.collect().await {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            error!(
                ?e,
                "container={}, key={}, Failed to read the body for the file",
                container_name.clone(),
                key.clone(),
            );
            Err(FAILURE)
        }
    }
}

fn group_files_by_directory(
    list: Vec<ConsolidationFile>,
) -> HashMap<String, Vec<ConsolidationFile>> {
    let mut hm: HashMap<String, Vec<ConsolidationFile>> = HashMap::new();
    for f in &list {
        let p = Path::new(&f.key);

        let base_path = match p.parent() {
            Some(p_base) => p_base.to_string_lossy().to_string(),
            None => "".to_string(),
        };

        if !hm.contains_key(&base_path) {
            let vec: Vec<ConsolidationFile> = Vec::new();
            hm.insert(base_path.clone(), vec);
        }

        let m = hm.get_mut(&base_path).unwrap();
        m.push(f.clone());
    }

    hm
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use flate2::Compression;
    use flate2::read::GzEncoder;
    use std::io::Read;

    use crate::sinks::azure_blob::file_consolidation_processor::ConsolidationFile;
    use crate::sinks::azure_blob::file_consolidation_processor::decompress_gzip;
    use crate::sinks::azure_blob::file_consolidation_processor::group_files_by_directory;
    use crate::sinks::azure_blob::file_consolidation_processor::splice_files_list;

    #[test]
    fn splice_empty_list() {
        let mut files: Vec<ConsolidationFile> = Vec::new();

        let result = splice_files_list(1000, &mut files);
        assert_eq!(files.len(), 0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn splice_single_item() {
        let mut files: Vec<ConsolidationFile> = Vec::new();
        for i in 0..1 {
            files.push(ConsolidationFile {
                compressed: false,
                size: 10,
                key: i.to_string().to_owned(),
            });
        }

        let result = splice_files_list(9, &mut files);
        assert_eq!(files.len(), 0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn splice_partial_list() {
        let mut files: Vec<ConsolidationFile> = Vec::new();
        for i in 0..10 {
            files.push(ConsolidationFile {
                compressed: false,
                size: 10,
                key: i.to_string().to_owned(),
            });
        }

        let result = splice_files_list(40, &mut files);
        assert_eq!(files.len(), 6);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn splice_entire_list() {
        let mut files: Vec<ConsolidationFile> = Vec::new();
        for i in 0..10 {
            files.push(ConsolidationFile {
                compressed: false,
                size: 10,
                key: i.to_string().to_owned(),
            });
        }

        let result = splice_files_list(1000, &mut files);
        assert_eq!(files.len(), 0);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn decompress_document_data() {
        let hello_world = "hello world".to_owned();

        // compress the text
        let mut ret_vec = [0; 100];
        let mut bytestring = hello_world.as_bytes();
        let mut gz = GzEncoder::new(&mut bytestring, Compression::fast());
        let count = gz.read(&mut ret_vec).unwrap();
        let vec = ret_vec[0..count].to_vec();

        let mut bytes_mut = BytesMut::with_capacity(0);
        bytes_mut.extend_from_slice(&vec);
        let bytes = Bytes::from(bytes_mut);

        //decompress
        let decompressed = decompress_gzip(&bytes);
        let s = std::str::from_utf8(&decompressed).unwrap();
        assert_eq!(hello_world, s);
    }

    #[test]
    fn test_group_files_by_directory() {
        let vec: Vec<ConsolidationFile> = vec![
            create_consolidation_file("base_file1.log"),
            create_consolidation_file("base_file2.log"),
            create_consolidation_file("/file1.log"),
            create_consolidation_file("/sub/file2.log"),
            create_consolidation_file("/sub/file3.log"),
            create_consolidation_file("/sub/file4.log"),
            create_consolidation_file("/sub/sub/file5.log"),
            create_consolidation_file("/sub/sub/file6.log"),
            create_consolidation_file("/sub/sub/file7.log"),
            create_consolidation_file("/sub/sub/file8.log"),
        ];

        let group = group_files_by_directory(vec);
        let keys = group.keys();

        assert_eq!(4, keys.len());

        assert_eq!(
            vec!["base_file1.log".to_string(), "base_file2.log".to_string()],
            group
                .get("")
                .unwrap()
                .iter()
                .map(|cf| cf.key.clone())
                .collect::<Vec<String>>()
        );

        assert_eq!(
            vec!["/file1.log"],
            group
                .get("/")
                .unwrap()
                .iter()
                .map(|cf| cf.key.clone())
                .collect::<Vec<String>>()
        );

        assert_eq!(
            vec!["/sub/file2.log", "/sub/file3.log", "/sub/file4.log"],
            group
                .get("/sub")
                .unwrap()
                .iter()
                .map(|cf| cf.key.clone())
                .collect::<Vec<String>>()
        );

        assert_eq!(
            vec![
                "/sub/sub/file5.log",
                "/sub/sub/file6.log",
                "/sub/sub/file7.log",
                "/sub/sub/file8.log",
            ],
            group
                .get("/sub/sub")
                .unwrap()
                .iter()
                .map(|cf| cf.key.clone())
                .collect::<Vec<String>>()
        );
    }

    fn create_consolidation_file(p: &str) -> ConsolidationFile {
        ConsolidationFile {
            compressed: false,
            size: 10,
            key: p.to_string().to_owned(),
        }
    }
}
