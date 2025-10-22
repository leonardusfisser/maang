use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Service entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub status: ServiceStatus,
    pub metadata: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Active,
    Inactive,
}

impl Service {
    #[must_use]
    pub fn new(id: String, tenant_id: String, name: String, description: String) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id,
            tenant_id,
            name,
            description,
            status: ServiceStatus::Active,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.status, ServiceStatus::Active)
    }
}