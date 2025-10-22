use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub csrf_token: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_activity: i64,
}

impl Session {
    #[must_use]
    pub fn new(
        id: String,
        user_id: String,
        tenant_id: String,
        csrf_token: String,
        ttl_secs: u64,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let ttl = i64::try_from(ttl_secs).unwrap_or(3600);

        Self {
            id,
            user_id,
            tenant_id,
            csrf_token,
            created_at: now,
            expires_at: now + ttl,
            last_activity: now,
        }
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        now > self.expires_at
    }

    pub fn touch(&mut self) {
        self.last_activity = OffsetDateTime::now_utc().unix_timestamp();
    }

    pub fn extend(&mut self, ttl_secs: u64) {
        let ttl = i64::try_from(ttl_secs).unwrap_or(3600);
        self.expires_at = OffsetDateTime::now_utc().unix_timestamp() + ttl;
    }
}