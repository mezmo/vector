use std::string::FromUtf8Error;
use async_trait::async_trait;
use futures::{future, stream::BoxStream, StreamExt};
use deadpool_postgres::{Config, Runtime};
use tokio_postgres::NoTls;
use url::Url;
use vector_common::finalization::Finalizable;
use crate::{
    event::{Event},
    sinks::{
        postgresql::{
            config::{
                PostgreSQLSinkConfig, PostgreSQLConflictsConfig, PostgreSQLFieldConfig,
                PostgreSQLSchemaConfig
            },
            service::{PostgreSQLService, PostgreSQLRequest},
        },
        util::{StreamSink, SinkBuilderExt},
    },
};

pub struct PostgreSQLSink {
    schema_config: PostgreSQLSchemaConfig,
    service: PostgreSQLService,
}

impl PostgreSQLSink {
    pub(crate) fn new(config: PostgreSQLSinkConfig) -> crate::Result<Self> {
        let pool_conf = pool_config(&config.connection)?;
        let connection_pool = pool_conf.create_pool(Some(Runtime::Tokio1), NoTls)?;

        let sql = generate_sql(&config.schema.table, &config.schema.fields, &config.conflicts);
        debug!("generated sql from sink config: {sql}");

        let service = PostgreSQLService::new(connection_pool, sql);
        let schema_config = config.schema;
        Ok(Self {schema_config, service})
    }
}

fn pool_config(connect_url: &str) -> crate::Result<Config> {
    let url = Url::parse(connect_url)?;
        if url.scheme() != "postgresql" && url.scheme() != "postgres" {
            error!(
                message = "Invalid scheme for sink connection string",
                scheme = url.scheme()
            );
        }

        let mut pool_conf = Config::new();
        pool_conf.host = url.host().map(|h| h.to_string());
        pool_conf.port = url.port();
        if !url.username().is_empty() {
            pool_conf.user = Some(url_decode(url.username())?);
        }
        if let Some(password) = url.password() {
            let password = url_decode(password)?;
            pool_conf.password = Some(password);
        }
        if let Some(mut path_seg) = url.path_segments() {
            if let Some(first) = path_seg.next() {
                pool_conf.dbname = Some(first.to_owned());
            }
        }

        Ok(pool_conf)
}

fn generate_sql(table: &str, fields: &Vec<PostgreSQLFieldConfig>, conflicts: &Option<PostgreSQLConflictsConfig>) -> String {
    let mut field_list = String::new();
    let mut param_list = String::new();
    let mut field_iter = fields.iter().enumerate();
    if let Some((_, field)) = field_iter.next() {
        field_list.push_str(&*field.name);
        param_list.push_str("$1");

        for (idx, field) in field_iter {
            field_list.push(',');
            field_list.push_str(&*field.name);

            param_list.push_str(",$");
            param_list.push_str(&*(idx + 1).to_string());
        }
    }

    let mut conflict_chunk = String::new();
    if let Some(conflicts) = conflicts {
        conflict_chunk.push_str(" ON CONFLICT (");
        match &conflicts {
            PostgreSQLConflictsConfig::Nothing {target} => {
                conflict_chunk.push_str(&target.join(","));
                conflict_chunk.push_str(") DO NOTHING");
            } ,
            PostgreSQLConflictsConfig::Update {target, fields: update_fields} => {
                conflict_chunk.push_str(&target.join(","));
                conflict_chunk.push_str(") DO UPDATE SET ");

                let mut update_iter = update_fields.iter();
                if let Some(u_field) = update_iter.next() {
                    let f_idx =
                        fields.iter().position(|c| c.name == *u_field).unwrap() + 1;
                    conflict_chunk.push_str(&*u_field);
                    conflict_chunk.push_str("=$");
                    conflict_chunk.push_str(&*f_idx.to_string());

                    for u_field in update_iter {
                        conflict_chunk.push(',');
                        let f_idx =
                            fields.iter().position(|c| c.name == *u_field).unwrap() + 1;
                        conflict_chunk.push_str(&*u_field);
                        conflict_chunk.push_str("=$");
                        conflict_chunk.push_str(&*f_idx.to_string());
                    }
                }
            }
        }
    }

    format!("INSERT INTO {table} ({field_list}) VALUES ({param_list}){conflict_chunk}")
}

fn url_decode(input: &str) -> Result<String, FromUtf8Error> {
    urlencoding::decode(input).map(|c| c.to_string())
}

pub(crate) async fn healthcheck(_config: PostgreSQLSinkConfig) -> crate::Result<()> {
    // Future enhancement: inspect the system tables to validate the config schema and
    // conflict options refer to actual items in the database.
    Ok(())
}

fn build_request(schema: &PostgreSQLSchemaConfig, mut e: Event) -> PostgreSQLRequest {
    let mut data = Vec::new();
    for field in &schema.fields {
        let v = e.as_log().get(&*field.path).unwrap().clone();
        data.push(v);
    }

    PostgreSQLRequest::new(data, e.take_finalizers())
}

#[async_trait]
impl StreamSink<Event> for PostgreSQLSink {
    async fn run(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let schema_config = self.schema_config.clone();
        let sink = input
            .filter_map(|e| {
                let req = build_request(&schema_config, e);
                future::ready(Some(req))
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
        let mut field_conf = Vec::new();
        field_conf.push(PostgreSQLFieldConfig {
            name: "message".to_owned(),
            path: ".message".to_owned()
        });
        let actual = generate_sql("tbl_123", &field_conf, &None);
        assert_eq!(actual, "INSERT INTO tbl_123 (message) VALUES ($1)");
    }

    #[test]
    fn generate_sql_multi_field_test() {
        let mut field_conf = Vec::new();
        field_conf.push( PostgreSQLFieldConfig {
            name: "timestamp".to_owned(),
            path: ".timestamp".to_owned()
        });
        field_conf.push(PostgreSQLFieldConfig {
            name: "message".to_owned(),
            path: ".message".to_owned()
        });
        let actual = generate_sql("tbl_123", &field_conf, &None);
        assert_eq!(actual, "INSERT INTO tbl_123 (timestamp,message) VALUES ($1,$2)");
    }

    #[test]
    fn generate_sql_confict_do_nothing_test() {
        let mut field_conf = Vec::new();
        field_conf.push( PostgreSQLFieldConfig {
            name: "timestamp".to_owned(),
            path: ".timestamp".to_owned()
        });
        field_conf.push(PostgreSQLFieldConfig {
            name: "host".to_owned(),
            path: ".host".to_owned()
        });
        field_conf.push(PostgreSQLFieldConfig {
            name: "message".to_owned(),
            path: ".message".to_owned()
        });
        let confict_conf = Some(PostgreSQLConflictsConfig::Nothing {
            target: vec!["host".to_owned(),"message".to_owned()],
        });
        let actual = generate_sql("tbl_123", &field_conf, &confict_conf);
        assert_eq!(
            actual,
            "INSERT INTO tbl_123 (timestamp,host,message) VALUES ($1,$2,$3) ON CONFLICT \
            (host,message) DO NOTHING"
        );
    }

    #[test]
    fn generate_sql_on_conflict_set_test() {
        let mut field_conf = Vec::new();
        field_conf.push( PostgreSQLFieldConfig {
            name: "timestamp".to_owned(),
            path: ".timestamp".to_owned()
        });
        field_conf.push(PostgreSQLFieldConfig {
            name: "ratio".to_owned(),
            path: ".ratio".to_owned()
        });
        field_conf.push(PostgreSQLFieldConfig {
            name: "message".to_owned(),
            path: ".message".to_owned()
        });
        let confict_conf = Some(PostgreSQLConflictsConfig::Update {
            target: vec!["message".to_owned()],
            fields: vec!["ratio".to_owned(), "timestamp".to_owned()]
        });
        let actual = generate_sql("tbl_123", &field_conf, &confict_conf);
        assert_eq!(
            actual,
            "INSERT INTO tbl_123 (timestamp,ratio,message) VALUES ($1,$2,$3) \
             ON CONFLICT (message) DO UPDATE SET ratio=$2,timestamp=$1"
        );
    }
}