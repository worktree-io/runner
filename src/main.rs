//! Worktree CLI binary — open GitHub issues as git worktree workspaces.
#![allow(missing_docs)] // binary crate; public API lives in the library
use anyhow::Result;
use clap::Parser;
mod cli;
mod commands;
use cli::{Cli, Commands};
use commands::config::cmd_config;
use commands::list::cmd_list;
use commands::open::cmd_open;
use commands::open_multi::cmd_open_multi;
use commands::prune::cmd_prune;
use commands::restore::cmd_restore;
use commands::scheme::cmd_scheme;
use commands::setup::cmd_setup;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Open {
            issue_ref,
            editor,
            no_hooks,
            headless,
            script,
        } => cmd_open(
            issue_ref.as_deref(),
            editor,
            no_hooks,
            headless,
            script.as_deref(),
        )?,
        Commands::OpenMulti { refs, no_hooks } => cmd_open_multi(&refs, no_hooks)?,
        Commands::Config { action } => cmd_config(action)?,
        Commands::List { json } => cmd_list(json)?,
        Commands::Prune { json } => cmd_prune(json)?,
        Commands::Restore => cmd_restore()?,
        Commands::Scheme { action } => cmd_scheme(action)?,
        Commands::Setup => cmd_setup()?,
        Commands::Version => println!("{}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
