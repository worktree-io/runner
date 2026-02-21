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
        // Unregister before removing
        let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
            LaunchServices.framework/Versions/A/Support/lsregister";
        let _ = std::process::Command::new(lsregister).args(["-u"]).arg(&app).status();

        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove {}", app.display()))?;
        println!("Removed {}", app.display());
    } else {
        println!("Not installed — nothing to remove.");
    }
    Ok(())
}

pub fn status() -> Result<SchemeStatus> {
    let app = app_dir();
    if app.join("Contents").join("Info.plist").exists() {
        Ok(SchemeStatus::Installed { path: app.display().to_string() })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}
