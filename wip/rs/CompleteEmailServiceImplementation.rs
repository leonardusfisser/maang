// crates/infrastructure/email/src/lib.rs
//! Email service with Askama templates and SMTP sending

use askama::Template;
use lettre::message::{header, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Failed to build email: {0}")]
    BuildError(String),

    #[error("Failed to send email: {0}")]
    SendError(String),

    #[error("Template rendering failed: {0}")]
    TemplateError(String),
}

// ============================================================================
// EMAIL SERVICE
// ============================================================================

pub struct EmailService {
    mailer: SmtpTransport,
    from_address: String,
    from_name: String,
}

impl EmailService {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
        from_name: String,
    ) -> Result<Self, EmailError> {
        let creds = Credentials::new(username, password);

        let mailer = SmtpTransport::relay(&smtp_host)
            .map_err(|e| EmailError::BuildError(e.to_string()))?
            .credentials(creds)
            .port(smtp_port)
            .build();

        Ok(Self {
            mailer,
            from_address,
            from_name,
        })
    }

    pub fn send<T: Template>(
        &self,
        to: &str,
        to_name: Option<&str>,
        subject: &str,
        template: T,
    ) -> Result<(), EmailError> {
        let html_body = template
            .render()
            .map_err(|e| EmailError::TemplateError(e.to_string()))?;

        let from = format!("{} <{}>", self.from_name, self.from_address);
        let to_formatted = if let Some(name) = to_name {
            format!("{} <{}>", name, to)
        } else {
            to.to_string()
        };

        let email = Message::builder()
            .from(from.parse().map_err(|e| EmailError::BuildError(format!("{:?}", e)))?)
            .to(to_formatted.parse().map_err(|e| EmailError::BuildError(format!("{:?}", e)))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(self.html_to_text(&html_body)),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )
            .map_err(|e| EmailError::BuildError(e.to_string()))?;

        self.mailer
            .send(&email)
            .map_err(|e| EmailError::SendError(e.to_string()))?;

        Ok(())
    }

    fn html_to_text(&self, html: &str) -> String {
        // Simple HTML to text conversion
        html.replace("<br>", "\n")
            .replace("</p>", "\n\n")
            .replace("<li>", "• ")
            .replace("</li>", "\n")
    }
}

// ============================================================================
// EMAIL TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "emails/welcome.html")]
pub struct WelcomeEmail {
    pub owner_name: String,
    pub domain: String,
    pub email: String,
    pub temp_password: String,
}

#[derive(Template)]
#[template(path = "emails/payment_success.html")]
pub struct PaymentSuccessEmail {
    pub owner_name: String,
    pub domain: String,
    pub amount: String,
    pub payment_date: String,
    pub invoice_number: String,
    pub invoice_url: String,
    pub next_billing_date: String,
}

#[derive(Template)]
#[template(path = "emails/payment_failed.html")]
pub struct PaymentFailedEmail {
    pub owner_name: String,
    pub domain: String,
    pub amount: String,
    pub attempted_date: String,
    pub grace_period_days: u8,
}

#[derive(Template)]
#[template(path = "emails/subscription_expiring.html")]
pub struct SubscriptionExpiringEmail {
    pub owner_name: String,
    pub domain: String,
    pub days_until: u8,
    pub amount: String,
    pub billing_date: String,
    pub payment_method: String,
}

#[derive(Template)]
#[template(path = "emails/account_suspended.html")]
pub struct AccountSuspendedEmail {
    pub owner_name: String,
    pub domain: String,
    pub suspension_reason: String,
    pub grace_period_days: u8,
    pub resolution_steps: String,
    pub outstanding_amount: String,
}

#[derive(Template)]
#[template(path = "emails/password_reset.html")]
pub struct PasswordResetEmail {
    pub owner_name: String,
    pub domain: String,
    pub reset_link: String,
    pub expires_in_minutes: u32,
}

// ============================================================================
// EMAIL SENDING FUNCTIONS
// ============================================================================

impl EmailService {
    pub fn send_welcome(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        temp_password: &str,
    ) -> Result<(), EmailError> {
        let template = WelcomeEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            email: email.to_string(),
            temp_password: temp_password.to_string(),
        };

        self.send(email, Some(owner_name), "Welcome to Lillpepe!", template)
    }

    pub fn send_payment_success(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        amount: &str,
        invoice_id: &str,
    ) -> Result<(), EmailError> {
        let now = time::OffsetDateTime::now_utc();
        let next_billing = now + time::Duration::days(7);

        let template = PaymentSuccessEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            amount: amount.to_string(),
            payment_date: now.date().to_string(),
            invoice_number: invoice_id.to_string(),
            invoice_url: format!("https://dashboard.stripe.com/invoices/{}", invoice_id),
            next_billing_date: next_billing.date().to_string(),
        };

        self.send(email, Some(owner_name), "Payment Received", template)
    }

    pub fn send_payment_failed(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        amount: &str,
    ) -> Result<(), EmailError> {
        let now = time::OffsetDateTime::now_utc();

        let template = PaymentFailedEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            amount: amount.to_string(),
            attempted_date: now.date().to_string(),
            grace_period_days: 7,
        };

        self.send(email, Some(owner_name), "Payment Failed - Action Required", template)
    }

    pub fn send_subscription_expiring(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        days_until: u8,
        amount: &str,
    ) -> Result<(), EmailError> {
        let now = time::OffsetDateTime::now_utc();
        let billing_date = now + time::Duration::days(days_until as i64);

        let template = SubscriptionExpiringEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            days_until,
            amount: amount.to_string(),
            billing_date: billing_date.date().to_string(),
            payment_method: "Card ending in ••••".to_string(), // TODO: Get from Stripe
        };

        self.send(
            email,
            Some(owner_name),
            &format!("Subscription Renewing in {} Days", days_until),
            template,
        )
    }

    pub fn send_account_suspended(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        outstanding_amount: &str,
    ) -> Result<(), EmailError> {
        let template = AccountSuspendedEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            suspension_reason: "Payment overdue".to_string(),
            grace_period_days: 30,
            resolution_steps: "Update your payment method and clear outstanding balance".to_string(),
            outstanding_amount: outstanding_amount.to_string(),
        };

        self.send(email, Some(owner_name), "Account Suspended", template)
    }

    pub fn send_password_reset(
        &self,
        email: &str,
        owner_name: &str,
        domain: &str,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let reset_link = format!("https://{}/reset-password?token={}", domain, reset_token);

        let template = PasswordResetEmail {
            owner_name: owner_name.to_string(),
            domain: domain.to_string(),
            reset_link,
            expires_in_minutes: 60,
        };

        self.send(email, Some(owner_name), "Password Reset Request", template)
    }
}

// ============================================================================
// BACKGROUND JOB HANDLERS
// ============================================================================

use infrastructure_db::Database;

pub async fn send_welcome_email_job(
    email_service: &EmailService,
    db: &Database,
    domain: &str,
    email: &str,
    password: &str,
) -> Result<(), EmailError> {
    // Get white label to find owner name
    let label = db
        .get_label_by_domain(domain)
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?
        .ok_or_else(|| EmailError::SendError("Label not found".to_string()))?;

    email_service.send_welcome(email, "Owner", domain, password)?;

    // Log activity
    db.log_activity(
        Some(&label.id),
        "welcome_email_sent",
        &format!("Welcome email sent to {}", email),
        false,
    )
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?;

    Ok(())
}

pub async fn send_payment_success_email_job(
    email_service: &EmailService,
    db: &Database,
    domain: &str,
    invoice_id: &str,
) -> Result<(), EmailError> {
    let label = db
        .get_label_by_domain(domain)
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?
        .ok_or_else(|| EmailError::SendError("Label not found".to_string()))?;

    let amount = format!("{:.2}", label.subscription.price_per_cycle);

    // TODO: Get user email from users table
    let user_email = "user@example.com";

    email_service.send_payment_success(user_email, "Owner", domain, &amount, invoice_id)?;

    db.log_activity(
        Some(&label.id),
        "payment_success_email_sent",
        &format!("Payment success email sent to {}", user_email),
        false,
    )
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?;

    Ok(())
}

pub async fn send_payment_failed_email_job(
    email_service: &EmailService,
    db: &Database,
    domain: &str,
    amount: &str,
) -> Result<(), EmailError> {
    let label = db
        .get_label_by_domain(domain)
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?
        .ok_or_else(|| EmailError::SendError("Label not found".to_string()))?;

    // TODO: Get user email from users table
    let user_email = "user@example.com";

    email_service.send_payment_failed(user_email, "Owner", domain, amount)?;

    db.log_activity(
        Some(&label.id),
        "payment_failed_email_sent",
        &format!("Payment failed email sent to {}", user_email),
        false,
    )
        .await
        .map_err(|e| EmailError::SendError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_email_renders() {
        let template = WelcomeEmail {
            owner_name: "Test User".to_string(),
            domain: "test.com".to_string(),
            email: "test@test.com".to_string(),
            temp_password: "temp123".to_string(),
        };

        let html = template.render().unwrap();
        assert!(html.contains("Welcome to Lillpepe"));
        assert!(html.contains("test.com"));
        assert!(html.contains("test@test.com"));
    }

    #[test]
    fn test_payment_success_email_renders() {
        let template = PaymentSuccessEmail {
            owner_name: "Test User".to_string(),
            domain: "test.com".to_string(),
            amount: "10.00".to_string(),
            payment_date: "2025-10-20".to_string(),
            invoice_number: "INV_123".to_string(),
            invoice_url: "https://stripe.com/inv/123".to_string(),
            next_billing_date: "2025-10-27".to_string(),
        };

        let html = template.render().unwrap();
        assert!(html.contains("Payment Received"));
        assert!(html.contains("10.00"));
        assert!(html.contains("test.com"));
    }
}