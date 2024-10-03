use snafu::Snafu;
use tokio_postgres::error::Error as PostgreSQLError;

pub(crate) mod config;
mod integration_tests;
mod metric_utils;
pub(crate) mod service;
pub(crate) mod sink;

#[derive(Debug, Snafu)]
pub enum PostgreSQLSinkError {
    #[snafu(display("Failed to obtain connection from pool: {}", message))]
    PoolError { message: String },

    #[snafu(display("Failed to execute DB statement: {}", source))]
    SqlError { source: PostgreSQLError },

    #[snafu(display("Cannot convert event type into Postgres request"))]
    UnsupportedEventType,

    #[snafu(display("Failed to find data for field {} in event object", field_name))]
    MissingField { field_name: String },

    #[snafu(display("Field {} in conflict setting does not appear in field map.", field))]
    UndefinedConflictField { field: String },
}
