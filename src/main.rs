use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
use commands::config::{cmd_config, ConfigAction};
use commands::open::cmd_open;
use commands::scheme::{cmd_scheme, SchemeAction};
use commands::setup::cmd_setup;

#[derive(Parser)]
#[command(name = "worktree", about = "Open GitHub issues as git worktree workspaces")]
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

    /// Run first-time setup: detect editor, write config, register URL scheme
    Setup,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Open { issue_ref, editor, print_path } => cmd_open(&issue_ref, editor, print_path)?,
        Commands::Config { action } => cmd_config(action)?,
        Commands::Scheme { action } => cmd_scheme(action)?,
        Commands::Setup => cmd_setup()?,
    }

    Ok(())
}
