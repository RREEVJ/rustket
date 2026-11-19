mod bit;
mod cli;
mod commands;
mod paths;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Install { name, skip_verify } => commands::install::run(&name, skip_verify),
        Command::Uninstall { name } => commands::uninstall::run(&name),
        Command::List => commands::list::run(),
        Command::Search { query } => commands::search::run(&query),
        Command::Info { name } => commands::info::run(&name),
        Command::Doctor => commands::doctor::run(),
        Command::Update => commands::update::run(),
        Command::Tap { command } => commands::tap::run(command),
    }
}
