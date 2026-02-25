use std::time::SystemTime;

use anyhow::{bail, Result};
use worktree_io::ttl::WorkspaceRegistry;
use worktree_io::{config::Config, ttl};

pub fn cmd_prune() -> Result<()> {
    let config = Config::load()?;
    let Some(ttl) = config.workspace.ttl else {
        bail!("No workspace TTL configured. Set workspace.ttl in your config (e.g. \"7days\").");
    };

    let mut registry = WorkspaceRegistry::load()?;
    let expired = ttl::prune(&registry.workspace, &ttl, SystemTime::now());

    if expired.is_empty() {
        eprintln!("No expired workspaces found.");
        return Ok(());
    }

    let expired_paths: Vec<_> = expired.iter().map(|r| r.path.clone()).collect();

    for path in &expired_paths {
        eprintln!("Removing expired workspace: {}", path.display());
        if let Err(e) = std::fs::remove_dir_all(path) {
            eprintln!("Warning: failed to remove {}: {e}", path.display());
        }
    }

    registry
        .workspace
        .retain(|r| !expired_paths.contains(&r.path));
    registry.save()?;

    eprintln!("Pruned {} expired workspace(s).", expired_paths.len());
    Ok(())
}
