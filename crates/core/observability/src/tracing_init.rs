use tracing_subscriber::{EnvFilter, fmt};

/// Initialize global tracing subscriber for structured logs.
///
/// This should be called once at app startup.
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    tracing::info!("✅ Tracing initialized");
}
