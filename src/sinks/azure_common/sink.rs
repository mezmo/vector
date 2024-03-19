use std::fmt;

use crate::sinks::{prelude::*, util::partitioner::KeyPartitioner};

// MEZMO: added dependency for s3-sink file consolidation
use crate::sinks::azure_blob::file_consolidator_async::FileConsolidatorAsync;

pub struct AzureBlobSink<Svc, RB> {
    service: Svc,
    request_builder: RB,
    partitioner: KeyPartitioner,
    batcher_settings: BatcherSettings,
    // MEZMO: added property for azure-blob-sink file consolidation
    file_consolidator: Option<FileConsolidatorAsync>,
}

impl<Svc, RB> AzureBlobSink<Svc, RB> {
    pub const fn new(
        service: Svc,
        request_builder: RB,
        partitioner: KeyPartitioner,
        batcher_settings: BatcherSettings,
        file_consolidator: Option<FileConsolidatorAsync>,
    ) -> Self {
        Self {
            service,
            request_builder,
            partitioner,
            batcher_settings,
            file_consolidator,
        }
    }
}

impl<Svc, RB> AzureBlobSink<Svc, RB>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(String, Vec<Event>)> + Send + Sync + 'static,
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
            .filter_map(|(key, batch)| async move {
                // We don't need to emit an error here if the event is dropped since this will occur if the template
                // couldn't be rendered during the partitioning. A `TemplateRenderingError` is already emitted when
                // that occurs.
                key.map(move |k| (k, batch))
            })
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
            .protocol("https")
            .run()
            .await;

        // MEZMO: added file consolidation processing
        //stop the file consolidation process if necessary
        file_consolidator.stop();

        result
    }
}

#[async_trait]
impl<Svc, RB> StreamSink<Event> for AzureBlobSink<Svc, RB>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(String, Vec<Event>)> + Send + Sync + 'static,
    RB::Error: fmt::Display + Send,
    RB::Request: Finalizable + MetaDescriptive + Send,
{
    async fn run(mut self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
