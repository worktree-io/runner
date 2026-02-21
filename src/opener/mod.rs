mod shell;
mod terminal;

pub use shell::augmented_path;

use anyhow::{Context, Result};
use std::path::Path;

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
        if app_exists(app) && terminal::try_terminal_with_init(path, cmd, init_script)? {
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
    if terminal::try_terminal_with_init(path, command, init_script)? {
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

    let cmd_str = if command.contains(" . ") || command.ends_with(" .") || command == "." {
        command.replacen(" .", &format!(" {path_str}"), 1)
    } else {
        format!("{command} {path_str}")
    };

    shell::run_shell_command(&cmd_str)
        .with_context(|| format!("Failed to open editor with command: {cmd_str}"))
}
