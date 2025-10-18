//! # tar_maang
//!
//! `tar_maang` is an operational utility binary in the MAANG framework.
//!
//! It performs a full backup of the `~/maangframe` project tree into
//! `~/DevBackups/maang_backups/maang_{timestamp}`.
//!
//! The process includes:
//! 1. Creating a timestamped backup directory.
//! 2. Copying all files and folders from `~/maangframe` using `cp -a --reflink=auto`.
//! 3. Removing transient build artefacts (`target/` directories and `.d` files).
//! 4. Compressing the cleaned backup into a `.tar.gz` archive in the same location.
//!
//! All operations are logged with `tracing`, and typed errors are used throughout
//! to ensure the binary remains panic-free and safe for automation.
//!
//! ## Example usage
//! ```bash
//! cargo run -p tar_maang --release
//! ```
//! This produces a ready-to-archive backup such as:
//! `~/DevBackups/maang_backups/maang_2025-10-17_21-00.tar.gz`
//!
use std::{fs, process::Command};
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Error)]
enum TarError {
    #[error("home directory not found")]
    HomeDir,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("time format error: {0}")]
    TimeFmt(#[from] time::error::Format),
    #[error("copy command failed: {0:?}")]
    Copy(Option<i32>),
    #[error("cleanup failed: {0}")]
    Cleanup(String),
    #[error("archive failed: {0}")]
    Archive(String),
}

fn main() -> Result<(), TarError> {
    // ─── observability ───────────────────────────────────────────────────────
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    let _ = tracing::subscriber::set_global_default(sub);

    // ─── timestamp ───────────────────────────────────────────────────────────
    let fmt = format_description!("[year]-[month]-[day]_[hour]-[minute]");
    let ts = OffsetDateTime::now_utc().format(&fmt)?;

    // ─── paths ───────────────────────────────────────────────────────────────
    let home = dirs::home_dir().ok_or(TarError::HomeDir)?;
    let dest_root = home.join("DevBackups/maang_backups");
    let dest = dest_root.join(format!("maang_{ts}"));
    let src = home.join("maangframe");
    fs::create_dir_all(&dest_root)?;

    info!("📦 Source: {}", src.display());
    info!("📁 Destination: {}", dest.display());

    // ─── 1. Copy project ─────────────────────────────────────────────────────
    let status = Command::new("cp")
        .args([
            "-a",
            "--reflink=auto",
            src.join(".").to_str().unwrap(),
            dest.to_str().unwrap(),
        ])
        .status()?;
    if !status.success() {
        return Err(TarError::Copy(status.code()));
    }

    // ─── 2. Cleanup build junk ───────────────────────────────────────────────
    let cleanup = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "find '{}' -type d -name target -prune -exec rm -rf {{}} +; \
             find '{}' -type f -name '*.d' -delete",
            dest.display(),
            dest.display()
        ))
        .status();

    match cleanup {
        Ok(s) if s.success() => (),
        Ok(s) => return Err(TarError::Cleanup(format!("exit {:?}", s.code()))),
        Err(e) => return Err(TarError::Cleanup(e.to_string())),
    }

    // ─── 3. Create tar.gz ────────────────────────────────────────────────────
    let archive = dest_root.join(format!("maang_{ts}.tar.gz"));
    let tar_status = Command::new("tar")
        .args([
            "-czf",
            archive.to_str().unwrap(),
            "-C",
            dest_root.to_str().unwrap(),
            dest.file_name().unwrap().to_str().unwrap(),
        ])
        .status();

    match tar_status {
        Ok(s) if s.success() => (),
        Ok(s) => return Err(TarError::Archive(format!("exit {:?}", s.code()))),
        Err(e) => return Err(TarError::Archive(e.to_string())),
    }

    info!("✅ Backup complete: {}", dest.display());
    info!("📦 Archive created: {}", archive.display());
    Ok(())
}
