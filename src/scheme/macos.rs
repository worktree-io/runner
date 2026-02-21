use anyhow::{bail, Context, Result};
use std::process::Command;

use super::SchemeStatus;

fn app_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join("Applications")
        .join("WorktreeRunner.app")
}

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let app = app_dir();

    // Remove any previous install so osacompile starts clean
    if app.exists() {
        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove existing app at {}", app.display()))?;
    }

    // macOS delivers URL scheme events as Apple Events (kAEGetURL / open location),
    // NOT as argv[1] to the executable.  A plain shell script never sees the URL.
    // Compiling an AppleScript applet that handles `on open location` is the
    // correct, documented way to receive the URL from the OS.
    let script_src = std::env::temp_dir().join("worktree-runner.applescript");
    let applescript = format!(
        "on open location this_URL\n\
         \tdo shell script {exe_q} & \" open \" & quoted form of this_URL\n\
         end open location\n",
        exe_q = applescript_quoted(&exe.display().to_string()),
    );
    std::fs::write(&script_src, &applescript)
        .context("Failed to write AppleScript source")?;

    // Compile the script into a .app bundle
    let status = Command::new("osacompile")
        .args(["-o"])
        .arg(&app)
        .arg(&script_src)
        .status()
        .context("Failed to run osacompile")?;
    let _ = std::fs::remove_file(&script_src);
    if !status.success() {
        bail!("osacompile failed");
    }

    // Patch the generated Info.plist: bundle identity + LSUIElement + URL scheme
    let plist = app.join("Contents").join("Info.plist");
    let pb = "/usr/libexec/PlistBuddy";

    // CFBundleIdentifier is absent from the osacompile-generated plist — Add it
    plist_buddy(pb, "Add :CFBundleIdentifier string io.worktree.runner", &plist)?;
    // CFBundleName is present but defaults to the script filename — override it
    plist_buddy(pb, "Set :CFBundleName WorktreeRunner", &plist)?;

    // LSUIElement keeps the applet out of the Dock; add if absent then set it
    let _ = Command::new(pb).args(["-c", "Add :LSUIElement bool true"]).arg(&plist).status();
    plist_buddy(pb, "Set :LSUIElement true", &plist)?;

    // URL scheme registration
    let _ = Command::new(pb).args(["-c", "Add :CFBundleURLTypes array"]).arg(&plist).status();
    plist_buddy(pb, "Add :CFBundleURLTypes:0 dict", &plist)?;
    plist_buddy(pb, "Add :CFBundleURLTypes:0:CFBundleURLName string Worktree URL", &plist)?;
    plist_buddy(pb, "Add :CFBundleURLTypes:0:CFBundleURLSchemes array", &plist)?;
    plist_buddy(pb, "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string worktree", &plist)?;

    // Register with Launch Services
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
        LaunchServices.framework/Versions/A/Support/lsregister";
    let status = Command::new(lsregister)
        .arg("-f")
        .arg(&app)
        .status()
        .context("Failed to run lsregister")?;

    if !status.success() {
        bail!("lsregister failed");
    }

    println!("Installed WorktreeRunner.app at {}", app.display());
    println!("The worktree:// URL scheme is now registered.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let app = app_dir();
    if app.exists() {
        // Unregister before removing
        let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
            LaunchServices.framework/Versions/A/Support/lsregister";
        let _ = Command::new(lsregister).args(["-u"]).arg(&app).status();

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

/// Wrap a string in AppleScript's `quoted form` equivalent for embedding in source.
/// Escapes backslashes and double-quotes so the path is safe inside a double-quoted
/// AppleScript string literal.
fn applescript_quoted(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Run a single PlistBuddy command, returning an error if it fails.
fn plist_buddy(pb: &str, cmd: &str, plist: &std::path::Path) -> Result<()> {
    let status = std::process::Command::new(pb)
        .args(["-c", cmd])
        .arg(plist)
        .status()
        .with_context(|| format!("Failed to run PlistBuddy: {cmd}"))?;
    if !status.success() {
        bail!("PlistBuddy failed: {cmd}");
    }
    Ok(())
}
