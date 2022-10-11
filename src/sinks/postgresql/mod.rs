use snafu::Snafu;
use deadpool_postgres::PoolError;
use tokio_postgres::error::Error as PostgreSQLError;
use crate::config::SinkDescription;

pub (crate) mod config;
pub (crate) mod service;
pub (crate) mod sink;
mod metric_utils;

use self::config::PostgreSQLSinkConfig;

inventory::submit! {
    SinkDescription::new::<PostgreSQLSinkConfig>("postgresql")
}

#[derive(Debug, Snafu)]
pub enum PostgreSQLSinkError {
    #[snafu(display("Failed to obtain connection from pool: {}", source))]
    PoolError {source: PoolError},

    #[snafu(display("Failed to execute DB statement: {}", source))]
    SqlError {source: PostgreSQLError},

    #[snafu(display("Cannot convert event type into Postgres request"))]
    UnsupportedEventType,

    #[snafu(display("Failed to find data for field {} in event object", field_name))]
    MissingField {field_name: String},

    #[snafu(display("Field {} in conflict setting does not appear in field map.", field))]
    UndefinedConflictField {field: String},
}
