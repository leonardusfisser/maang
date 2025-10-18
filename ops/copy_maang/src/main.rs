//! Binary: copy_maang
//!
//! Copies the MAANG project tree to `~/DevBackups/maang_backups/maang_{timestamp}`,
//! removes all build artefacts (`target/`, `.d` files),
//! and optionally compresses the result into a `.tar.gz` archive.

use std::{fs, process::Command};
use thiserror::Error;
use time::{macros::format_description, OffsetDateTime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Error)]
enum CopyError {
    #[error("home directory not found")]
    HomeDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("time format error: {0}")]
    TimeFmt(#[from] time::error::Format),
    #[error("copy command failed: {0:?}")]
    Copy(Option<i32>),
    #[error("cleanup failed: {0}")]
    Cleanup(String),
}

fn main() -> Result<(), CopyError> {
    // observability
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    let _ = tracing::subscriber::set_global_default(sub);

    // timestamp
    let fmt = format_description!("[year]-[month]-[day]_[hour]-[minute]");
    let now = OffsetDateTime::now_utc();
    let ts = now.format(&fmt)?;

    // paths
    let home = dirs::home_dir().ok_or(CopyError::HomeDir)?;
    let dest = home.join("DevBackups/maang_backups").join(format!("maang_{ts}"));
    let maang = home.join("maangframe");

    info!("📦 Source: {}", maang.display());
    info!("📁 Destination: {}", dest.display());

    fs::create_dir_all(&dest)?;

    // 1. copy everything
    let status = Command::new("cp")
        .args([
            "-a",
            "--reflink=auto",
            maang.join(".").to_str().unwrap(),
            dest.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        return Err(CopyError::Copy(status.code()));
    }

    // 2. remove target dirs and .d files
    let cleanup = Command::new("find")
        .arg(dest.to_str().unwrap())
        .args([
            "(",
            "-type", "d", "-name", "target",
            "-prune", "-exec", "rm", "-rf", "{}", "+",
            ")",
            "-o",
            "(",
            "-type", "f", "-name", "*.d",
            "-delete",
            ")",
        ])
        .status();

    match cleanup {
        Ok(s) if s.success() => (),
        Ok(s) => return Err(CopyError::Cleanup(format!("exit {:?}", s.code()))),
        Err(e) => return Err(CopyError::Cleanup(e.to_string())),
    }

    info!("✅ Backup complete: {}", dest.display());
    Ok(())
}
