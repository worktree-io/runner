use std::path::Path;

use serde::{Deserialize, Serialize};

/// The commented-only scaffold written to `.worktree.toml` when the file is
/// absent. Every line is a comment so the file has no effect until edited.
pub const SCAFFOLD: &str = "# .worktree.toml — per-repo worktree configuration\n\
# Commit this file to version-control to share settings with your team.\n\
\n\
# [hooks]\n\
# Configure lifecycle hooks that run when a worktree for this repo is opened.\n\
# Each hook is a shell command (string) executed in the worktree directory.\n\
# Mustache templating is supported (same variables as global hooks).\n\
#\n\
# Hook ordering controls how a per-repo hook interacts with the global hook\n\
# configured in the runner's main config. Allowed values:\n\
#   \"before\"  — run the per-repo hook first, then the global hook\n\
#   \"after\"   — run the global hook first, then the per-repo hook\n\
#   \"replace\" — skip the global hook entirely, run only the per-repo hook\n\
#\n\
# \"pre:open\" = \"cargo build\"\n\
# \"pre:open:order\" = \"before\"   # default: \"before\"\n\
#\n\
# \"post:open\" = \"npm install\"\n\
# \"post:open:order\" = \"before\"  # default: \"before\"\n";

/// Declares how a per-repo hook relates to the matching global hook.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HookOrder {
    /// Run the per-repo hook first, then the global hook.
    #[default]
    Before,
    /// Run the global hook first, then the per-repo hook.
    After,
    /// Disable the global hook entirely; run only the per-repo hook.
    Replace,
}

/// A single per-repo hook with its script and its ordering relationship to the
/// global hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoHookEntry {
    /// The hook script. Supports the same Mustache-style placeholders as
    /// global hooks: `{{owner}}`, `{{repo}}`, `{{issue}}`, `{{branch}}`,
    /// `{{worktree_path}}`.
    pub script: String,
    /// How this hook relates to the global hook. Defaults to `before`.
    #[serde(default)]
    pub order: HookOrder,
}

/// Per-repo hooks configuration from `.worktree.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoHooksConfig {
    /// Script run before the workspace is opened.
    #[serde(rename = "pre:open", skip_serializing_if = "Option::is_none", default)]
    pub pre_open: Option<RepoHookEntry>,
    /// Script run after the workspace is opened.
    #[serde(rename = "post:open", skip_serializing_if = "Option::is_none", default)]
    pub post_open: Option<RepoHookEntry>,
}

/// Per-repository configuration loaded from `.worktree.toml` in the worktree
/// root.
///
/// The file is version-controlled alongside the repo so that every developer
/// who uses `worktree-io` gets the same lifecycle hooks automatically.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RepoConfig {
    /// Lifecycle hooks scoped to this repository.
    pub hooks: RepoHooksConfig,
}

/// Write [`SCAFFOLD`] to `<worktree_path>/.worktree.toml` if the file does not
/// already exist.
///
/// Returns `true` when the file was created, `false` when it was already
/// present (no write is performed in that case).
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn scaffold_if_missing(worktree_path: &Path) -> std::io::Result<bool> {
    let path = worktree_path.join(".worktree.toml");
    if path.exists() {
        return Ok(false);
    }
    std::fs::write(path, SCAFFOLD)?;
    Ok(true)
}

impl RepoConfig {
    /// Load `.worktree.toml` from `worktree_path`.
    ///
    /// Returns `None` if the file does not exist or cannot be parsed, so the
    /// caller can safely fall back to global-only behavior.
    #[must_use]
    pub fn load_from(worktree_path: &Path) -> Option<Self> {
        let path = worktree_path.join(".worktree.toml");
        let contents = std::fs::read_to_string(path).ok()?;
        toml::from_str(&contents).ok()
    }
}

/// Combine a global hook script with an optional per-repo hook entry into a
/// single effective script.
///
/// | global  | repo entry | result                              |
/// |---------|------------|-------------------------------------|
/// | `None`  | `None`     | `None`                              |
/// | `Some`  | `None`     | global only                         |
/// | `None`  | `Some`     | repo script only                    |
/// | `Some`  | `Some`     | ordered per `entry.order`           |
///
/// When both sides are present the ordering rules apply:
/// * `before` — repo script, newline, global script
/// * `after`  — global script, newline, repo script
/// * `replace` — repo script only (global is suppressed)
#[must_use]
pub fn combined_script(global: Option<&str>, repo_entry: Option<&RepoHookEntry>) -> Option<String> {
    match (global, repo_entry) {
        (None, None) => None,
        (Some(g), None) => Some(g.to_owned()),
        (None, Some(r)) => Some(r.script.clone()),
        (Some(g), Some(r)) => Some(match r.order {
            HookOrder::Before => format!("{}\n{}", r.script, g),
            HookOrder::After => format!("{}\n{}", g, r.script),
            HookOrder::Replace => r.script.clone(),
        }),
    }
}

#[cfg(test)]
#[path = "repo_hooks_load_tests.rs"]
mod load_tests;
#[cfg(test)]
#[path = "repo_hooks_tests.rs"]
mod tests;
