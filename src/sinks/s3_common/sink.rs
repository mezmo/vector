use std::fmt;

use crate::sinks::prelude::*;

use super::partitioner::{S3KeyPartitioner, S3PartitionKey};

// MEZMO: added dependency for s3-sink file consolidation
use crate::sinks::aws_s3::file_consolidator_async::FileConsolidatorAsync;

pub struct S3Sink<Svc, RB> {
    service: Svc,
    request_builder: RB,
    partitioner: S3KeyPartitioner,
    batcher_settings: BatcherSettings,

    // MEZMO: added property for s3-sink file consolidation
    file_consolidator: Option<FileConsolidatorAsync>,
}

impl<Svc, RB> S3Sink<Svc, RB> {
    pub const fn new(
        service: Svc,
        request_builder: RB,
        partitioner: S3KeyPartitioner,
        batcher_settings: BatcherSettings,
        file_consolidator: Option<FileConsolidatorAsync>,
    ) -> Self {
        Self {
            partitioner,
            service,
            request_builder,
            batcher_settings,
            file_consolidator,
        }
    }
}

impl<Svc, RB> S3Sink<Svc, RB>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(S3PartitionKey, Vec<Event>)> + Send + Sync + 'static,
    RB::Error: fmt::Display + Send,
    RB::Request: Finalizable + MetaDescriptive + Send,
{
    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let partitioner = self.partitioner;
        let settings = self.batcher_settings;
        let request_builder = self.request_builder;

        // MEZMO: added file consolidation processing
        // initiate the file consolidation process if necessary
        let mut file_consolidator = self.file_consolidator.unwrap_or_default();
        file_consolidator.start();

        let result = input
            .batched_partitioned(partitioner, || settings.as_byte_size_config())
            .filter_map(|(key, batch)| async move { key.map(move |k| (k, batch)) })
            .request_builder(default_request_builder_concurrency_limit(), request_builder)
            .filter_map(|request| async move {
                match request {
                    Err(error) => {
                        emit!(SinkRequestBuildError { error });
                        None
                    }
                    Ok(req) => Some(req),
                }
            })
            .into_driver(self.service)
            .run()
            .await;

        // MEZMO: added file consolidation processing
        //stop the file consolidation process if necessary
        file_consolidator.stop();

        result
    }
}

#[async_trait]
impl<Svc, RB> StreamSink<Event> for S3Sink<Svc, RB>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(S3PartitionKey, Vec<Event>)> + Send + Sync + 'static,
    RB::Error: fmt::Display + Send,
    RB::Request: Finalizable + MetaDescriptive + Send,
{
    async fn run(mut self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
