// Safely disable + archive a tenant (non-destructive delete):
// 1) stop and disable service + timers
// 2) backup (tar.zst) to /backups
// 3) disable caddy vhost (rename file) + reload
// 4) move tenant folder to /srv/lillpepe/archived/<domain>-<epoch>
use std::{fs, path::Path, process::Command, time::{SystemTime, UNIX_EPOCH}};
use thiserror::Error;

type Result<T> = std::result::Result<T, DelError>;

#[derive(Debug, Error)]
enum DelError {
    #[error("missing arg: {0}")]
    Missing(&'static str),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("cmd failed: {cmd} (code: {code:?})")]
    Cmd { cmd: &'static str, code: Option<i32> },
    #[error("state: {0}")]
    State(&'static str),
}

fn epoch() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| DelError::State("clock"))?.as_secs())
}

fn sh(cmd: &str, args: &[&str]) -> Result<()> {
    let st = Command::new(cmd).args(args).status()?;
    if !st.success() {
        return Err(DelError::Cmd { cmd, code: st.code() });
    }
    Ok(())
}

fn main() -> Result<()> {
    // Usage: delete_tenant <domain>
    let domain = std::env::args().nth(1).ok_or(DelError::Missing("domain"))?;
    let root = format!("/srv/lillpepe/tenants/{}", domain);
    if !Path::new(&root).exists() {
        return Err(DelError::State("tenant folder not found"));
    }

    // 1) stop & disable service + timers
    let unit = format!("lillpepe@{}.service", domain);
    let timer = format!("backup-tenant@{}.timer", domain);
    let _ = Command::new("systemctl").args(["stop", &unit]).status();
    let _ = Command::new("systemctl").args(["disable", &unit]).status();
    let _ = Command::new("systemctl").args(["disable", &timer]).status();

    // 2) backup archive
    fs::create_dir_all("/backups")?;
    let ts = epoch()?.to_string();
    let archive = format!("/backups/{}-{}.tar.zst", domain, ts);
    sh("tar", &["-I", "zstd", "-cf", &archive, &root])?;

    // 3) disable caddy vhost if present
    let vhost = format!("/etc/caddy/tenants/{}.caddy", domain);
    if Path::new(&vhost).exists() {
        let disabled = format!("{}.disabled-{}", vhost, ts);
        fs::rename(&vhost, &disabled)?;
        let _ = Command::new("caddy").args(["validate","--config","/etc/caddy/Caddyfile"]).status();
        let _ = Command::new("systemctl").args(["reload","caddy"]).status();
    }

    // 4) move folder to archived/
    let archived_root = "/srv/lillpepe/archived";
    fs::create_dir_all(archived_root)?;
    let target = format!("{}/{}-{}", archived_root, domain, ts);
    fs::rename(&root, &target)?;

    println!("✅ tenant {} disabled and archived → {}", domain, target);
    println!("🔒 backup at {}", archive);
    Ok(())
}
