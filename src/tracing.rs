use tracing_subscriber::EnvFilter;

use crate::config::Config;

pub use tracing::{error, info};

pub fn init_subscriber(config: &Config) {
    let common_subscriber = tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(EnvFilter::from_default_env());

    if config.pretty_log {
        common_subscriber.pretty().init();
    } else {
        common_subscriber.init();
    }
}

