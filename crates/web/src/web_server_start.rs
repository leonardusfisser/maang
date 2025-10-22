use actix_web::{web, App, HttpServer};
use actix_files as fs;
use frame::{AppConfig, AppError};
use deadpool_postgres::Pool as PgPool;
use redis::aio::ConnectionManager as RedisManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisManager,
    pub config: AppConfig,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db", &"PgPool")
            .field("redis", &"RedisManager")
            .field("config", &self.config)
            .finish()
    }
}

pub async fn start_server(
    config: AppConfig,
    db: PgPool,
    redis: RedisManager,
) -> Result<(), AppError> {
    let bind_addr = config.server.bind_address;
    let state = AppState {
        db,
        redis,
        config: config.clone(),
    };

    tracing::info!("Starting server on {}", bind_addr);
    tracing::info!("Configuring routes...");

    HttpServer::new(move || {
        tracing::info!("Creating new app instance");
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(fs::Files::new("/static", "crates/web/static").show_files_listing())
            .configure(crate::web_routes_shared::configure)
            .configure(crate::web_routes_tenants::configure)
            .configure(crate::web_routes_webmaster::configure)
    })
        .bind(bind_addr)
        .map_err(|e| AppError::init(format!("Failed to bind server: {e}")))?
        .run()
        .await
        .map_err(|e| AppError::init(format!("Server error: {e}")))?;

    Ok(())
}