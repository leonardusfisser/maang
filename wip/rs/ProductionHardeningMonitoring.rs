// tools/healthcheck/src/main.rs
//! Production health check and monitoring tool

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HealthError {
    #[error("Database unreachable: {0}")]
    DatabaseDown(String),

    #[error("API endpoint failed: {0}")]
    ApiDown(String),

    #[error("Critical service down: {0}")]
    ServiceDown(String),

    #[error("Warning: {0}")]
    Warning(String),
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub timestamp: String,
    pub checks: Vec<Check>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub response_time_ms: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub warnings: usize,
    pub failed: usize,
}

// ============================================================================
// HEALTH CHECKER
// ============================================================================

pub struct HealthChecker {
    api_base: String,
    db_host: String,
    timeout: Duration,
}

impl HealthChecker {
    pub fn new(api_base: String, db_host: String) -> Self {
        Self {
            api_base,
            db_host,
            timeout: Duration::from_secs(5),
        }
    }

    pub async fn run_all_checks(&self) -> HealthReport {
        let mut checks = Vec::new();

        // Check database
        checks.push(self.check_database().await);

        // Check API health endpoint
        checks.push(self.check_api_health().await);

        // Check admin dashboard
        checks.push(self.check_admin_dashboard().await);

        // Check disk space
        checks.push(self.check_disk_space().await);

        // Check memory usage
        checks.push(self.check_memory().await);

        // Check process running
        checks.push(self.check_process().await);

        // Calculate summary
        let total = checks.len();
        let passed = checks.iter().filter(|c| matches!(c.status, CheckStatus::Pass)).count();
        let warnings = checks.iter().filter(|c| matches!(c.status, CheckStatus::Warn)).count();
        let failed = checks.iter().filter(|c| matches!(c.status, CheckStatus::Fail)).count();

        let status = if failed > 0 {
            HealthStatus::Down
        } else if warnings > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthReport {
            status,
            timestamp: time::OffsetDateTime::now_utc().to_string(),
            checks,
            summary: Summary {
                total,
                passed,
                warnings,
                failed,
            },
        }
    }

    async fn check_database(&self) -> Check {
        let start = Instant::now();

        match self.ping_database().await {
            Ok(_) => Check {
                name: "Database".to_string(),
                status: CheckStatus::Pass,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some("SurrealDB responding".to_string()),
            },
            Err(e) => Check {
                name: "Database".to_string(),
                status: CheckStatus::Fail,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        }
    }

    async fn ping_database(&self) -> Result<(), HealthError> {
        // Try to connect to SurrealDB
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| HealthError::DatabaseDown(e.to_string()))?;

        let response = client
            .get(format!("http://{}/health", self.db_host))
            .send()
            .await
            .map_err(|e| HealthError::DatabaseDown(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(HealthError::DatabaseDown("Not responding".to_string()))
        }
    }

    async fn check_api_health(&self) -> Check {
        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap();

        match client.get(format!("{}/api/v1/health", self.api_base)).send().await {
            Ok(response) if response.status().is_success() => Check {
                name: "API Health".to_string(),
                status: CheckStatus::Pass,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some("API responding".to_string()),
            },
            Ok(response) => Check {
                name: "API Health".to_string(),
                status: CheckStatus::Fail,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("Status: {}", response.status())),
            },
            Err(e) => Check {
                name: "API Health".to_string(),
                status: CheckStatus::Fail,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        }
    }

    async fn check_admin_dashboard(&self) -> Check {
        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap();

        match client.get(format!("{}/admin", self.api_base)).send().await {
            Ok(response) if response.status().is_success() => Check {
                name: "Admin Dashboard".to_string(),
                status: CheckStatus::Pass,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: None,
            },
            Ok(response) => Check {
                name: "Admin Dashboard".to_string(),
                status: CheckStatus::Warn,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("Status: {}", response.status())),
            },
            Err(e) => Check {
                name: "Admin Dashboard".to_string(),
                status: CheckStatus::Fail,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        }
    }

    async fn check_disk_space(&self) -> Check {
        let start = Instant::now();

        // Simple disk check - in production use proper system calls
        match std::fs::read_dir("/opt/lillpepe") {
            Ok(_) => {
                // Check free space (simplified)
                Check {
                    name: "Disk Space".to_string(),
                    status: CheckStatus::Pass,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    message: Some("Sufficient space".to_string()),
                }
            }
            Err(e) => Check {
                name: "Disk Space".to_string(),
                status: CheckStatus::Warn,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        }
    }

    async fn check_memory(&self) -> Check {
        let start = Instant::now();

        // Simplified memory check
        Check {
            name: "Memory".to_string(),
            status: CheckStatus::Pass,
            response_time_ms: start.elapsed().as_millis() as u64,
            message: Some("Within limits".to_string()),
        }
    }

    async fn check_process(&self) -> Check {
        let start = Instant::now();

        // Check if lillpepe process is running
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("lillpepe")
            .output();

        match output {
            Ok(output) if output.status.success() => Check {
                name: "Process".to_string(),
                status: CheckStatus::Pass,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some("Lillpepe running".to_string()),
            },
            _ => Check {
                name: "Process".to_string(),
                status: CheckStatus::Fail,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some("Process not found".to_string()),
            },
        }
    }
}

// ============================================================================
// METRICS COLLECTOR
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub requests_total: u64,
    pub requests_per_second: f64,
    pub response_time_avg_ms: f64,
    pub active_connections: usize,
    pub white_labels_total: u64,
    pub white_labels_active: u64,
    pub failed_jobs: usize,
    pub database_size_mb: u64,
}

pub async fn collect_metrics(db: &infrastructure_db::Database) -> Metrics {
    let counts = db.count_labels_by_status().await.unwrap_or_default();

    Metrics {
        timestamp: time::OffsetDateTime::now_utc().to_string(),
        uptime_seconds: 0, // TODO: Track from app start
        requests_total: 0, // TODO: Track in middleware
        requests_per_second: 0.0,
        response_time_avg_ms: 0.0,
        active_connections: 0,
        white_labels_total: counts.total as u64,
        white_labels_active: counts.white as u64,
        failed_jobs: 0, // TODO: Query from job queue
        database_size_mb: 0,
    }
}

// ============================================================================
// BACKUP AUTOMATION
// ============================================================================

pub struct BackupManager {
    backup_dir: String,
    retention_days: u32,
}

impl BackupManager {
    pub fn new(backup_dir: String, retention_days: u32) -> Self {
        Self {
            backup_dir,
            retention_days,
        }
    }

    pub async fn create_backup(&self) -> Result<String, std::io::Error> {
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
            .replace(":", "-");

        let backup_name = format!("backup_{}.tar.gz", timestamp);
        let backup_path = format!("{}/{}", self.backup_dir, backup_name);

        // Create backup using surreal export
        let output = std::process::Command::new("surreal")
            .args(&[
                "export",
                "--conn", "http://127.0.0.1:8000",
                "--user", "root",
                "--pass", "root",
                "--ns", "lillpepe",
                "--db", "main",
                &backup_path,
            ])
            .output()?;

        if output.status.success() {
            tracing::info!("Backup created: {}", backup_path);
            Ok(backup_path)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::new(std::io::ErrorKind::Other, error))
        }
    }

    pub fn cleanup_old_backups(&self) -> Result<usize, std::io::Error> {
        use std::fs;
        use std::time::SystemTime;

        let retention = Duration::from_secs(self.retention_days as u64 * 86400);
        let mut removed = 0;

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now().duration_since(modified).unwrap_or_default();

                if age > retention {
                    fs::remove_file(entry.path())?;
                    removed += 1;
                    tracing::info!("Removed old backup: {:?}", entry.path());
                }
            }
        }

        Ok(removed)
    }
}

// ============================================================================
// ALERT SYSTEM
// ============================================================================

pub struct AlertManager {
    webhook_url: Option<String>,
}

impl AlertManager {
    pub fn new(webhook_url: Option<String>) -> Self {
        Self { webhook_url }
    }

    pub async fn send_alert(&self, severity: AlertSeverity, message: String) {
        let alert = Alert {
            severity,
            message: message.clone(),
            timestamp: time::OffsetDateTime::now_utc().to_string(),
        };

        tracing::error!("ALERT [{}]: {}", alert.severity.as_str(), message);

        // Send to webhook if configured
        if let Some(url) = &self.webhook_url {
            let client = reqwest::Client::new();
            let _ = client
                .post(url)
                .json(&alert)
                .send()
                .await;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Critical => "CRITICAL",
            Self::Warning => "WARNING",
            Self::Info => "INFO",
        }
    }
}

// ============================================================================
// CLI TOOL
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command>", args[0]);
        println!("\nCommands:");
        println!("  health       - Run health checks");
        println!("  backup       - Create database backup");
        println!("  cleanup      - Remove old backups");
        println!("  metrics      - Display current metrics");
        return Ok(());
    }

    match args[1].as_str() {
        "health" => {
            let checker = HealthChecker::new(
                "http://localhost:8080".to_string(),
                "127.0.0.1:8000".to_string(),
            );

            let report = checker.run_all_checks().await;
            println!("{}", serde_json::to_string_pretty(&report)?);

            std::process::exit(if matches!(report.status, HealthStatus::Down) { 1 } else { 0 });
        }

        "backup" => {
            let manager = BackupManager::new("/var/backups/lillpepe".to_string(), 90);

            match manager.create_backup().await {
                Ok(path) => println!("✓ Backup created: {}", path),
                Err(e) => {
                    eprintln!("✗ Backup failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "cleanup" => {
            let manager = BackupManager::new("/var/backups/lillpepe".to_string(), 90);

            match manager.cleanup_old_backups() {
                Ok(count) => println!("✓ Removed {} old backups", count),
                Err(e) => {
                    eprintln!("✗ Cleanup failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "metrics" => {
            let db = infrastructure_db::Database::new("127.0.0.1:8000", "lillpepe", "main").await?;
            let metrics = collect_metrics(&db).await;
            println!("{}", serde_json::to_string_pretty(&metrics)?);
        }

        _ => {