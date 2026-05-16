use clap::{Parser, Subcommand};

use crate::commands::config::ConfigAction;
use crate::commands::scheme::SchemeAction;

#[derive(Parser)]
#[command(
    name = "worktree",
    about = "Open GitHub issues as git worktree workspaces",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse an issue reference, create a worktree, and open it
    Open {
        /// Issue reference (omit to detect from current repo's origin remote)
        #[arg(value_name = "REF")]
        issue_ref: Option<String>,
        /// Force open in editor
        #[arg(long)]
        editor: bool,
        /// Skip pre/post-open hooks
        #[arg(long)]
        no_hooks: bool,
        /// Skip opening editor/terminal (hooks still run); useful for programmatic invocation
        #[arg(long)]
        headless: bool,
        /// Run a script from .worktree/ as post:open, replacing all other hooks
        #[arg(long, value_name = "NAME")]
        script: Option<String>,
        /// Environment variables to inject (KEY=VALUE), may be repeated
        #[arg(long = "env", value_name = "KEY=VALUE", action = clap::ArgAction::Append)]
        env: Vec<String>,
        /// Output JSON with worktree path and created flag instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Open multiple repos as a unified workspace under ~/workspaces/<name>/
    #[command(name = "open-multi")]
    OpenMulti {
        /// Issue references (owner/repo#N or full GitHub URL), at least two
        #[arg(value_name = "REF", num_args = 1..)]
        refs: Vec<String>,
        /// Skip pre/post-open hooks
        #[arg(long)]
        no_hooks: bool,
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
    /// List all registered workspaces with their TTL status
    List {
        /// Emit a JSON report to stdout instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Remove expired worktrees based on workspace.ttl config
    Prune {
        /// Emit a JSON report to stdout instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Restore worktrees whose directories were manually deleted
    Restore,
    /// Run first-time setup: detect editor, write config, register URL scheme
    Setup,
    /// Print the current version
    Version,
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
