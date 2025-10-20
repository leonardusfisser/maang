// tools/admin-cli/src/main.rs
//! Command-line admin tool for Lillpepe management

use clap::{Parser, Subcommand};
use infrastructure_db::{Database, LabelStatus};
use infrastructure_session::{create_initial_admin, PasswordHasher};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "lillpepe-admin")]
#[command(about = "Lillpepe Admin CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create initial admin user
    CreateAdmin {
        #[arg(short, long)]
        email: String,

        #[arg(short, long)]
        password: String,
    },

    /// List all white labels
    ListLabels {
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Create test white label
    CreateTestLabel {
        #[arg(short, long)]
        domain: String,
    },

    /// Seed database with sample data
    Seed {
        #[arg(short, long, default_value = "10")]
        count: usize,
    },

    /// Show statistics
    Stats,

    /// Change white label status
    UpdateStatus {
        #[arg(short, long)]
        domain: String,

        #[arg(short, long)]
        status: String,
    },

    /// Reset admin password
    ResetPassword {
        #[arg(short, long)]
        email: String,

        #[arg(short, long)]
        new_password: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Connect to database
    let db = Database::new("127.0.0.1:8000", "lillpepe", "main").await?;

    match cli.command {
        Commands::CreateAdmin { email, password } => {
            create_initial_admin(&db, &email, &password).await?;
            println!("✓ Admin user created: {}", email);
        }

        Commands::ListLabels { status } => {
            let labels = if let Some(status) = status {
                db.get_labels_by_status(&status).await?
            } else {
                db.get_all_labels().await?
            };

            println!("\n{:<30} {:<10} {:<20} {:<10}", "Domain", "Status", "Created", "Storage");
            println!("{}", "-".repeat(70));

            for label in labels {
                let storage = label.storage.images_bytes
                    + label.storage.videos_bytes
                    + label.storage.db_bytes;
                let storage_mb = storage / 1_048_576;

                println!(
                    "{:<30} {:<10} {:<20} {} MB",
                    label.domain,
                    format!("{:?}", label.status),
                    label.created_at.split('T').next().unwrap_or(""),
                    storage_mb
                );
            }

            println!();
        }

        Commands::CreateTestLabel { domain } => {
            create_test_label(&db, &domain).await?;
            println!("✓ Test white label created: {}", domain);
        }

        Commands::Seed { count } => {
            seed_database(&db, count).await?;
            println!("✓ Seeded {} white labels with sample data", count);
        }

        Commands::Stats => {
            show_statistics(&db).await?;
        }

        Commands::UpdateStatus { domain, status } => {
            let label = db
                .get_label_by_domain(&domain)
                .await?
                .ok_or("White label not found")?;

            let new_status = match status.as_str() {
                "white" => LabelStatus::White,
                "gray" => LabelStatus::Gray,
                "black" => LabelStatus::Black,
                "red" => LabelStatus::Red,
                _ => return Err("Invalid status. Use: white, gray, black, red".into()),
            };

            db.update_label_status(&label.id, new_status).await?;
            println!("✓ Updated {} to status: {}", domain, status);
        }

        Commands::ResetPassword { email, new_password } => {
            let hash = PasswordHasher::hash(&new_password)?;

            let query = "UPDATE admin_users SET password_hash = $hash WHERE email = $email";
            db.client()
                .query(query)
                .bind(("email", email.clone()))
                .bind(("hash", hash))
                .await?;

            println!("✓ Password reset for: {}", email);
        }
    }

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn create_test_label(db: &Database, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    let query = r#"
        CREATE white_labels CONTENT {
            domain: $domain,
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
        }
    "#;

    db.client()
        .query(query)
        .bind(("domain", domain))
        .await?;

    Ok(())
}

async fn seed_database(db: &Database, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    use rand::Rng;

    let statuses = ["white", "gray", "black"];
    let colors = [
        ("#3b82f6", "#10b981", "#f59e0b"),
        ("#8b5cf6", "#ec4899", "#f97316"),
        ("#06b6d4", "#14b8a6", "#84cc16"),
    ];

    for i in 0..count {
        let mut rng = rand::thread_rng();
        let status = statuses[rng.gen_range(0..statuses.len())];
        let (primary, secondary, accent) = colors[rng.gen_range(0..colors.len())];

        let domain = format!("test-{}.example.com", i + 1);

        let query = r#"
            CREATE white_labels CONTENT {
                domain: $domain,
                status: $status,
                created_at: time::now(),
                updated_at: time::now(),
                subscription: {
                    stripe_customer_id: $customer_id,
                    plan_type: "basic",
                    price_per_cycle: 10.00,
                    billing_cycle: "weekly",
                    payment_failed_count: $failed_count
                },
                customization: {
                    primary_color: $primary,
                    secondary_color: $secondary,
                    accent_color: $accent,
                    background_color: "#ffffff",
                    text_color: "#111827"
                },
                storage: {
                    images_bytes: $images,
                    videos_bytes: $videos,
                    audio_bytes: 0,
                    db_bytes: $db
                }
            }
        "#;

        db.client()
            .query(query)
            .bind(("domain", domain))
            .bind(("status", status))
            .bind(("customer_id", format!("cus_test_{}", i + 1)))
            .bind(("failed_count", if status == "gray" { rng.gen_range(1..3) } else { 0 }))
            .bind(("primary", primary))
            .bind(("secondary", secondary))
            .bind(("accent", accent))
            .bind(("images", rng.gen_range(0..50_000_000) as i64))
            .bind(("videos", rng.gen_range(0..100_000_000) as i64))
            .bind(("db", rng.gen_range(0..10_000_000) as i64))
            .await?;

        // Add some sample products
        seed_products(db, &format!("test-{}.example.com", i + 1), rng.gen_range(3..8)).await?;
    }

    Ok(())
}

async fn seed_products(db: &Database, domain: &str, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let label = db.get_label_by_domain(domain).await?.ok_or("Label not found")?;

    let products = [
        ("Premium Widget", "High-quality widget for professionals", 99.99),
        ("Basic Package", "Starter package for beginners", 29.99),
        ("Pro Bundle", "Everything you need", 199.99),
        ("Enterprise License", "Unlimited usage", 499.99),
        ("Student Edition", "Special pricing for students", 19.99),
        ("Lifetime Access", "One-time payment", 999.99),
        ("Monthly Plan", "Flexible monthly subscription", 9.99),
        ("Annual Plan", "Save with annual billing", 99.00),
    ];

    for i in 0..count.min(products.len()) {
        let (title, desc, price) = products[i];

        let query = r#"
            CREATE products CONTENT {
                white_label_id: $white_label_id,
                title: $title,
                description: $description,
                price: $price,
                currency: "EUR",
                is_published: true,
                created_at: time::now(),
                updated_at: time::now()
            }
        "#;

        db.client()
            .query(query)
            .bind(("white_label_id", format!("white_labels:{}", label.id)))
            .bind(("title", title))
            .bind(("description", desc))
            .bind(("price", price))
            .await?;
    }

    Ok(())
}

async fn show_statistics(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let counts = db.count_labels_by_status().await?;
    let metrics = db.get_dashboard_metrics().await?;

    println!("\n╔════════════════════════════════════════╗");
    println!("║        LILLPEPE STATISTICS             ║");
    println!("╚════════════════════════════════════════╝");
    println!();

    println!("White Labels:");
    println!("  ✓ Active (White):     {}", counts.white);
    println!("  ⚠ Overdue (Gray):     {}", counts.gray);
    println!("  ✗ Defaulted (Black):  {}", counts.black);
    println!("  ⊗ Deleted (Red):      {}", counts.red);
    println!("  ━━━━━━━━━━━━━━━━━━━━");
    println!("  Total:                {}", counts.total);
    println!();

    println!("Revenue:");
    println!("  MRR:                  €{:.2}", metrics.mrr);
    println!("  Failed Payments:      {}", metrics.failed_payments_count);
    println!();

    println!("Storage:");
    let storage_gb = metrics.total_storage_bytes as f64 / 1_073_741_824.0;
    println!("  Total Used:           {:.2} GB", storage_gb);
    println!("  Capacity:             500 GB");
    println!("  Usage:                {:.1}%", (storage_gb / 500.0) * 100.0);
    println!();

    Ok(())
}

// ============================================================================
// CARGO.TOML for this tool
// ============================================================================

/*
[package]
name = "admin-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "lillpepe-admin"
path = "src/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.48", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
rand = "0.8"

# Internal crates
infrastructure-db = { path = "../../crates/infrastructure/db" }
infrastructure-session = { path = "../../crates/infrastructure/session" }
*/