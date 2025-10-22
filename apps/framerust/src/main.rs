use frame::{init_tracing, AppConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing("framerust")?;

    // Load configuration
    let config = AppConfig::from_env()?;
    config.validate()?;

    // Create database pool
    let db = infra::create_pool(&config.database).await?;

    // Create Redis pool
    let redis = infra::create_redis_pool(&config.redis).await?;

    // Start server
    web::start_server(config, db, redis).await?;

    Ok(())
}