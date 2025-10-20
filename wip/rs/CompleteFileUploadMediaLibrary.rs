// crates/infrastructure/storage/src/lib.rs
//! File storage and upload management

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Result as ActixResult};
use futures_util::stream::StreamExt as _;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File too large: {0} bytes (max: {1})")]
    FileTooLarge(usize, usize),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),
}

// ============================================================================
// STORAGE CONFIGURATION
// ============================================================================

pub struct StorageConfig {
    pub base_path: PathBuf,
    pub max_file_size: usize,
    pub allowed_types: Vec<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("/opt/lillpepe/storage"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "audio/mpeg".to_string(),
                "audio/wav".to_string(),
            ],
        }
    }
}

// ============================================================================
// STORAGE SERVICE
// ============================================================================

pub struct StorageService {
    config: StorageConfig,
}

impl StorageService {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    pub async fn init_storage(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.config.base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))
    }

    pub async fn save_file(
        &self,
        white_label_id: &str,
        filename: &str,
        mime_type: &str,
        data: Vec<u8>,
    ) -> Result<String, StorageError> {
        // Validate file size
        if data.len() > self.config.max_file_size {
            return Err(StorageError::FileTooLarge(
                data.len(),
                self.config.max_file_size,
            ));
        }

        // Validate mime type
        if !self.config.allowed_types.contains(&mime_type.to_string()) {
            return Err(StorageError::InvalidFileType(mime_type.to_string()));
        }

        // Sanitize filename
        let safe_filename = sanitize_filename(filename);

        // Create unique filename
        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
        let unique_filename = format!("{}_{}", timestamp, safe_filename);

        // Create directory structure: /storage/{white_label_id}/media/
        let label_dir = self.config.base_path.join(white_label_id).join("media");
        fs::create_dir_all(&label_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Write file
        let file_path = label_dir.join(&unique_filename);
        fs::write(&file_path, data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Return URL path
        Ok(format!("/media/{}/{}", white_label_id, unique_filename))
    }

    pub async fn delete_file(
        &self,
        white_label_id: &str,
        filename: &str,
    ) -> Result<(), StorageError> {
        let file_path = self
            .config
            .base_path
            .join(white_label_id)
            .join("media")
            .join(filename);

        fs::remove_file(file_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))
    }

    pub async fn list_files(
        &self,
        white_label_id: &str,
    ) -> Result<Vec<FileInfo>, StorageError> {
        let label_dir = self.config.base_path.join(white_label_id).join("media");

        let mut files = Vec::new();
        let mut entries = fs::read_dir(&label_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| StorageError::IoError(e.to_string()))? {
            let metadata = entry.metadata().await.map_err(|e| StorageError::IoError(e.to_string()))?;

            if metadata.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let size = metadata.len();

                files.push(FileInfo {
                    filename: filename.clone(),
                    size,
                    url: format!("/media/{}/{}", white_label_id, filename),
                    mime_type: guess_mime_type(&filename),
                });
            }
        }

        Ok(files)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub url: String,
    pub mime_type: String,
}

// ============================================================================
// UPLOAD HANDLER
// ============================================================================

pub async fn upload_file(
    mut payload: Multipart,
    storage: web::Data<StorageService>,
    db: web::Data<infrastructure_db::Database>,
    white_label_id: String,
) -> ActixResult<HttpResponse> {
    let mut filename = String::new();
    let mut mime_type = String::new();
    let mut data = Vec::new();

    // Process multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(actix_web::error::ErrorBadRequest)?;

        // Get content disposition
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.get_name() {
            if name == "file" {
                filename = content_disposition
                    .get_filename()
                    .unwrap_or("unnamed")
                    .to_string();

                mime_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                // Read file data
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(actix_web::error::ErrorBadRequest)?;
                    data.extend_from_slice(&chunk);
                }
            }
        }
    }

    // Save file
    let url = storage
        .save_file(&white_label_id, &filename, &mime_type, data.clone())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    // Save to database
    let query = r#"
        CREATE media CONTENT {
            white_label_id: $white_label_id,
            filename: $filename,
            media_type: $media_type,
            mime_type: $mime_type,
            size_bytes: $size,
            url: $url,
            created_at: time::now()
        }
    "#;

    db.client()
        .query(query)
        .bind(("white_label_id", format!("white_labels:{}", white_label_id)))
        .bind(("filename", filename))
        .bind(("media_type", detect_media_type(&mime_type)))
        .bind(("mime_type", mime_type))
        .bind(("size", data.len() as i64))
        .bind(("url", url.clone()))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "url": url,
        "filename": filename
    })))
}

pub async fn delete_file(
    path: web::Path<String>,
    storage: web::Data<StorageService>,
    db: web::Data<infrastructure_db::Database>,
    white_label_id: String,
) -> ActixResult<HttpResponse> {
    let media_id = path.into_inner();

    // Get media from DB
    let media: Option<infrastructure_db::Media> = db
        .client()
        .select(("media", media_id.as_str()))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(media) = media {
        // Extract filename from URL
        let filename = media.url.split('/').last().unwrap_or("");

        // Delete file
        storage
            .delete_file(&white_label_id, filename)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        // Delete from DB
        db.client()
            .delete(("media", media_id))
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true
    })))
}

// ============================================================================
// HELPERS
// ============================================================================

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn guess_mime_type(filename: &str) -> String {
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
        .to_string()
}

fn detect_media_type(mime_type: &str) -> String {
    if mime_type.starts_with("image/") {
        "image"
    } else if mime_type.starts_with("video/") {
        "video"
    } else if mime_type.starts_with("audio/") {
        "audio"
    } else {
        "other"
    }
        .to_string()
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/dashboard/upload", web::post().to(upload_file))
        .route("/dashboard/media/{id}", web::delete().to(delete_file));
}