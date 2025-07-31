//! Logging and tracing configuration.
//!
//! This module sets up structured logging for the application,
//! with support for both pretty-printed and JSON formats.

use tracing_subscriber::EnvFilter;

use crate::config::Config;

pub use tracing::{error, info};

/// Initializes the tracing subscriber based on configuration.
///
/// Sets up logging with environment-based filtering and configurable
/// output format (pretty-printed for development, structured for production).
pub fn initialize_tracing(config: &Config) {
    let common_subscriber = tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(EnvFilter::from_default_env());

    if config.pretty_log {
        common_subscriber.pretty().init();
    } else {
        common_subscriber.init();
    }
}
