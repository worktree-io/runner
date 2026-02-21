use anyhow::Result;
use clap::Subcommand;
use worktree_io::scheme;

#[derive(Subcommand)]
pub enum SchemeAction {
    /// Unregister the worktree:// URL scheme handler
    Uninstall,
    /// Check whether the URL scheme handler is registered
    Status,
}

pub fn cmd_scheme(action: SchemeAction) -> Result<()> {
    match action {
        SchemeAction::Uninstall => scheme::uninstall()?,
        SchemeAction::Status => println!("{}", scheme::status()?),
    }
    Ok(())
}
