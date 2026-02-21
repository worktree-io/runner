mod branch;
mod clone;

pub use branch::{branch_exists_remote, detect_default_branch};
pub use clone::{bare_clone, git_fetch};

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

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
        // Check out the existing remote branch, tracking it locally
        cmd.arg(dest).arg("--track").arg(format!("origin/{branch}"));
    } else {
        // Create a new branch from the default base
        cmd.arg(dest)
            .arg("-b")
            .arg(branch)
            .arg(format!("origin/{base_branch}"));
    }

    let status = cmd.status().context("Failed to run `git worktree add`")?;

    if !status.success() {
        bail!("git worktree add failed for branch {branch}");
    }
    Ok(())
}
