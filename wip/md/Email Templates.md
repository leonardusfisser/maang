Email Templates - Lillpepe
Architecture
Template Engine: Askama (same as web templates)
Email Library: lettre for SMTP
Storage: templates/emails/ directory
Variables: Rust structs with serde serialization

Template Structure
templates/emails/
├── base.html           (common layout, header, footer)
├── welcome.html        (new white label signup)
├── payment_success.html
├── payment_failed.html
├── subscription_expiring.html
├── account_suspended.html
└── password_reset.html

1. Base Layout Template
   File: templates/emails/base.html
   html<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            line-height: 1.6;
            color: #111827;
            background-color: #f9fafb;
            margin: 0;
            padding: 0;
        }
        .container {
            max-width: 600px;
            margin: 0 auto;
            background-color: #ffffff;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        .header {
            background-color: #3b82f6;
            color: #ffffff;
            padding: 2rem;
            text-align: center;
        }
        .content {
            padding: 2rem;
        }
        .footer {
            background-color: #f9fafb;
            padding: 1.5rem;
            text-align: center;
            font-size: 0.875rem;
            color: #6b7280;
            border-top: 1px solid #e5e7eb;
        }
        .button {
            display: inline-block;
            background-color: #3b82f6;
            color: #ffffff;
            text-decoration: none;
            padding: 0.75rem 1.5rem;
            border-radius: 6px;
            margin: 1rem 0;
        }
        .alert {
            background-color: #fef2f2;
            border-left: 4px solid #ef4444;
            padding: 1rem;
            margin: 1rem 0;
        }
        .success {
            background-color: #f0fdf4;
            border-left: 4px solid #10b981;
            padding: 1rem;
            margin: 1rem 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Lillpepe</h1>
        </div>
        <div class="content">
            {% block content %}{% endblock %}
        </div>
        <div class="footer">
            <p>© 2025 Lillpepe. All rights reserved.</p>
            <p>
                <a href="https://lillpepe.com/privacy">Privacy Policy</a> | 
                <a href="https://lillpepe.com/terms">Terms of Service</a> | 
                <a href="https://lillpepe.com/contact">Contact Support</a>
            </p>
        </div>
    </div>
</body>
</html>

2. Welcome Email
   File: templates/emails/welcome.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Welcome to Lillpepe! 🎉</h2>

<p>Hi {{ owner_name }},</p>

<p>Your white label website is ready at <strong>{{ domain }}</strong>!</p>

<div class="success">
    <strong>What's Next?</strong>
    <ul>
        <li>Customize your 5 brand colors</li>
        <li>Upload your logo and background media</li>
        <li>Add your first products or services</li>
        <li>Edit your About and Contact pages</li>
    </ul>
</div>

<p style="text-align: center;">
    <a href="https://{{ domain }}/login" class="button">Access Your Dashboard</a>
</p>

<h3>Login Credentials</h3>
<p>
    <strong>Email:</strong> {{ email }}<br>
    <strong>Password:</strong> {{ temp_password }}
</p>

<p><em>Please change your password after first login.</em></p>

<h3>Need Help?</h3>
<p>Check out our <a href="https://lillpepe.com/docs">documentation</a> or reply to this email.</p>

<p>Happy building!<br>The Lillpepe Team</p>
{% endblock %}
Rust struct:
rust#[derive(Template)]
#[template(path = "emails/welcome.html")]
pub struct WelcomeEmail {
    pub owner_name: String,
    pub domain: String,
    pub email: String,
    pub temp_password: String,
}

3. Payment Success
   File: templates/emails/payment_success.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Payment Received ✓</h2>

<p>Hi {{ owner_name }},</p>

<p>Your payment has been processed successfully.</p>

<div class="success">
    <p><strong>Receipt Details</strong></p>
    <ul style="list-style: none; padding: 0;">
        <li>Amount: <strong>€{{ amount }}</strong></li>
        <li>Date: {{ payment_date }}</li>
        <li>Invoice: #{{ invoice_number }}</li>
    </ul>
</div>

<p>Your subscription for <strong>{{ domain }}</strong> is active.</p>

<p style="text-align: center;">
    <a href="{{ invoice_url }}" class="button">View Invoice</a>
</p>

<p>Next billing date: <strong>{{ next_billing_date }}</strong></p>

<p>Questions? Contact us anytime.</p>

<p>Thank you for using Lillpepe!</p>
{% endblock %}
Rust struct:
rust#[derive(Template)]
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

4. Payment Failed
   File: templates/emails/payment_failed.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Payment Failed</h2>

<p>Hi {{ owner_name }},</p>

<div class="alert">
    <p><strong>We couldn't process your payment for {{ domain }}</strong></p>
    <p>Amount due: <strong>€{{ amount }}</strong></p>
    <p>Attempted on: {{ attempted_date }}</p>
</div>

<h3>Why This Happened</h3>
<p>Common reasons:</p>
<ul>
    <li>Insufficient funds</li>
    <li>Expired card</li>
    <li>Card declined by bank</li>
</ul>

<h3>What You Need to Do</h3>
<p>Update your payment method within <strong>{{ grace_period_days }} days</strong> to avoid service interruption.</p>

<p style="text-align: center;">
    <a href="https://{{ domain }}/admin/billing" class="button">Update Payment Method</a>
</p>

<p>Your site will remain active during this grace period.</p>

<p>Need help? Reply to this email.</p>

<p>The Lillpepe Team</p>
{% endblock %}
Rust struct:
rust#[derive(Template)]
#[template(path = "emails/payment_failed.html")]
pub struct PaymentFailedEmail {
    pub owner_name: String,
    pub domain: String,
    pub amount: String,
    pub attempted_date: String,
    pub grace_period_days: u8,
}

5. Subscription Expiring
   File: templates/emails/subscription_expiring.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Subscription Renewing Soon</h2>

<p>Hi {{ owner_name }},</p>

<p>Your Lillpepe subscription for <strong>{{ domain }}</strong> will renew in <strong>{{ days_until }} days</strong>.</p>

<div class="success">
    <p><strong>Renewal Details</strong></p>
    <ul style="list-style: none; padding: 0;">
        <li>Amount: <strong>€{{ amount }}</strong></li>
        <li>Billing date: {{ billing_date }}</li>
        <li>Payment method: {{ payment_method }}</li>
    </ul>
</div>

<p>No action needed - we'll automatically charge your payment method on file.</p>

<p style="text-align: center;">
    <a href="https://{{ domain }}/admin/billing" class="button">Manage Subscription</a>
</p>

<p>Want to make changes? Update your billing details before the renewal date.</p>

<p>Thank you for being with us!</p>
{% endblock %}

6. Account Suspended
   File: templates/emails/account_suspended.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Account Suspended</h2>

<p>Hi {{ owner_name }},</p>

<div class="alert">
    <p><strong>Your account for {{ domain }} has been suspended.</strong></p>
    <p>Reason: {{ suspension_reason }}</p>
</div>

<h3>What This Means</h3>
<ul>
    <li>Your website is currently offline</li>
    <li>Your data is preserved</li>
    <li>You have {{ grace_period_days }} days to resolve this</li>
</ul>

<h3>How to Restore Access</h3>
<p>{{ resolution_steps }}</p>

<p style="text-align: center;">
    <a href="https://{{ domain }}/admin/billing" class="button">Restore Account</a>
</p>

<p>Outstanding balance: <strong>€{{ outstanding_amount }}</strong></p>

<p>After {{ grace_period_days }} days, your account and data will be permanently deleted.</p>

<p>Need assistance? Reply to this email immediately.</p>

<p>The Lillpepe Team</p>
{% endblock %}

7. Password Reset
   File: templates/emails/password_reset.html
   html{% extends "emails/base.html" %}

{% block content %}
<h2>Password Reset Request</h2>

<p>Hi {{ owner_name }},</p>

<p>We received a request to reset your password for <strong>{{ domain }}</strong>.</p>

<p style="text-align: center;">
    <a href="{{ reset_link }}" class="button">Reset Password</a>
</p>

<p>This link will expire in <strong>{{ expires_in_minutes }} minutes</strong>.</p>

<div class="alert">
    <p><strong>Didn't request this?</strong></p>
    <p>Ignore this email. Your password won't be changed.</p>
</div>

<p>For security, never share this email or link with anyone.</p>

<p>The Lillpepe Team</p>
{% endblock %}

Implementation Code
Email Service (Rust)
File: crates/infrastructure/email/src/lib.rs
rustuse askama::Template;
use lettre::{
transport::smtp::authentication::Credentials,
Message, SmtpTransport, Transport,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
#[error("Failed to build email: {0}")]
BuildError(String),

    #[error("Failed to send email: {0}")]
    SendError(String),
}

pub struct EmailService {
mailer: SmtpTransport,
from_address: String,
}

impl EmailService {
pub fn new(smtp_host: String, smtp_port: u16, username: String, password: String, from: String) -> Result<Self, EmailError> {
let creds = Credentials::new(username, password);

        let mailer = SmtpTransport::relay(&smtp_host)
            .map_err(|e| EmailError::BuildError(e.to_string()))?
            .credentials(creds)
            .port(smtp_port)
            .build();
            
        Ok(Self {
            mailer,
            from_address: from,
        })
    }
    
    pub fn send<T: Template>(&self, to: &str, subject: &str, template: T) -> Result<(), EmailError> {
        let html_body = template
            .render()
            .map_err(|e| EmailError::BuildError(e.to_string()))?;
            
        let email = Message::builder()
            .from(self.from_address.parse().map_err(|e| EmailError::BuildError(format!("{}", e)))?)
            .to(to.parse().map_err(|e| EmailError::BuildError(format!("{}", e)))?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| EmailError::BuildError(e.to_string()))?;
            
        self.mailer
            .send(&email)
            .map_err(|e| EmailError::SendError(e.to_string()))?;
            
        Ok(())
    }
}
Usage Example
rustuse email_templates::WelcomeEmail;

let template = WelcomeEmail {
owner_name: "John Doe".to_string(),
domain: "example.com".to_string(),
email: "john@example.com".to_string(),
temp_password: "temp123".to_string(),
};

email_service.send(
"john@example.com",
"Welcome to Lillpepe!",
template,
)?;

Configuration
Environment variables (.env):
SMTP_HOST=smtp.eu.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@lillpepe.com
SMTP_PASSWORD=your_password_here
EMAIL_FROM=noreply@lillpepe.com

Testing
rust#[cfg(test)]
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
    }
}

Email Triggers
EventTemplateSent WhenWhite label signupwelcome.htmlAccount createdPayment successpayment_success.htmlStripe invoice.paid webhookPayment failedpayment_failed.htmlStripe invoice.payment_failed webhookRenewal remindersubscription_expiring.html3 days before billing dateSuspensionaccount_suspended.htmlPayment overdue > grace periodPassword resetpassword_reset.htmlUser requests reset

Production Checklist

SMTP credentials in secure config (not git)
SPF/DKIM records configured for domain
Unsubscribe link in footer (EU/CAN-SPAM compliance)
Rate limiting on email sending
Bounce/complaint handling
Email queue for reliability (use background jobs)
Test all templates before launch
