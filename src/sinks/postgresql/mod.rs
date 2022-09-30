use crate::config::SinkDescription;

pub (crate) mod config;
pub (crate) mod service;
pub (crate) mod sink;

use self::config::PostgreSQLSinkConfig;

inventory::submit! {
    SinkDescription::new::<PostgreSQLSinkConfig>("postgresql")
}