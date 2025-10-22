use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Product entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub currency: String,
    pub status: ProductStatus,
    pub image_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductStatus {
    Active,
    Inactive,
    Archived,
}

impl Product {
    #[must_use]
    pub fn new(
        id: String,
        tenant_id: String,
        name: String,
        description: String,
        price: i64,
        currency: String,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id,
            tenant_id,
            name,
            description,
            price,
            currency,
            status: ProductStatus::Active,
            image_url: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.status, ProductStatus::Active)
    }
}

/// Product creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProduct {
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub currency: String,
}