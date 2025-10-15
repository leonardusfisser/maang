#![forbid(unsafe_code)]
#![deny(warnings)]

use clap::{Parser, Subcommand};
use std::process::Command;
use thiserror::Error;
use tracing::info;

fn main() -> XtaskResult<()> {
    tracing_subscriber::fmt::init();
    let args = Xtask::parse();

    match args.command {
        Commands::BuildTenant { name } => build_tenant(&name)?,
        Commands::CheckAll => check_all()?,
    }

    Ok(())
}


fn check_all() -> XtaskResult<()> {
    let cmds: &[&[&str]] = &[
        &["check", "--workspace", "--all-targets", "--all-features"],
        &["clippy", "--workspace", "--all-targets", "--all-features", "-D", "warnings"],
        &["test", "--workspace", "--all-features"],
    ];
    for args in cmds {
        let status = Command::new("cargo").args(args).status()?;
        if !status.success() {
            return Err(XtaskError::Command(format!("cargo {} failed", args[0])));
        }
    }
    Ok(())
}
