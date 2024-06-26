pub(crate) mod config;
pub(crate) mod encoding;
pub(crate) mod healthcheck;
pub(crate) mod models;
pub(crate) mod service;
pub(crate) mod sink;

pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod traces;

#[cfg(feature = "opentelemetry-sink-integration-tests")]
#[cfg(test)]
pub(crate) mod integration_tests;

use vector_lib::configurable::configurable_component;
use vector_lib::sensitive_string::SensitiveString;

/// Authentication strategies.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "strategy")]
#[configurable(metadata(docs::enum_tag_description = "The authentication strategy to use."))]
pub enum OpentelemetrySinkAuth {
    /// HTTP Basic Authentication.
    Basic {
        /// Basic authentication username.
        user: String,

        /// Basic authentication password.
        password: String,
    },

    /// Bearer authentication.
    ///
    /// A bearer token (OAuth2, JWT, etc) is passed as-is.
    Bearer {
        /// The bearer token to send.
        token: SensitiveString,
    },
}

#[derive(Debug, Clone)]
pub enum Auth {
    Basic(crate::http::Auth),
}
