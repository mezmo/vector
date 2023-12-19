use bytes::{Bytes, BytesMut};
use std::{
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::read::GzDecoder;
use std::io;
use std::io::Cursor;

use aws_sdk_s3::{
    model::{CompletedMultipartUpload, CompletedPart},
    types::ByteStream,
    Client as S3Client, Error,
};

use aws_smithy_types::DateTime as AwsDateTime;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use md5::Digest;

const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
const MULTIPART_FILE_MIN_MB_I32: i32 = 5 * 1024 * 1024;
const ONE_MEGABYTE_USIZE: usize = 1024 * 1024;

// handles consolidating the small files within AWS into much larger files
#[derive(Debug)]
pub struct FileConsolidationProcessor<'a> {
    s3_client: &'a S3Client,
    bucket: String,
    key_prefix: String,
    requested_size_bytes: i32,
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
        key_prefix: String,
        requested_size_bytes: i32,
    ) -> Self {
        FileConsolidationProcessor {
            s3_client,
            bucket,
            key_prefix,
            requested_size_bytes,
        }
    }

    pub async fn run(self) {
        // retrieve the files list from s3 that we can process
        let mut files_to_consolidate: Vec<ConsolidationFile> = match get_files_to_consolidate(
            self.s3_client,
            self.bucket.clone(),
            self.key_prefix.clone(),
        )
        .await
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    ?e,
                    "bucket={}, prefix={}, Failed to retrieve files to consolidate",
                    self.bucket.clone(),
                    self.key_prefix.clone(),
                );
                let empty_files: Vec<ConsolidationFile> = Vec::new();
                empty_files
            }
        };

        // if we have no files to combine
        if files_to_consolidate.len() <= 1 {
            return;
        }

        // break the files into groups so we can generate a file of the
        // size requested by the customer
        while !files_to_consolidate.is_empty() {
            // keep track of the processed files to delete
            let mut files_to_delete: Vec<String> = Vec::new();
            let mut completed_parts: Vec<CompletedPart> = Vec::new();

            let upload_file_parts: Vec<ConsolidationFile> =
                splice_files_list(self.requested_size_bytes, &mut files_to_consolidate);

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
                new_file_key =
                    format!("{}{}_merged.log", self.key_prefix.clone(), time_since_epoch);

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

            //calculate the size of all the files
            //we make no assumptions about compression here if the file is gzip'd
            let mut total_bytes_of_all_files: i32 = 0;
            for record in upload_file_parts.iter() {
                total_bytes_of_all_files += record.size;
            }

            // there is a mimimum size of a multipart upload so we'll just upload a single file
            // if the directory has less than that amount.
            if total_bytes_of_all_files <= MULTIPART_FILE_MIN_MB_I32 {
                let bytes = match download_all_files_as_bytes(
                    self.s3_client,
                    self.bucket.clone(),
                    &upload_file_parts,
                    &mut files_to_delete,
                )
                .await
                {
                    Ok(data) => data,
                    Err(e) => {
                        error!(
                            ?e,
                            "bucket={}, Failed to download files",
                            self.bucket.clone(),
                        );
                        continue;
                    }
                };

                if bytes.is_empty() {
                    info!(
                        "bucket={}, Failed to download files={:?}",
                        self.bucket.clone(),
                        upload_file_parts,
                    );
                    continue;
                }

                let content_md5 = BASE64_STANDARD.encode(md5::Md5::digest(bytes.clone()));
                match self
                    .s3_client
                    .put_object()
                    .body(bytes_to_bytestream(bytes))
                    .bucket(self.bucket.clone())
                    .key(new_file_key.clone())
                    .set_content_type(Some(content_type))
                    .set_tagging(Some(tags))
                    .content_md5(content_md5)
                    .send()
                    .await
                {
                    Ok(f) => {
                        info!(
                            "bucket={}, Successfully put single consolidated file={} for files={:?}",
                            self.bucket.clone(),
                            new_file_key.clone(),
                            upload_file_parts
                        );
                        f
                    }
                    Err(e) => {
                        error!(
                            ?e,
                            "bucket={}, Failed to put single consolidated file={}",
                            self.bucket.clone(),
                            new_file_key.clone(),
                        );
                        continue;
                    }
                };
            } else {
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

                for file in &upload_file_parts {
                    // if we've got a plaintext file that meets the multipart min, then we can straight copy it via sdk
                    if !file.compressed && file.size >= MULTIPART_FILE_MIN_MB_I32 {
                        part_num += 1;

                        let source = format!("{}/{}", self.bucket.clone(), file.key.clone());
                        let encoded_source = urlencoding::encode(&source);

                        let copy = match self
                            .s3_client
                            .upload_part_copy()
                            .bucket(self.bucket.clone())
                            .key(new_file_key.clone())
                            .upload_id(upload_id.clone())
                            .copy_source(encoded_source)
                            .part_number(part_num)
                            .send()
                            .await
                        {
                            Ok(c) => {
                                info!(
                                    "bucket={}, upload_id={}, Copied part={} ({}) for file={}",
                                    self.bucket.clone(),
                                    upload_id.clone(),
                                    part_num,
                                    file.key.clone(),
                                    new_file_key.clone()
                                );
                                c
                            }
                            Err(e) => {
                                error!(
                                    ?e,
                                    "bucket={}, upload_id={}, Failed to put copy part file={}",
                                    self.bucket.clone(),
                                    upload_id.clone(),
                                    file.key.clone(),
                                );
                                part_num -= 1;
                                continue;
                            }
                        };

                        // keep track of the part for completion and deletion
                        completed_parts.push(
                            CompletedPart::builder()
                                .e_tag(copy.copy_part_result().unwrap().e_tag().unwrap())
                                .part_number(part_num)
                                .build(),
                        );

                        files_to_delete.push(file.key.clone());
                        continue;
                    }

                    // if the file is compressed, we need to pull and decompress it
                    // so we can join it with other files
                    // if its less than 5 megs, we need to pull it too to consolidate with other files
                    let vector =
                        match download_file_as_vec(self.s3_client, self.bucket.clone(), file).await
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

                    buf.extend_from_slice(&vector);
                    buf.extend_from_slice(b"\n"); //newline between file
                    buf_files.push(file.key.clone());

                    // if we've got the minimum for a multipart chunk, send it on to the server
                    if buf.len() as i32 >= MULTIPART_FILE_MIN_MB_I32 {
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
                            .send()
                            .await
                        {
                            Ok(_v) => info!("bucket={}, upload_id={}, Aborted multipart upload for file={}", self.bucket.clone(), upload_id.clone(), new_file_key.clone()),
                            Err(e) => error!(?e, "bucket={}, upload_id={}, Failed to abort multipart upload for file={}", self.bucket.clone(), upload_id.clone(), new_file_key.clone()),
                        };

                        continue;
                    }
                };
            }

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
            } // end else multipart logic
        } // end files to consolidate loop
    }
}

// helper class for the files that we're consolidating into a single file
#[derive(Debug)]
pub struct ConsolidationFile {
    pub compressed: bool,
    pub size: i32,
    pub key: String,
}

impl ConsolidationFile {
    pub const fn new(compressed: bool, size: i32, key: String) -> ConsolidationFile {
        ConsolidationFile {
            compressed,
            size,
            key,
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
    requested_size_bytes: i32,
    files: &mut Vec<ConsolidationFile>,
) -> Vec<ConsolidationFile> {
    let mut total_bytes: i32 = 0;
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
    @key_prefix: the prefix path for the files
    @@returns: Vector<ConsolidationFile>, the files which can be merged.
*/
pub async fn get_files_to_consolidate(
    client: &S3Client,
    bucket: String,
    key_prefix: String,
) -> Result<Vec<ConsolidationFile>, Error> {
    let list_result = client
        .list_objects_v2()
        .bucket(bucket.clone())
        .prefix(key_prefix.clone())
        .send()
        .await?;

    if list_result.contents.is_none() {
        info!(
            "bucket={}, prefix={}, No files found",
            bucket.clone(),
            key_prefix.clone(),
        );
        let v: Vec<ConsolidationFile> = Vec::new();
        return Ok(v);
    }

    let mut sorted_objects = list_result.contents().unwrap().to_vec();
    sorted_objects.sort_by_key(|x| x.last_modified().unwrap().secs());

    let mut files_to_consolidate: Vec<ConsolidationFile> = Vec::new();
    for key_object in sorted_objects {
        let key = key_object.key().unwrap();

        let tag_result = client
            .get_object_tagging()
            .bucket(bucket.clone())
            .key(key)
            .send()
            .await?;

        // this file is the result of a previous merge
        let mut mezmo_merged_file = false;
        // this file wasn't produced by the mezmo s3 process
        let mut mezmo_produced_file = false;
        // not breaking down standard json files as we don't want to load download
        // the whole file into memory. We're trying to straight memory copy here.
        let mut can_combine = false;

        let tags = tag_result.tag_set().unwrap_or_default();
        for tag in tags.iter() {
            match tag.key().unwrap_or_default() {
                "mezmo_pipeline_merged" => mezmo_merged_file = true,
                "mezmo_pipeline_s3_sink" => mezmo_produced_file = true,
                "mezmo_pipeline_s3_type" => match tag.value().unwrap() {
                    "ndjson" => can_combine = true,
                    "text" => can_combine = true,
                    "json" => can_combine = false,
                    _ => can_combine = false,
                },
                _ => info!(message = "unrecognized tag:".to_owned() + tag.key().unwrap()),
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
                let size = head.content_length() as i32;
                let key = key.to_string();

                files_to_consolidate.push(ConsolidationFile::new(compressed, size, key));
            }
            Err(e) => error!(?e, "bucket={}, Failed to head file={}", bucket.clone(), key),
        };
    } // end retrieving objects and sorting

    Ok(files_to_consolidate)
}

/*
    Handles downloading the byte data from all the provided files.
    Internally handles failures to retrieve a file by only populating
    the files_to_delete with files which were successfully downloaded.
    If the file is compressed, handles also decompressing the document
    via gzip compression.
    @client: the s3 client
    @bucket: the s3 bucket
    @files_to_delete: populated with the files which were successfully downloaded
    @@returns: Bytes, the byte data representing all the downloaded files
*/
async fn download_all_files_as_bytes(
    client: &S3Client,
    bucket: String,
    files: &[ConsolidationFile],
    files_to_delete: &mut Vec<String>,
) -> Result<Bytes, Error> {
    let mut buf = BytesMut::with_capacity(0);
    for file in files.iter() {
        let b: Bytes = match download_bytes(client, bucket.clone(), file.key.clone()).await {
            Ok(b) => b,
            Err(e) => {
                error!(
                    ?e,
                    "bucket={}, Failed to download file={}",
                    bucket.clone(),
                    file.key.clone(),
                );
                continue;
            }
        };

        if file.compressed {
            let decompressed = decompress_gzip(&b);
            buf.extend_from_slice(&decompressed);
        } else {
            buf.extend_from_slice(&b);
        }

        //add a newline as a separator
        buf.extend_from_slice(b"\n");

        // file downloaded successfully so mark it for potential deletion
        files_to_delete.push(file.key.clone());
    }

    Ok(Bytes::from(buf))
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
async fn download_file_as_vec(
    client: &S3Client,
    bucket: String,
    file: &ConsolidationFile,
) -> Result<Vec<u8>, Error> {
    let b: Bytes = download_bytes(client, bucket.clone(), file.key.clone()).await?;

    let mut buf = BytesMut::with_capacity(0);
    if file.compressed {
        let decompressed = decompress_gzip(&b);
        buf.extend_from_slice(&decompressed);
    } else {
        buf.extend_from_slice(&b);
    }

    Ok(buf.to_vec())
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
        Err(e) => return Err(Error::Unhandled(Box::new(e))),
    };

    Ok(body)
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use flate2::read::GzEncoder;
    use flate2::Compression;
    use std::io::Read;

    use crate::sinks::aws_s3::file_consolidation_processor::decompress_gzip;
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
}
