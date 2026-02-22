use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn detect_default_branch(bare: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to run `git symbolic-ref`")?;

    if output.status.success() {
        let full = String::from_utf8_lossy(&output.stdout);
        if let Some(branch) = full.trim().strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args(["remote", "show", "origin"])
        .output()
        .context("Failed to run `git remote show origin`")?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if let Some(branch) = line.strip_prefix("HEAD branch: ") {
                return Ok(branch.to_string()); // LLVM_COV_EXCL_LINE
            }
        }
    }

    for candidate in ["main", "master", "develop"] {
        let output = Command::new("git")
            .args(["-C"])
            .arg(bare)
            .args([
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{candidate}"),
            ])
            .output()
            .context("Failed to run `git rev-parse`")?;
        if output.status.success() {
            return Ok(candidate.to_string()); // LLVM_COV_EXCL_LINE
        }
    }

    bail!("Could not detect default branch for the repository"); // LLVM_COV_EXCL_LINE
}

pub fn branch_exists_remote(bare: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args([
            "rev-parse",
            "--verify",
            &format!("refs/remotes/origin/{branch}"),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Detect the current branch of a local (non-bare) repository.
pub fn detect_local_default_branch(repo: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to run `git rev-parse --abbrev-ref HEAD`")?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Ok(branch);
        }
    }

    // Fallback: check common branch names
    for candidate in ["main", "master", "develop"] {
        let output = Command::new("git")
            .args(["-C"])
            .arg(repo)
            .args(["rev-parse", "--verify", &format!("refs/heads/{candidate}")])
            .output()
            .context("Failed to run `git rev-parse`")?;
        if output.status.success() {
            return Ok(candidate.to_string()); // LLVM_COV_EXCL_LINE
        }
    }

    bail!("Could not detect default branch for the local repository"); // LLVM_COV_EXCL_LINE
}

/// Check whether a local branch exists in a non-bare repository.
pub fn branch_exists_local(repo: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
