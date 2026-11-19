use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustket", version, about = "Tiny package manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Install bit
    Install {
        name: String,
        /// Skip sha256 check (dev only)
        #[arg(long)]
        skip_verify: bool,
    },
    /// Remove package
    Uninstall { name: String },
    /// List installed packages
    List,
    /// Search bits
    Search { query: String },
    /// Show bit info
    Info { name: String },
    /// Check system for problems
    Doctor,
    /// Update rustket itself
    Update,
    /// Manage package taps
    Tap {
        #[command(subcommand)]
        command: TapCommand,
    },
}

#[derive(Subcommand)]
pub enum TapCommand {
    /// Add a GitHub tap (owner/repo or GitHub URL)
    Add { repository: String },
    /// List configured taps
    List,
    /// Remove a tap
    Remove { repository: String },
    /// Pull updates for all configured taps
    Update,
}
