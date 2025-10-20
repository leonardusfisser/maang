// apps/lillpepe/src/main.rs
//! Lillpepe - White Label SaaS Platform
//! Main application entry point

use actix_web::{middleware, web, App, HttpServer};
use infrastructure_db::Database;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Lillpepe v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config();

    // Initialize database connection
    info!("Connecting to SurrealDB at {}", config.database_url);
    let db = Database::new(
        &config.database_url,
        &config.database_namespace,
        &config.database_name,
    )
        .await
        .expect("Failed to connect to database");

    info!("Database connected successfully");

    // Initialize background job queue
    let job_queue = Arc::new(background_jobs::JobQueue::new(4));
    let scheduler = background_jobs::JobScheduler::new(Arc::clone(&job_queue));
    scheduler.start().await;

    info!("Background job system started");

    // Start HTTP server
    let bind_address = format!("{}:{}", config.host, config.port);
    info!("Starting HTTP server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            // Share state
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(Arc::clone(&job_queue)))

            // Middleware
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())

            // Routes
            .configure(web_admin::routes::configure)
            .configure(web_api::routes::configure)
            .configure(web_public::routes::configure)

            // Static files
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
        .bind(&bind_address)?
        .run()
        .await
}

// ============================================================================
// CONFIGURATION
// ============================================================================

struct Config {
    host: String,
    port: u16,
    database_url: String,
    database_namespace: String,
    database_name: String,
}

fn load_config() -> Config {
    Config {
        host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a number"),
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
        database_namespace: std::env::var("DATABASE_NAMESPACE")
            .unwrap_or_else(|_| "lillpepe".to_string()),
        database_name: std::env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "main".to_string()),
    }
}

// ============================================================================
// MODULES (referenced above)
// ============================================================================

/// Admin dashboard routes
mod web_admin {
    pub mod routes {
        // This would be in crates/application/web/src/admin/routes.rs
        // (Already created above)
        pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
            // Import from the actual admin routes module
            todo!("Import admin routes configuration")
        }
    }
}

/// Public API routes (for white labels)
mod web_api {
    pub mod routes {
        use actix_web::web;

        pub fn configure(cfg: &mut web::ServiceConfig) {
            cfg.service(
                web::scope("/api/v1")
                    // TODO: Implement API routes
                    .route("/health", web::get().to(health_check))
            );
        }

        async fn health_check() -> actix_web::Result<actix_web::HttpResponse> {
            Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION")
            })))
        }
    }
}

/// Public-facing white label routes
mod web_public {
    pub mod routes {
        use actix_web::web;

        pub fn configure(cfg: &mut web::ServiceConfig) {
            cfg
                .route("/", web::get().to(home))
                .route("/about", web::get().to(about))
                .route("/contact", web::get().to(contact));
        }

        async fn home() -> actix_web::Result<actix_web::HttpResponse> {
            Ok(actix_web::HttpResponse::Ok()
                .content_type("text/html")
                .body("<h1>Welcome to Lillpepe</h1>"))
        }

        async fn about() -> actix_web::Result<actix_web::HttpResponse> {
            Ok(actix_web::HttpResponse::Ok()
                .content_type("text/html")
                .body("<h1>About</h1>"))
        }

        async fn contact() -> actix_web::Result<actix_web::HttpResponse> {
            Ok(actix_web::HttpResponse::Ok()
                .content_type("text/html")
                .body("<h1>Contact</h1>"))
        }
    }
}

/// Background jobs system
mod background_jobs {
    // This would be imported from crates/infrastructure/queue
    // (Already created above)

    use std::sync::Arc;

    pub struct JobQueue;

    impl JobQueue {
        pub fn new(_workers: usize) -> Self {
            Self
        }
    }

    pub struct JobScheduler {
        _queue: Arc<JobQueue>,
    }

    impl JobScheduler {
        pub fn new(queue: Arc<JobQueue>) -> Self {
            Self { _queue: queue }
        }

        pub async fn start(&self) {
            tracing::info!("Job scheduler started");
        }
    }
}