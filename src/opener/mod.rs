mod editor;
mod shell;
mod terminal;

pub use editor::resolve_editor_command;
pub use shell::augmented_path;

use anyhow::{Context, Result};
use std::path::Path;

/// Check whether a macOS application bundle is installed.
#[cfg(target_os = "macos")]
fn app_exists(name: &str) -> bool {
    std::path::Path::new(&format!("/Applications/{name}.app")).exists()
        || std::path::Path::new(&format!("/System/Applications/{name}.app")).exists()
}

/// Check whether Windows Terminal (`wt`) is available on `PATH`.
#[cfg(windows)]
fn wt_exists() -> bool {
    std::process::Command::new("where")
        .arg("wt")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// For the IDE case: find an available terminal app and run `init_script` inside it.
fn open_hook_in_auto_terminal(path: &Path, init_script: &str) -> Result<bool> {
    #[cfg(windows)]
    if wt_exists() && terminal::try_terminal_with_init(path, "wt", init_script)? {
        return Ok(true);
    }

    #[cfg(target_os = "macos")]
    {
        let candidates: &[(&str, &str)] = &[
            ("iTerm", "open -a iTerm ."),
            ("Warp", "open -a Warp ."),
            ("Ghostty", "open -a Ghostty ."),
            ("Terminal", "open -a Terminal ."),
        ];
        for &(app, cmd) in candidates {
            // LLVM_COV_EXCL_START
            if app_exists(app) && terminal::try_terminal_with_init(path, cmd, init_script)? {
                return Ok(true);
            }
            // LLVM_COV_EXCL_STOP
        }
    }

    Ok(false)
}

/// Open `path` with `command` and run `init_script` inside the resulting window.
///
/// # Errors
///
/// Returns an error if the editor or terminal command fails to spawn.
pub fn open_with_hook(path: &Path, command: &str, init_script: &str) -> Result<bool> {
    if terminal::try_terminal_with_init(path, command, init_script)? {
        // LLVM_COV_EXCL_START
        return Ok(true);
        // LLVM_COV_EXCL_STOP
    }
    open_in_editor(path, command)?;
    open_hook_in_auto_terminal(path, init_script)
}

/// Open the workspace path in the configured editor.
///
/// # Errors
///
/// Returns an error if the workspace path is not valid UTF-8 or the editor
/// command fails to spawn.
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    #[test]
    fn test_app_exists_nonexistent() {
        assert!(!app_exists("__NoSuchApp__"));
    }
    #[test]
    fn test_open_with_hook_ide_no_terminal() {
        let p = std::path::Path::new("/tmp");
        // "code ." is not a terminal, and no /Applications/iTerm.app etc in CI
        let _ = open_with_hook(p, "echo .", "true");
    }
    #[test]
    fn test_open_in_editor_dot_substitution() {
        let p = std::path::Path::new("/tmp/myproject");
        open_in_editor(p, "echo .").unwrap();
    }
    #[test]
    fn test_open_in_editor_no_dot() {
        let p = std::path::Path::new("/tmp/myproject");
        open_in_editor(p, "echo").unwrap();
    }
}
