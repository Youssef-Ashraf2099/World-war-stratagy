//! Tracing and logging setup

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with default configuration
pub fn init_tracing() {
    init_tracing_with_filter("alalamien_engine=debug,alalamien_api=debug")
}

/// Initialize tracing with custom filter
pub fn init_tracing_with_filter(filter: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
}

/// Initialize tracing for tests (with compact output)
#[cfg(test)]
pub fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}
