pub mod state {
    use crate::mezmo_env_config;
    use std::time::Duration;

    const ENV_MEZMO_STATE_CONNECTION_STRING: &str = "MEZMO_STATE_CONNECTION_STRING";
    const ENV_MEZMO_STATE_CONNECTION_RETRY_FACTOR_MS: &str =
        "MEZMO_STATE_CONNECTION_RETRY_FACTOR_MS";
    const ENV_MEZMO_STATE_CONNECTION_RETRY_COUNT: &str = "MEZMO_STATE_CONNECTION_RETRY_COUNT";
    const ENV_MEZMO_STATE_CONNECTION_RETRY_MAX_DELAY_MS: &str =
        "MEZMO_STATE_CONNECTION_RETRY_MAX_DELAY_MS";
    const ENV_MEZMO_STATE_CONNECTION_TIMEOUT_MS: &str = "MEZMO_STATE_CONNECTION_TIMEOUT_MS";
    const ENV_MEZMO_STATE_CONNECTION_RESPONSE_TIMEOUT_MS: &str =
        "MEZMO_STATE_CONNECTION_RESPONSE_TIMEOUT_MS";

    const DEFAULT_CONNECTION_STRING: &str = "redis://127.0.0.1:6379/0";
    const DEFAULT_CONNECTION_RETRY_FACTOR_MS: u64 = 150;
    const DEFAULT_CONNECTION_RETRY_COUNT: usize = 10;
    const DEFAULT_CONNECTION_RETRY_MAX_DELAY_MS: u64 = 7_500;
    const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 2_500;
    const DEFAULT_CONNECTION_RESPONSE_TIMEOUT_MS: u64 = 2_500;

    pub fn default_connection_string() -> String {
        match std::env::var_os(ENV_MEZMO_STATE_CONNECTION_STRING) {
            Some(s) => s.into_string(),
            None => Ok(DEFAULT_CONNECTION_STRING.to_string()),
        }
        .unwrap_or_else(|_| DEFAULT_CONNECTION_STRING.to_string())
    }

    pub fn default_connection_retry_factor_ms() -> u64 {
        mezmo_env_config!(
            ENV_MEZMO_STATE_CONNECTION_RETRY_FACTOR_MS,
            DEFAULT_CONNECTION_RETRY_FACTOR_MS
        )
    }

    pub fn default_connection_retry_count() -> usize {
        mezmo_env_config!(
            ENV_MEZMO_STATE_CONNECTION_RETRY_COUNT,
            DEFAULT_CONNECTION_RETRY_COUNT
        )
    }

    pub fn default_connection_retry_max_delay_ms() -> u64 {
        mezmo_env_config!(
            ENV_MEZMO_STATE_CONNECTION_RETRY_MAX_DELAY_MS,
            DEFAULT_CONNECTION_RETRY_MAX_DELAY_MS
        )
    }

    pub fn default_connection_timeout_ms() -> Duration {
        let ms = mezmo_env_config!(
            ENV_MEZMO_STATE_CONNECTION_TIMEOUT_MS,
            DEFAULT_CONNECTION_TIMEOUT_MS
        );

        Duration::from_millis(ms)
    }

    pub fn default_connection_response_timeout_ms() -> Duration {
        let ms = mezmo_env_config!(
            ENV_MEZMO_STATE_CONNECTION_RESPONSE_TIMEOUT_MS,
            DEFAULT_CONNECTION_RESPONSE_TIMEOUT_MS
        );

        Duration::from_millis(ms)
    }
}
