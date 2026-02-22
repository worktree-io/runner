use anyhow::{bail, Context, Result};
use std::process::{Command, Stdio};

/// Return the `PATH` string augmented with common homebrew and system locations.
#[must_use]
pub fn augmented_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let extras = ["/usr/local/bin", "/opt/homebrew/bin", "/opt/homebrew/sbin"];
    let mut parts: Vec<&str> = extras.to_vec();
    for p in current.split(':').filter(|s| !s.is_empty()) {
        if !parts.contains(&p) {
            parts.push(p);
        }
    }
    parts.join(":")
}

pub(super) fn run_shell_command(cmd: &str) -> Result<()> {
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

fn shlex_split(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in s.chars() {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shlex_simple() {
        assert_eq!(
            shlex_split("git commit -m init"),
            vec!["git", "commit", "-m", "init"]
        );
    }
    #[test]
    fn test_shlex_quoted() {
        assert_eq!(
            shlex_split(r#"git commit -m "hello world""#),
            vec!["git", "commit", "-m", "hello world"]
        );
    }
    #[test]
    fn test_shlex_tabs() {
        assert_eq!(shlex_split("a\tb"), vec!["a", "b"]);
    }
    #[test]
    fn test_shlex_empty() {
        assert!(shlex_split("").is_empty());
    }
    #[test]
    fn test_augmented_path_contains_homebrew() {
        let p = augmented_path();
        assert!(p.contains("/opt/homebrew/bin"));
    }
    #[test]
    fn test_run_shell_command_empty() {
        assert!(run_shell_command("").is_err());
    }
    #[test]
    fn test_run_shell_command_success() {
        run_shell_command("true").unwrap();
    }
    #[test]
    fn test_run_shell_command_bad_program() {
        assert!(run_shell_command("__nonexistent_xyz_wt__").is_err());
    }
}
