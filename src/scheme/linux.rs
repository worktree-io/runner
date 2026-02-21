use anyhow::{Context, Result};
use std::process::Command;

use super::SchemeStatus;

fn desktop_file() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("applications")
        .join("worktree-runner.desktop")
}

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let path = desktop_file();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let content = format!(
        "[Desktop Entry]\n\
         Name=Worktree Runner\n\
         Exec={exe} open %u\n\
         Type=Application\n\
         NoDisplay=true\n\
         MimeType=x-scheme-handler/worktree;\n",
        exe = exe.display()
    );
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write desktop file to {}", path.display()))?;

    Command::new("xdg-mime")
        .args([
            "default",
            "worktree-runner.desktop",
            "x-scheme-handler/worktree",
        ])
        .status()
        .context("Failed to run xdg-mime")?;

    println!("Installed desktop entry at {}", path.display());
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let path = desktop_file();
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
        println!("Removed {}", path.display());
    } else {
        println!("Not installed — nothing to remove.");
    }
    Ok(())
}

pub fn status() -> Result<SchemeStatus> {
    let path = desktop_file();
    if path.exists() {
        Ok(SchemeStatus::Installed {
            path: path.display().to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}
