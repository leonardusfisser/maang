use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl User {
    #[must_use]
    pub fn new(
        id: String,
        tenant_id: String,
        email: String,
        password_hash: String,
        salt: String,
        role: UserRole,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id,
            tenant_id,
            email,
            password_hash,
            salt,
            role,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Active)
    }

    #[must_use]
    pub const fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    pub fn suspend(&mut self) {
        self.status = UserStatus::Suspended;
        self.updated_at = OffsetDateTime::now_utc().unix_timestamp();
    }

    pub fn activate(&mut self) {
        self.status = UserStatus::Active;
        self.updated_at = OffsetDateTime::now_utc().unix_timestamp();
    }
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub tenant_id: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}