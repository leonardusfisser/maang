#![allow(missing_docs)]

use std::process::Command;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    let now = OffsetDateTime::now_utc();
    let formatted = now.format(&Rfc3339).unwrap_or_else(|_| "unknown".into());

    let git_hash = git_hash.trim();
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=BUILD_TIME={formatted}");
}
