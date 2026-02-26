use std::path::Path;

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

#[cfg(test)]
#[path = "repo_hooks_scaffold_tests.rs"]
mod tests;
