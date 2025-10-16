// ops/provision_tenant/src/main.rs
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use thiserror::Error;

type Result<T> = std::result::Result<T, OpsError>;

#[derive(Debug, Error)]
enum OpsError {
    #[error("missing argument: {0}")]
    MissingArg(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn djb2_hash(s: &str) -> u32 {
    let mut h: u32 = 5381;
    for b in s.bytes() {
        h = ((h << 5).wrapping_add(h)).wrapping_add(b as u32);
    }
    h
}

// Map domain → port in 10000..19999 (deterministic, avoids collisions in most cases)
fn port_for(domain: &str) -> u16 {
    let h = djb2_hash(domain);
    let base = 10_000u16;
    let span = 10_000u16; // 10000..19999
    base + ((h as u16) % span)
}

fn write(path: &str, content: &str) -> Result<()> {
    let mut f = File::create(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let domain = std::env::args().nth(1).ok_or(OpsError::MissingArg("domain"))?;
    let ns = domain.replace('.', "_");
    let root = format!("/srv/lillpepe/tenants/{domain}");

    // dirs
    for d in ["", "/static", "/media", "/logs", "/data", "/cache", "/backups", "/tmp"] {
        fs::create_dir_all(format!("{root}{d}"))?;
    }

    // env
    let port = port_for(&domain);
    let env_content = format!(
        "TENANT_ID={domain}\nDOMAIN={domain}\nHOST=127.0.0.1\nPORT={port}\n\
SURREAL_URL=http://127.0.0.1:8000\nSURREAL_NS={ns}\nSURREAL_DB=site\n\
LOG_DIR={root}/logs\nDATA_DIR={root}/data\n"
    );
    write(&format!("{root}/env"), &env_content)?;

    // config.toml (minimal defaults; you’ll regenerate from DB later)
    let cfg = format!(
        r#"[tenant]
id = "{domain}"
name = "{domain}"
domain = "{domain}"
plan = "shared"
enabled = true

[branding]
title = "{domain}"
tagline = ""
primary = "#3498db"
secondary = "#2ecc71"
background = "#ffffff"
text_color = "#222222"
logo = "img/logo.png"
favicon = "favicon.ico"

[db]
namespace = "{ns}"
database  = "site"
path      = "{root}/data/surreal.db"

[features]
enable_posts = true
enable_products = false
enable_services = false
enable_gallery = true
enable_payments = false
enable_blog_rss = true
"#);
    write(&format!("{root}/config.toml"), &cfg)?;

    // license placeholder (bind later)
    write(&format!("{root}/license.json"), r#"{"status":"pending"}"#)?;

    // perms hint (you can enforce via a separate ops tool or installer)
    if !Path::new(&root).exists() {
        return Err(OpsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "tenant root not found after creation",
        )));
    }

    println!("✅ provisioned {domain} at {root} (PORT {port}, NS {ns})");
    Ok(())
}
