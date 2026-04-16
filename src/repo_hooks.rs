use std::path::Path;

/// Declares how a per-repo hook relates to the matching global hook.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HookOrder {
    /// Run the per-repo hook first, then the global hook.
    #[default]
    Before,
    /// Run the global hook first, then the per-repo hook.
    After,
    /// Disable the global hook entirely; run only the per-repo hook.
    Replace,
}

/// A single per-repo hook with its script and ordering relationship to the
/// global hook.
#[derive(Debug, Clone)]
pub struct RepoHookEntry {
    /// The hook script. Supports the same Mustache-style placeholders as
    /// global hooks: `{{owner}}`, `{{repo}}`, `{{issue}}`, `{{branch}}`,
    /// `{{worktree_path}}`.
    pub script: String,
    /// How this hook relates to the global hook. Defaults to [`HookOrder::Before`].
    pub order: HookOrder,
}

/// Per-repo hooks configuration parsed from `.worktree.toml`.
#[derive(Debug, Clone, Default)]
pub struct RepoHooksConfig {
    /// Script run before the workspace is opened.
    pub pre_open: Option<RepoHookEntry>,
    /// Script run after the workspace is opened.
    pub post_open: Option<RepoHookEntry>,
}

/// Per-repository configuration loaded from `.worktree.toml` in the worktree
/// root.
///
/// The file is version-controlled alongside the repo so that every developer
/// who uses `worktree-io` gets the same lifecycle hooks automatically.
#[derive(Debug, Clone, Default)]
pub struct RepoConfig {
    /// Lifecycle hooks scoped to this repository.
    pub hooks: RepoHooksConfig,
}

impl RepoConfig {
    /// Load `.worktree.toml` from `worktree_path`.
    ///
    /// Returns `None` when the file is missing. When the file exists but
    /// cannot be parsed, prints a warning to stderr and also returns `None`
    /// so the caller falls back to global-only behavior.
    #[must_use]
    pub fn load_from(worktree_path: &Path) -> Option<Self> {
        let path = worktree_path.join(".worktree.toml");
        let contents = std::fs::read_to_string(&path).ok()?;
        match crate::repo_hooks_parse::parse(&contents) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                eprintln!("warning: ignoring {}: {e}", path.display());
                None
            }
        }
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
