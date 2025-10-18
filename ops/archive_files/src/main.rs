//! # archive_slices
//!
//! Creates multiple **type-specific** `.tar.gz` archives from the MAANG tree:
//! - `maang_md.tar.gz`  → all `*.md`
//! - `maang_rs.tar.gz`  → all `*.rs`
//! - `maang_cargotoml.tar.gz` → all `Cargo.toml`
//!
//! Excludes any `target/` directory, the archives output dir itself, and the
//! project root as a file. Logs progress; skips unreadable files with warnings.
//!
//! Output location: `<maang_root>/var/archives/`.
//! 
use std::{
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
    #[error("finish encoder error: {0}")]
    Finish(String),
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

fn archives_dir(maang_root: &Path) -> PathBuf {
    maang_root.join("var").join("archives")
}

fn is_excluded(path: &Path, maang_root: &Path, archives_dir: &Path) -> bool {
    if path.components().any(|c| c.as_os_str() == "target") {
        return true;
    }
    if path.starts_with(archives_dir) {
        return true;
    }
    path == maang_root
}

fn should_visit(entry: &DirEntry, maang_root: &Path, archives_dir: &Path) -> bool {
    !is_excluded(entry.path(), maang_root, archives_dir)
}

fn create_archive<F>(
    maang_root: &Path,
    archives_dir: &Path,
    out_name: &str,
    mut include_file: F,
) -> Result<(), AppError>
where
    F: FnMut(&Path) -> bool,
{
    if !archives_dir.exists() {
        std::fs::create_dir_all(archives_dir)?;
    }
    let out_path = archives_dir.join(out_name);
    info!("Creating {}", out_path.display());

    let out_file = File::create(&out_path)?;
    let enc = GzEncoder::new(out_file, Compression::default());
    let mut builder = Builder::new(enc);

    for entry in WalkDir::new(maang_root).follow_links(false) {
        let entry = entry?;
        if !should_visit(&entry, maang_root, archives_dir) {
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

        if meta.is_dir() {
            // Don’t write dirs for type-specific archives; files will imply dirs.
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        if !include_file(path) {
            continue;
        }

        let rel = path
            .strip_prefix(maang_root)
            .map_err(|_| AppError::StripPrefix {
                path: path.to_path_buf(),
            })?;

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
    let enc = builder
        .into_inner()
        .map_err(|e| AppError::Finish(e.to_string()))?;
    enc.finish().map_err(|e| AppError::Finish(e.to_string()))?;
    info!("✅ Wrote {}", out_path.display());
    Ok(())
}

fn main() -> Result<(), AppError> {
    init_tracing();

    let cwd = std::env::current_dir()?;
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let maang_root = exe_dir
        .as_deref()
        .and_then(find_maang_root)
        .or_else(|| find_maang_root(&cwd))
        .unwrap_or(cwd);

    let archives = archives_dir(&maang_root);
    info!("Project root: {}", maang_root.display());
    info!("Archives dir: {}", archives.display());

    // *.md
    create_archive(&maang_root, &archives, "maang_md.tar.gz", |p: &Path| {
        p.extension().and_then(|e| e.to_str()) == Some("md")
    })?;

    // *.rs
    create_archive(&maang_root, &archives, "maang_rs.tar.gz", |p: &Path| {
        p.extension().and_then(|e| e.to_str()) == Some("rs")
    })?;

    // Cargo.toml (exact file name)
    create_archive(
        &maang_root,
        &archives,
        "maang_cargotoml.tar.gz",
        |p: &Path| p.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml"),
    )?;

    info!("🎯 Done.");
    Ok(())
}
