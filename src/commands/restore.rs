use anyhow::Result;
use worktree_io::git::{create_worktree, git_worktree_prune};
use worktree_io::ttl::WorkspaceRegistry;

/// Restore worktrees whose directories were manually deleted.
///
/// Scans the workspace registry for entries whose paths no longer exist on
/// disk and attempts to recreate them by pruning the stale git worktree
/// reference and re-adding the worktree at the original path.
///
/// Local worktrees (under `~/worktrees/local/`) cannot be restored
/// automatically because the original project path is not stored in the
/// registry; the user is asked to run `worktree open <issue-ref>` instead.
pub fn cmd_restore() -> Result<()> {
    let home = dirs::home_dir().expect("could not determine home directory");
    let local_prefix = home.join("worktrees").join("local");

    let registry = WorkspaceRegistry::load()?;
    let orphaned: Vec<_> = registry
        .workspace
        .iter()
        .filter(|r| !r.path.exists())
        .collect();

    if orphaned.is_empty() {
        eprintln!("No orphaned worktrees found.");
        return Ok(());
    }

    eprintln!("Found {} orphaned worktree(s).", orphaned.len());

    for record in &orphaned {
        let path = &record.path;

        if path.starts_with(&local_prefix) {
            eprintln!(
                "Skipping local worktree: {}\n  Run `worktree open <issue-ref>` to restore it.",
                path.display()
            );
            continue;
        }

        let Some(bare_path) = path.parent() else {
            eprintln!(
                "Skipping {} — cannot determine bare clone path.",
                path.display()
            ); // LLVM_COV_EXCL_LINE
            continue; // LLVM_COV_EXCL_LINE
        };

        let Some(branch) = path.file_name().and_then(|n| n.to_str()) else {
            eprintln!(
                "Skipping {} — cannot determine branch name.",
                path.display()
            ); // LLVM_COV_EXCL_LINE
            continue; // LLVM_COV_EXCL_LINE
        };

        if !bare_path.exists() {
            eprintln!(
                "Skipping {} — bare clone no longer exists at {}.",
                path.display(),
                bare_path.display()
            );
            continue;
        }

        eprintln!("Restoring {}…", path.display());

        if let Err(e) = git_worktree_prune(bare_path) {
            eprintln!("  Warning: worktree prune failed: {e}"); // LLVM_COV_EXCL_LINE
        }

        match create_worktree(bare_path, path, branch, "", true) {
            Ok(()) => eprintln!("  Restored: {}", path.display()),
            Err(e) => eprintln!("  Failed to restore {}: {e}", path.display()), // LLVM_COV_EXCL_LINE
        }
    }

    Ok(())
}
