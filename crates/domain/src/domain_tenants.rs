use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Tenant entity - represents isolated customer instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub status: TenantStatus,
    pub config: TenantConfig,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    Active,
    Suspended,
    Provisioning,
    Deactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub background_media: Option<String>,
    pub logo_url: Option<String>,
    pub custom_css_url: Option<String>,
}

impl Tenant {
    #[must_use]
    pub fn new(id: String, name: String, domain: String) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id,
            name,
            domain,
            status: TenantStatus::Provisioning,
            config: TenantConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active)
    }

    pub fn activate(&mut self) {
        self.status = TenantStatus::Active;
        self.updated_at = OffsetDateTime::now_utc().unix_timestamp();
    }

    pub fn suspend(&mut self) {
        self.status = TenantStatus::Suspended;
        self.updated_at = OffsetDateTime::now_utc().unix_timestamp();
    }

    pub fn update_config(&mut self, config: TenantConfig) {
        self.config = config;
        self.updated_at = OffsetDateTime::now_utc().unix_timestamp();
    }
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            primary_color: "#000000".to_string(),
            secondary_color: "#ffffff".to_string(),
            accent_color: "#ff0000".to_string(),
            background_color: "#f5f5f5".to_string(),
            text_color: "#333333".to_string(),
            background_media: None,
            logo_url: None,
            custom_css_url: None,
        }
    }
}

/// Tenant creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenant {
    pub name: String,
    pub domain: String,
}

/// Tenant update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenant {
    pub name: Option<String>,
    pub status: Option<TenantStatus>,
    pub config: Option<TenantConfig>,
}