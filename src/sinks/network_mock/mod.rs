//! A mock network sink for realistic throughput testing.
//!
//! This sink exercises the full Tower service pipeline (batching, encoding,
//! request building, concurrency control, retries) while replacing the actual
//! network call with a configurable sleep, optionally simulating request errors.

mod config;
mod encoder;
mod request_builder;
mod service;
mod sink;
