pub mod cmd;
pub mod engine;
pub mod models;

use anyhow::Result;
use clap::Parser;
use cmd::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("🔥 Ferropress Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { force, name } => {
            cmd::init::execute(force, name)?;
        }
        Commands::Build => {
            cmd::build::execute()?;
        }
        Commands::Preview => {
            println!("🔥 Starting dev server with hot-reload...");
            cmd::serve::execute(true).await?;
        }
        Commands::Serve => {
            println!("🌍 Serving production build on http://localhost:3000");
            cmd::serve::execute(false).await?;
        }
    }
    Ok(())
}
