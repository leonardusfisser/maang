Lillpepe - Setup & Running Instructions
What's Been Built
✅ SurrealDB Schema - Complete database schema with migrations
✅ Database Module - Full CRUD operations for white labels, payments, activity logs
✅ Admin Dashboard - HTMX-based single-page admin interface
✅ Background Jobs - Async task queue with scheduler
✅ Email Templates - Askama-based transactional emails
✅ Legal Documentation - Terms, Privacy Policy, GDPR compliance templates
✅ Main Application - Actix-Web server with all routes configured

Prerequisites

Rust 1.89.0+

bash   rustup update

SurrealDB (running instance)

bash   # Install SurrealDB
curl -sSf https://install.surrealdb.com | sh

# Start SurrealDB
surreal start --log trace --user root --pass root memory

Fish Shell (your preference)

bash   sudo apt install fish  # Debian

Project Structure
maang/
├── apps/
│   └── lillpepe/
│       ├── src/
│       │   └── main.rs                    ✅ Created
│       └── Cargo.toml                     ✅ Created
├── crates/
│   ├── infrastructure/
│   │   ├── db/
│   │   │   └── src/lib.rs                 ✅ Created (full DB module)
│   │   ├── email/
│   │   │   └── src/lib.rs                 ✅ Created (email service)
│   │   └── queue/
│   │       └── src/lib.rs                 ✅ Created (background jobs)
│   ├── application/
│   │   └── web/
│   │       └── src/admin/
│   │           └── routes.rs              ✅ Created (admin routes)
│   └── core/
│       ├── errors/
│       └── config/
├── templates/
│   ├── admin/
│   │   ├── base.html                      ✅ Created (HTMX layout)
│   │   └── partials/
│   │       ├── dashboard.html             ✅ Created
│   │       ├── labels.html                ✅ Created
│   │       ├── label_detail.html          ✅ Created
│   │       ├── label_detail_overview.html ✅ Created
│   │       ├── payments.html              ✅ Created
│   │       └── system.html                ✅ Created
│   └── emails/
│       ├── base.html                      ✅ Created
│       ├── welcome.html                   ✅ Created
│       ├── payment_success.html           ✅ Created
│       └── payment_failed.html            ✅ Created
├── docs/
│   └── legal/
│       ├── TERMS_OF_SERVICE.md            ✅ Created
│       ├── PRIVACY_POLICY.md              ✅ Created
│       └── GDPR_COMPLIANCE.md             ✅ Created
└── Cargo.toml                             ✅ Updated

Step-by-Step Setup
1. Initialize Database Schema
   bash# Start SurrealDB (in separate terminal)
   surreal start --log info --user root --pass root memory

# Import schema
surreal import \
--conn http://localhost:8000 \
--user root \
--pass root \
--ns lillpepe \
--db main \
migrations/001_initial_schema.surql
Create migration file:
bashmkdir -p migrations
# Copy the SQL schema from artifact "SurrealDB Schema Migrations"
# Save as migrations/001_initial_schema.surql
2. Create Template Files
   bash# Create template directories
   mkdir -p templates/admin/partials
   mkdir -p templates/emails

# Copy HTML files from artifacts:
# - "Admin Base Layout - HTMX" → templates/admin/base.html
# - "Admin Partials - HTMX Content" → templates/admin/partials/*.html
# - "Email Templates" sections → templates/emails/*.html
3. Set Up Environment
   bash# Create .env file
   cat > .env << 'EOF'
   HOST=0.0.0.0
   PORT=8080
   DATABASE_URL=127.0.0.1:8000
   DATABASE_NAMESPACE=lillpepe
   DATABASE_NAME=main
   RUST_LOG=info
   EOF
4. Build the Project
   bash# From project root
   cargo build --release

# This compiles:
# - apps/lillpepe (main binary)
# - All crates (db, email, queue, etc.)
5. Run the Application
   bash# Development mode (with hot reload)
   cargo run

# Production mode
./target/release/lillpepe

# Should see:
# INFO Starting Lillpepe v0.1.0
# INFO Connecting to SurrealDB at 127.0.0.1:8000
# INFO Database connected successfully
# INFO Background job system started
# INFO Starting HTTP server on 0.0.0.0:8080
6. Access Admin Dashboard
   Open browser: http://localhost:8080/admin

Testing the Setup
1. Check Health Endpoint
   bashcurl http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","version":"0.1.0"}
2. Create Test White Label (via SurrealDB CLI)
   bashsurreal sql \
   --conn http://localhost:8000 \
   --user root \
   --pass root \
   --ns lillpepe \
   --db main
   sqlCREATE white_labels CONTENT {
   domain: "example.com",
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
   images_bytes: 12582912,
   videos_bytes: 8388608,
   audio_bytes: 0,
   db_bytes: 4194304
   }
   };
3. Verify Dashboard Shows Data

Refresh /admin - should see metrics updated
Click "White Labels" - should see example.com listed
Click on example.com - should see detail page


What Still Needs Implementation
Immediate (Week 1)

Template files - Copy HTML from artifacts to templates/
Static CSS - Extract CSS from base.html to static/admin.css
Migration runner - Script to apply migrations automatically
Admin authentication - Login system for admin access

Near-term (Week 2-3)

Stripe integration - Webhook handlers, subscription management
Email service config - SMTP credentials, email sending
White label provisioning - Binary compilation + deployment
Tab content - Payments history, Activity logs, System databases

Future (Month 2+)

Load testing - Performance benchmarks
Monitoring - Netdata integration
Backup automation - Scheduled database backups
White label customization UI - Color picker, file uploads


Development Workflow
File watching during development
bash# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
Run specific crate tests
bashcargo test -p infrastructure-db
cargo test -p infrastructure-email
cargo test -p infrastructure-queue
Check code quality
bash# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Check without building
cargo check --all

Common Issues & Solutions
SurrealDB Connection Fails
bash# Check if SurrealDB is running
lsof -i :8000

# Restart SurrealDB
pkill surreal
surreal start --log info --user root --pass root memory
Template Not Found Errors
bash# Verify templates exist
ls -la templates/admin/
ls -la templates/admin/partials/

# Check Cargo.toml has askama configured
grep -A 5 "\[build-dependencies\]" Cargo.toml
HTMX Not Loading

Check browser console for 404 on htmx.org CDN
Verify /static route is configured
Check firewall isn't blocking CDN


Production Deployment
Build optimized binary
bashcargo build --release --target x86_64-unknown-linux-gnu

# Binary location:
# ./target/release/lillpepe
Create systemd service
bashsudo nano /etc/systemd/system/lillpepe.service
ini[Unit]
Description=Lillpepe White Label SaaS
After=network.target surrealdb.service

[Service]
Type=simple
User=lillpepe
WorkingDirectory=/opt/lillpepe
Environment="DATABASE_URL=127.0.0.1:8000"
Environment="HOST=0.0.0.0"
Environment="PORT=8080"
ExecStart=/opt/lillpepe/lillpepe
Restart=always

[Install]
WantedBy=multi-user.target
bashsudo systemctl daemon-reload
sudo systemctl enable lillpepe
sudo systemctl start lillpepe
sudo systemctl status lillpepe
Using rsync for deployment
bash# Your preferred method (no Docker/Git Actions)
rsync -avz --progress \
./target/release/lillpepe \
user@mp100:/opt/lillpepe/

rsync -avz --progress \
./templates/ \
user@mp100:/opt/lillpepe/templates/

ssh user@mp100 'sudo systemctl restart lillpepe'

Next Steps

Copy template files from artifacts to templates/ directory
Create migration file from SurrealDB schema artifact
Run database migration to initialize schema
Start the app with cargo run
Access admin at http://localhost:8080/admin
Create test data using SurrealDB CLI
Verify everything works - dashboard loads, tabs switch, data displays


Support & Documentation

Your Preferences: Production-grade, safe, secure, fast, minimal libraries
No Docker/K8s/YAML: Pure Rust binary + rsync deployment
Tools: Netdata (monitoring), rsync (deploy), ssh (access)
Hardware: MP80 (dev/Parrot OS), MP100 (prod/Debian)

You're 90% there. The scaffold is done. Now fill in the remaining pieces!