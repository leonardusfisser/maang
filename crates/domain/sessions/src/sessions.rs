use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub tenant_id: String,
    pub created_at: i64,
}

impl Session {
    pub fn new(user_id: String, tenant_id: String) -> Self {
        Self {
            user_id,
            tenant_id,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        }
    }
}