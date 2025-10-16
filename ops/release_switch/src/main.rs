use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

type Result<T> = std::result::Result<T, SwitchError>;

#[derive(Debug, Error)]
enum SwitchError {
    #[error("missing argument: {0}")]
    MissingArg(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid state: {0}")]
    InvalidState(&'static str),
    #[error("command failed: {cmd} (code: {code:?})")]
    CommandFailed { cmd: &'static str, code: Option<i32> },
}

struct Args {
    version: Option<String>,
    canary_domain: Option<String>,
    restart_all: bool,
    rollback: bool,
}

fn parse_args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut version = None;
    let mut canary = None;
    let mut restart_all = false;
    let mut rollback = false;

    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => restart_all = true,
            "--rollback" => rollback = true,
            _ => {
                if version.is_none() { version = Some(a); }
                else if canary.is_none() { canary = Some(a); }
                else { /* ignore extras */ }
            }
        }
    }
    Ok(Args { version, canary_domain: canary, restart_all, rollback })
}

fn replace_symlink(target: &Path, link: &str) -> Result<()> {
    if Path::new(link).exists() {
        fs::remove_file(link)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
    }
    Ok(())
}

fn read_symlink(p: &str) -> Option<PathBuf> {
    fs::read_link(p).ok()
}

fn systemctl_restart(unit: &str) -> Result<()> {
    let st = Command::new("systemctl").args(["restart", unit]).status()?;
    if !st.success() {
        return Err(SwitchError::CommandFailed { cmd: "systemctl restart", code: st.code() });
    }
    Ok(())
}

fn curl_health(domain: &str) -> Result<()> {
    let url = format!("https://{}/health", domain);
    let st = Command::new("curl")
        .args(["-fsS", "--connect-timeout", "5", "--max-time", "8", &url, "-o", "/dev/null"])
        .status()?;
    if !st.success() {
        return Err(SwitchError::CommandFailed { cmd: "curl /health", code: st.code() });
    }
    Ok(())
}

fn restart_all_tenants() -> Result<()> {
    for entry in fs::read_dir("/srv/lillpepe/tenants")? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let domain = entry.file_name().to_string_lossy().to_string();
        let unit = format!("lillpepe@{}.service", domain);
        systemctl_restart(&unit)?;
    }
    Ok(())
}

fn do_rollback(prev_target: PathBuf, canary: Option<&str>) -> Result<()> {
    let link = "/opt/lillpepe/bin/current";
    replace_symlink(&prev_target, link)?;
    if let Some(dom) = canary {
        systemctl_restart(&format!("lillpepe@{}.service", dom))?;
        let _ = curl_health(dom);
    }
    // best-effort restart all (optional; comment if you prefer manual)
    restart_all_tenants()?;
    println!("↩ rolled back to {}", prev_target.display());
    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let link = "/opt/lillpepe/bin/current";

    // find previous release (2nd newest dir under /opt/lillpepe/releases)
    let releases_root = PathBuf::from("/opt/lillpepe/releases");
    let mut dirs: Vec<_> = fs::read_dir(&releases_root)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    dirs.sort_by(|a, b| b.cmp(a)); // newest first (lexicographic by version string)

    // rollback mode
    if args.rollback {
        let prev = dirs.get(1)
            .map(|p| p.join("lillpepe"))
            .ok_or(SwitchError::InvalidState("no previous release found"))?;
        if !prev.exists() {
            return Err(SwitchError::InvalidState("previous release binary missing"));
        }
        return do_rollback(prev, args.canary_domain.as_deref());
    }

    // switch mode requires version + canary
    let ver = args.version.ok_or(SwitchError::MissingArg("version (e.g. 2025.10.16+001)"))?;
    let canary = args.canary_domain.as_deref().ok_or(SwitchError::MissingArg("canary domain"))?;

    let new_target = releases_root.join(&ver).join("lillpepe");
    if !new_target.exists() {
        return Err(SwitchError::InvalidState("new release binary not found under /opt/lillpepe/releases/<version>/lillpepe"));
    }

    let prev = read_symlink(link);

    // switch symlink to new version
    replace_symlink(&new_target, link)?;
    println!("→ switched current → {}", new_target.display());

    // canary restart + health
    let canary_unit = format!("lillpepe@{}.service", canary);
    systemctl_restart(&canary_unit)?;
    if let Err(e) = curl_health(canary) {
        eprintln!("✖ canary failed: {e}. rolling back…");
        if let Some(prev_target) = prev {
            do_rollback(prev_target, Some(canary))?;
        }
        return Err(SwitchError::InvalidState("canary health failed"));
    }
    println!("✅ canary healthy");

    if args.restart_all {
        restart_all_tenants()?;
        println!("✅ restarted all tenants");
    } else {
        println!("ℹ add --all to restart all tenants after canary");
    }

    Ok(())
}
