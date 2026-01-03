pub mod build;
pub mod init;
pub mod serve;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ferropress", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize new site structure
    Init {
        /// Name of the project folder to create
        name: Option<String>,
        /// Overwrite existing files in the current directory
        #[arg(short, long)]
        force: bool,
    },
    /// Run development server with hot reload and file watching
    Preview,
    /// Build the static site
    Build,
    /// Serve the web app
    Serve,
}
