#![forbid(unsafe_code)]
#![deny(warnings)]// crates/infrastructure/db/src/lib.rs
//! SurrealDB database connection and query helpers

use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Result as SurrealResult, Surreal};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    /// Create new database connection
    pub async fn new(url: &str, namespace: &str, database: &str) -> Result<Self, DbError> {
        let client = Surreal::new::<Ws>(url)
            .await
            .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;

        // Sign in as root user
        client
            .signin(Root {
                username: "root",
                password: "root", // TODO: Move to config
            })
            .await
            .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;

        // Use namespace and database
        client
            .use_ns(namespace)
            .use_db(database)
            .await
            .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;

        Ok(Self { client })
    }

    /// Get reference to underlying client
    pub fn client(&self) -> &Surreal<Client> {
        &self.client
    }
}

// ============================================================================
// WHITE LABEL QUERIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabel {
    pub id: String,
    pub domain: String,
    pub status: LabelStatus,
    pub created_at: String,
    pub updated_at: String,
    pub subscription: Subscription,
    pub customization: Customization,
    pub storage: Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelStatus {
    White,
    Gray,
    Black,
    Red,
}

impl LabelStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::White => "white",
            Self::Gray => "gray",
            Self::Black => "black",
            Self::Red => "red",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub plan_type: String,
    pub price_per_cycle: f64,
    pub billing_cycle: String,
    pub next_billing_date: Option<String>,
    pub last_payment_date: Option<String>,
    pub payment_failed_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customization {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub logo_url: Option<String>,
    pub background_media_url: Option<String>,
    pub custom_css_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub images_bytes: i64,
    pub videos_bytes: i64,
    pub audio_bytes: i64,
    pub db_bytes: i64,
}

impl Database {
    /// Get all white labels
    pub async fn get_all_labels(&self) -> Result<Vec<WhiteLabel>, DbError> {
        let result: Vec<WhiteLabel> = self
            .client
            .select("white_labels")
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(result)
    }

    /// Get white labels by status
    pub async fn get_labels_by_status(&self, status: &str) -> Result<Vec<WhiteLabel>, DbError> {
        let query = "SELECT * FROM white_labels WHERE status = $status ORDER BY created_at DESC";

        let mut result = self
            .client
            .query(query)
            .bind(("status", status))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let labels: Vec<WhiteLabel> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(labels)
    }

    /// Get white label by ID
    pub async fn get_label_by_id(&self, id: &str) -> Result<Option<WhiteLabel>, DbError> {
        let label: Option<WhiteLabel> = self
            .client
            .select(("white_labels", id))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(label)
    }

    /// Get white label by domain
    pub async fn get_label_by_domain(&self, domain: &str) -> Result<Option<WhiteLabel>, DbError> {
        let query = "SELECT * FROM white_labels WHERE domain = $domain LIMIT 1";

        let mut result = self
            .client
            .query(query)
            .bind(("domain", domain))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let labels: Vec<WhiteLabel> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(labels.into_iter().next())
    }

    /// Count labels by status
    pub async fn count_labels_by_status(&self) -> Result<StatusCounts, DbError> {
        let query = r#"
            SELECT
                count() AS total,
                count(status = 'white') AS white,
                count(status = 'gray') AS gray,
                count(status = 'black') AS black,
                count(status = 'red') AS red
            FROM white_labels
            GROUP ALL
        "#;

        let mut result = self
            .client
            .query(query)
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let counts: Option<StatusCounts> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(counts.unwrap_or_default())
    }

    /// Update white label status
    pub async fn update_label_status(&self, id: &str, status: LabelStatus) -> Result<(), DbError> {
        let query = "UPDATE $id SET status = $status, updated_at = time::now()";

        self.client
            .query(query)
            .bind(("id", format!("white_labels:{}", id)))
            .bind(("status", status.as_str()))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete white label (soft delete - set status to red)
    pub async fn delete_label(&self, id: &str) -> Result<(), DbError> {
        self.update_label_status(id, LabelStatus::Red).await
    }

    /// Search white labels by domain
    pub async fn search_labels(&self, search_term: &str) -> Result<Vec<WhiteLabel>, DbError> {
        let query = "SELECT * FROM white_labels WHERE domain ~ $pattern ORDER BY created_at DESC";

        let mut result = self
            .client
            .query(query)
            .bind(("pattern", format!("(?i){}", search_term))) // Case-insensitive regex
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let labels: Vec<WhiteLabel> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(labels)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatusCounts {
    pub total: i64,
    pub white: i64,
    pub gray: i64,
    pub black: i64,
    pub red: i64,
}

// ============================================================================
// PAYMENT QUERIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub white_label_id: String,
    pub stripe_invoice_id: String,
    pub stripe_payment_intent_id: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub status: PaymentStatus,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

impl Database {
    /// Get failed payments
    pub async fn get_failed_payments(&self) -> Result<Vec<Payment>, DbError> {
        let query = "SELECT * FROM payments WHERE status = 'failed' ORDER BY created_at DESC";

        let mut result = self
            .client
            .query(query)
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let payments: Vec<Payment> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(payments)
    }

    /// Get payments for a white label
    pub async fn get_payments_for_label(&self, label_id: &str) -> Result<Vec<Payment>, DbError> {
        let query = "SELECT * FROM payments WHERE white_label_id = $label_id ORDER BY created_at DESC";

        let mut result = self
            .client
            .query(query)
            .bind(("label_id", format!("white_labels:{}", label_id)))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let payments: Vec<Payment> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(payments)
    }
}

// ============================================================================
// ACTIVITY LOG QUERIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: String,
    pub white_label_id: Option<String>,
    pub user_id: Option<String>,
    pub admin_action: bool,
    pub action_type: String,
    pub description: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

impl Database {
    /// Get recent activity
    pub async fn get_recent_activity(&self, limit: i64) -> Result<Vec<ActivityLog>, DbError> {
        let query = "SELECT * FROM activity_log ORDER BY created_at DESC LIMIT $limit";

        let mut result = self
            .client
            .query(query)
            .bind(("limit", limit))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let activity: Vec<ActivityLog> = result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(activity)
    }

    /// Log an activity
    pub async fn log_activity(
        &self,
        white_label_id: Option<&str>,
        action_type: &str,
        description: &str,
        admin_action: bool,
    ) -> Result<(), DbError> {
        let query = r#"
            CREATE activity_log CONTENT {
                white_label_id: $label_id,
                admin_action: $admin_action,
                action_type: $action_type,
                description: $description,
                created_at: time::now()
            }
        "#;

        self.client
            .query(query)
            .bind(("label_id", white_label_id.map(|id| format!("white_labels:{}", id))))
            .bind(("admin_action", admin_action))
            .bind(("action_type", action_type))
            .bind(("description", description))
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// METRICS QUERIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardMetrics {
    pub status_counts: StatusCounts,
    pub mrr: f64,
    pub failed_payments_count: i64,
    pub total_storage_bytes: i64,
}

impl Database {
    /// Get dashboard metrics
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics, DbError> {
        let status_counts = self.count_labels_by_status().await?;

        // Calculate MRR (Monthly Recurring Revenue)
        let mrr_query = r#"
            SELECT math::sum(subscription.price_per_cycle) AS mrr
            FROM white_labels
            WHERE status = 'white' AND subscription.billing_cycle = 'weekly'
            GROUP ALL
        "#;

        let mut mrr_result = self
            .client
            .query(mrr_query)
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        #[derive(Deserialize)]
        struct MrrResult {
            mrr: Option<f64>,
        }

        let mrr_data: Option<MrrResult> = mrr_result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let weekly_revenue = mrr_data.and_then(|m| m.mrr).unwrap_or(0.0);
        let mrr = weekly_revenue * 4.33; // Convert weekly to monthly

        // Count failed payments
        let failed_query = "SELECT count() AS count FROM payments WHERE status = 'failed' GROUP ALL";

        let mut failed_result = self
            .client
            .query(failed_query)
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        #[derive(Deserialize)]
        struct CountResult {
            count: i64,
        }

        let failed_data: Option<CountResult> = failed_result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let failed_payments_count = failed_data.map(|c| c.count).unwrap_or(0);

        // Calculate total storage
        let storage_query = r#"
            SELECT
                math::sum(storage.images_bytes) +
                math::sum(storage.videos_bytes) +
                math::sum(storage.audio_bytes) +
                math::sum(storage.db_bytes) AS total
            FROM white_labels
            WHERE status IN ['white', 'gray', 'black']
            GROUP ALL
        "#;

        let mut storage_result = self
            .client
            .query(storage_query)
            .await
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        #[derive(Deserialize)]
        struct StorageResult {
            total: Option<i64>,
        }

        let storage_data: Option<StorageResult> = storage_result
            .take(0)
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        let total_storage_bytes = storage_data.and_then(|s| s.total).unwrap_or(0);

        Ok(DashboardMetrics {
            status_counts,
            mrr,
            failed_payments_count,
            total_storage_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        // This test requires a running SurrealDB instance
        let result = Database::new("127.0.0.1:8000", "lillpepe", "main").await;
        assert!(result.is_ok());
    }
}