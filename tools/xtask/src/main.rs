use std::process::Command;
use clap::{Parser, Subcommand};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
enum XtaskError {
    #[error("{0}")]
    Command(String),
}

type XtaskResult<T> = Result<T, XtaskError>;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Workspace utilities")]
struct Xtask {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check everything fast (cargo check --workspace)
    CheckAll,
    /// Build a tenant (placeholder)
    BuildTenant { name: String },
}

fn main() -> XtaskResult<()> {
    let args = Xtask::parse();
    match args.cmd {
        Commands::BuildTenant { name } => build_tenant(&name)?,
        Commands::CheckAll => check_all()?,
    }
    Ok(())
}

fn check_all() -> XtaskResult<()> {
    info!("running cargo check --workspace");
    let status = Command::new("cargo")
        .args(["check", "--workspace"])
        .status()
        .map_err(|e| XtaskError::Command(format!("spawn cargo failed: {e}")))?;
    if !status.success() {
        return Err(XtaskError::Command("cargo check failed".into()));
    }
    Ok(())
}

fn build_tenant(name: &str) -> XtaskResult<()> {
    info!("placeholder build for tenant: {name}");
    Ok(())
}
