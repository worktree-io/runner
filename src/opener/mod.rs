mod shell;
mod terminal;

pub use shell::augmented_path;

use anyhow::{Context, Result};
use std::path::Path;

/// Map a short editor name (e.g. "terminal", "vscode", "cursor") to a shell command.
pub fn resolve_editor_command(name: &str) -> String {
    let candidates: &[(&str, &str)] = &[
        ("vscode", "code ."),
        ("cursor", "cursor ."),
        ("code", "code ."),
        ("zed", "zed ."),
        ("subl", "subl ."),
        ("nvim", "nvim ."),
        ("vim", "vim ."),
        ("iterm", "open -a iTerm ."),
        ("iterm2", "open -a iTerm ."),
        ("warp", "open -a Warp ."),
        ("ghostty", "open -a Ghostty ."),
        ("alacritty", "alacritty --working-directory ."),
        ("kitty", "kitty --directory ."),
        ("wezterm", "wezterm start --cwd ."),
        ("wt", "wt -d ."),
        ("windowsterminal", "wt -d ."),
    ];
    for &(sym, cmd) in candidates {
        if name.eq_ignore_ascii_case(sym) {
            return cmd.to_string();
        }
    }
    if name.eq_ignore_ascii_case("terminal") {
        #[cfg(target_os = "macos")]
        return "open -a Terminal .".to_string();
        #[cfg(target_os = "windows")]
        return "wt -d .".to_string();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return "xterm".to_string();
    }
    name.to_string()
}

/// Check whether a macOS application bundle is installed.
fn app_exists(name: &str) -> bool {
    std::path::Path::new(&format!("/Applications/{name}.app")).exists()
        || std::path::Path::new(&format!("/System/Applications/{name}.app")).exists()
}

/// For the IDE case: find an available terminal app and run `init_script` inside it.
fn open_hook_in_auto_terminal(path: &Path, init_script: &str) -> Result<bool> {
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
    Ok(false)
}

/// Open `path` with `command` and run `init_script` inside the resulting window.
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
