#[actix_web::main]
async fn main() -> std::io::Result<()> {
    core_observability::tracing_init::init();

    // dev: tries ENV_FILE/apps/lillpepe/.env/root .env; prod: real env only (inside core_config)
    let _ = core_config::load_env("lillpepe");

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let addr = format!("{host}:{port}");

    application_web::server::run(&addr).await
}
