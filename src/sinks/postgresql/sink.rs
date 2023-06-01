use crate::sinks::postgresql::metric_utils::get_from_metric;
use crate::sinks::postgresql::PostgreSQLSinkError;
use crate::{
    event::Event,
    sinks::{
        postgresql::{
            config::{
                PostgreSQLConflictsConfig, PostgreSQLFieldConfig, PostgreSQLSchemaConfig,
                PostgreSQLSinkConfig,
            },
            service::{PostgreSQLRequest, PostgreSQLService},
        },
        util::{SinkBuilderExt, StreamSink},
    },
};
use async_trait::async_trait;
use deadpool_postgres::{Config, PoolConfig, Runtime};
use futures::{future, stream::BoxStream, StreamExt};
use std::string::FromUtf8Error;
use tokio_postgres::NoTls;
use url::Url;
use vector_common::finalization::Finalizable;

pub struct PostgreSQLSink {
    schema_config: PostgreSQLSchemaConfig,
    service: PostgreSQLService,
}

impl PostgreSQLSink {
    pub(crate) fn new(config: PostgreSQLSinkConfig) -> crate::Result<Self> {
        let pool_conf = pool_config(&config.connection, config.max_pool_size)?;
        let connection_pool = pool_conf.create_pool(Some(Runtime::Tokio1), NoTls)?;

        let sql = generate_sql(
            &config.schema.table,
            &config.schema.fields,
            &config.conflicts,
        )?;
        debug!("generated sql from sink config: {sql}");

        let service = PostgreSQLService::new(connection_pool, sql);
        let schema_config = config.schema;
        Ok(Self {
            schema_config,
            service,
        })
    }
}

fn pool_config(connect_url: &str, max_pool_size: usize) -> crate::Result<Config> {
    let url = Url::parse(connect_url)?;
    if url.scheme() != "postgresql" && url.scheme() != "postgres" {
        error!(
            message = "Invalid scheme for sink connection string",
            scheme = url.scheme()
        );
    }

    let mut conf = Config::new();
    conf.host = url.host().map(|h| h.to_string());
    conf.port = url.port();
    if !url.username().is_empty() {
        conf.user = Some(url_decode(url.username())?);
    }
    if let Some(password) = url.password() {
        let password = url_decode(password)?;
        conf.password = Some(password);
    }
    if let Some(mut path_seg) = url.path_segments() {
        if let Some(first) = path_seg.next() {
            conf.dbname = Some(first.to_owned());
        }
    }

    let max_pool_size = if max_pool_size == 0 {
        warn!(
            "Configuration attempted to set max_pool_size to 0. Using the default of {}",
            super::config::default_max_pool_size()
        );
        super::config::default_max_pool_size()
    } else {
        max_pool_size
    };

    conf.pool = Some(PoolConfig::new(max_pool_size));

    Ok(conf)
}

/// Build up the sql insert statement trying to avoid intermediate memory allocations while building
/// up the statement string.
fn generate_sql(
    table: &str,
    fields: &[PostgreSQLFieldConfig],
    conflicts: &Option<PostgreSQLConflictsConfig>,
) -> crate::Result<String> {
    let mut field_list = String::new();
    let mut param_list = String::new();
    let mut field_iter = fields.iter().enumerate();
    if let Some((_, field)) = field_iter.next() {
        field_list.push_str(&field.name);
        param_list.push_str("$1");

        for (idx, field) in field_iter {
            field_list.push(',');
            field_list.push_str(&field.name);

            param_list.push_str(",$");
            param_list.push_str(&(idx + 1).to_string());
        }
    }

    let mut conflict_chunk = String::new();
    if let Some(conflicts) = conflicts {
        conflict_chunk.push_str(" ON CONFLICT (");
        match &conflicts {
            PostgreSQLConflictsConfig::Nothing { target } => {
                conflict_chunk.push_str(&target.join(","));
                conflict_chunk.push_str(") DO NOTHING");
            }
            PostgreSQLConflictsConfig::Update {
                target,
                fields: update_fields,
            } => {
                conflict_chunk.push_str(&target.join(","));
                conflict_chunk.push_str(") DO UPDATE SET ");

                let mut update_iter = update_fields.iter();
                if let Some(u_field) = update_iter.next() {
                    let f_idx = match fields.iter().position(|c| c.name == *u_field) {
                        Some(i) => i + 1,
                        None => {
                            let field = u_field.to_owned();
                            return Err(Box::new(PostgreSQLSinkError::UndefinedConflictField {
                                field,
                            }));
                        }
                    };
                    conflict_chunk.push_str(u_field);
                    conflict_chunk.push_str("=$");
                    conflict_chunk.push_str(&f_idx.to_string());

                    for u_field in update_iter {
                        conflict_chunk.push(',');
                        let f_idx = match fields.iter().position(|c| c.name == *u_field) {
                            Some(i) => i + 1,
                            None => {
                                let field = u_field.to_owned();
                                return Err(Box::new(
                                    PostgreSQLSinkError::UndefinedConflictField { field },
                                ));
                            }
                        };
                        conflict_chunk.push_str(u_field);
                        conflict_chunk.push_str("=$");
                        conflict_chunk.push_str(&f_idx.to_string());
                    }
                }
            }
        }
    }

    Ok(format!(
        "INSERT INTO {table} ({field_list}) VALUES ({param_list}){conflict_chunk}"
    ))
}

fn url_decode(input: &str) -> Result<String, FromUtf8Error> {
    urlencoding::decode(input).map(|c| c.to_string())
}

pub(crate) async fn healthcheck(_config: PostgreSQLSinkConfig) -> crate::Result<()> {
    // Future enhancement: inspect the system tables to validate the config schema and
    // conflict options refer to actual items in the database.
    Ok(())
}

fn build_request(
    schema: &PostgreSQLSchemaConfig,
    mut e: Event,
) -> crate::Result<PostgreSQLRequest> {
    let mut data = Vec::new();
    for field in &schema.fields {
        let value = match e {
            Event::Log(_) => e.as_log().get(&*field.path).map(ToOwned::to_owned).ok_or(
                PostgreSQLSinkError::MissingField {
                    field_name: field.path.to_owned(),
                },
            ),
            Event::Metric(_) => get_from_metric(e.as_metric(), &*field.path).ok_or(
                PostgreSQLSinkError::MissingField {
                    field_name: field.path.to_owned(),
                },
            ),
            _ => Err(PostgreSQLSinkError::UnsupportedEventType),
        }?;
        data.push(value);
    }
    Ok(PostgreSQLRequest::new(data, e.take_finalizers()))
}

#[async_trait]
impl StreamSink<Event> for PostgreSQLSink {
    async fn run(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let schema_config = self.schema_config.clone();
        let sink = input
            .filter_map(|e| {
                let req = build_request(&schema_config, e)
                    .map_err(|err| {
                        error!("Failed to convert event into PostgresSQL request: {err}")
                    })
                    .ok();
                future::ready(req)
            })
            .into_driver(self.service);
        sink.run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sql_single_field_test() {
        let field_conf = vec![PostgreSQLFieldConfig {
            name: "message".to_owned(),
            path: ".message".to_owned(),
        }];
        let actual = generate_sql("tbl_123", &field_conf, &None).unwrap();
        assert_eq!(actual, "INSERT INTO tbl_123 (message) VALUES ($1)");
    }

    #[test]
    fn generate_sql_multi_field_test() {
        let field_conf = vec![
            PostgreSQLFieldConfig {
                name: "timestamp".to_owned(),
                path: ".timestamp".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "message".to_owned(),
                path: ".message".to_owned(),
            },
        ];
        let actual = generate_sql("tbl_123", &field_conf, &None).unwrap();
        assert_eq!(
            actual,
            "INSERT INTO tbl_123 (timestamp,message) VALUES ($1,$2)"
        );
    }

    #[test]
    fn generate_sql_confict_do_nothing_test() {
        let field_conf = vec![
            PostgreSQLFieldConfig {
                name: "timestamp".to_owned(),
                path: ".timestamp".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "host".to_owned(),
                path: ".host".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "message".to_owned(),
                path: ".message".to_owned(),
            },
        ];
        let confict_conf = Some(PostgreSQLConflictsConfig::Nothing {
            target: vec!["host".to_owned(), "message".to_owned()],
        });
        let actual = generate_sql("tbl_123", &field_conf, &confict_conf).unwrap();
        assert_eq!(
            actual,
            "INSERT INTO tbl_123 (timestamp,host,message) VALUES ($1,$2,$3) ON CONFLICT \
            (host,message) DO NOTHING"
        );
    }

    #[test]
    fn generate_sql_on_conflict_set_test() {
        let field_conf = vec![
            PostgreSQLFieldConfig {
                name: "timestamp".to_owned(),
                path: ".timestamp".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "ratio".to_owned(),
                path: ".ratio".to_owned(),
            },
            PostgreSQLFieldConfig {
                name: "message".to_owned(),
                path: ".message".to_owned(),
            },
        ];
        let confict_conf = Some(PostgreSQLConflictsConfig::Update {
            target: vec!["message".to_owned()],
            fields: vec!["ratio".to_owned(), "timestamp".to_owned()],
        });
        let actual = generate_sql("tbl_123", &field_conf, &confict_conf).unwrap();
        assert_eq!(
            actual,
            "INSERT INTO tbl_123 (timestamp,ratio,message) VALUES ($1,$2,$3) \
             ON CONFLICT (message) DO UPDATE SET ratio=$2,timestamp=$1"
        );
    }
}
