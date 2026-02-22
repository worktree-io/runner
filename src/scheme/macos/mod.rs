mod install;

pub use install::install;

use anyhow::{Context, Result};

use super::SchemeStatus;

pub(super) fn app_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join("Applications")
        .join("WorktreeRunner.app")
}

pub fn uninstall() -> Result<()> {
    let app = app_dir();
    if app.exists() {
        // LLVM_COV_EXCL_START
        let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
            LaunchServices.framework/Versions/A/Support/lsregister";
        let _ = std::process::Command::new(lsregister)
            .args(["-u"])
            .arg(&app)
            .status();
        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove {}", app.display()))?;
        println!("Removed {}", app.display());
        // LLVM_COV_EXCL_STOP
    } else {
        println!("Not installed — nothing to remove."); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}

pub fn status() -> Result<SchemeStatus> {
    let app = app_dir();
    if app.join("Contents").join("Info.plist").exists() {
        Ok(SchemeStatus::Installed {
            path: app.display().to_string(),
        }) // LLVM_COV_EXCL_LINE
    } else {
        Ok(SchemeStatus::NotInstalled) // LLVM_COV_EXCL_LINE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_dir_ends_with_app() {
        let d = app_dir();
        assert!(d.to_string_lossy().ends_with("WorktreeRunner.app"));
    }
    #[test]
    fn test_status_returns_ok() {
        assert!(status().is_ok());
    }
}
