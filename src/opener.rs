use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Open the workspace path in the configured editor.
/// `command` is a shell template, e.g. `"code ."` or `"nvim ."`.
/// The trailing `.` (or any `.`) in the command is replaced by the actual path.
pub fn open_in_editor(path: &Path, command: &str) -> Result<()> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;

    // Replace standalone `.` tokens with the actual path, or append it
    let cmd_str = if command.contains(" . ") || command.ends_with(" .") || command == "." {
        command.replacen(" .", &format!(" {path_str}"), 1)
    } else {
        format!("{command} {path_str}")
    };

    run_shell_command(&cmd_str)
        .with_context(|| format!("Failed to open editor with command: {cmd_str}"))
}

/// Open the workspace path in the platform file explorer.
pub fn open_in_explorer(path: &Path) -> Result<()> {
    platform_open_in_explorer(path)
}

#[cfg(target_os = "macos")]
fn platform_open_in_explorer(path: &Path) -> Result<()> {
    Command::new("open")
        .arg(path)
        .spawn()
        .context("Failed to open Finder")?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_open_in_explorer(path: &Path) -> Result<()> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .context("Failed to open file manager")?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_open_in_explorer(path: &Path) -> Result<()> {
    Command::new("explorer")
        .arg(path)
        .spawn()
        .context("Failed to open Explorer")?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_open_in_explorer(_path: &Path) -> Result<()> {
    bail!("open_in_explorer is not implemented for this platform")
}

/// Open a terminal window in the workspace path.
/// If `command` is provided it is run as a shell template (same `.` replacement as editor).
/// Otherwise a platform-appropriate default terminal is used.
pub fn open_in_terminal(path: &Path, command: Option<&str>) -> Result<()> {
    if let Some(cmd) = command {
        let path_str = path
            .to_str()
            .context("Workspace path contains non-UTF-8 characters")?;
        let cmd_str = if cmd.contains(" . ") || cmd.ends_with(" .") || cmd == "." {
            cmd.replacen(" .", &format!(" {path_str}"), 1)
        } else {
            format!("{cmd} {path_str}")
        };
        return run_shell_command(&cmd_str)
            .with_context(|| format!("Failed to open terminal with command: {cmd_str}"));
    }

    open_default_terminal(path)
}

#[cfg(target_os = "macos")]
fn open_default_terminal(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;
    // Escape single quotes in the path for AppleScript
    let escaped = path_str.replace('\'', "'\\''");
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "cd '{escaped}'"
end tell"#
    );
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .context("Failed to open Terminal.app via osascript")?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_default_terminal(path: &Path) -> Result<()> {
    // Try common terminal emulators in order
    let terminals: &[&[&str]] = &[
        &["gnome-terminal", "--working-directory"],
        &["xterm", "-e", "bash -c 'cd \"$1\" && exec bash' -- "],
        &["konsole", "--workdir"],
        &["xfce4-terminal", "--working-directory"],
    ];

    for args in terminals {
        let (prog, rest) = args.split_first().unwrap();
        let mut cmd = Command::new(prog);
        for arg in rest {
            cmd.arg(arg);
        }
        cmd.arg(path);
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    bail!("No suitable terminal emulator found on this Linux system")
}

#[cfg(target_os = "windows")]
fn open_default_terminal(path: &Path) -> Result<()> {
    // Try Windows Terminal first, then cmd.exe
    let result = Command::new("wt")
        .args(["--startingDirectory"])
        .arg(path)
        .spawn();

    if result.is_ok() {
        return Ok(());
    }

    Command::new("cmd")
        .args(["/c", "start", "cmd.exe", "/k"])
        .arg(format!("cd /d \"{}\"", path.display()))
        .spawn()
        .context("Failed to open cmd.exe")?;

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn open_default_terminal(_path: &Path) -> Result<()> {
    bail!("open_in_terminal is not implemented for this platform")
}

/// Split a command string on whitespace and run it.
fn run_shell_command(cmd: &str) -> Result<()> {
    let mut parts = shlex_split(cmd);
    if parts.is_empty() {
        bail!("Empty command");
    }
    let program = parts.remove(0);
    Command::new(&program)
        .args(&parts)
        .spawn()
        .with_context(|| format!("Failed to spawn {program}"))?;
    Ok(())
}

/// Very simple whitespace-based command splitter that respects double-quoted strings.
fn shlex_split(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}
