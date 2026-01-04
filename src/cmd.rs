pub mod build;
pub mod init;
pub mod serve;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        /// Directory to initialize (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Overwrite only FerroPress scaffold files if they already exist
        #[arg(long)]
        overwrite: bool,

        /// Remove existing scaffold directories/files before initializing (DANGEROUS)
        #[arg(long)]
        clean: bool,
    },
    /// Run development server with hot reload and file watching
    Preview,
    /// Build the static site
    Build,
    /// Serve the production build
    Serve,
}
