//! Multi-repo workspace creation: one unified folder across several repos.
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    git::{
        bare_clone, branch_exists_remote, create_worktree, detect_default_branch, git_fetch,
        git_worktree_prune,
    },
    issue::IssueRef,
    name_gen,
    ttl::WorkspaceRegistry,
};

/// Create a unified workspace under `workspaces_root`, one sub-dir per issue.
///
/// For each `IssueRef` in `specs`:
/// - Bare-clones the repo (or fetches if already cached).
/// - Creates a worktree at `<root>/<repo>-<id>/`.
///
/// The root directory is registered in the `WorkspaceRegistry` so TTL pruning
/// applies to the entire unified workspace.
///
/// Returns the absolute path to the workspace root.
///
/// # Errors
///
/// Returns an error if any directory cannot be created, any repo cannot be
/// cloned/fetched, or any worktree cannot be created.
// LLVM_COV_EXCL_START
pub fn create_multi_workspace(specs: &[IssueRef], workspaces_root: &Path) -> Result<PathBuf> {
    let root = workspaces_root.join(name_gen::generate_name());
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create workspace root {}", root.display()))?;
    for issue in specs {
        open_one(issue, &root)?;
    }
    if let Ok(mut registry) = WorkspaceRegistry::load() {
        registry.register(root.clone());
        let _ = registry.save();
    }
    Ok(root)
}

fn open_one(issue: &IssueRef, root: &Path) -> Result<()> {
    let bare_path = issue.bare_clone_path();
    if bare_path.exists() {
        eprintln!("Fetching origin for {}…", bare_path.display());
        git_fetch(&bare_path)?;
    } else {
        eprintln!("Cloning {} (bare)…", issue.clone_url());
        bare_clone(&issue.clone_url(), &bare_path)?;
    }
    let base_branch = detect_default_branch(&bare_path)?;
    let branch = issue.branch_name();
    let branch_exists = branch_exists_remote(&bare_path, &branch);
    let _ = git_worktree_prune(&bare_path);
    let dest = root.join(issue.multi_dir_name());
    create_worktree(&bare_path, &dest, &branch, &base_branch, branch_exists)
        .with_context(|| format!("failed to create worktree at {}", dest.display()))
}
// LLVM_COV_EXCL_STOP
