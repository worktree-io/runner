use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let app = super::app_dir();
    if app.exists() {
        std::fs::remove_dir_all(&app)
            .with_context(|| format!("Failed to remove existing app at {}", app.display()))?;
    }
    let script_src = std::env::temp_dir().join("worktree-runner.applescript");
    let applescript = format!(
        "on open location this_URL\n\
         \tdo shell script {exe_q} & \" open \" & quoted form of this_URL\n\
         end open location\n",
        exe_q = applescript_quoted(&exe.display().to_string()),
    );
    std::fs::write(&script_src, &applescript).context("Failed to write AppleScript source")?;
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
    let plist = app.join("Contents").join("Info.plist");
    let pb = "/usr/libexec/PlistBuddy";
    plist_buddy(
        pb,
        "Add :CFBundleIdentifier string io.worktree.runner",
        &plist,
    )?;
    plist_buddy(pb, "Set :CFBundleName WorktreeRunner", &plist)?;
    let _ = Command::new(pb)
        .args(["-c", "Add :LSUIElement bool true"])
        .arg(&plist)
        .status();
    plist_buddy(pb, "Set :LSUIElement true", &plist)?;
    let _ = Command::new(pb)
        .args(["-c", "Add :CFBundleURLTypes array"])
        .arg(&plist)
        .status();
    plist_buddy(pb, "Add :CFBundleURLTypes:0 dict", &plist)?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLName string Worktree URL",
        &plist,
    )?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLSchemes array",
        &plist,
    )?;
    plist_buddy(
        pb,
        "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string worktree",
        &plist,
    )?;
    // Ad-hoc sign the bundle so Gatekeeper does not quarantine or evict it.
    let status = Command::new("codesign")
        .args(["--sign", "-", "--force", "--deep"])
        .arg(&app)
        .status()
        .context("Failed to run codesign")?;
    if !status.success() {
        bail!("codesign failed");
    }
    let lsregister = super::LSREGISTER;
    let status = Command::new(lsregister)
        .arg("-f")
        .arg(&app)
        .status()
        .context("Failed to run lsregister")?;
    if !status.success() {
        bail!("lsregister failed");
    }
    install_launch_agent(&app)?;
    println!("Installed WorktreeRunner.app at {}", app.display());
    println!("The worktree:// URL scheme is now registered.");
    Ok(())
}

fn install_launch_agent(app: &std::path::Path) -> Result<()> {
    let agents_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join("Library")
        .join("LaunchAgents");
    std::fs::create_dir_all(&agents_dir).context("Failed to create LaunchAgents directory")?;
    let plist_path = agents_dir.join("io.worktree.runner.plist");
    std::fs::write(&plist_path, launch_agent_plist_content(app)).with_context(|| {
        format!(
            "Failed to write LaunchAgent plist at {}",
            plist_path.display()
        )
    })?;
    Ok(())
}

pub(super) fn launch_agent_plist_content(app: &std::path::Path) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>io.worktree.runner</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         \t\t<string>{lsregister}</string>\n\
         \t\t<string>-f</string>\n\
         \t\t<string>{app}</string>\n\
         \t</array>\n\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         </dict>\n\
         </plist>\n",
        lsregister = super::LSREGISTER,
        app = app.display(),
    )
}

fn applescript_quoted(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

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

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
