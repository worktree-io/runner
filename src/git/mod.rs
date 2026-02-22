mod branch;
mod clone;
mod local_branch;

pub use branch::{branch_exists_remote, detect_default_branch};
pub use clone::{bare_clone, git_fetch};
pub use local_branch::{branch_exists_local, detect_local_default_branch};

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Create a worktree from a local (non-bare) repository without referencing a remote.
///
/// When `branch_exists` is false a new branch is created from HEAD.
/// When `branch_exists` is true the existing local branch is checked out.
pub fn create_local_worktree(
    repo: &Path,
    dest: &Path,
    branch: &str,
    branch_exists: bool,
) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(["-C"]).arg(repo).arg("worktree").arg("add");

    if branch_exists {
        cmd.arg(dest).arg(branch);
    } else {
        // Create new branch from HEAD (no origin/ reference needed)
        cmd.arg(dest).arg("-b").arg(branch);
    }

    let status = cmd.status().context("Failed to run `git worktree add`")?;

    if !status.success() {
        bail!("git worktree add failed for branch {branch}"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}

pub fn create_worktree(
    bare: &Path,
    dest: &Path,
    branch: &str,
    base_branch: &str,
    branch_exists: bool,
) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(["-C"]).arg(bare).arg("worktree").arg("add");

    if branch_exists {
        cmd.arg(dest).arg(branch);
    } else {
        cmd.arg(dest)
            .arg("-b")
            .arg(branch)
            .arg(format!("origin/{base_branch}"));
    }

    let status = cmd.status().context("Failed to run `git worktree add`")?;

    if !status.success() {
        bail!("git worktree add failed for branch {branch}"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}
