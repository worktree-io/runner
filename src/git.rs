use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn bare_clone(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["clone", "--bare", url])
        .arg(dest)
        .status()
        .context("Failed to run `git clone --bare`")?;

    if !status.success() {
        bail!("git clone --bare failed for {url}");
    }

    // Set up the remote tracking so `git fetch` and `symbolic-ref` work correctly
    // for a bare clone we need to configure remote.origin.fetch
    let fetch_refspec = "+refs/heads/*:refs/remotes/origin/*";
    let status = Command::new("git")
        .args(["-C"])
        .arg(dest)
        .args(["config", "remote.origin.fetch", fetch_refspec])
        .status()
        .context("Failed to configure remote.origin.fetch")?;

    if !status.success() {
        bail!("Failed to set remote.origin.fetch");
    }

    // Fetch so that refs/remotes/origin/HEAD is populated
    let status = Command::new("git")
        .args(["-C"])
        .arg(dest)
        .args(["fetch", "origin"])
        .status()
        .context("Failed to run `git fetch origin`")?;

    if !status.success() {
        bail!("git fetch origin failed after bare clone");
    }

    Ok(())
}

pub fn git_fetch(bare: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["-C"])
        .arg(bare)
        .args(["fetch", "origin"])
        .status()
        .context("Failed to run `git fetch`")?;

    if !status.success() {
        bail!("git fetch origin failed");
    }
    Ok(())
}

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
        cmd.arg(dest)
            .arg("--track")
            .arg(format!("origin/{branch}"));
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
