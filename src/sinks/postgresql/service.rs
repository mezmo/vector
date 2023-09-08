use crate::{event::EventStatus, sinks::postgresql::PostgreSQLSinkError};
use bytes::BytesMut;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use futures::future::BoxFuture;
use std::{
    borrow::Cow,
    error::Error,
    sync::Arc,
    task::{Context, Poll},
};
use tokio_postgres::types::{to_sql_checked, IsNull, ToSql, Type};
use tower::Service;
use vector_common::byte_size_of::ByteSizeOf;
use vector_common::finalization::{EventFinalizers, Finalizable};
use vector_common::internal_event::{
    ByteSize, BytesSent, InternalEventHandle, Protocol, Registered,
};
use vector_common::request_metadata::{MetaDescriptive, RequestMetadata};
use vector_core::{internal_event::CountByteSize, stream::DriverResponse};
use vrl::value::Value;

pub struct PostgreSQLRequest {
    data: Vec<Value>,
    finalizers: EventFinalizers,
}

impl PostgreSQLRequest {
    pub(crate) fn new(data: Vec<Value>, finalizers: EventFinalizers) -> Self {
        Self { data, finalizers }
    }
}

impl Finalizable for PostgreSQLRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        self.finalizers.take_finalizers()
    }
}

impl MetaDescriptive for PostgreSQLRequest {
    fn get_metadata(&self) -> RequestMetadata {
        // RequestMetadata is not relevant for the output of this sink, but
        // is required by the trait bounds of service driver
        RequestMetadata::new(0, 0, 0, 0, 0)
    }
}

pub struct PostgreSQLResponse {
    bytes_sent: usize,
}

impl DriverResponse for PostgreSQLResponse {
    fn event_status(&self) -> EventStatus {
        EventStatus::Delivered
    }

    fn events_sent(&self) -> CountByteSize {
        CountByteSize(1, self.bytes_sent)
    }
}

pub struct PostgreSQLService {
    connection_pool: Arc<Pool>,
    sql: String,
    bytes_sent_handle: Registered<BytesSent>,
}

impl PostgreSQLService {
    pub(crate) fn new(connection_pool: Pool, sql: String) -> Self {
        let connection_pool = Arc::new(connection_pool);
        Self {
            connection_pool,
            sql,
            bytes_sent_handle: register!(BytesSent::from(Protocol::from("postgresql"))),
        }
    }
}

impl Service<PostgreSQLRequest> for PostgreSQLService {
    type Response = PostgreSQLResponse;
    type Error = PostgreSQLSinkError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: PostgreSQLRequest) -> Self::Future {
        let connection_pool = Arc::clone(&self.connection_pool);
        let sql = self.sql.clone();
        let bytes_sent_handle = self.bytes_sent_handle.clone();
        Box::pin(async move {
            let client = match connection_pool.get().await {
                Ok(client) => client,
                Err(source) => return Err(PostgreSQLSinkError::PoolError { source }),
            };

            let prep_stmt = match client.prepare_cached(&sql).await {
                Ok(prep_stmt) => prep_stmt,
                Err(source) => return Err(PostgreSQLSinkError::SqlError { source }),
            };

            let mut bytes_sent = 0;
            let params = req
                .data
                .iter()
                .map(|value| {
                    bytes_sent += value.allocated_bytes();
                    ValueSqlAdapter(value)
                })
                .collect::<Vec<_>>();
            let params = params.iter().map(|x| x as &(dyn ToSql + Sync));

            let res = match client.execute_raw(&prep_stmt, params).await {
                Ok(res) => res,
                Err(source) => return Err(PostgreSQLSinkError::SqlError { source }),
            };

            debug!("postgres execute successful; {res} rows modified");
            bytes_sent_handle.emit(ByteSize(bytes_sent));
            Ok(PostgreSQLResponse { bytes_sent })
        })
    }
}

#[derive(Debug)]
struct ValueSqlAdapter<'a>(&'a Value);

impl ToSql for ValueSqlAdapter<'_> {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        let value = self.0;
        match value.kind_str() {
            "string" => {
                let raw_value = value.as_str().expect("string");
                <Cow<str> as ToSql>::to_sql(&raw_value, ty, out)
            }
            "timestamp" => {
                let raw_value = value.as_timestamp().expect("timestamp");
                <DateTime<Utc> as ToSql>::to_sql(raw_value, ty, out)
            }
            "integer" => {
                let raw_value = value.as_integer().expect("integer");
                <i64 as ToSql>::to_sql(&raw_value, ty, out)
            }
            "float" => {
                let raw_value = value.as_float().expect("float").into_inner();
                <f64 as ToSql>::to_sql(&raw_value, ty, out)
            }
            "boolean" => {
                let raw_value = value.as_boolean().expect("boolean");
                <bool as ToSql>::to_sql(&raw_value, ty, out)
            }
            _ => {
                // Treat unhandled cases, map, array and null as null values
                <Option<i64> as ToSql>::to_sql(&None, ty, out)
            }
        }
    }

    fn accepts(_ty: &Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    to_sql_checked!();
}
