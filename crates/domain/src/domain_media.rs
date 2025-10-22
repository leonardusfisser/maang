use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Media entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub media_type: MediaType,
    pub url: String,
    pub size_bytes: i64,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
}

impl Media {
    #[must_use]
    pub fn new(
        id: String,
        tenant_id: String,
        name: String,
        media_type: MediaType,
        url: String,
        size_bytes: i64,
    ) -> Self {
        Self {
            id,
            tenant_id,
            name,
            media_type,
            url,
            size_bytes,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}