use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use serde_json::json;
use warp::{reply::json, Rejection, Reply};

// Health handler, responds with '{ ok: true }' when running and '{ ok: false}'
// when shutting down
pub(super) async fn health(running: Arc<AtomicBool>) -> Result<impl Reply, Rejection> {
    if running.load(atomic::Ordering::Relaxed) {
        Ok(warp::reply::with_status(
            json(&json!({"ok": true})),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            json(&json!({"ok": false})),
            warp::http::StatusCode::SERVICE_UNAVAILABLE,
        ))
    }
}

// Config loaded handler, for use with config providers only, returns '{loaded:
// true}' when a config provider has sucessfully loaded a configuration for the
// first time; otherwise, '{loaded: false}'.
pub(super) async fn config(
    running: Arc<AtomicBool>,
    config_loaded: Arc<AtomicBool>,
) -> Result<impl Reply, Rejection> {
    if running.load(atomic::Ordering::Relaxed) && config_loaded.load(atomic::Ordering::Relaxed) {
        Ok(warp::reply::with_status(
            json(&json!({"loaded": true})),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            json(&json!({"loaded": false})),
            warp::http::StatusCode::SERVICE_UNAVAILABLE,
        ))
    }
}
