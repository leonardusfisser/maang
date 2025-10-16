// ops/backup_tenant/src/main.rs
use std::{fs, path::Path, process::Command, time::{SystemTime, UNIX_EPOCH}};

use thiserror::Error;

type Result<T> = std::result::Result<T, OpsError>;

#[derive(Debug, Error)]
enum OpsError {
    #[error("missing argument: {0}")]
    MissingArg(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("time error")]
    Time,
    #[error("command failed: {cmd} (code: {code:?})")]
    CommandFailed { cmd: &'static str, code: Option<i32> },
}

fn epoch_secs() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OpsError::Time)?
        .as_secs())
}

fn main() -> Result<()> {
    let tenant = std::env::args().nth(1).ok_or(OpsError::MissingArg("tenant domain"))?;
    let ns = tenant.replace('.', "_");
    let ts = epoch_secs()?.to_string();

    let tenant_root = format!("/srv/lillpepe/tenants/{tenant}");
    let dump_path = format!("/tmp/{tenant}-{ts}.surql");
    let backup_dir = "/backups";
    let archive_path = format!("{backup_dir}/{tenant}-{ts}.tar.zst");

    // ensure paths
    if !Path::new(&tenant_root).exists() {
        return Err(OpsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("tenant folder not found: {tenant_root}"),
        )));
    }
    fs::create_dir_all(backup_dir)?;

    // 1) Export SurrealDB (per-tenant namespace)
    // Note: relies on `surreal` CLI in PATH; adjust if needed.
    let status = Command::new("surreal")
        .args(["export", &dump_path, "--ns", &ns, "--db", "site"])
        .status()?;
    if !status.success() {
        return Err(OpsError::CommandFailed { cmd: "surreal export", code: status.code() });
    }

    // 2) Archive tenant folder + dump (zstd-compressed)
    // Example: tar -I zstd -cf /backups/tenant-TS.tar.zst /srv/.../tenant /tmp/tenant-TS.surql
    let status = Command::new("tar")
        .args(["-I", "zstd", "-cf", &archive_path, &tenant_root, &dump_path])
        .status()?;
    if !status.success() {
        return Err(OpsError::CommandFailed { cmd: "tar", code: status.code() });
    }

    // 3) Clean temp dump
    let _ = fs::remove_file(&dump_path);

    println!("✅ backup complete: {archive_path}");
    Ok(())
}
