//! Worktree CLI binary — open GitHub issues as git worktree workspaces.
#![allow(missing_docs)] // binary crate; public API lives in the library
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
use commands::config::{cmd_config, ConfigAction};
use commands::open::cmd_open;
use commands::open_multi::cmd_open_multi;
use commands::prune::cmd_prune;
use commands::restore::cmd_restore;
use commands::scheme::{cmd_scheme, SchemeAction};
use commands::setup::cmd_setup;

#[derive(Parser)]
#[command(
    name = "worktree",
    about = "Open GitHub issues as git worktree workspaces",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an issue reference, create a worktree, and open it
    Open {
        /// Issue reference: GitHub URL, worktree:// deep link, or owner/repo#N
        #[arg(value_name = "REF")]
        issue_ref: String,
        /// Force open in editor
        #[arg(long)]
        editor: bool,
        /// Print the workspace path and exit without opening anything
        #[arg(long)]
        print_path: bool,
    },
    /// Open multiple repos as a unified workspace under ~/workspaces/<name>/
    #[command(name = "open-multi")]
    OpenMulti {
        /// Issue references (owner/repo#N or full GitHub URL), at least two
        #[arg(value_name = "REF", num_args = 1..)]
        refs: Vec<String>,
    },
    /// Manage worktree configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage the worktree:// URL scheme handler
    Scheme {
        #[command(subcommand)]
        action: SchemeAction,
    },
    /// Remove expired worktrees based on workspace.ttl config
    Prune,
    /// Restore worktrees whose directories were manually deleted
    Restore,
    /// Run first-time setup: detect editor, write config, register URL scheme
    Setup,
    /// Print the current version
    Version,
}

fn main() -> Result<()> {
    // LLVM_COV_EXCL_LINE
    // LLVM_COV_EXCL_START
    let cli = Cli::parse();
    match cli.command {
        Commands::Open {
            issue_ref,
            editor,
            print_path,
        } => cmd_open(&issue_ref, editor, print_path)?,
        Commands::OpenMulti { refs } => cmd_open_multi(&refs)?,
        Commands::Config { action } => cmd_config(action)?,
        Commands::Prune => cmd_prune()?,
        Commands::Restore => cmd_restore()?,
        Commands::Scheme { action } => cmd_scheme(action)?,
        Commands::Setup => cmd_setup()?,
        Commands::Version => println!("{}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
    // LLVM_COV_EXCL_STOP
}
