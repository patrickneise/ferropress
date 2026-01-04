pub mod cmd;
pub mod engine;
pub mod models;

use crate::models::ServeMode;
use anyhow::{Context, Result};
use clap::Parser;
use cmd::{Cli, Commands};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("FerroPress Error: {:#}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init {
            path,
            overwrite,
            clean,
        } => {
            cmd::init::execute(path, overwrite, clean)
                .await
                .context("init failed")?;
        }
        Commands::Build => {
            cmd::build::execute().await.context("build failed")?;
        }
        Commands::Preview => {
            cmd::serve::execute(ServeMode::Dev)
                .await
                .context("preview failed")?;
        }
        Commands::Serve => {
            cmd::serve::execute(ServeMode::Prod)
                .await
                .context("serve failed")?;
        }
    }
    Ok(())
}
