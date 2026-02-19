use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

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
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
