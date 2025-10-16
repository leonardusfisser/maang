// ops/verify_tenants/src/main.rs
use std::{fs, process::Command};

use thiserror::Error;

type Result<T> = std::result::Result<T, OpsError>;

#[derive(Debug, Error)]
enum OpsError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("command failed: {cmd} (code: {code:?})")]
    CommandFailed { cmd: &'static str, code: Option<i32> },
}

fn main() -> Result<()> {
    let dir = "/srv/lillpepe/tenants";
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let domain = entry.file_name().to_string_lossy().to_string();
        let url = format!("https://{domain}/health");

        // curl -fsS --connect-timeout 5 --max-time 8 https://domain/health
        let status = Command::new("curl")
            .args(["-fsS", "--connect-timeout", "5", "--max-time", "8", &url, "-o", "/dev/null"])
            .status()?;

        if status.success() {
            println!("✅ {domain} OK");
        } else {
            eprintln!("⚠️  {domain} bad/failed health (curl code: {:?})", status.code());
        }
    }
    Ok(())
}
