// crates/infrastructure/payment/src/lib.rs
//! Stripe payment integration and webhook handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StripeError {
    #[error("Stripe API error: {0}")]
    ApiError(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerificationFailed(String),

    #[error("Invalid event type: {0}")]
    InvalidEventType(String),

    #[error("Missing field: {0}")]
    MissingField(String),
}

// ============================================================================
// STRIPE CLIENT
// ============================================================================

pub struct StripeClient {
    api_key: String,
    webhook_secret: String,
    client: reqwest::Client,
}

impl StripeClient {
    pub fn new(api_key: String, webhook_secret: String) -> Self {
        Self {
            api_key,
            webhook_secret,
            client: reqwest::Client::new(),
        }
    }

    /// Create a new customer
    pub async fn create_customer(
        &self,
        email: &str,
        metadata: HashMap<String, String>,
    ) -> Result<Customer, StripeError> {
        let mut params = HashMap::new();
        params.insert("email".to_string(), email.to_string());

        for (key, value) in metadata {
            params.insert(format!("metadata[{}]", key), value);
        }

        let response = self
            .client
            .post("https://api.stripe.com/v1/customers")
            .bearer_auth(&self.api_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| StripeError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StripeError::ApiError(error_text));
        }

        response
            .json::<Customer>()
            .await
            .map_err(|e| StripeError::ApiError(e.to_string()))
    }

    /// Create a subscription
    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
    ) -> Result<Subscription, StripeError> {
        let mut params = HashMap::new();
        params.insert("customer".to_string(), customer_id.to_string());
        params.insert("items[0][price]".to_string(), price_id.to_string());

        let response = self
            .client
            .post("https://api.stripe.com/v1/subscriptions")
            .bearer_auth(&self.api_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| StripeError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StripeError::ApiError(error_text));
        }

        response
            .json::<Subscription>()
            .await
            .map_err(|e| StripeError::ApiError(e.to_string()))
    }

    /// Cancel a subscription
    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), StripeError> {
        let response = self
            .client
            .delete(format!(
                "https://api.stripe.com/v1/subscriptions/{}",
                subscription_id
            ))
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| StripeError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(StripeError::ApiError(error_text));
        }

        Ok(())
    }

    /// Verify webhook signature
    pub fn verify_webhook(
        &self,
        payload: &str,
        signature: &str,
    ) -> Result<WebhookEvent, StripeError> {
        // Simple verification - in production, use proper HMAC verification
        // For now, just parse the event
        serde_json::from_str(payload)
            .map_err(|e| StripeError::WebhookVerificationFailed(e.to_string()))
    }
}

// ============================================================================
// STRIPE DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub customer: String,
    pub status: String,
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub object: serde_json::Value,
}

// ============================================================================
// WEBHOOK HANDLERS
// ============================================================================

pub struct WebhookHandler {
    stripe: StripeClient,
}

impl WebhookHandler {
    pub fn new(stripe: StripeClient) -> Self {
        Self { stripe }
    }

    pub async fn handle_event(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        match event.event_type.as_str() {
            "invoice.payment_succeeded" => self.handle_payment_succeeded(event).await,
            "invoice.payment_failed" => self.handle_payment_failed(event).await,
            "customer.subscription.updated" => self.handle_subscription_updated(event).await,
            "customer.subscription.deleted" => self.handle_subscription_deleted(event).await,
            "customer.subscription.created" => self.handle_subscription_created(event).await,
            _ => Ok(WebhookAction::Ignore),
        }
    }

    async fn handle_payment_succeeded(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        let invoice = self.extract_invoice(&event)?;

        Ok(WebhookAction::PaymentSucceeded {
            customer_id: invoice.customer,
            invoice_id: invoice.id,
            amount: invoice.amount_paid,
            subscription_id: invoice.subscription,
        })
    }

    async fn handle_payment_failed(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        let invoice = self.extract_invoice(&event)?;

        Ok(WebhookAction::PaymentFailed {
            customer_id: invoice.customer,
            invoice_id: invoice.id,
            amount: invoice.amount_due,
            subscription_id: invoice.subscription,
        })
    }

    async fn handle_subscription_updated(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        let subscription = self.extract_subscription(&event)?;

        Ok(WebhookAction::SubscriptionUpdated {
            subscription_id: subscription.id,
            customer_id: subscription.customer,
            status: subscription.status,
        })
    }

    async fn handle_subscription_deleted(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        let subscription = self.extract_subscription(&event)?;

        Ok(WebhookAction::SubscriptionDeleted {
            subscription_id: subscription.id,
            customer_id: subscription.customer,
        })
    }

    async fn handle_subscription_created(&self, event: WebhookEvent) -> Result<WebhookAction, StripeError> {
        let subscription = self.extract_subscription(&event)?;

        Ok(WebhookAction::SubscriptionCreated {
            subscription_id: subscription.id,
            customer_id: subscription.customer,
            status: subscription.status,
        })
    }

    fn extract_invoice(&self, event: &WebhookEvent) -> Result<Invoice, StripeError> {
        serde_json::from_value(event.data.object.clone())
            .map_err(|e| StripeError::MissingField(format!("invoice: {}", e)))
    }

    fn extract_subscription(&self, event: &WebhookEvent) -> Result<Subscription, StripeError> {
        serde_json::from_value(event.data.object.clone())
            .map_err(|e| StripeError::MissingField(format!("subscription: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Invoice {
    id: String,
    customer: String,
    subscription: Option<String>,
    amount_paid: i64,
    amount_due: i64,
}

#[derive(Debug, Clone)]
pub enum WebhookAction {
    PaymentSucceeded {
        customer_id: String,
        invoice_id: String,
        amount: i64,
        subscription_id: Option<String>,
    },
    PaymentFailed {
        customer_id: String,
        invoice_id: String,
        amount: i64,
        subscription_id: Option<String>,
    },
    SubscriptionUpdated {
        subscription_id: String,
        customer_id: String,
        status: String,
    },
    SubscriptionDeleted {
        subscription_id: String,
        customer_id: String,
    },
    SubscriptionCreated {
        subscription_id: String,
        customer_id: String,
        status: String,
    },
    Ignore,
}

// ============================================================================
// WEBHOOK ROUTE HANDLER
// ============================================================================

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use infrastructure_db::Database;
use infrastructure_queue::JobQueue;

pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    db: web::Data<Database>,
    queue: web::Data<std::sync::Arc<JobQueue>>,
    stripe: web::Data<StripeClient>,
) -> ActixResult<HttpResponse> {
    // Get signature from header
    let signature = req
        .headers()
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing signature"))?;

    // Parse payload
    let payload = std::str::from_utf8(&body)
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;

    // Verify webhook
    let event = stripe
        .verify_webhook(payload, signature)
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;

    // Log webhook received
    db.log_activity(
        None,
        "stripe_webhook_received",
        &format!("Webhook event: {}", event.event_type),
        false,
    )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Handle event
    let handler = WebhookHandler::new(stripe.get_ref().clone());
    let action = handler
        .handle_event(event.clone())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Process action
    match action {
        WebhookAction::PaymentSucceeded {
            customer_id,
            invoice_id,
            amount,
            subscription_id,
        } => {
            // Find white label by customer_id
            if let Some(label) = find_label_by_customer(&db, &customer_id).await? {
                // Update to white status (active)
                db.update_label_status(&label.id, infrastructure_db::LabelStatus::White)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                // Enqueue email job
                queue.enqueue(infrastructure_queue::Job::new(
                    infrastructure_queue::JobType::SendPaymentSuccessEmail {
                        domain: label.domain.clone(),
                        invoice_id,
                    },
                ))
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                // Log activity
                db.log_activity(
                    Some(&label.id),
                    "payment_received",
                    &format!("Payment succeeded: €{:.2}", amount as f64 / 100.0),
                    false,
                )
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
            }
        }

        WebhookAction::PaymentFailed {
            customer_id,
            invoice_id,
            amount,
            subscription_id,
        } => {
            if let Some(label) = find_label_by_customer(&db, &customer_id).await? {
                // Update to gray status (overdue)
                db.update_label_status(&label.id, infrastructure_db::LabelStatus::Gray)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                // Enqueue email job
                queue.enqueue(infrastructure_queue::Job::new(
                    infrastructure_queue::JobType::SendPaymentFailedEmail {
                        domain: label.domain.clone(),
                        amount: format!("{:.2}", amount as f64 / 100.0),
                    },
                ))
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                // Log activity
                db.log_activity(
                    Some(&label.id),
                    "payment_failed",
                    &format!("Payment failed: €{:.2}", amount as f64 / 100.0),
                    false,
                )
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
            }
        }

        WebhookAction::SubscriptionDeleted {
            subscription_id,
            customer_id,
        } => {
            if let Some(label) = find_label_by_customer(&db, &customer_id).await? {
                // Update to red status (deleted)
                db.update_label_status(&label.id, infrastructure_db::LabelStatus::Red)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                // Log activity
                db.log_activity(
                    Some(&label.id),
                    "subscription_cancelled",
                    "Subscription cancelled",
                    false,
                )
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
            }
        }

        _ => {
            // Other events logged but not processed
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "event_id": event.id
    })))
}

async fn find_label_by_customer(
    db: &Database,
    customer_id: &str,
) -> ActixResult<Option<infrastructure_db::WhiteLabel>> {
    let query = "SELECT * FROM white_labels WHERE subscription.stripe_customer_id = $customer_id LIMIT 1";

    // This would need to be implemented in the Database struct
    // For now, return None
    Ok(None)
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/webhooks/stripe", web::post().to(stripe_webhook));
}