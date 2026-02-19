use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum DaemonStatus {
    Installed { path: String },
    NotInstalled,
}

impl std::fmt::Display for DaemonStatus {
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

pub fn status() -> Result<DaemonStatus> {
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
    let contents = app.join("Contents");
    let macos_dir = contents.join("MacOS");

    std::fs::create_dir_all(&macos_dir)
        .with_context(|| format!("Failed to create {}", macos_dir.display()))?;

    // Shell handler script that forwards the URL as the first argument
    let handler = macos_dir.join("runner-handler");
    let script = format!(
        "#!/bin/sh\nexec {exe} open \"$1\"\n",
        exe = exe.display()
    );
    std::fs::write(&handler, &script)
        .with_context(|| format!("Failed to write handler script to {}", handler.display()))?;

    // Make it executable
    Command::new("chmod")
        .args(["+x"])
        .arg(&handler)
        .status()
        .context("Failed to chmod handler script")?;

    // Info.plist
    let plist_path = contents.join("Info.plist");
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>io.worktree.runner</string>
    <key>CFBundleName</key>
    <string>WorktreeRunner</string>
    <key>CFBundleExecutable</key>
    <string>runner-handler</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Worktree URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>worktree</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#
    );
    std::fs::write(&plist_path, plist)
        .with_context(|| format!("Failed to write Info.plist to {}", plist_path.display()))?;

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
fn platform_status() -> Result<DaemonStatus> {
    let app = app_dir();
    if app.join("Contents").join("Info.plist").exists() {
        Ok(DaemonStatus::Installed {
            path: app.display().to_string(),
        })
    } else {
        Ok(DaemonStatus::NotInstalled)
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
fn platform_status() -> Result<DaemonStatus> {
    let path = desktop_file();
    if path.exists() {
        Ok(DaemonStatus::Installed {
            path: path.display().to_string(),
        })
    } else {
        Ok(DaemonStatus::NotInstalled)
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
fn platform_status() -> Result<DaemonStatus> {
    use std::process::Command;

    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Classes\worktree"])
        .output()
        .context("Failed to query registry")?;

    if output.status.success() {
        Ok(DaemonStatus::Installed {
            path: r"HKCU\Software\Classes\worktree".to_string(),
        })
    } else {
        Ok(DaemonStatus::NotInstalled)
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
fn platform_status() -> Result<DaemonStatus> {
    Ok(DaemonStatus::NotInstalled)
}
