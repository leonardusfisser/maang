/// # 🧾 MAANG Backup Utility
///
/// This binary performs a **timestamped rsync backup** of the local `~/maangframe` directory.
/// Designed for production use — safe, observable, and minimal.
///
/// ## ✅ Features
/// - Panic-free (typed `BackupError` enum)
/// - `rsync`-based, fast incremental backups
/// - UTC timestamped destination folders
/// - Excludes build output (`target`, `*.d`)
/// - Logs via `tracing` (INFO level)
///
/// ## 🧩 Directory Layout
/// ```text
/// ~/maangframe/DevBackups/maang_backups/maang_YYYY-MM-DD_HH-MM/
/// ```
///
/// ## ⚙️ Dependencies
/// ```toml
/// [dependencies]
/// thiserror = "1.0"
/// tracing = "0.1"
/// tracing-subscriber = "0.3"
/// time = { version = "0.3", features = ["macros"] }
/// dirs = "5.0"
/// ```
///
/// ## 🧠 Usage
/// ```fish
/// cargo build --release
/// ./target/release/backup_tenant
/// ```
///
/// ## 📜 Example Output
/// ```text
/// 🔄 Source: /home/leon/maangframe
/// 📦 Destination: /home/leon/maangframe/DevBackups/maang_backups/maang_2025-10-17_19-03
/// sending incremental file list
/// ...
/// ✅ Backup complete at 2025-10-17_19-03
/// ```
///
/// ## 🧰 Notes
/// - Uses `OffsetDateTime::now_utc()` for portable timestamps.
/// - Uses `format_description!` macro for compile-time date formatting.
/// - Avoids unwraps and panics entirely.
/// - Exits with an error message if:
///   - Home directory not found
///   - rsync fails (non-zero exit code)
///   - I/O or time format errors occur.
///
/// ## 🧱 Future Improvements
/// - Add gzip/tar compression after backup
/// - Keep last N backups only (auto-prune)
/// - Add optional CLI args for source/destination overrides
///
/// ---

use std::{fs, process::Command};
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Error)]
enum BackupError {
    #[error("home directory not found")]
    HomeDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("time format error: {0}")]
    TimeFmt(#[from] time::error::Format),
    #[error("rsync failed: exit code {0:?}")]
    Rsync(Option<i32>),
}

fn main() -> Result<(), BackupError> {
    // Observability
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Timestamp (UTC), using time's format_description! macro
    let now = OffsetDateTime::now_utc();
    let fmt = format_description!("[year]-[month]-[day]_[hour]-[minute]");
    let timestamp = now.format(&fmt)?; // -> String

    // Paths
    let home = dirs::home_dir().ok_or(BackupError::HomeDir)?;
    let source = home.join("maangframe");
    let dest = home
        .join("maangframe")
        .join("DevBackups")
        .join("maang_backups")
        .join(format!("maang_{timestamp}"));

    info!("🔄 Source: {}", source.display());
    info!("📦 Destination: {}", dest.display());
    fs::create_dir_all(&dest)?;

    // Build args without unwraps
    let source_str = source.to_string_lossy().into_owned();
    let dest_str = dest.to_string_lossy().into_owned();

    let status = Command::new("rsync")
        .args([
            "-av",
            "--progress",
            "--exclude",
            "target",
            "--exclude",
            "*.d",
            &source_str,
            &dest_str,
        ])
        .status()?;

    if !status.success() {
        return Err(BackupError::Rsync(status.code()));
    }

    info!("✅ Backup complete at {timestamp}");
    Ok(())
}
