use frame::{AppError, DatabaseConfig, InfraError};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

/// Create Postgres connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<Pool, AppError> {
    let mut cfg = Config::new();
    cfg.url = Some(config.url.clone());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|e| AppError::init(format!("Failed to create DB pool: {e}")))?;

    // Verify connection
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::init(format!("Failed to get DB connection: {e}")))?;

    let _ = client
        .execute("SELECT 1", &[])
        .await
        .map_err(|e| AppError::init(format!("DB health check failed: {e}")))?;

    tracing::info!("Postgres pool created successfully");
    Ok(pool)
}

/// Execute a query and return results
pub async fn query<T>(
    pool: &Pool,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<Vec<tokio_postgres::Row>, InfraError> {
    let client = pool.get().await.map_err(InfraError::from)?;

    client
        .query(query, params)
        .await
        .map_err(InfraError::from)
}

/// Execute a statement (INSERT, UPDATE, DELETE)
pub async fn execute(
    pool: &Pool,
    statement: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<u64, InfraError> {
    let client = pool.get().await.map_err(InfraError::from)?;

    client
        .execute(statement, params)
        .await
        .map_err(InfraError::from)
}