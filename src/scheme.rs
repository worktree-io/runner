use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum SchemeStatus {
    Installed { path: String },
    NotInstalled,
}

impl std::fmt::Display for SchemeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed { path } => write!(f, "Installed at {path}"),
            Self::NotInstalled => write!(f, "Not installed"),
        }
    }
}

pub fn install() -> Result<()> {
    platform_install()
}

pub fn uninstall() -> Result<()> {
    platform_uninstall()
}

pub fn status() -> Result<SchemeStatus> {
    platform_status()
}

// ──────────────────────────── macOS ────────────────────────────

#[cfg(target_os = "macos")]
fn app_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join("Applications")
        .join("WorktreeRunner.app")
}

#[cfg(target_os = "macos")]
fn platform_install() -> Result<()> {
    use std::process::Command;

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

/// Wrap a string in AppleScript's `quoted form` equivalent for embedding in source.
/// Escapes backslashes and double-quotes so the path is safe inside a double-quoted
/// AppleScript string literal.
#[cfg(target_os = "macos")]
fn applescript_quoted(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Run a single PlistBuddy command, returning an error if it fails.
#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn platform_uninstall() -> Result<()> {
    use std::process::Command;

    let app = app_dir();
    if app.exists() {
        // Unregister before removing
        let lsregister = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/\
            LaunchServices.framework/Versions/A/Support/lsregister";
        let _ = Command::new(lsregister)
            .args(["-u"])
            .arg(&app)
            .status();

        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove {}", app.display()))?;
        println!("Removed {}", app.display());
    } else {
        println!("Not installed — nothing to remove.");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn platform_status() -> Result<SchemeStatus> {
    let app = app_dir();
    if app.join("Contents").join("Info.plist").exists() {
        Ok(SchemeStatus::Installed {
            path: app.display().to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}

// ──────────────────────────── Linux ────────────────────────────

#[cfg(target_os = "linux")]
fn desktop_file() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("applications")
        .join("worktree-runner.desktop")
}

#[cfg(target_os = "linux")]
fn platform_install() -> Result<()> {
    use std::process::Command;

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
        .args(["default", "worktree-runner.desktop", "x-scheme-handler/worktree"])
        .status()
        .context("Failed to run xdg-mime")?;

    println!("Installed desktop entry at {}", path.display());
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_uninstall() -> Result<()> {
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

#[cfg(target_os = "linux")]
fn platform_status() -> Result<SchemeStatus> {
    let path = desktop_file();
    if path.exists() {
        Ok(SchemeStatus::Installed {
            path: path.display().to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}

// ──────────────────────────── Windows ────────────────────────────

#[cfg(target_os = "windows")]
fn platform_install() -> Result<()> {
    use std::process::Command;

    let exe = std::env::current_exe()
        .context("Failed to get current executable path")?
        .display()
        .to_string();

    let run = |args: &[&str]| -> Result<()> {
        let status = Command::new("reg")
            .args(args)
            .status()
            .context("Failed to run `reg`")?;
        if !status.success() {
            bail!("reg command failed");
        }
        Ok(())
    };

    run(&[
        "add",
        r"HKCU\Software\Classes\worktree",
        "/d",
        "URL:Worktree Protocol",
        "/f",
    ])?;
    run(&[
        "add",
        r"HKCU\Software\Classes\worktree",
        "/v",
        "URL Protocol",
        "/d",
        "",
        "/f",
    ])?;
    run(&[
        "add",
        r"HKCU\Software\Classes\worktree\shell\open\command",
        "/d",
        &format!(r#""{exe}" open "%1""#),
        "/f",
    ])?;

    println!("Registered worktree:// URL scheme in Windows registry.");
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_uninstall() -> Result<()> {
    use std::process::Command;

    let status = Command::new("reg")
        .args(["delete", r"HKCU\Software\Classes\worktree", "/f"])
        .status()
        .context("Failed to run `reg delete`")?;

    if !status.success() {
        bail!("reg delete failed");
    }
    println!("Unregistered worktree:// URL scheme.");
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_status() -> Result<SchemeStatus> {
    use std::process::Command;

    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Classes\worktree"])
        .output()
        .context("Failed to query registry")?;

    if output.status.success() {
        Ok(SchemeStatus::Installed {
            path: r"HKCU\Software\Classes\worktree".to_string(),
        })
    } else {
        Ok(SchemeStatus::NotInstalled)
    }
}

// ──────────────────────────── Fallback ────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_install() -> Result<()> {
    bail!("URL scheme registration is not supported on this platform")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_uninstall() -> Result<()> {
    bail!("URL scheme registration is not supported on this platform")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_status() -> Result<SchemeStatus> {
    Ok(SchemeStatus::NotInstalled)
}
