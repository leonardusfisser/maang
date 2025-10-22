use askama::Template;
use serde::Deserialize;

// PUBLIC TEMPLATES
#[derive(Debug, Template)]
#[template(path = "public_home.html")]
pub struct HomeTemplate {
    pub tenant_name: String,
    pub tagline: String,
    pub background_video: String,
}

#[derive(Debug, Template)]
#[template(path = "public_about.html")]
pub struct AboutTemplate {
    pub tenant_name: String,
    pub description: String,
}

#[derive(Debug, Template)]
#[template(path = "public_contact.html")]
pub struct ContactTemplate {
    pub tenant_name: String,
}

#[derive(Debug, Template)]
#[template(path = "public_products.html")]
pub struct ProductsTemplate {
    pub tenant_name: String,
}

#[derive(Debug, Template)]
#[template(path = "public_services.html")]
pub struct ServicesTemplate {
    pub tenant_name: String,
}

#[derive(Debug, Template)]
#[template(path = "public_media.html")]
pub struct MediaTemplate {
    pub tenant_name: String,
}

#[derive(Debug, Template)]
#[template(path = "public_login.html")]
pub struct LoginTemplate {
    pub tenant_name: String,
    pub csrf_token: String,
    pub error: String,
}

#[derive(Debug, Template)]
#[template(path = "public_signup.html")]
pub struct SignupTemplate {
    pub tenant_name: String,
    pub csrf_token: String,
    pub error: String,
}

// ADMIN TEMPLATES
#[derive(Debug, Template)]
#[template(path = "admin_dashboard.html")]
pub struct DashboardTemplate {
    pub tenant_name: String,
    pub user_email: String,
    pub product_count: usize,
    pub service_count: usize,
}

#[derive(Debug, Template)]
#[template(path = "admin_users.html")]
pub struct UsersTemplate {
    pub tenant_name: String,
    pub user_email: String,
}

// FORM STRUCTS
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SignupForm {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub csrf_token: String,
}