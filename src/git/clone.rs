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
