//! # archive_maang
//!
//! `archive_maang` is an operational binary in the MAANG framework.
//! It walks the entire `~/maangframe` project directory, excluding build artefacts,
//! and creates a compressed `archive_maang.tar.gz` inside `maangframe/var/archives`.
//!
//! ## Features
//! - Finds the MAANG project root automatically from the current or executable directory.
//! - Excludes all `target/` directories and previously created archives.
//! - Streams files directly into a `.tar.gz` using `flate2` and `tar` crates.
//! - Logs progress and skipped files with `tracing`.
//!
//! ## Example usage
//! ```bash
//! cargo run -p archive_maang --release
//! ```
//! Produces a compressed archive at:
//! ```text
//! ~/maangframe/var/archives/archive_maang.tar.gz
//! ```
//!
//! ## Safety
//! - No panics (`Result`-based error propagation).
//! - Uses typed errors (`AppError`).
//! - Ignores unreadable files safely, logging warnings instead of failing the run.

use std::{
    env,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use flate2::{Compression, write::GzEncoder};
use tar::Builder;
use thiserror::Error;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Error)]
enum AppError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("failed to strip prefix from path {path:?}")]
    StripPrefix { path: PathBuf },
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .try_init();
}

fn find_maang_root(start: &Path) -> Option<PathBuf> {
    for anc in start.ancestors() {
        if anc.file_name().and_then(|s| s.to_str()) == Some("maangframe") {
            return Some(anc.to_path_buf());
        }
    }
    None
}

// underscore added to silence lint cleanly
fn is_excluded(path: &Path, _maang_root: &Path, archives_dir: &Path) -> bool {
    if path.components().any(|c| c.as_os_str() == "target") {
        return true;
    }
    if path.starts_with(archives_dir) {
        return true;
    }
    let out_file = archives_dir.join("archive_maang.tar.gz");
    if path == out_file {
        return true;
    }
    false
}

fn should_visit(entry: &DirEntry, maang_root: &Path, archives_dir: &Path) -> bool {
    !is_excluded(entry.path(), maang_root, archives_dir)
}

fn archive_maang() -> Result<(), AppError> {
    init_tracing();

    let cwd = env::current_dir()?;
    let exe = env::current_exe().ok();
    let start = exe
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(cwd.as_path());

    let maang_root = find_maang_root(start)
        .or_else(|| find_maang_root(&cwd))
        .unwrap_or(cwd.clone());

    let archives_dir = maang_root.join("var").join("archives");
    if !archives_dir.exists() {
        std::fs::create_dir_all(&archives_dir)?;
    }

    let out_path = archives_dir.join("archive_maang.tar.gz");
    info!("Project root: {}", maang_root.display());
    info!("Output dir  : {}", archives_dir.display());
    info!("Archive file: {}", out_path.display());

    let out_file = File::create(&out_path)?;
    let enc = GzEncoder::new(out_file, Compression::default());
    let mut builder = Builder::new(enc);

    for entry in WalkDir::new(&maang_root).follow_links(false) {
        let entry = entry?;
        if !should_visit(&entry, &maang_root, &archives_dir) {
            continue;
        }

        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("Skipping (metadata error): {} ({e})", path.display());
                continue;
            }
        };

        if !meta.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(&maang_root)
            .map_err(|_| AppError::StripPrefix {
                path: path.to_path_buf(),
            })?;

        if rel.as_os_str().is_empty() {
            continue;
        }

        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Skipping (open error): {} ({e})", path.display());
                continue;
            }
        };

        builder.append_file(rel, &mut f)?;
    }

    builder.finish()?;
    let enc = builder.into_inner()?;
    enc.finish()?;

    info!("✅ Done. Created {}", out_path.display());
    Ok(())
}

fn main() -> Result<(), AppError> {
    archive_maang()
}
