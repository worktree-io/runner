use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Write a bootstrap script (hook + `exec "${SHELL:-sh}"`) to a temp file and
/// spawn the terminal running it. Returns `true` if the command was recognised
/// as a terminal emulator, `false` otherwise (IDE / unknown command).
fn try_terminal_with_init(path: &Path, command: &str, init_script: &str) -> Result<bool> {
    let path_str = path
        .to_str()
        .context("Workspace path contains non-UTF-8 characters")?;

    // Escape single quotes for use inside a single-quoted sh string.
    let path_escaped = path_str.replace('\'', "'\\''");

    let bootstrap = format!(
        "#!/bin/sh\ncd '{}'\n{}\nexec \"${{SHELL:-sh}}\"\n",
        path_escaped, init_script
    );

    let tmp_path = std::env::temp_dir()
        .join(format!("worktree-hook-open-{}.sh", std::process::id()));
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

/// Check whether a macOS application bundle is installed.
fn app_exists(name: &str) -> bool {
    std::path::Path::new(&format!("/Applications/{name}.app")).exists()
        || std::path::Path::new(&format!("/System/Applications/{name}.app")).exists()
}

/// For the IDE case: find an available terminal app and run `init_script` inside it.
/// Probes in order: iTerm → Warp → Ghostty → Terminal.app.
/// Returns `true` if a terminal window was opened.
fn open_hook_in_auto_terminal(path: &Path, init_script: &str) -> Result<bool> {
    let candidates: &[(&str, &str)] = &[
        ("iTerm", "open -a iTerm ."),
        ("Warp", "open -a Warp ."),
        ("Ghostty", "open -a Ghostty ."),
        ("Terminal", "open -a Terminal ."),
    ];
    for &(app, cmd) in candidates {
        if app_exists(app) && try_terminal_with_init(path, cmd, init_script)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Open `path` with `command` and run `init_script` inside the resulting window.
/// Returns `true` when the hook ran inside a terminal window, `false` when an
/// IDE was opened and no terminal was available (caller should run the hook as
/// a fallback).
pub fn open_with_hook(path: &Path, command: &str, init_script: &str) -> Result<bool> {
    if try_terminal_with_init(path, command, init_script)? {
        return Ok(true);
    }
    // IDE path: open the editor then try to show the hook in a separate terminal.
    open_in_editor(path, command)?;
    open_hook_in_auto_terminal(path, init_script)
}

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

/// Split a command string on whitespace and run it.
fn run_shell_command(cmd: &str) -> Result<()> {
    let mut parts = shlex_split(cmd);
    if parts.is_empty() {
        bail!("Empty command");
    }
    let program = parts.remove(0);
    Command::new(&program)
        .args(&parts)
        .env("PATH", augmented_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn {program}"))?;
    Ok(())
}

/// Return a PATH that includes common binary directories that GUI-launched
/// processes (e.g. via AppleScript `do shell script`) typically lack.
pub fn augmented_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let extras = [
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
    ];
    let mut parts: Vec<&str> = extras.iter().copied().collect();
    for p in current.split(':').filter(|s| !s.is_empty()) {
        if !parts.contains(&p) {
            parts.push(p);
        }
    }
    parts.join(":")
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
