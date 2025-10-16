// Move one tenant from this server (source) to a destination host.
// Steps (idempotent where possible):
// 1) Export SurrealDB NS/DB to /tmp/*.surql
// 2) Rsync tenant folder to dest
// 3) Copy dump to dest
// 4) On dest: import DB, ensure service + caddy reload
// 5) On source: stop service (optional), leave folder for 24–48h
use std::process::Command;
use std::{fs, path::Path};
use thiserror::Error;

type Result<T> = std::result::Result<T, MigError>;

#[derive(Debug, Error)]
enum MigError {
    #[error("missing argument: {0}")]
    Missing(&'static str),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("command failed: {cmd} (code: {code:?})")]
    Cmd { cmd: &'static str, code: Option<i32> },
    #[error("invalid state: {0}")]
    State(&'static str),
}

fn sh(cmd: &str, args: &[&str]) -> Result<()> {
    let st = Command::new(cmd).args(args).status()?;
    if !st.success() {
        return Err(MigError::Cmd { cmd, code: st.code() });
    }
    Ok(())
}

fn main() -> Result<()> {
    // Usage:
    //   migrate_tenant <domain> <dest_host> [dest_root=/srv/lillpepe/tenants]
    let mut it = std::env::args().skip(1);
    let domain = it.next().ok_or(MigError::Missing("domain"))?;
    let dest_host = it.next().ok_or(MigError::Missing("dest_host (ssh host)"))?;
    let dest_root = it.next().unwrap_or_else(|| "/srv/lillpepe/tenants".to_string());

    let ns = domain.replace('.', "_");
    let src_root = format!("/srv/lillpepe/tenants/{domain}");
    if !Path::new(&src_root).exists() {
        return Err(MigError::State("source tenant folder not found"));
    }

    // 0) preflight: ensure service exists (non-fatal if not)
    // systemctl daemon-reload (ignored) – we won’t fail if missing.

    // 1) export DB
    let dump = format!("/tmp/{domain}.surql");
    sh("surreal", &["export", &dump, "--ns", &ns, "--db", "site"])?;

    // 2) rsync tenant folder → dest (exclude volatile)
    let dest_path = format!("{dest_host}:{dest_root}/{domain}");
    sh("ssh", &[&dest_host, "sudo", "mkdir", "-p", &format!("{dest_root}/{domain}")])?;
    sh(
        "rsync",
        &[
            "-a",
            "--delete",
            "--exclude", "cache/",
            "--exclude", "tmp/",
            "--exclude", "backups/",
            &format!("{}/", &src_root),
            &dest_path,
        ],
    )?;

    // 3) copy DB dump → dest
    sh("scp", &[&dump, &format!("{dest_host}:/tmp/{}.surql", domain)])?;
    let _ = fs::remove_file(&dump);

    // 4) on dest: import DB, enable service, reload caddy
    // NOTE: assumes same systemd template and caddy include pattern on dest.
    let import_cmd = format!(
        "sudo surreal import --ns {} --db site /tmp/{}.surql && sudo rm -f /tmp/{}.surql",
        ns, domain, domain
    );
    sh("ssh", &[&dest_host, import_cmd.as_str()])?;

    // ensure systemd service and start
    sh(
        "ssh",
        &[
            &dest_host,
            &format!("sudo systemctl enable --now lillpepe@{}.service", domain),
        ],
    )?;

    // caddy reload (non-fatal if not configured yet)
    let _ = Command::new("ssh")
        .args([&dest_host, "sudo systemctl reload caddy"])
        .status();

    // 5) (optional) stop source service but leave data for fallback window
    let _ = Command::new("systemctl")
        .args(["stop", &format!("lillpepe@{}.service", domain)])
        .status();

    println!("✅ migrated {} → {}:{}", domain, dest_host, format!("{dest_root}/{domain}"));
    println!("ℹ️  remember to update DNS (A/AAAA) to point to {}", dest_host);
    Ok(())
}
