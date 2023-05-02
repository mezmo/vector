use std::time::SystemTime;

use bytes::Bytes;
use futures::{FutureExt, SinkExt};
use http::{Request, StatusCode, Uri};
use once_cell::sync::Lazy;
use serde_json::json;
use value::Value;
use vector_common::sensitive_string::SensitiveString;
use vector_config::configurable_component;

use crate::{
    codecs::Transformer,
    config::{AcknowledgementsConfig, GenerateConfig, Input, SinkConfig, SinkContext},
    event::Event,
    http::{Auth, HttpClient},
    mezmo::user_trace::MezmoUserLog,
    sinks::util::{
        http::{HttpEventEncoder, HttpSink, PartitionHttpSink},
        BatchConfig, BoxedRawValue, JsonArrayBuffer, PartitionBuffer, PartitionInnerBuffer,
        RealtimeSizeBasedDefaultBatchSettings, TowerRequestConfig, UriSerde,
    },
    template::{Template, TemplateRenderingError},
};

static HOST: Lazy<Uri> = Lazy::new(|| Uri::from_static("https://logs.logdna.com"));

const PATH: &str = "/logs/ingest";
const LINE_KEY: &str = "line";
const APP_KEY: &str = "app";
const FILE_KEY: &str = "file";
const ENV_KEY: &str = "env";
const DEFAULT_VALUE: Value = Value::Null;

/// Configuration for the `logdna` sink.
#[configurable_component(sink("logdna"))]
#[derive(Clone, Debug)]
pub struct LogdnaConfig {
    /// The Ingestion API key.
    api_key: SensitiveString,

    /// The endpoint to send logs to.
    #[serde(alias = "host")]
    endpoint: Option<UriSerde>,

    /// Optional line field selector, only one of `line_field` and `line_template` can be specified
    line_field: Option<String>,

    /// Optional line template, only one of `line_field` and `line_template` can be specified
    line_template: Option<Template>,

    /// Optional field selector for the log line's timestamp
    timestamp_field: Option<String>,

    /// Optional app template
    app_template: Option<Template>,

    /// Optional template for the file that supplied the log line
    file_template: Option<Template>,

    /// Optional template for the environment the log line came from
    env_template: Option<Template>,

    /// The hostname that will be attached to each batch of events.
    hostname: Template,

    /// The MAC address that will be attached to each batch of events.
    mac_template: Option<Template>,

    /// The IP address that will be attached to each batch of events.
    ip_template: Option<Template>,

    /// The tags that will be attached to each batch of events.
    tags: Option<Vec<Template>>,

    #[configurable(derived)]
    #[serde(
        default,
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    pub encoding: Transformer,

    /// The default app that will be set for events that do not contain a `file` or `app` field.
    default_app: Option<String>,

    /// The default environment that will be set for events that do not contain an `env` field.
    default_env: Option<String>,

    #[configurable(derived)]
    #[serde(default)]
    batch: BatchConfig<RealtimeSizeBasedDefaultBatchSettings>,

    #[configurable(derived)]
    #[serde(default)]
    request: TowerRequestConfig,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    acknowledgements: AcknowledgementsConfig,
}

impl GenerateConfig for LogdnaConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(
            r#"hostname = "hostname"
            api_key = "${LOGDNA_API_KEY}""#,
        )
        .unwrap()
    }
}

#[async_trait::async_trait]
impl SinkConfig for LogdnaConfig {
    async fn build(
        &self,
        cx: SinkContext,
    ) -> crate::Result<(super::VectorSink, super::Healthcheck)> {
        if self.line_field.is_some() && self.line_template.is_some() {
            return Err("only one of `line_field` and `line_template` can be provided".into());
        }

        let request_settings = self.request.unwrap_with(&TowerRequestConfig::default());
        let batch_settings = self.batch.into_batch_settings()?;
        let client = HttpClient::new(None, cx.proxy())?;

        let logdna_sink = LogdnaSink {
            cx: cx.clone(),
            cfg: self.clone(),
        };
        let sink = PartitionHttpSink::new(
            logdna_sink.clone(),
            PartitionBuffer::new(JsonArrayBuffer::new(batch_settings.size)),
            request_settings,
            batch_settings.timeout,
            client.clone(),
            cx,
        )
        .sink_map_err(|error| error!(message = "Fatal logdna sink error.", %error));

        let uri = logdna_sink.build_uri("");
        let healthcheck = healthcheck(uri, client).boxed();

        Ok((super::VectorSink::from_event_sink(sink), healthcheck))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct PartitionKey {
    hostname: String,
    tags: Option<Vec<String>>,
    ip: Option<String>,
    mac: Option<String>,
}

pub struct LogdnaEventEncoder {
    cx: SinkContext,
    line_field: Option<String>,
    line_template: Option<Template>,
    timestamp_field: Option<String>,
    app_template: Option<Template>,
    file_template: Option<Template>,
    env_template: Option<Template>,
    hostname: Template,
    tags: Option<Vec<Template>>,
    ip_template: Option<Template>,
    mac_template: Option<Template>,
    transformer: Transformer,
    default_app: Option<String>,
    default_env: Option<String>,
}

impl LogdnaEventEncoder {
    fn render_key(&self, event: &Event) -> Option<PartitionKey> {
        let tags = self
            .tags
            .as_ref()
            .map(|tags| {
                let mut vec = Vec::with_capacity(tags.len());
                for tag in tags {
                    let t = tag.render_string(event);
                    match t {
                        Ok(t) => vec.push(t),
                        Err(error) => {
                            self.log_template_error("tag", error, false);
                        }
                    }
                }
                if !vec.is_empty() {
                    Some(vec)
                } else {
                    None
                }
            })
            .unwrap_or(None);
        let ip = self
            .ip_template
            .as_ref()
            .map(|i| {
                let m = i.render_string(event);
                match m {
                    Ok(m) => Some(m),
                    Err(error) => {
                        self.log_template_error("ip", error, false);
                        None
                    }
                }
            })
            .unwrap_or(None);
        let mac = self
            .mac_template
            .as_ref()
            .map(|m| {
                let s = m.render_string(event);
                match s {
                    Ok(s) => Some(s),
                    Err(error) => {
                        self.log_template_error("mac", error, false);
                        None
                    }
                }
            })
            .unwrap_or(None);

        let hostname_result = self.hostname.render_string(event);
        match hostname_result {
            Ok(hostname) => Some(PartitionKey {
                hostname,
                tags,
                ip,
                mac,
            }),
            Err(error) => {
                self.log_template_error("hostname", error, false);
                None // hostname is required by ingest API, so fail key generation without it
            }
        }
    }
    fn log_template_error(&self, field: &str, error: TemplateRenderingError, drop_event: bool) {
        emit!(crate::internal_events::TemplateRenderingError {
            error: error.clone(),
            field: Some(field),
            drop_event,
        });
        self.cx
            .mezmo_ctx
            .error(Value::from(format!("{field} template error - {error}")));
    }
}

impl HttpEventEncoder<PartitionInnerBuffer<serde_json::Value, PartitionKey>>
    for LogdnaEventEncoder
{
    fn encode_event(
        &mut self,
        mut event: Event,
    ) -> Option<PartitionInnerBuffer<serde_json::Value, PartitionKey>> {
        let key = self.render_key(&event)?;

        let message_key: &str = crate::config::log_schema().message_key();

        self.transformer.transform(&mut event);
        let mut log = event.into_log();
        let mut paths_to_remove = Vec::new();

        let mut map = serde_json::map::Map::new();

        // line
        if let Some(line_template) = &self.line_template {
            match line_template.render_string(&log) {
                Ok(line) => {
                    // Remove the template parts later so we don't put them in the meta
                    let parts = line_template.get_fields().unwrap_or_default();
                    for path in &parts {
                        paths_to_remove.push(path.to_owned());
                    }

                    map.insert(LINE_KEY.to_string(), json!(line));
                }
                Err(error) => {
                    self.log_template_error("line", error, true);
                }
            };
        } else if let Some(path) = &self.line_field {
            paths_to_remove.push(path.to_string());
            let line = log.get(path.as_str()).unwrap_or(&DEFAULT_VALUE);

            match line.is_object() {
                false => map.insert(LINE_KEY.to_string(), json!(line)),
                true => {
                    let encoded = serde_json::to_string(&line)
                        .ok()
                        .unwrap_or_else(|| "".into());
                    map.insert(LINE_KEY.to_string(), json!(encoded))
                }
            };
        }

        // timestamp
        if let Some(path) = &self.timestamp_field {
            paths_to_remove.push(path.to_string());
            if let Some(ts) = log.get(path.as_str()) {
                map.insert("timestamp".to_string(), json!(ts));
            }
        } else if let Some(ts) = log.get(crate::config::log_schema().timestamp_key()) {
            map.insert("timestamp".to_string(), json!(ts));
        }

        // app
        if let Some(app_template) = &self.app_template {
            match app_template.render_string(&log) {
                Ok(app) => {
                    // Remove the template parts so we don't put them in the meta
                    let parts = app_template.get_fields().unwrap_or_default();
                    for path in &parts {
                        paths_to_remove.push(path.to_owned());
                    }

                    map.insert(APP_KEY.to_string(), json!(app));
                }
                Err(error) => {
                    self.log_template_error("app", error, false);
                }
            };
        }

        // file
        if let Some(file_template) = &self.file_template {
            match file_template.render_string(&log) {
                Ok(file) => {
                    // Remove the template parts so we don't put them in the meta
                    let parts = file_template.get_fields().unwrap_or_default();
                    for path in &parts {
                        paths_to_remove.push(path.to_owned());
                    }

                    map.insert(FILE_KEY.to_string(), json!(file));
                }
                Err(error) => {
                    self.log_template_error("file", error, false);
                }
            };
        }

        // app fallback
        if !map.contains_key(APP_KEY) && !map.contains_key(FILE_KEY) {
            let default_app = match &self.default_app {
                Some(l) => l,
                None => "vector",
            };

            map.insert(APP_KEY.to_string(), json!(default_app));
        }

        // env
        if let Some(env_template) = &self.env_template {
            match env_template.render_string(&log) {
                Ok(env) => {
                    // Remove the template parts so we don't put them in the meta
                    let parts = env_template.get_fields().unwrap_or_default();
                    for path in &parts {
                        paths_to_remove.push(path.to_owned());
                    }

                    map.insert(ENV_KEY.to_string(), json!(env));
                }
                Err(error) => {
                    self.log_template_error("env", error, false);
                }
            };
        }

        if !map.contains_key(ENV_KEY) {
            map.insert(
                ENV_KEY.to_string(),
                json!(self.default_env.as_deref().unwrap_or("production")),
            );
        }

        //
        // Handle catch-all cases
        //

        // Remove used properties
        for path in paths_to_remove {
            log.remove(path.as_str());
        }

        // Handle the default whole message as line or remaining message as meta cases
        //  after removing other used properties
        if let Some(message) = log.remove(message_key) {
            let key = if !map.contains_key(LINE_KEY) {
                LINE_KEY
            } else {
                "meta"
            };

            if message.is_object() {
                let encoded = serde_json::to_string(&message)
                    .ok()
                    .unwrap_or_else(|| "".into());
                map.insert(key.to_string(), json!(encoded));
            } else {
                map.insert(key.to_string(), json!(message));
            }
        }

        Some(PartitionInnerBuffer::new(map.into(), key))
    }
}

#[derive(Clone, Debug)]
struct LogdnaSink {
    cx: SinkContext,
    cfg: LogdnaConfig,
}

#[async_trait::async_trait]
impl HttpSink for LogdnaSink {
    type Input = PartitionInnerBuffer<serde_json::Value, PartitionKey>;
    type Output = PartitionInnerBuffer<Vec<BoxedRawValue>, PartitionKey>;
    type Encoder = LogdnaEventEncoder;

    fn build_encoder(&self) -> Self::Encoder {
        LogdnaEventEncoder {
            cx: self.cx.clone(),
            line_field: self.cfg.line_field.clone(),
            line_template: self.cfg.line_template.clone(),
            timestamp_field: self.cfg.timestamp_field.clone(),
            app_template: self.cfg.app_template.clone(),
            file_template: self.cfg.file_template.clone(),
            env_template: self.cfg.env_template.clone(),
            hostname: self.cfg.hostname.clone(),
            tags: self.cfg.tags.clone(),
            ip_template: self.cfg.ip_template.clone(),
            mac_template: self.cfg.mac_template.clone(),
            transformer: self.cfg.encoding.clone(),
            default_app: self.cfg.default_app.clone(),
            default_env: self.cfg.default_env.clone(),
        }
    }

    async fn build_request(&self, output: Self::Output) -> crate::Result<http::Request<Bytes>> {
        let (events, key) = output.into_parts();
        let mut query = url::form_urlencoded::Serializer::new(String::new());

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time can't drift behind the epoch!")
            .as_millis();

        query.append_pair("hostname", &key.hostname);
        query.append_pair("now", &now.to_string());

        if let Some(mac) = &key.mac {
            query.append_pair("mac", mac);
        }

        if let Some(ip) = &key.ip {
            query.append_pair("ip", ip);
        }

        if let Some(tags) = &key.tags {
            let tags = tags.join(",");
            query.append_pair("tags", &tags);
        }

        let query = query.finish();

        let body = crate::serde::json::to_bytes(&json!({
            "lines": events,
        }))
        .unwrap()
        .freeze();

        let uri = self.build_uri(&query);

        let mut request = Request::builder()
            .uri(uri)
            .method("POST")
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap();

        let auth = Auth::Basic {
            user: self.cfg.api_key.inner().to_string(),
            password: SensitiveString::default(),
        };

        auth.apply(&mut request);

        Ok(request)
    }
}

impl LogdnaSink {
    fn build_uri(&self, query: &str) -> Uri {
        let host = self
            .cfg
            .endpoint
            .as_ref()
            .map(|endpoint| &endpoint.uri)
            .unwrap_or_else(|| &*HOST);

        let uri = format!("{}{}?{}", host, PATH, query);

        uri.parse::<http::Uri>()
            .expect("This should be a valid uri")
    }
}

async fn healthcheck(uri: Uri, client: HttpClient) -> crate::Result<()> {
    let req = Request::post(uri).body(hyper::Body::empty()).unwrap();

    let res = client.send(req).await?;

    if res.status().is_server_error() {
        return Err("Server returned a server error".into());
    }

    if res.status() == StatusCode::FORBIDDEN {
        return Err("Token is not valid, 403 returned.".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use futures::{channel::mpsc, StreamExt};
    use futures_util::stream;
    use http::{request::Parts, StatusCode};
    use serde_json::json;
    use temp_env::with_var;
    use vector_core::event::{BatchNotifier, BatchStatus, Event, LogEvent};

    use super::*;
    use crate::{
        config::SinkConfig,
        sinks::util::test::{build_test_server_status, load_sink},
        test_util::{
            components::{assert_sink_compliance, HTTP_SINK_TAGS},
            next_addr, random_lines,
        },
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<LogdnaConfig>();
    }

    #[tokio::test]
    async fn build_config() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            default_env = "acceptance"
            codec.except_fields = ["magic"]
            line_field = ".message.line"
            line_template = "{{.message.line}} - {{.message.other.thing}}"
        "#,
        )
        .unwrap();

        let built = config.build(cx).await;
        assert!(built.is_err());
    }

    #[test]
    fn encode_event_defaults() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            app_template = "{{ .message.app }}"
            file_template = "{{ .message.file }}"
            env_template = "{{ .message.env }}"
            timestamp_field = ".message._ts"
            default_env = "default"
            default_app = "default"
            codec.except_fields = ["magic"]
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();

        let event1 = Event::Log(LogEvent::from("hello world"));
        let event1_out = encoder.encode_event(event1).unwrap().into_parts().0;
        let event1_out = event1_out.as_object().unwrap();
        assert_eq!(event1_out.get("app"), Some(&json!("default")));
        assert!(event1_out.get("file").is_none());
        assert_eq!(event1_out.get("env"), Some(&json!("default")));
        assert_eq!(event1_out.get("line"), Some(&json!("hello world")));
        assert!(event1_out.get("meta").is_none());

        let message_object = json!({
        "message": "hello world",
        "app": "notvector",
        "file": "log.txt",
        "env": "staging",
        "first": "prop",
        "_ts": "1682022085309",
        });
        let mut event2 = Event::Log(LogEvent::from("hello world"));
        event2.as_mut_log().insert(".message", message_object);
        let event2_out = encoder.encode_event(event2).unwrap().into_parts().0;
        let event2_out = event2_out.as_object().unwrap();
        assert_eq!(event2_out.get("app"), Some(&json!("notvector")));
        assert_eq!(event2_out.get("file"), Some(&json!("log.txt")));
        assert_eq!(event2_out.get("env"), Some(&json!("staging")));
        assert_eq!(event2_out.get("timestamp"), Some(&json!("1682022085309")));
        assert_eq!(
            event2_out.get("line"),
            Some(&json!("{\"first\":\"prop\",\"message\":\"hello world\"}"))
        );
        assert!(event1_out.get("meta").is_none());
    }

    #[test]
    fn encode_event_line_field() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            default_env = "acceptance"
            codec.except_fields = ["magic"]
            line_field = ".message.line"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();

        let payload = json!({
        "code": 200,
        "success": true,
        "payload": {
            "features": [
                "serde",
                "json"
            ]
        }});
        let mut event = Event::Log(LogEvent::try_from(payload).unwrap());

        let message = json!({
            "line": "hello world",
            "other": "stuff"
        });
        event.as_mut_log().insert("message", message);

        let event_out = encoder.encode_event(event).unwrap().into_parts().0;
        let event_out = event_out.as_object().unwrap();

        assert_eq!(event_out.get("line"), Some(&json!("hello world")));
        assert_eq!(event_out.get("meta"), Some(&json!("{\"other\":\"stuff\"}")));
    }

    #[test]
    fn encode_event_line_template() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            default_env = "acceptance"
            codec.except_fields = ["magic"]
            line_template = "{{.message.line}} - {{.message.other.thing}}"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();
        let mut event = Event::Log(LogEvent::from("goodbye world"));

        let message = json!({
            "line": "hello world",
            "other": {
                "thing": "stuff",
                "nested": "remaining"
            },
            "third": "thing"
        });
        event.as_mut_log().insert("message", message);

        let event_out = encoder.encode_event(event).unwrap().into_parts().0;
        let event_out = event_out.as_object().unwrap();

        assert_eq!(event_out.get("line"), Some(&json!("hello world - stuff")));
        assert_eq!(
            event_out.get("meta"),
            Some(&json!(
                "{\"other\":{\"nested\":\"remaining\"},\"third\":\"thing\"}"
            ))
        );
    }

    #[test]
    fn encode_event_object_line() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            default_env = "acceptance"
            codec.except_fields = ["magic"]
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();
        let mut event = Event::Log(LogEvent::from("goodbye world"));

        let message = json!({
            "line": "hello world",
            "other": "stuff",
            "third": "thing"
        });
        event.as_mut_log().insert("message", message);

        let event_out = encoder.encode_event(event).unwrap().into_parts().0;
        let event_out = event_out.as_object().unwrap();

        assert_eq!(
            event_out.get("line"),
            Some(&json!(
                "{\"line\":\"hello world\",\"other\":\"stuff\",\"third\":\"thing\"}"
            ))
        );
        assert!(event_out.get("meta").is_none());
    }

    #[test]
    fn encode_event_app_template() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            default_env = "acceptance"
            codec.except_fields = ["magic"]
            default_app = "app-name"
            app_template = "{{.message.third}} - {{.message.other.thing}}"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();
        let mut event = Event::Log(LogEvent::from("goodbye world"));

        let message = json!({
            "line": "hello world",
            "other": {
                "thing": "stuff",
                "nested": "remaining"
            },
            "third": "thing"
        });
        event.as_mut_log().insert("message", message);

        let event_out = encoder.encode_event(event).unwrap().into_parts().0;
        let event_out = event_out.as_object().unwrap();

        assert_eq!(event_out.get("app"), Some(&json!("thing - stuff")));
        assert_eq!(
            event_out.get("line"),
            Some(&json!(
                "{\"line\":\"hello world\",\"other\":{\"nested\":\"remaining\"}}"
            ))
        );
        assert!(event_out.get("meta").is_none()); // whole .message went to the line
    }

    #[test]
    fn encode_event_nothing_to_reshape() {
        // Since Log Analysis root-level properties don't contain `message`, there should
        // be nothing to reshape even if the env var is set.

        with_var("MEZMO_RESHAPE_MESSAGE", Some("1"), || {
            let (config, cx) = load_sink::<LogdnaConfig>(
                r#"
                api_key = "mylogtoken"
                hostname = "vector"
                app_template = "{{ .message.app }}"
                file_template = "{{ .message.file }}"
                env_template = "{{ .message.env }}"
                default_env = "acceptance"
                codec.except_fields = ["magic"]
            "#,
            )
            .unwrap();
            let sink = LogdnaSink { cx, cfg: config };
            let mut encoder = sink.build_encoder();

            let mut event1 = Event::Log(LogEvent::from("hello world"));
            event1.as_mut_log().insert(".message.app", "notvector");
            event1.as_mut_log().insert("magic", "vector");

            let mut event2 = Event::Log(LogEvent::from("hello world"));
            event2.as_mut_log().insert(".message.file", "log.txt");

            let event3 = Event::Log(LogEvent::from("hello world"));

            let mut event4 = Event::Log(LogEvent::from("hello world"));
            event4.as_mut_log().insert(".message.env", "staging");

            let event1_out = encoder.encode_event(event1).unwrap().into_parts().0;
            let event1_out = event1_out.as_object().unwrap();
            let event2_out = encoder.encode_event(event2).unwrap().into_parts().0;
            let event2_out = event2_out.as_object().unwrap();
            let event3_out = encoder.encode_event(event3).unwrap().into_parts().0;
            let event3_out = event3_out.as_object().unwrap();
            let event4_out = encoder.encode_event(event4).unwrap().into_parts().0;
            let event4_out = event4_out.as_object().unwrap();

            assert_eq!(event1_out.get("app"), Some(&json!("notvector")));
            assert_eq!(event2_out.get("file"), Some(&json!("log.txt")));
            assert_eq!(event3_out.get("app"), Some(&json!("vector")));
            assert_eq!(event3_out.get("env"), Some(&json!("acceptance")));
            assert_eq!(event4_out.get("env"), Some(&json!("staging")));
        });
    }

    #[test]
    fn render_key_la_values() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            mac_template = "{{ .metadata.query.mac }}"
            ip_template = "{{ .metadata.query.ip }}"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let encoder = sink.build_encoder();

        let message_object = json!({
        "message": "hello world",
        "_file": "log.txt",
        "env": "staging",
        "_ts": "1682022085309",
        "_meta": {
            "first": "prop"
        }
        });
        let metadata_object = json!({
            "query": {
                "app": "la_app",
                "ip": "127.0.0.1",
                "mac": "some-mac-addr"
            }
        });
        let mut event = Event::Log(LogEvent::from("hello world"));
        event.as_mut_log().insert(".message", message_object);
        event.as_mut_log().insert(".metadata", metadata_object);

        let key = encoder.render_key(&event).unwrap();

        assert_eq!(key.hostname, "vector".to_string());
        assert_eq!(key.ip, Some("127.0.0.1".to_string()));
        assert_eq!(key.mac, Some("some-mac-addr".to_string()));
        assert!(key.tags.is_none());
    }

    #[test]
    fn encode_event_render_key_host_error() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "{{ .metadata.query.host }}"
            mac_template = "{{ .metadata.query.mac }}"
            ip_template = "{{ .metadata.query.ip }}"
            line_field = ".message.message"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();

        let message_object = json!({
        "message": "hello world",
        "_file": "log.txt",
        "env": "staging",
        "_ts": "1682022085309",
        "_meta": {
            "first": "prop"
        }
        });
        let metadata_object = json!({
            "query": {
                "app": "la_app"
            }
        });
        let mut event = Event::Log(LogEvent::from("hello world"));
        event.as_mut_log().insert(".message", message_object);
        event.as_mut_log().insert(".metadata", metadata_object);

        let event_out = encoder.encode_event(event);
        assert!(event_out.is_none());
    }

    #[test]
    fn encode_event_render_key_optional_error() {
        let (config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            hostname = "vector"
            tags = ["{{ .metadata.query.tags }}"]
            ip_template = "{{ .metadata.query.ip }}"
            mac_template = "{{ .metadata.query.mac }}"
            line_field = ".message.message"
        "#,
        )
        .unwrap();
        let sink = LogdnaSink { cx, cfg: config };
        let mut encoder = sink.build_encoder();

        let message_object = json!({
        "message": "hello world",
        "_file": "log.txt",
        "env": "staging",
        "_ts": "1682022085309",
        "_meta": {
            "first": "prop"
        }
        });
        let metadata_object = json!({
            "query": {
                "app": "la_app"
            }
        });
        let mut event = Event::Log(LogEvent::from("hello world"));
        event.as_mut_log().insert(".message", message_object);
        event.as_mut_log().insert(".metadata", metadata_object);

        let event_out = encoder.encode_event(event).unwrap().into_parts().0;
        let event_out = event_out.as_object().unwrap();
        // Template errors on optional params don't stop encoding
        assert_eq!(event_out.get("line"), Some(&json!("hello world")));
    }

    async fn smoke_start(
        status_code: StatusCode,
        batch_status: BatchStatus,
    ) -> (
        Vec<&'static str>,
        Vec<Vec<String>>,
        mpsc::Receiver<(Parts, bytes::Bytes)>,
    ) {
        let (mut config, cx) = load_sink::<LogdnaConfig>(
            r#"
            api_key = "mylogtoken"
            ip_template = "127.0.0.1"
            mac_template = "some-mac-addr"
            hostname = "{{ hostname }}"
            tags = ["test","maybeanothertest"]
        "#,
        )
        .unwrap();

        // Make sure we can build the config
        let _ = config.build(cx.clone()).await.unwrap();

        let addr = next_addr();
        // Swap out the host so we can force send it
        // to our local server
        let endpoint = format!("http://{}", addr).parse::<http::Uri>().unwrap();
        config.endpoint = Some(endpoint.into());

        let (sink, _) = config.build(cx).await.unwrap();

        let (rx, _trigger, server) = build_test_server_status(addr, status_code);
        tokio::spawn(server);

        let lines = random_lines(100).take(10).collect::<Vec<_>>();
        let mut events = Vec::new();
        let hosts = vec!["host0", "host1"];

        let (batch, mut receiver) = BatchNotifier::new_with_receiver();
        let mut partitions = vec![Vec::new(), Vec::new()];
        // Create 10 events where the first one contains custom
        // fields that are not just `message`.
        for (i, line) in lines.iter().enumerate() {
            let mut event = LogEvent::from(line.as_str()).with_batch_notifier(&batch);
            let p = i % 2;
            event.insert("hostname", hosts[p]);

            partitions[p].push(line.into());
            events.push(Event::Log(event));
        }
        drop(batch);

        let events = stream::iter(events).map(Into::into);
        sink.run(events).await.expect("Running sink failed");

        assert_eq!(receiver.try_recv(), Ok(batch_status));

        (hosts, partitions, rx)
    }

    #[tokio::test]
    async fn smoke_fails() {
        let (_hosts, _partitions, mut rx) =
            smoke_start(StatusCode::FORBIDDEN, BatchStatus::Rejected).await;
        assert!(matches!(rx.try_next(), Err(mpsc::TryRecvError { .. })));
    }

    #[tokio::test]
    async fn smoke() {
        assert_sink_compliance(&HTTP_SINK_TAGS, async {
            let (hosts, partitions, mut rx) =
                smoke_start(StatusCode::OK, BatchStatus::Delivered).await;

            for _ in 0..partitions.len() {
                let output = rx.next().await.unwrap();

                let request = &output.0;
                let body: serde_json::Value = serde_json::from_slice(&output.1[..]).unwrap();

                let query = request.uri.query().unwrap();

                let (p, _) = hosts
                    .iter()
                    .enumerate()
                    .find(|(_, host)| query.contains(&format!("hostname={}", host)))
                    .expect("invalid hostname");
                let lines = &partitions[p];

                assert!(query.contains("ip=127.0.0.1"));
                assert!(query.contains("mac=some-mac-addr"));
                assert!(query.contains("tags=test%2Cmaybeanothertest"));

                let output = body
                    .as_object()
                    .unwrap()
                    .get("lines")
                    .unwrap()
                    .as_array()
                    .unwrap();

                for (i, line) in output.iter().enumerate() {
                    // All lines are json objects
                    let line = line.as_object().unwrap();

                    assert_eq!(line.get("app"), Some(&json!("vector")));
                    assert_eq!(line.get("env"), Some(&json!("production")));
                    assert_eq!(line.get("line"), Some(&json!(lines[i])));

                    assert!(line.get("meta").is_none());
                }
            }
        })
        .await;
    }
}
