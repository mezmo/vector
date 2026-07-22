#![allow(missing_docs)]

// `reshape_log_event_by_message` moved to the `mezmo` crate (lib/mezmo) so the
// `codecs` crate can use it after the upstream `src/codecs` -> `lib/codecs` move.
// Re-exported here to keep existing `crate::mezmo::reshape_log_event_by_message`
// call sites working.
pub use ::mezmo::reshape_log_event_by_message;

pub mod config;
pub mod event_trace;
#[cfg(feature = "component-persistence")]
pub mod persistence;
#[cfg(feature = "api-client")]
pub mod remote_task_execution;
pub mod user_trace;

#[macro_export]
macro_rules! mezmo_env_config {
    ($key:expr, $default:expr) => {
        match std::env::var($key) {
            Ok(ref value) if value.is_empty() => {
                debug!(
                    "{} was empty, using default value of {}",
                    $key, $default
                );
                $default
            },
            Ok(value) => match value.parse() {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        error = %err,
                        "Unable to parse {value} for config key {}, using default value of {}",
                        $key, $default
                    );
                    $default
                }
            },
            Err(err) => {
                debug!(
                    error = %err,
                    "{} was not set, using default value of {}",
                    $key, $default
                );
                $default
            }
        }
    };
}
