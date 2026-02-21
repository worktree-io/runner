use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn detect_default_branch(bare: &Path) -> Result<String> {
    // Try symbolic-ref first (works when remote HEAD is set)
    let output = Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to run `git symbolic-ref`")?;

    if output.status.success() {
        let full = String::from_utf8_lossy(&output.stdout);
        // Output looks like "refs/remotes/origin/main\n"
        if let Some(branch) = full.trim().strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    // Fall back: try `git remote show origin` to detect the default branch name
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
                return Ok(branch.to_string());
            }
        }
    }

    // Last resort: try common names
    for candidate in ["main", "master", "develop"] {
        let output = Command::new("git")
            .args(["-C"])
            .arg(bare)
            .args(["rev-parse", "--verify", &format!("refs/remotes/origin/{candidate}")])
            .output()
            .context("Failed to run `git rev-parse`")?;
        if output.status.success() {
            return Ok(candidate.to_string());
        }
    }

    bail!("Could not detect default branch for the repository");
}

pub fn branch_exists_remote(bare: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args(["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
