// ops/provision_tenant/src/main.rs
//! White Label Provisioning System
//! Compiles and deploys isolated white label binaries

use infrastructure_db::{Database, WhiteLabel};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum ProvisionError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

// ============================================================================
// PROVISIONING CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionConfig {
    pub deploy_root: PathBuf,
    pub template_path: PathBuf,
    pub binary_name: String,
    pub surrealdb_host: String,
    pub surrealdb_port: u16,
}

impl Default for ProvisionConfig {
    fn default() -> Self {
        Self {
            deploy_root: PathBuf::from("/opt/lillpepe/tenants"),
            template_path: PathBuf::from("./template"),
            binary_name: "tenant_app".to_string(),
            surrealdb_host: "127.0.0.1".to_string(),
            surrealdb_port: 8000,
        }
    }
}

// ============================================================================
// PROVISIONER
// ============================================================================

pub struct Provisioner {
    config: ProvisionConfig,
    db: Database,
}

impl Provisioner {
    pub fn new(config: ProvisionConfig, db: Database) -> Self {
        Self { config, db }
    }

    /// Provision a new white label
    pub async fn provision(&self, domain: &str, white_label_id: &str) -> Result<(), ProvisionError> {
        info!("Starting provisioning for domain: {}", domain);

        // 1. Verify white label exists in database
        let label = self
            .db
            .get_label_by_id(white_label_id)
            .await
            .map_err(|e| ProvisionError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ProvisionError::ConfigError("White label not found".to_string()))?;

        info!("White label found: {}", label.domain);

        // 2. Create tenant directory structure
        let tenant_dir = self.create_tenant_directory(&label)?;
        info!("Tenant directory created: {:?}", tenant_dir);

        // 3. Generate tenant configuration
        let config = self.generate_tenant_config(&label)?;
        self.write_config(&tenant_dir, &config)?;
        info!("Configuration written");

        // 4. Create isolated SurrealDB namespace for tenant
        self.create_tenant_database(&label).await?;
        info!("Database namespace created");

        // 5. Copy static assets
        self.copy_static_assets(&tenant_dir, &label)?;
        info!("Static assets copied");

        // 6. Generate customized CSS
        self.generate_custom_css(&tenant_dir, &label)?;
        info!("Custom CSS generated");

        // 7. Compile binary (optional - can use shared binary)
        // For now, we'll use a shared binary approach
        let binary_path = self.get_shared_binary_path()?;
        info!("Using shared binary: {:?}", binary_path);

        // 8. Create systemd service (deployment)
        self.create_systemd_service(&label, &tenant_dir, &binary_path)?;
        info!("Systemd service created");

        // 9. Log activity
        self.db
            .log_activity(
                Some(white_label_id),
                "provisioning_completed",
                &format!("White label {} provisioned successfully", domain),
                true,
            )
            .await
            .map_err(|e| ProvisionError::DatabaseError(e.to_string()))?;

        info!("Provisioning completed for {}", domain);
        Ok(())
    }

    /// Create tenant directory structure
    fn create_tenant_directory(&self, label: &WhiteLabel) -> Result<PathBuf, ProvisionError> {
        let tenant_dir = self.config.deploy_root.join(&label.domain);

        std::fs::create_dir_all(&tenant_dir)
            .map_err(|e| ProvisionError::FileError(e.to_string()))?;

        // Create subdirectories
        for subdir in &["static", "media", "config", "logs"] {
            std::fs::create_dir_all(tenant_dir.join(subdir))
                .map_err(|e| ProvisionError::FileError(e.to_string()))?;
        }

        Ok(tenant_dir)
    }

    /// Generate tenant-specific configuration
    fn generate_tenant_config(&self, label: &WhiteLabel) -> Result<TenantConfig, ProvisionError> {
        Ok(TenantConfig {
            domain: label.domain.clone(),
            white_label_id: label.id.clone(),
            database_namespace: format!("tenant_{}", sanitize_namespace(&label.domain)),
            database_name: "main".to_string(),
            port: self.get_available_port()?,
            colors: TenantColors {
                primary: label.customization.primary_color.clone(),
                secondary: label.customization.secondary_color.clone(),
                accent: label.customization.accent_color.clone(),
                background: label.customization.background_color.clone(),
                text: label.customization.text_color.clone(),
            },
        })
    }

    /// Write configuration to file
    fn write_config(&self, tenant_dir: &Path, config: &TenantConfig) -> Result<(), ProvisionError> {
        let config_path = tenant_dir.join("config/tenant.toml");
        let config_str = toml::to_string_pretty(config)
            .map_err(|e| ProvisionError::FileError(e.to_string()))?;

        std::fs::write(&config_path, config_str)
            .map_err(|e| ProvisionError::FileError(e.to_string()))?;

        Ok(())
    }

    /// Create isolated database namespace for tenant
    async fn create_tenant_database(&self, label: &WhiteLabel) -> Result<(), ProvisionError> {
        let namespace = format!("tenant_{}", sanitize_namespace(&label.domain));

        // Execute SurrealDB command to create namespace
        let output = Command::new("surreal")
            .args(&[
                "sql",
                "--conn",
                &format!("{}:{}", self.config.surrealdb_host, self.config.surrealdb_port),
                "--user",
                "root",
                "--pass",
                "root",
                "--ns",
                &namespace,
                "--db",
                "main",
                "--",
                "DEFINE NAMESPACE IF NOT EXISTS tenant; USE NS tenant; DEFINE DATABASE IF NOT EXISTS main;",
            ])
            .output()
            .map_err(|e| ProvisionError::DatabaseError(e.to_string()))?;

        if !output.status.success() {
            return Err(ProvisionError::DatabaseError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        info!("Created database namespace: {}", namespace);
        Ok(())
    }

    /// Copy static assets
    fn copy_static_assets(&self, tenant_dir: &Path, label: &WhiteLabel) -> Result<(), ProvisionError> {
        let source = self.config.template_path.join("static");
        let dest = tenant_dir.join("static");

        // Copy default static files
        copy_dir_all(&source, &dest)
            .map_err(|e| ProvisionError::FileError(e.to_string()))?;

        Ok(())
    }

    /// Generate custom CSS file with tenant colors
    fn generate_custom_css(&self, tenant_dir: &Path, label: &WhiteLabel) -> Result<(), ProvisionError> {
        let css_content = format!(
            r#"/* Auto-generated custom CSS for {} */
:root {{
    --primary-color: {};
    --secondary-color: {};
    --accent-color: {};
    --background-color: {};
    --text-color: {};
}}

body {{
    background-color: var(--background-color);
    color: var(--text-color);
}}

.btn-primary {{
    background-color: var(--primary-color);
}}

.btn-secondary {{
    background-color: var(--secondary-color);
}}

a {{
    color: var(--accent-color);
}}
"#,
            label.domain,
            label.customization.primary_color,
            label.customization.secondary_color,
            label.customization.accent_color,
            label.customization.background_color,
            label.customization.text_color,
        );

        let css_path = tenant_dir.join("static/custom.css");
        std::fs::write(&css_path, css_content)
            .map_err(|e| ProvisionError::FileError(e.to_string()))?;

        Ok(())
    }

    /// Get shared binary path (avoids compiling for each tenant)
    fn get_shared_binary_path(&self) -> Result<PathBuf, ProvisionError> {
        let binary_path = PathBuf::from("/opt/lillpepe/bin").join(&self.config.binary_name);

        if !binary_path.exists() {
            return Err(ProvisionError::ConfigError(format!(
                "Binary not found: {:?}",
                binary_path
            )));
        }

        Ok(binary_path)
    }

    /// Create systemd service for tenant
    fn create_systemd_service(
        &self,
        label: &WhiteLabel,
        tenant_dir: &Path,
        binary_path: &Path,
    ) -> Result<(), ProvisionError> {
        let service_name = format!("lillpepe-{}", sanitize_namespace(&label.domain));
        let service_content = format!(
            r#"[Unit]
Description=Lillpepe White Label - {}
After=network.target surrealdb.service

[Service]
Type=simple
User=lillpepe
WorkingDirectory={}
Environment="TENANT_CONFIG={}/config/tenant.toml"
Environment="RUST_LOG=info"
ExecStart={} --config {}/config/tenant.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"#,
            label.domain,
            tenant_dir.display(),
            tenant_dir.display(),
            binary_path.display(),
            tenant_dir.display(),
        );

        let service_path = PathBuf::from(format!("/etc/systemd/system/{}.service", service_name));

        // Write service file (requires root)
        std::fs::write(&service_path, service_content)
            .map_err(|e| ProvisionError::DeploymentFailed(e.to_string()))?;

        // Reload systemd
        Command::new("systemctl")
            .args(&["daemon-reload"])
            .output()
            .map_err(|e| ProvisionError::DeploymentFailed(e.to_string()))?;

        // Enable and start service
        Command::new("systemctl")
            .args(&["enable", &service_name])
            .output()
            .map_err(|e| ProvisionError::DeploymentFailed(e.to_string()))?;

        Command::new("systemctl")
            .args(&["start", &service_name])
            .output()
            .map_err(|e| ProvisionError::DeploymentFailed(e.to_string()))?;

        info!("Systemd service created and started: {}", service_name);
        Ok(())
    }

    fn get_available_port(&self) -> Result<u16, ProvisionError> {
        // Simple port allocation - in production, track used ports
        // For now, use a deterministic approach based on domain hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.config.deploy_root.hash(&mut hasher);
        let hash = hasher.finish();

        // Port range: 9000-9999
        let port = 9000 + (hash % 1000) as u16;
        Ok(port)
    }
}

// ============================================================================
// DEPROVISIONER
// ============================================================================

impl Provisioner {
    /// Deprovision a white label (cleanup)
    pub async fn deprovision(&self, domain: &str, white_label_id: &str) -> Result<(), ProvisionError> {
        info!("Starting deprovisioning for domain: {}", domain);

        let service_name = format!("lillpepe-{}", sanitize_namespace(domain));

        // 1. Stop systemd service
        let _ = Command::new("systemctl")
            .args(&["stop", &service_name])
            .output();

        // 2. Disable service
        let _ = Command::new("systemctl")
            .args(&["disable", &service_name])
            .output();

        // 3. Remove service file
        let service_path = PathBuf::from(format!("/etc/systemd/system/{}.service", service_name));
        let _ = std::fs::remove_file(&service_path);

        // 4. Remove tenant directory
        let tenant_dir = self.config.deploy_root.join(domain);
        if tenant_dir.exists() {
            std::fs::remove_dir_all(&tenant_dir)
                .map_err(|e| ProvisionError::FileError(e.to_string()))?;
        }

        // 5. Drop database namespace (optional - keep for recovery period)
        // For now, leave the database for 30 days

        // 6. Log activity
        self.db
            .log_activity(
                Some(white_label_id),
                "deprovisioning_completed",
                &format!("White label {} deprovisioned", domain),
                true,
            )
            .await
            .map_err(|e| ProvisionError::DatabaseError(e.to_string()))?;

        info!("Deprovisioning completed for {}", domain);
        Ok(())
    }
}

// ============================================================================
// TENANT CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TenantConfig {
    domain: String,
    white_label_id: String,
    database_namespace: String,
    database_name: String,
    port: u16,
    colors: TenantColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TenantColors {
    primary: String,
    secondary: String,
    accent: String,
    background: String,
    text: String,
}

// ============================================================================
// HELPERS
// ============================================================================

fn sanitize_namespace(domain: &str) -> String {
    domain
        .replace('.', "_")
        .replace('-', "_")
        .to_lowercase()
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

// ============================================================================
// BACKGROUND JOB HANDLER
// ============================================================================

pub async fn provision_white_label_job(
    provisioner: &Provisioner,
    domain: &str,
    white_label_id: &str,
) -> Result<(), ProvisionError> {
    provisioner.provision(domain, white_label_id).await
}

pub async fn delete_white_label_job(
    provisioner: &Provisioner,
    domain: &str,
    white_label_id: &str,
) -> Result<(), ProvisionError> {
    provisioner.deprovision(domain, white_label_id).await
}

// ============================================================================
// CLI TOOL
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <provision|deprovision> <domain>", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let domain = &args[2];

    // Initialize database
    let db = Database::new("127.0.0.1:8000", "lillpepe", "main").await?;

    // Get white label ID
    let label = db
        .get_label_by_domain(domain)
        .await?
        .ok_or("White label not found")?;

    // Create provisioner
    let config = ProvisionConfig::default();
    let provisioner = Provisioner::new(config, db);

    match command.as_str() {
        "provision" => {
            provisioner.provision(domain, &label.id).await?;
            println!("✓ Provisioned {}", domain);
        }
        "deprovision" => {
            provisioner.deprovision(domain, &label.id).await?;
            println!("✓ Deprovisioned {}", domain);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }

    Ok(())
}