// Admin Routes Implementation
// crates/application/web/src/admin/routes.rs

use actix_web::{web, HttpResponse, Result};
use askama::Template;
use serde::Deserialize;

// ============================================================================
// TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "admin/base.html")]
struct BaseTemplate;

#[derive(Template)]
#[template(path = "admin/partials/dashboard.html")]
struct DashboardPartial {
    white_count: u32,
    gray_count: u32,
    black_count: u32,
    red_count: u32,
    white_change: i32,
    mrr: String,
    mrr_growth: i32,
    failed_payments: u32,
    storage_gb: u32,
    storage_percent: u32,
    recent_activity: Vec<Activity>,
}

#[derive(Template)]
#[template(path = "admin/partials/labels.html")]
struct LabelsPartial {
    status: String,
    total_count: u32,
    white_count: u32,
    gray_count: u32,
    black_count: u32,
    red_count: u32,
    labels: Vec<LabelRow>,
}

#[derive(Template)]
#[template(path = "admin/partials/label_detail.html")]
struct LabelDetailPartial {
    label: LabelDetail,
}

#[derive(Template)]
#[template(path = "admin/partials/label_detail_overview.html")]
struct LabelOverviewPartial {
    label: LabelDetail,
}

#[derive(Template)]
#[template(path = "admin/partials/payments.html")]
struct PaymentsPartial {
    tab: String,
    upcoming_count: u32,
    failed_count: u32,
    failed_payments: Vec<FailedPayment>,
}

#[derive(Template)]
#[template(path = "admin/partials/system.html")]
struct SystemPartial {
    tab: String,
    version: String,
    deployed_at: String,
    uptime: String,
    cpu_percent: u32,
    memory_percent: u32,
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

struct Activity {
    timestamp: String,
    domain: String,
    action: String,
    admin: String,
}

struct LabelRow {
    id: String,
    domain: String,
    status: String,
    created_at: String,
    last_payment: String,
    storage_mb: u32,
}

struct LabelDetail {
    id: String,
    domain: String,
    status: String,
    created_at: String,
    plan_type: String,
    price: String,
    next_billing: String,
    card_last4: String,
    images_mb: u32,
    videos_mb: u32,
    db_mb: u32,
    total_mb: u32,
    products_count: u32,
    services_count: u32,
    posts_count: u32,
    last_login: String,
}

struct FailedPayment {
    label_id: String,
    domain: String,
    amount: String,
    failed_date: String,
    retry_count: u32,
}

#[derive(Deserialize)]
struct LabelQuery {
    status: Option<String>,
    q: Option<String>,
}

#[derive(Deserialize)]
struct TabQuery {
    tab: Option<String>,
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

/// Base admin page (loads once)
pub async fn admin_base() -> Result<HttpResponse> {
    let template = BaseTemplate;
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// Dashboard partial
pub async fn dashboard_partial(
    // Add DB connection pool, etc
) -> Result<HttpResponse> {
    // TODO: Fetch real data from SurrealDB
    let template = DashboardPartial {
        white_count: 42,
        gray_count: 3,
        black_count: 1,
        red_count: 8,
        white_change: 12,
        mrr: "2,100".to_string(),
        mrr_growth: 8,
        failed_payments: 2,
        storage_gb: 127,
        storage_percent: 63,
        recent_activity: vec![
            Activity {
                timestamp: "2025-10-20 09:15".to_string(),
                domain: "example.com".to_string(),
                action: "White label created".to_string(),
                admin: "You".to_string(),
            },
            Activity {
                timestamp: "2025-10-20 08:42".to_string(),
                domain: "test.com".to_string(),
                action: "Payment received".to_string(),
                admin: "System".to_string(),
            },
        ],
    };

    render_template(template)
}

/// Labels list partial
pub async fn labels_partial(
    query: web::Query<LabelQuery>,
) -> Result<HttpResponse> {
    let status = query.status.as_deref().unwrap_or("all");
    let search = query.q.as_deref();

    // TODO: Fetch from SurrealDB with filters
    let labels = vec![
        LabelRow {
            id: "label_123".to_string(),
            domain: "example.com".to_string(),
            status: "white".to_string(),
            created_at: "2025-10-15".to_string(),
            last_payment: "2025-10-19".to_string(),
            storage_mb: 245,
        },
        LabelRow {
            id: "label_456".to_string(),
            domain: "test.com".to_string(),
            status: "gray".to_string(),
            created_at: "2025-09-01".to_string(),
            last_payment: "2025-10-12".to_string(),
            storage_mb: 128,
        },
    ];

    let template = LabelsPartial {
        status: status.to_string(),
        total_count: 54,
        white_count: 42,
        gray_count: 3,
        black_count: 1,
        red_count: 8,
        labels,
    };

    render_template(template)
}

/// Label detail partial
pub async fn label_detail_partial(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Fetch from SurrealDB
    let label = LabelDetail {
        id: label_id,
        domain: "example.com".to_string(),
        status: "white".to_string(),
        created_at: "2025-10-15".to_string(),
        plan_type: "basic".to_string(),
        price: "10.00".to_string(),
        next_billing: "2025-10-27".to_string(),
        card_last4: "4242".to_string(),
        images_mb: 120,
        videos_mb: 80,
        db_mb: 45,
        total_mb: 245,
        products_count: 12,
        services_count: 5,
        posts_count: 23,
        last_login: "2025-10-20 08:30".to_string(),
    };

    let template = LabelDetailPartial { label };
    render_template(template)
}

/// Label detail overview tab
pub async fn label_overview_partial(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Fetch from SurrealDB
    let label = LabelDetail {
        id: label_id,
        domain: "example.com".to_string(),
        status: "white".to_string(),
        created_at: "2025-10-15".to_string(),
        plan_type: "basic".to_string(),
        price: "10.00".to_string(),
        next_billing: "2025-10-27".to_string(),
        card_last4: "4242".to_string(),
        images_mb: 120,
        videos_mb: 80,
        db_mb: 45,
        total_mb: 245,
        products_count: 12,
        services_count: 5,
        posts_count: 23,
        last_login: "2025-10-20 08:30".to_string(),
    };

    let template = LabelOverviewPartial { label };
    render_template(template)
}

/// Payments partial
pub async fn payments_partial(
    query: web::Query<TabQuery>,
) -> Result<HttpResponse> {
    let tab = query.tab.as_deref().unwrap_or("upcoming");

    // TODO: Fetch from SurrealDB
    let failed_payments = if tab == "failed" {
        vec![
            FailedPayment {
                label_id: "label_789".to_string(),
                domain: "overdue.com".to_string(),
                amount: "10.00".to_string(),
                failed_date: "2025-10-18".to_string(),
                retry_count: 2,
            },
        ]
    } else {
        vec![]
    };

    let template = PaymentsPartial {
        tab: tab.to_string(),
        upcoming_count: 7,
        failed_count: 2,
        failed_payments,
    };

    render_template(template)
}

/// System health partial
pub async fn system_partial(
    query: web::Query<TabQuery>,
) -> Result<HttpResponse> {
    let tab = query.tab.as_deref().unwrap_or("overview");

    // TODO: Fetch real system metrics
    let template = SystemPartial {
        tab: tab.to_string(),
        version: "1.0.0".to_string(),
        deployed_at: "2025-10-19 22:00".to_string(),
        uptime: "14d 6h 23m".to_string(),
        cpu_percent: 23,
        memory_percent: 42,
    };

    render_template(template)
}

// ============================================================================
// API ENDPOINTS (for actions)
// ============================================================================

/// Delete a white label
pub async fn delete_label(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Soft delete in SurrealDB
    // TODO: Enqueue background job for cleanup

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Label deleted"
    })))
}

/// Suspend a white label
pub async fn suspend_label(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Update status in SurrealDB

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "status": "suspended"
    })))
}

/// Send email to white label owner
pub async fn email_label(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Enqueue email job

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "sent": true
    })))
}

// ============================================================================
// HELPERS
// ============================================================================

fn render_template<T: Template>(template: T) -> Result<HttpResponse> {
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // TODO: Add admin authentication middleware
            // Base page (loads once)
            .route("", web::get().to(admin_base))

            // Partials (HTMX endpoints)
            .route("/partials/dashboard", web::get().to(dashboard_partial))
            .route("/partials/labels", web::get().to(labels_partial))
            .route("/partials/labels/{id}", web::get().to(label_detail_partial))
            .route("/partials/labels/{id}/overview", web::get().to(label_overview_partial))
            .route("/partials/payments", web::get().to(payments_partial))
            .route("/partials/system", web::get().to(system_partial))

            // API endpoints (actions)
            .route("/api/labels/{id}", web::delete().to(delete_label))
            .route("/api/labels/{id}/suspend", web::post().to(suspend_label))
            .route("/api/labels/{id}/email", web::post().to(email_label))
    );
}

// ============================================================================
// USAGE IN MAIN APP
// ============================================================================

/*
// In apps/lillpepe/src/main.rs

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(web_admin::routes::configure)
            // ... other routes
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
*/