use crate::{
    config::{AcknowledgementsConfig, DataType, GenerateConfig, Input, SinkConfig, SinkContext},
    sinks::{
        Healthcheck, VectorSink,
        postgresql::sink::{PostgreSQLSink, healthcheck},
    },
};
use async_trait::async_trait;
use futures::FutureExt;
use typetag::serde;
use vector_lib::{config::log_schema, configurable::configurable_component};

/// Column/Event field mapping.
#[configurable_component]
#[derive(Clone, Debug, PartialEq)]
pub struct PostgreSQLFieldConfig {
    /// The name of the table column to write the data into.
    pub name: String,

    /// The VRL path used to access the data from the event object.
    pub path: String,
}

impl Default for PostgreSQLFieldConfig {
    fn default() -> Self {
        Self {
            name: "message".to_owned(),
            path: log_schema().message_key().unwrap().to_string(),
        }
    }
}

/// Schema information for the output table in PostgreSQL.
#[configurable_component]
#[derive(Clone, Debug, PartialEq)]
pub struct PostgreSQLSchemaConfig {
    /// Name of the table to write information to.
    pub table: String,

    #[configurable(derived)]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub fields: Vec<PostgreSQLFieldConfig>,
}

impl Default for PostgreSQLSchemaConfig {
    fn default() -> Self {
        Self {
            table: "vector_data".to_owned(),
            fields: vec![Default::default()],
        }
    }
}

/// Supported options to deal with insert conflicts.
#[configurable_component]
#[derive(Clone, Debug, PartialEq)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PostgreSQLConflictsConfig {
    /// Drop conflicting insert values without generating an error.
    Nothing {
        /// The list of unique constrained fields that would cause a conflict.
        target: Vec<String>,
    },

    /// Update fields of the existing row if the insert causes a conflict.
    Update {
        /// The list of unique constrained fields that would cause a conflict.
        target: Vec<String>,

        /// The list of fields that should be updated with event object. These fields
        /// need to be defined in the schema configuration section.
        fields: Vec<String>,
    },
}

/// Configuration for the `postgresql` sink.
#[configurable_component(sink("postgresql"))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostgreSQLSinkConfig {
    /// The connection URI for the postgres database to write data into. This is of the form
    /// `postgresql://[userspec@][hostspec][/dbname]`
    ///    where userspec is `user[:password]`
    ///    and hostspec is `[host][:port]`
    pub connection: String,

    #[configurable(derived)]
    pub schema: PostgreSQLSchemaConfig,

    #[configurable(derived)]
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub conflicts: Option<PostgreSQLConflictsConfig>,

    /// Maximum size of the Postgres connection pool for this instance.
    /// Defaults to 4
    #[serde(default = "default_max_pool_size")]
    pub max_pool_size: usize,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::is_default"
    )]
    pub acknowledgements: AcknowledgementsConfig,
}

impl Default for PostgreSQLSinkConfig {
    fn default() -> Self {
        Self {
            connection: "postgres://localhost:5432/db".to_owned(),
            schema: Default::default(),
            conflicts: None,
            max_pool_size: 1,
            acknowledgements: Default::default(),
        }
    }
}

pub(crate) const fn default_max_pool_size() -> usize {
    4
}

impl GenerateConfig for PostgreSQLSinkConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            connection: "postgresql://postgres@localhost:5431/vector".to_owned(),
            schema: Default::default(),
            conflicts: Default::default(),
            max_pool_size: default_max_pool_size(),
            acknowledgements: AcknowledgementsConfig::default(),
        })
        .unwrap()
    }
}

#[async_trait]
#[typetag::serde(name = "postgresql")]
impl SinkConfig for PostgreSQLSinkConfig {
    async fn build(&self, _cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let sink = PostgreSQLSink::new(self.clone())?;
        let hc = healthcheck(self.clone()).boxed();
        Ok((VectorSink::from_event_streamsink(sink), hc))
    }

    fn input(&self) -> Input {
        Input::new(DataType::Log | DataType::Metric)
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}
