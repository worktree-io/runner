use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Write a bootstrap script (hook + `exec "${SHELL:-sh}"`) to a temp file and
/// spawn the terminal running it. Returns `true` if the command was recognised
/// as a terminal emulator, `false` otherwise (IDE / unknown command).
pub(super) fn try_terminal_with_init(
    path: &Path,
    command: &str,
    init_script: &str,
) -> Result<bool> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;

    let path_escaped = path_str.replace('\'', "'\\''");
    let bootstrap = format!(
        "#!/bin/sh\ncd '{}'\n{}\nexec \"${{SHELL:-sh}}\"\n",
        path_escaped, init_script
    );

    let tmp_path =
        std::env::temp_dir().join(format!("worktree-hook-open-{}.sh", std::process::id()));
    std::fs::write(&tmp_path, bootstrap.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let tmp_str = tmp_path
        .to_str()
        .context("Temp path contains non-UTF-8 characters")?;
    let cmd_lower = command.to_ascii_lowercase();

    if cmd_lower.contains("iterm") {
        let script = format!(
            r#"tell application "iTerm2" to create window with default profile command "sh {}""#,
            tmp_str
        );
        Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else if cmd_lower.contains("open -a terminal") {
        Command::new("open")
            .args(["-a", "Terminal", tmp_str])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else if cmd_lower.starts_with("alacritty") {
        Command::new("alacritty")
            .args(["--working-directory", path_str, "-e", "sh", tmp_str])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else if cmd_lower.starts_with("kitty") {
        Command::new("kitty")
            .args(["--directory", path_str, "sh", tmp_str])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else if cmd_lower.starts_with("wezterm") {
        Command::new("wezterm")
            .args(["start", "--cwd", path_str, "--", "sh", tmp_str])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(true)
    } else {
        Ok(false)
    }
}
