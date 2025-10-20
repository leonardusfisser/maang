// crates/application/web/src/admin/routes.rs
//! Admin dashboard routes with SurrealDB integration

use actix_web::{web, HttpResponse, Result};
use askama::Template;
use serde::Deserialize;
use infrastructure_db::{Database, LabelStatus};

// ============================================================================
// TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "admin/base.html")]
struct BaseTemplate;

#[derive(Template)]
#[template(path = "admin/partials/dashboard.html")]
struct DashboardPartial {
    white_count: i64,
    gray_count: i64,
    black_count: i64,
    red_count: i64,
    white_change: i32,
    mrr: String,
    mrr_growth: i32,
    failed_payments: i64,
    storage_gb: String,
    storage_percent: u32,
    recent_activity: Vec<ActivityRow>,
}

#[derive(Template)]
#[template(path = "admin/partials/labels.html")]
struct LabelsPartial {
    status: String,
    total_count: i64,
    white_count: i64,
    gray_count: i64,
    black_count: i64,
    red_count: i64,
    labels: Vec<LabelRow>,
}

#[derive(Template)]
#[template(path = "admin/partials/label_detail.html")]
struct LabelDetailPartial {
    label: LabelDetailView,
}

#[derive(Template)]
#[template(path = "admin/partials/label_detail_overview.html")]
struct LabelOverviewPartial {
    label: LabelDetailView,
}

#[derive(Template)]
#[template(path = "admin/partials/payments.html")]
struct PaymentsPartial {
    tab: String,
    upcoming_count: u32,
    failed_count: i64,
    failed_payments: Vec<FailedPaymentRow>,
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
// VIEW MODELS
// ============================================================================

struct ActivityRow {
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

struct LabelDetailView {
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

struct FailedPaymentRow {
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
    render_template(template)
}

/// Dashboard partial with real metrics
pub async fn dashboard_partial(
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let metrics = db
        .get_dashboard_metrics()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recent_activity_data = db
        .get_recent_activity(10)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recent_activity = recent_activity_data
        .into_iter()
        .map(|log| ActivityRow {
            timestamp: format_timestamp(&log.created_at),
            domain: extract_domain(&log.description),
            action: log.action_type,
            admin: if log.admin_action { "Admin".to_string() } else { "System".to_string() },
        })
        .collect();

    let storage_gb = (metrics.total_storage_bytes as f64 / 1_073_741_824.0);
    let storage_percent = ((storage_gb / 500.0) * 100.0) as u32; // Assuming 500GB capacity

    let template = DashboardPartial {
        white_count: metrics.status_counts.white,
        gray_count: metrics.status_counts.gray,
        black_count: metrics.status_counts.black,
        red_count: metrics.status_counts.red,
        white_change: 12, // TODO: Calculate from historical data
        mrr: format!("{:.2}", metrics.mrr),
        mrr_growth: 8, // TODO: Calculate from historical data
        failed_payments: metrics.failed_payments_count,
        storage_gb: format!("{:.1}", storage_gb),
        storage_percent,
        recent_activity,
    };

    render_template(template)
}

/// Labels list partial with filtering
pub async fn labels_partial(
    db: web::Data<Database>,
    query: web::Query<LabelQuery>,
) -> Result<HttpResponse> {
    let status = query.status.as_deref().unwrap_or("all");
    let search = query.q.as_deref();

    // Get counts for all statuses
    let counts = db
        .count_labels_by_status()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Fetch labels based on filters
    let labels_data = if let Some(search_term) = search {
        db.search_labels(search_term)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else if status == "all" {
        db.get_all_labels()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        db.get_labels_by_status(status)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let labels = labels_data
        .into_iter()
        .map(|label| LabelRow {
            id: label.id,
            domain: label.domain,
            status: format!("{:?}", label.status).to_lowercase(),
            created_at: format_date(&label.created_at),
            last_payment: label
                .subscription
                .last_payment_date
                .as_ref()
                .map(|d| format_date(d))
                .unwrap_or_else(|| "Never".to_string()),
            storage_mb: bytes_to_mb(
                label.storage.images_bytes
                    + label.storage.videos_bytes
                    + label.storage.audio_bytes
                    + label.storage.db_bytes,
            ),
        })
        .collect();

    let template = LabelsPartial {
        status: status.to_string(),
        total_count: counts.total,
        white_count: counts.white,
        gray_count: counts.gray,
        black_count: counts.black,
        red_count: counts.red,
        labels,
    };

    render_template(template)
}

/// Label detail partial
pub async fn label_detail_partial(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    let label = db
        .get_label_by_id(&label_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Label not found"))?;

    let total_storage = label.storage.images_bytes
        + label.storage.videos_bytes
        + label.storage.audio_bytes
        + label.storage.db_bytes;

    let view = LabelDetailView {
        id: label.id,
        domain: label.domain,
        status: format!("{:?}", label.status).to_lowercase(),
        created_at: format_date(&label.created_at),
        plan_type: label.subscription.plan_type,
        price: format!("{:.2}", label.subscription.price_per_cycle),
        next_billing: label
            .subscription
            .next_billing_date
            .as_ref()
            .map(|d| format_date(d))
            .unwrap_or_else(|| "N/A".to_string()),
        card_last4: label
            .subscription
            .stripe_customer_id
            .as_ref()
            .and_then(|_| Some("4242".to_string())) // TODO: Get from Stripe
            .unwrap_or_else(|| "N/A".to_string()),
        images_mb: bytes_to_mb(label.storage.images_bytes),
        videos_mb: bytes_to_mb(label.storage.videos_bytes),
        db_mb: bytes_to_mb(label.storage.db_bytes),
        total_mb: bytes_to_mb(total_storage),
        products_count: 0, // TODO: Query products table
        services_count: 0, // TODO: Query services table
        posts_count: 0,    // TODO: Query posts table
        last_login: "2025-10-20 08:30".to_string(), // TODO: Get from users table
    };

    let template = LabelDetailPartial { label: view };
    render_template(template)
}

/// Label detail overview tab
pub async fn label_overview_partial(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    let label = db
        .get_label_by_id(&label_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Label not found"))?;

    let total_storage = label.storage.images_bytes
        + label.storage.videos_bytes
        + label.storage.audio_bytes
        + label.storage.db_bytes;

    let view = LabelDetailView {
        id: label.id,
        domain: label.domain,
        status: format!("{:?}", label.status).to_lowercase(),
        created_at: format_date(&label.created_at),
        plan_type: label.subscription.plan_type,
        price: format!("{:.2}", label.subscription.price_per_cycle),
        next_billing: label
            .subscription
            .next_billing_date
            .as_ref()
            .map(|d| format_date(d))
            .unwrap_or_else(|| "N/A".to_string()),
        card_last4: "4242".to_string(), // TODO: Get from Stripe
        images_mb: bytes_to_mb(label.storage.images_bytes),
        videos_mb: bytes_to_mb(label.storage.videos_bytes),
        db_mb: bytes_to_mb(label.storage.db_bytes),
        total_mb: bytes_to_mb(total_storage),
        products_count: 0,
        services_count: 0,
        posts_count: 0,
        last_login: "2025-10-20 08:30".to_string(),
    };

    let template = LabelOverviewPartial { label: view };
    render_template(template)
}

/// Payments partial
pub async fn payments_partial(
    db: web::Data<Database>,
    query: web::Query<TabQuery>,
) -> Result<HttpResponse> {
    let tab = query.tab.as_deref().unwrap_or("upcoming");

    let failed_payments_data = if tab == "failed" {
        let payments = db
            .get_failed_payments()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        let mut rows = Vec::new();
        for payment in payments {
            // Get label domain
            if let Some(label_id) = payment.white_label_id.strip_prefix("white_labels:") {
                if let Ok(Some(label)) = db.get_label_by_id(label_id).await {
                    rows.push(FailedPaymentRow {
                        label_id: label_id.to_string(),
                        domain: label.domain,
                        amount: format!("{:.2}", payment.amount),
                        failed_date: format_date(&payment.created_at),
                        retry_count: 0, // TODO: Track retry attempts
                    });
                }
            }
        }
        rows
    } else {
        vec![]
    };

    let failed_count = db
        .get_failed_payments()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .len() as i64;

    let template = PaymentsPartial {
        tab: tab.to_string(),
        upcoming_count: 7, // TODO: Query upcoming renewals
        failed_count,
        failed_payments: failed_payments_data,
    };

    render_template(template)
}

/// System health partial
pub async fn system_partial(
    query: web::Query<TabQuery>,
) -> Result<HttpResponse> {
    let tab = query.tab.as_deref().unwrap_or("overview");

    // TODO: Get real system metrics
    let template = SystemPartial {
        tab: tab.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        deployed_at: "2025-10-19 22:00".to_string(),
        uptime: "14d 6h 23m".to_string(),
        cpu_percent: 23,
        memory_percent: 42,
    };

    render_template(template)
}

// ============================================================================
// API ENDPOINTS
// ============================================================================

/// Delete a white label
pub async fn delete_label(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    db.delete_label(&label_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log activity
    db.log_activity(
        Some(&label_id),
        "label_deleted",
        &format!("White label {} deleted", label_id),
        true,
    )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Label deleted successfully"
    })))
}

/// Suspend a white label
pub async fn suspend_label(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    db.update_label_status(&label_id, LabelStatus::Black)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log activity
    db.log_activity(
        Some(&label_id),
        "label_suspended",
        &format!("White label {} suspended", label_id),
        true,
    )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "status": "suspended"
    })))
}

/// Send email to white label owner
pub async fn email_label(
    path: web::Path<String>,
    // TODO: Add job queue
) -> Result<HttpResponse> {
    let label_id = path.into_inner();

    // TODO: Enqueue email job

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "sent": true,
        "message": format!("Email queued for label {}", label_id)
    })))
}

// ============================================================================
// HELPERS
// ============================================================================

fn render_template<T: Template>(template: T) -> Result<HttpResponse> {
    let html = template
        .render()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn bytes_to_mb(bytes: i64) -> u32 {
    (bytes / 1_048_576) as u32
}

fn format_date(date_str: &str) -> String {
    // TODO: Parse and format properly
    date_str.split('T').next().unwrap_or(date_str).to_string()
}

fn format_timestamp(timestamp: &str) -> String {
    // TODO: Parse and format with time
    timestamp.split('.').next().unwrap_or(timestamp).replace('T', " ")
}

fn extract_domain(description: &str) -> String {
    // Simple extraction - improve as needed
    description
        .split_whitespace()
        .find(|word| word.contains('.'))
        .unwrap_or("N/A")
        .to_string()
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // TODO: Add AdminAuth middleware
            .route("", web::get().to(admin_base))
            .route("/partials/dashboard", web::get().to(dashboard_partial))
            .route("/partials/labels", web::get().to(labels_partial))
            .route("/partials/labels/{id}", web::get().to(label_detail_partial))
            .route("/partials/labels/{id}/overview", web::get().to(label_overview_partial))
            .route("/partials/payments", web::get().to(payments_partial))
            .route("/partials/system", web::get().to(system_partial))
            .route("/api/labels/{id}", web::delete().to(delete_label))
            .route("/api/labels/{id}/suspend", web::post().to(suspend_label))
            .route("/api/labels/{id}/email", web::post().to(email_label))
    );
}