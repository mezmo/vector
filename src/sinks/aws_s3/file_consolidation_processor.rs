use bytes::{Bytes, BytesMut};
use std::{
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::read::GzDecoder;
use std::io;
use std::io::Cursor;

use std::collections::HashMap;
use std::path::Path;

use aws_sdk_s3::{
    types::{CompletedMultipartUpload, CompletedPart, RequestPayer},
    Client as S3Client, Error,
};

use aws_smithy_types::{byte_stream::ByteStream, DateTime as AwsDateTime};

const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
const MULTIPART_FILE_MIN_MB_I64: i64 = 5 * 1024 * 1024;
const ONE_MEGABYTE_USIZE: usize = 1024 * 1024;
const MAX_PARTS_MULTIPART_FILE: i32 = 10_000;

// handles consolidating the small files within AWS into much larger files
#[derive(Debug)]
pub struct FileConsolidationProcessor<'a> {
    s3_client: &'a S3Client,
    bucket: String,
    requested_size_bytes: i64,
    base_path: String,
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
        s3_client: &'a S3Client,
        bucket: String,
        requested_size_bytes: i64,
        base_path: String,
        output_format: String,
    ) -> Self {
        FileConsolidationProcessor {
            s3_client,
            bucket,
            requested_size_bytes,
            output_format,
            base_path,
        }
    }

    pub async fn run(self) {
        // retrieve the files list from s3 that we can process
        let files_to_consolidate: Vec<ConsolidationFile> = match get_files_to_consolidate(
            self.s3_client,
            self.bucket.clone(),
            self.base_path.clone(),
            self.output_format.clone(),
        )
        .await
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    ?e,
                    "bucket={}, base_path={}, Failed to retrieve files to consolidate",
                    self.bucket.clone(),
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

            while files.len() > 1 {
                // break the files into groups so we can generate a file of the size requested by the customer
                let mut upload_file_parts: Vec<ConsolidationFile> =
                    splice_files_list(self.requested_size_bytes, &mut files);

                // keep track of the processed files to delete
                let mut files_to_delete: Vec<String> = Vec::new();
                let mut completed_parts: Vec<CompletedPart> = Vec::new();

                // if there's a directory, the trailing slash is not present so add it
                let newfile_dir = if (dir.is_empty()) || (dir.ends_with('/')) {
                    dir.clone()
                } else {
                    format!("{}/", dir.clone())
                };

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
                    new_file_key = format!("{newfile_dir}merged_{time_since_epoch}.log");

                    match self
                        .s3_client
                        .get_object()
                        .bucket(self.bucket.clone())
                        .key(new_file_key.clone())
                        .send()
                        .await
                    {
                        Ok(_data) => {
                            info!(
                                "bucket={}, Merged file already exists, file={}",
                                self.bucket.clone(),
                                new_file_key.clone(),
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                        Err(_e) => {
                            // the file doesn't exist, break the loop and move on.
                            break;
                        }
                    };
                }
                info!(
                    "bucket={}, Starting consolidated file={}",
                    self.bucket.clone(),
                    new_file_key.clone(),
                );

                let tags = {
                    let mut tagging = url::form_urlencoded::Serializer::new(String::new());
                    tagging.append_pair("mezmo_pipeline_merged", "true");
                    tagging.finish()
                };

                let content_type = "text/x-log".to_owned();

                // use a multi-part upload for all the files that we need to process
                // set an expiration time for the file in case things go awry (vector dies, sink reloads, etc)
                // with the expiration time, the file will be auto-deleted if necessary.
                // we'll build the files up to the maximum size requested
                let expires = time_since_epoch + (TWO_HOURS_IN_SECONDS);
                let aws_time = AwsDateTime::from_secs(expires as i64);
                let multi_part_upload = match self
                    .s3_client
                    .create_multipart_upload()
                    .bucket(self.bucket.clone())
                    .key(new_file_key.clone())
                    .set_content_type(Some(content_type))
                    .set_tagging(Some(tags))
                    .set_expires(Some(aws_time))
                    .send()
                    .await
                {
                    Ok(m) => {
                        info!(
                            "bucket={}, Successfully created multipart doc for file={}",
                            self.bucket.clone(),
                            new_file_key.clone(),
                        );
                        m
                    }
                    Err(e) => {
                        error!(
                            ?e,
                            "bucket={}, Failed to invoke multipart upload file={}",
                            self.bucket.clone(),
                            new_file_key.clone(),
                        );
                        continue;
                    }
                };

                let upload_id: String = multi_part_upload.upload_id().unwrap().to_owned();

                // The minimum file size for upload parts and copy parts 5 MB, so we'll manually consolidate
                // small files into one larger to fill the void. Using 5 MB files will allow us to achieve
                // 50 GB total over 10_000 parts
                let mut part_num: i32 = 0;
                let mut buf = BytesMut::with_capacity(0);
                let mut buf_files: Vec<String> = Vec::new();

                upload_file_parts.reverse(); //reverse the list so we can pop off in the correct order
                while let Some(file) = upload_file_parts.pop() {
                    let consolidated_file_has_data =
                        !files_to_delete.is_empty() || !buf_files.is_empty();

                    let (trim_open_bracket, trim_close_bracket, prepend_char) =
                        determine_download_properties(
                            self.output_format.clone(),
                            consolidated_file_has_data,
                            &upload_file_parts,
                        );

                    // if the file is compressed, we need to pull and decompress it
                    // so we can join it with other files
                    let file_bytes = match download_file_as_bytes(
                        self.s3_client,
                        self.bucket.clone(),
                        &file,
                        trim_open_bracket,
                        trim_close_bracket,
                        prepend_char,
                    )
                    .await
                    {
                        Ok(v) => {
                            info!("bucket={}, Downloaded file={:?}", self.bucket.clone(), file);
                            v
                        }
                        Err(e) => {
                            error!(
                                ?e,
                                "bucket={}, Failed to download file={}",
                                self.bucket.clone(),
                                file.key.clone(),
                            );
                            continue;
                        }
                    };

                    buf.extend_from_slice(&file_bytes);
                    buf_files.push(file.key.clone());

                    // if we've got the minimum for a multipart chunk, send it on to the server
                    if buf.len() as i64 >= MULTIPART_FILE_MIN_MB_I64 {
                        part_num += 1;

                        // cloning the buffer so its not moved
                        let body = bytes_to_bytestream(Bytes::from(buf.clone()));
                        let upload = match self
                            .s3_client
                            .upload_part()
                            .bucket(self.bucket.clone())
                            .key(new_file_key.clone())
                            .upload_id(upload_id.clone())
                            .part_number(part_num)
                            .body(body)
                            .send()
                            .await
                        {
                            Ok(u) => {
                                info!(
                                    "bucket={}, upload_id={}, Uploaded part={} for file={}",
                                    self.bucket.clone(),
                                    upload_id.clone(),
                                    part_num,
                                    new_file_key.clone(),
                                );
                                u
                            }
                            Err(e) => {
                                error!(
                                    ?e,
                                    "bucket={}, upload_id={}, Failed to upload new part={} for file={}",
                                    self.bucket.clone(),
                                    upload_id.clone(),
                                    part_num,
                                    new_file_key.clone(),
                                );
                                part_num -= 1;
                                continue;
                            }
                        };

                        // keep track of the part for completion
                        completed_parts.push(
                            CompletedPart::builder()
                                .e_tag(upload.e_tag().unwrap())
                                .part_number(part_num)
                                .build(),
                        );

                        for file in &buf_files {
                            files_to_delete.push(file.clone())
                        }

                        // reset the buffer by clearing the memory
                        buf = BytesMut::with_capacity(0);
                        buf_files.clear();
                    }

                    // make sure to not go over the max parts and leave
                    // one slot for any buffer that hasn't been pushed
                    if (part_num + 1) >= MAX_PARTS_MULTIPART_FILE {
                        break;
                    }
                }

                // there's still data in the buffer, so make that the final part.
                // the final upload part doesn't have to be the 5 MB min
                if !buf.is_empty() {
                    part_num += 1;

                    let upload = match self
                        .s3_client
                        .upload_part()
                        .bucket(self.bucket.clone())
                        .key(new_file_key.clone())
                        .upload_id(upload_id.clone())
                        .part_number(part_num)
                        .body(bytes_to_bytestream(Bytes::from(buf)))
                        .send()
                        .await
                    {
                        Ok(u) => {
                            info!(
                                "bucket={}, upload_id={}, Uploaded part={} for file={}",
                                self.bucket.clone(),
                                upload_id.clone(),
                                part_num,
                                new_file_key.clone(),
                            );
                            u
                        }
                        Err(e) => {
                            error!(
                                ?e,
                                "bucket={}, upload_id={}, Failed to upload new part={} for file={}",
                                self.bucket.clone(),
                                upload_id.clone(),
                                part_num,
                                new_file_key.clone()
                            );
                            continue;
                        }
                    };

                    // keep track of the part for completion
                    completed_parts.push(
                        CompletedPart::builder()
                            .e_tag(upload.e_tag().unwrap())
                            .part_number(part_num)
                            .build(),
                    );

                    for file in &buf_files {
                        files_to_delete.push(file.clone())
                    }
                }

                // time to mark the entire file as complete
                match self
                    .s3_client
                    .complete_multipart_upload()
                    .bucket(self.bucket.clone())
                    .key(new_file_key.clone())
                    .upload_id(upload_id.clone())
                    .request_payer(RequestPayer::Requester)
                    .multipart_upload(
                        CompletedMultipartUpload::builder()
                            .set_parts(Some(completed_parts))
                            .build(),
                    )
                    .send()
                    .await
                {
                    Ok(u) => {
                        info!(
                            "bucket={}, upload_id={}, Completed multipart upload for file={}",
                            self.bucket.clone(),
                            upload_id.clone(),
                            new_file_key.clone()
                        );
                        u
                    }
                    Err(e) => {
                        error!(?e, "bucket={}, upload_id={}, Failed to complete multipart upload for file={}", self.bucket.clone(), upload_id.clone(), new_file_key.clone());

                        // completing the file didn't work out, so abort it completely.
                        match self
                            .s3_client
                            .abort_multipart_upload()
                            .bucket(self.bucket.clone())
                            .key(new_file_key.clone())
                            .upload_id(upload_id.clone())
                            .request_payer(RequestPayer::Requester)
                            .send()
                            .await
                        {
                            Ok(_v) => info!("bucket={}, upload_id={}, Aborted multipart upload for file={}", self.bucket.clone(), upload_id.clone(), new_file_key.clone()),
                            Err(e) => error!(?e, "bucket={}, upload_id={}, Failed to abort multipart upload for file={}", self.bucket.clone(), upload_id.clone(), new_file_key.clone()),
                        };

                        continue;
                    }
                };

                // remove all the files from S3 that have been merged into the larger file
                for file in files_to_delete {
                    match self
                        .s3_client
                        .delete_object()
                        .bucket(self.bucket.clone())
                        .key(file.clone())
                        .send()
                        .await
                    {
                        Ok(_result) => {
                            info!(
                                message = format!(
                                    "File={} removed from bucket={} after merge successful file consolidation",
                                    file.clone(),
                                    self.bucket.clone()
                                )
                            )
                        }
                        Err(e) => error!(
                            ?e,
                            "bucket={}, Failed to delete merged file={}",
                            self.bucket.clone(),
                            file.clone()
                        ),
                    };
                }
            } // end files to consolidate loop
        } // end files by directory loop
    }
}

// helper class for the files that we're consolidating into a single file
#[derive(Debug, Clone)]
pub struct ConsolidationFile {
    pub compressed: bool,
    pub size: i64,
    pub key: String,
    pub last_modified: i64,
}

impl ConsolidationFile {
    pub const fn new(
        compressed: bool,
        size: i64,
        key: String,
        last_modified: i64,
    ) -> ConsolidationFile {
        ConsolidationFile {
            compressed,
            size,
            key,
            last_modified,
        }
    }
}

fn bytes_to_bytestream(buf: Bytes) -> ByteStream {
    ByteStream::from(buf)
}

/*
    handles taking in a list of files and grabbing however many
    files which combined is the requested size.
    @requested_size_bytes: the total size of data requested
    @files: the list of files to pick from.
    @@returns: a vector of consolidation files
*/
fn splice_files_list(
    requested_size_bytes: i64,
    files: &mut Vec<ConsolidationFile>,
) -> Vec<ConsolidationFile> {
    let mut total_bytes: i64 = 0;
    for i in 0..files.len() {
        total_bytes += files[i].size;

        if total_bytes >= requested_size_bytes {
            return files.drain(0..i + 1).collect();
        }
    }

    std::mem::take(files)
}

/*
    Handles reading the s3 bucket and evaluating the files
    which can be merged into larger files
    @client: the s3 client
    @bucket: the s3 bucket
    @base_path: the base path for the files
    @requested_file_type: the type of file to find
    @@returns: Vector<ConsolidationFile>, the files which can be merged.
*/
pub async fn get_files_to_consolidate(
    client: &S3Client,
    bucket: String,
    base_path: String,
    requested_file_type: String,
) -> Result<Vec<ConsolidationFile>, Error> {
    let mut files_to_consolidate: Vec<ConsolidationFile> = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let list_result = client
            .list_objects_v2()
            .bucket(bucket.clone())
            .prefix(base_path.clone())
            .set_continuation_token(continuation_token)
            .send()
            .await?;

        if list_result.contents.is_none() {
            info!(
                "bucket={}, base_path={}, No files found",
                bucket.clone(),
                base_path.clone(),
            );
            break;
        }

        //determine if there is more records to be retrieved in another request
        //the default is 1000 records which we'll stick with until we need to do
        //some tuning
        if list_result.is_truncated().is_some_and(|r| r) {
            continuation_token = Some(list_result.next_continuation_token().unwrap().to_string());
        } else {
            continuation_token = None;
        }

        for key_object in list_result.contents() {
            let key = key_object.key().unwrap();

            let tag_result = client
                .get_object_tagging()
                .bucket(bucket.clone())
                .key(key)
                .send()
                .await?;

            let mut mezmo_merged_file = false;
            let mut mezmo_produced_file = false;
            let mut can_combine = false;

            let tags = tag_result.tag_set();
            for tag in tags.iter() {
                match tag.key() {
                    "mezmo_pipeline_merged" => mezmo_merged_file = true,
                    "mezmo_pipeline_s3_sink" => mezmo_produced_file = true,
                    "mezmo_pipeline_s3_type" => can_combine = requested_file_type == tag.value(),
                    _ => (),
                }
            }

            // scroll through the tags and determine if we can even combine the file
            if mezmo_merged_file || !mezmo_produced_file || !can_combine {
                continue;
            }

            // figure out the object size and keys
            match client
                .head_object()
                .bucket(bucket.clone())
                .key(key)
                .send()
                .await
            {
                Ok(head) => {
                    let compressed = head.content_encoding().unwrap_or_default() == "gzip";
                    let size = head.content_length().expect("object size missing");
                    let key = key.to_string();
                    let last_modified = key_object.last_modified().unwrap().secs();

                    files_to_consolidate.push(ConsolidationFile::new(
                        compressed,
                        size,
                        key,
                        last_modified,
                    ));
                }
                Err(e) => error!(?e, "bucket={}, Failed to head file={}", bucket.clone(), key),
            };
        } // end retrieving objects and sorting

        // complete processing if there is no token to continue with
        if continuation_token.is_none() {
            break;
        }
    }

    files_to_consolidate.sort_by_key(|x| x.last_modified);
    Ok(files_to_consolidate)
}

/*
    Handles downloading the byte data from the provided file and returns
    the vector representation of the bytes.
    If the file is compressed, handles also decompressing the document
    via gzip compression.
    @client: the s3 client
    @bucket: the s3 bucket
    @files_to_delete: populated with the files which were successfully downloaded
    @@returns: Bytes, the byte data representing all the downloaded files
*/
async fn download_file_as_bytes(
    client: &S3Client,
    bucket: String,
    file: &ConsolidationFile,
    trim_open_bracket: bool,
    trim_close_bracket: bool,
    prepend_char: Option<char>,
) -> Result<Bytes, Error> {
    let b: Bytes = download_bytes(client, bucket.clone(), file.key.clone()).await?;

    let mut vec: Vec<u8>;
    if file.compressed {
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
                break;
            }
        }
    }

    if trim_close_bracket && !vec.is_empty() {
        loop {
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
    Determines how we should handle the file when downloading by
    automatically aoppending commas or newlines between data as well
    as trimming brackets to handle json files
    @output_format: the type of file being written
    @consolidated_file_has_data: indicates if we've already written data
    @upload_file_parts: the remaining parts for the file
    @@returns: tuple containing [
        trim_open_bracket (bool),
        trim_close_bracket (bool),
        prepend_char (Option<char>)
    ]
*/
fn determine_download_properties(
    output_format: String,
    consolidated_file_has_data: bool,
    upload_file_parts: &[ConsolidationFile],
) -> (bool, bool, Option<char>) {
    let is_standard_json_file: bool = output_format == "json";

    let trim_open_bracket = is_standard_json_file && consolidated_file_has_data;
    let trim_close_bracket = is_standard_json_file && !upload_file_parts.is_empty();

    let prepend_char: Option<char> = if is_standard_json_file && consolidated_file_has_data {
        Some(',')
    } else if consolidated_file_has_data {
        Some('\n')
    } else {
        None
    };

    (trim_open_bracket, trim_close_bracket, prepend_char)
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
    _ = in_buf.read_to_end(&mut vec);
    vec
}

/*
    Handles retrieval of the s3 document from storage
    @client: the s3 client
    @bucket: the s3 bucket
    @key: the file key
    @@returns: the byte data of the file
*/
async fn download_bytes(client: &S3Client, bucket: String, key: String) -> Result<Bytes, Error> {
    let object = client.get_object().bucket(bucket).key(key).send().await?;

    let body = match object.body.collect().await {
        Ok(body) => body.into_bytes(),
        Err(e) => {
            return Err(Error::NotFound(
                aws_sdk_s3::types::error::NotFound::builder()
                    .message(format!("{e}"))
                    .build(),
            ))
        }
    };

    Ok(body)
}

fn group_files_by_directory(
    list: Vec<ConsolidationFile>,
) -> HashMap<String, Vec<ConsolidationFile>> {
    let mut hm: HashMap<String, Vec<ConsolidationFile>> = HashMap::new();
    for f in list {
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
    use flate2::read::GzEncoder;
    use flate2::Compression;
    use std::io::Read;

    use crate::sinks::aws_s3::file_consolidation_processor::decompress_gzip;
    use crate::sinks::aws_s3::file_consolidation_processor::group_files_by_directory;
    use crate::sinks::aws_s3::file_consolidation_processor::splice_files_list;
    use crate::sinks::aws_s3::file_consolidation_processor::ConsolidationFile;

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
                last_modified: 1,
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
                last_modified: 1,
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
                last_modified: 1,
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
            last_modified: 1,
        }
    }
}
