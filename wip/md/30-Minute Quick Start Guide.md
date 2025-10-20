# 🚀 30-Minute Quick Start Guide

Get Lillpepe running from zero to admin dashboard in 30 minutes.

---

## ⏱️ Minute 0-5: Prerequisites

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install SurrealDB
curl -sSf https://install.surrealdb.com | sh

# Verify installations
rustc --version  # Should be 1.79+
surreal version  # Should show version
```

---

## ⏱️ Minute 5-10: Create Project Structure

```bash
# Create project directory
mkdir -p ~/maang
cd ~/maang

# Create all directories
mkdir -p apps/lillpepe/src
mkdir -p crates/infrastructure/{db,email,payment,queue,session}/src
mkdir -p crates/application/{web,api}/src
mkdir -p crates/domain/users/src
mkdir -p ops/provision_tenant/src
mkdir -p templates/{admin/partials,tenant,emails}
mkdir -p migrations
mkdir -p docs/legal
mkdir -p static
```

---

## ⏱️ Minute 10-15: Copy Core Files

### 1. Root Cargo.toml

Copy from artifact: **"Cargo.toml - Lillpepe App"** → `Cargo.toml`

### 2. Main Application

Copy from artifact: **"Lillpepe Main App"** → `apps/lillpepe/src/main.rs`

### 3. Database Module

Copy from artifact: **"SurrealDB Connection Module"** → `crates/infrastructure/db/src/lib.rs`

### 4. Create Minimal Cargo.toml Files

```bash
# Each crate needs a Cargo.toml
# Use the structure from "Cargo.toml - Lillpepe App" artifact

# Quick minimal versions:

# crates/infrastructure/db/Cargo.toml
cat > crates/infrastructure/db/Cargo.toml << 'EOF'
[package]
name = "infrastructure-db"
version = "0.1.0"
edition = "2021"

[dependencies]
surrealdb = "2.5.2"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.48", features = ["macros", "rt-multi-thread"] }
thiserror = "2.0"
time = { version = "0.3", features = ["formatting", "macros"] }
EOF

# Repeat for other crates (or copy from detailed artifact)
```

---

## ⏱️ Minute 15-20: Database Setup

### 1. Start SurrealDB

```bash
# Terminal 1 - keep this running
surreal start --log info --user root --pass root memory
```

### 2. Create Migration File

Copy from artifact: **"SurrealDB Schema Migrations"** → `migrations/001_initial_schema.surql`

### 3. Run Migration

```bash
# Terminal 2
surreal import \
  --conn http://localhost:8000 \
  --user root \
  --pass root \
  --ns lillpepe \
  --db main \
  migrations/001_initial_schema.surql
```

---

## ⏱️ Minute 20-25: Templates

### 1. Admin Base Layout

Copy from artifact: **"Admin Base Layout - HTMX"** → `templates/admin/base.html`

### 2. Admin Partials

Copy from artifact: **"Admin Partials - HTMX Content"** → `templates/admin/partials/`

Split the artifact into separate files:
- dashboard.html
- labels.html
- label_detail.html
- label_detail_overview.html
- payments.html
- system.html

### 3. Admin Login

Copy from artifact: **"Admin Login Template"** → `templates/admin/login.html`

---

## ⏱️ Minute 25-28: Configuration

### 1. Create .env

```bash
cat > .env << 'EOF'
HOST=0.0.0.0
PORT=8080
DATABASE_URL=127.0.0.1:8000
DATABASE_NAMESPACE=lillpepe
DATABASE_NAME=main
RUST_LOG=info

# Stripe (test mode for now)
STRIPE_API_KEY=sk_test_your_key_here
STRIPE_WEBHOOK_SECRET=whsec_your_secret_here

# Email (optional for now)
SMTP_HOST=smtp.eu.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@yourdomain.com
SMTP_PASSWORD=your_password
EMAIL_FROM=noreply@yourdomain.com
EMAIL_FROM_NAME=Lillpepe
EOF
```

### 2. Create Placeholder Static File

```bash
echo "/* Custom CSS will be generated per tenant */" > static/custom.css
```

---

## ⏱️ Minute 28-30: Build and Run

### 1. First Build (might take 2-3 minutes)

```bash
cargo build --release
```

If you see errors about missing crates, add minimal lib.rs files:

```bash
# For any missing crate:
echo "// Placeholder" > crates/path/to/crate/src/lib.rs
```

### 2. Run Application

```bash
cargo run --release
```

### 3. Open Admin Dashboard

```
http://localhost:8080/admin
```

You should see the admin dashboard!

---

## 🎉 Success! What Now?

### Create Your First Admin User

```bash
# Connect to SurrealDB
surreal sql --conn http://localhost:8000 --user root --pass root --ns lillpepe --db main

# Run this query:
CREATE admin_users CONTENT {
    email: "admin@test.com",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$VGVzdFNhbHQAAAAAAAAA$YourHashHere",
    is_active: true,
    created_at: time::now()
};
```

**Note:** For password hash, use online Argon2 generator or add password from code.

### Create Test White Label

```sql
CREATE white_labels CONTENT {
    domain: "test.localhost",
    status: "white",
    created_at: time::now(),
    updated_at: time::now(),
    subscription: {
        plan_type: "basic",
        price_per_cycle: 10.00,
        billing_cycle: "weekly",
        payment_failed_count: 0
    },
    customization: {
        primary_color: "#3b82f6",
        secondary_color: "#10b981",
        accent_color: "#f59e0b",
        background_color: "#ffffff",
        text_color: "#111827"
    },
    storage: {
        images_bytes: 0,
        videos_bytes: 0,
        audio_bytes: 0,
        db_bytes: 0
    }
};
```

### Test Admin Dashboard

1. Refresh http://localhost:8080/admin
2. Should see 1 white label
3. Click on it → see details
4. All tabs should work (HTMX navigation)

---

## 🔧 If Something Goes Wrong

### Build Errors

```bash
# Missing crate? Add minimal version:
cargo add thiserror --package crate-name

# Or create placeholder:
mkdir -p crates/path/src
echo "// TODO" > crates/path/src/lib.rs
```

### Database Errors

```bash
# Restart SurrealDB
pkill surreal
surreal start --log info --user root --pass root memory

# Re-run migration
surreal import ...
```

### Template Errors

```bash
# Verify template paths match Askama annotations
# Example: #[template(path = "admin/base.html")]
# Must exist at: templates/admin/base.html
```

---

## 📝 Minimal Working Version

If you just want to see **something** working in 10 minutes:

### 1. Minimal main.rs

```rust
use actix_web::{web, App, HttpServer, HttpResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().body("<h1>Lillpepe is Running!</h1>")
            }))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

### 2. Minimal Cargo.toml

```toml
[package]
name = "lillpepe"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.11.0"
tokio = { version = "1.48.0", features = ["macros", "rt-multi-thread"] }
```

### 3. Run

```bash
cargo run
# Open http://localhost:8080
```

---

## 🎓 Next Steps After Quick Start

1. **Copy remaining artifacts** - Admin routes, email service, etc.
2. **Add authentication** - Copy session management code
3. **Test Stripe webhooks** - Use Stripe CLI
4. **Deploy to MP100** - Use deploy.fish script
5. **Polish UI** - Customize colors, add features

---

## 📞 Quick Reference

### Start Services
```bash
# Terminal 1: SurrealDB
surreal start --user root --pass root memory

# Terminal 2: Application
cargo run
```

### Useful Commands
```bash
# Check what's running
lsof -i :8080  # Lillpepe
lsof -i :8000  # SurrealDB

# View logs
cargo run 2>&1 | grep -i error

# Database shell
surreal sql --conn http://localhost:8000 --user root --pass root --ns lillpepe --db main
```

### URLs
- Admin: http://localhost:8080/admin
- API Health: http://localhost:8080/api/v1/health
- White Label: http://localhost:8080 (when domain configured)

---

**🚀 You should now have Lillpepe running locally!**

**Total time: 30 minutes (or less with practice)**

**Next: Copy all artifacts and build out the complete system.**