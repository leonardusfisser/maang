// crates/application/api/src/lib.rs
//! Public API for white label content management

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use infrastructure_db::Database;
use serde::{Deserialize, Serialize};

// ============================================================================
// PRODUCTS API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub white_label_id: String,
    pub title: String,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: String,
    pub image_url: Option<String>,
    pub is_published: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub title: String,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub is_published: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub is_published: Option<bool>,
}

/// Get all products for current white label
pub async fn get_products(
    req: HttpRequest,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM products WHERE white_label_id = $white_label_id AND is_published = true ORDER BY created_at DESC";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let products: Vec<Product> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(products))
}

/// Get single product
pub async fn get_product(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let product_id = path.into_inner();
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM products WHERE id = $id AND white_label_id = $white_label_id LIMIT 1";

    let mut result = db
        .client()
        .query(query)
        .bind(("id", format!("products:{}", product_id)))
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let products: Vec<Product> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let product = products
        .into_iter()
        .next()
        .ok_or_else(|| actix_web::error::ErrorNotFound("Product not found"))?;

    Ok(HttpResponse::Ok().json(product))
}

/// Create product (authenticated)
pub async fn create_product(
    req: HttpRequest,
    body: web::Json<CreateProductRequest>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = r#"
        CREATE products CONTENT {
            white_label_id: $white_label_id,
            title: $title,
            description: $description,
            price: $price,
            currency: $currency,
            image_url: $image_url,
            is_published: $is_published,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .bind(("title", &body.title))
        .bind(("description", &body.description))
        .bind(("price", body.price))
        .bind(("currency", body.currency.as_deref().unwrap_or("EUR")))
        .bind(("image_url", &body.image_url))
        .bind(("is_published", body.is_published.unwrap_or(false)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let product: Option<Product> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(product))
}

/// Update product (authenticated)
pub async fn update_product(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateProductRequest>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let product_id = path.into_inner();
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let mut query = "UPDATE $id SET updated_at = time::now()".to_string();

    if body.title.is_some() {
        query.push_str(", title = $title");
    }
    if body.description.is_some() {
        query.push_str(", description = $description");
    }
    if body.price.is_some() {
        query.push_str(", price = $price");
    }
    if body.currency.is_some() {
        query.push_str(", currency = $currency");
    }
    if body.image_url.is_some() {
        query.push_str(", image_url = $image_url");
    }
    if body.is_published.is_some() {
        query.push_str(", is_published = $is_published");
    }

    query.push_str(" WHERE white_label_id = $white_label_id");

    let mut q = db
        .client()
        .query(query)
        .bind(("id", format!("products:{}", product_id)))
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)));

    if let Some(t) = &body.title {
        q = q.bind(("title", t));
    }
    if let Some(d) = &body.description {
        q = q.bind(("description", d));
    }
    if let Some(p) = body.price {
        q = q.bind(("price", p));
    }
    if let Some(c) = &body.currency {
        q = q.bind(("currency", c));
    }
    if let Some(i) = &body.image_url {
        q = q.bind(("image_url", i));
    }
    if let Some(pub_val) = body.is_published {
        q = q.bind(("is_published", pub_val));
    }

    q.await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Product updated"
    })))
}

/// Delete product (authenticated)
pub async fn delete_product(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let product_id = path.into_inner();
    let _white_label_id = extract_white_label_id(&req, &db).await?;

    db.client()
        .delete(("products", product_id))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Product deleted"
    })))
}

// ============================================================================
// SERVICES API (similar to products)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub white_label_id: String,
    pub title: String,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: String,
    pub image_url: Option<String>,
    pub is_published: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_services(
    req: HttpRequest,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM services WHERE white_label_id = $white_label_id AND is_published = true ORDER BY created_at DESC";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let services: Vec<Service> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(services))
}

// ============================================================================
// POSTS API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub white_label_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub is_published: bool,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_posts(
    req: HttpRequest,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM posts WHERE white_label_id = $white_label_id AND is_published = true ORDER BY published_at DESC";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let posts: Vec<Post> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn get_post(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let slug = path.into_inner();
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM posts WHERE white_label_id = $white_label_id AND slug = $slug AND is_published = true LIMIT 1";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .bind(("slug", slug))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let posts: Vec<Post> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let post = posts
        .into_iter()
        .next()
        .ok_or_else(|| actix_web::error::ErrorNotFound("Post not found"))?;

    Ok(HttpResponse::Ok().json(post))
}

// ============================================================================
// PAGES API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub white_label_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub is_published: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_page(
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let slug = path.into_inner();
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM pages WHERE white_label_id = $white_label_id AND slug = $slug AND is_published = true LIMIT 1";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .bind(("slug", slug))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let pages: Vec<Page> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let page = pages
        .into_iter()
        .next()
        .ok_or_else(|| actix_web::error::ErrorNotFound("Page not found"))?;

    Ok(HttpResponse::Ok().json(page))
}

// ============================================================================
// MEDIA API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub white_label_id: String,
    pub filename: String,
    pub media_type: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub url: String,
    pub created_at: String,
}

pub async fn get_media(
    req: HttpRequest,
    db: web::Data<Database>,
) -> ActixResult<HttpResponse> {
    let white_label_id = extract_white_label_id(&req, &db).await?;

    let query = "SELECT * FROM media WHERE white_label_id = $white_label_id ORDER BY created_at DESC";

    let mut result = db
        .client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let media: Vec<Media> = result
        .take(0)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(media))
}

// ============================================================================
// HELPERS
// ============================================================================

async fn extract_white_label_id(
    req: &HttpRequest,
    db: &Database,
) -> ActixResult<String> {
    // Extract domain from Host header
    let domain = req
        .headers()
        .get("Host")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing Host header"))?;

    // Get white label by domain
    let label = db
        .get_label_by_domain(domain)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("White label not found"))?;

    Ok(label.id)
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Health check
            .route("/health", web::get().to(health_check))

            // Products
            .route("/products", web::get().to(get_products))
            .route("/products/{id}", web::get().to(get_product))
            .route("/products", web::post().to(create_product))
            .route("/products/{id}", web::put().to(update_product))
            .route("/products/{id}", web::delete().to(delete_product))

            // Services
            .route("/services", web::get().to(get_services))

            // Posts
            .route("/posts", web::get().to(get_posts))
            .route("/posts/{slug}", web::get().to(get_post))

            // Pages
            .route("/pages/{slug}", web::get().to(get_page))

            // Media
            .route("/media", web::get().to(get_media))
    );
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    })))
}