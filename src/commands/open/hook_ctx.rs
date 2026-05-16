use anyhow::{bail, Context, Result};
use std::path::{Component, Path};
use worktree_io::{
    config::Config,
    hooks::{run_hook, HookContext},
    opener,
    repo_hooks::{combined_script, RepoConfig},
};

pub(super) fn load_worktree_io_script(worktree_path: &Path, name: &str) -> Result<String> {
    let components: Vec<_> = Path::new(name).components().collect();
    if !matches!(components.as_slice(), [Component::Normal(_)]) {
        bail!("invalid script name {name:?} — must be a plain filename, no path separators");
    }
    let script_path = worktree_path.join(".worktree").join(name);
    std::fs::read_to_string(&script_path)
        .with_context(|| format!("script not found: {}", script_path.display()))
}

pub(super) fn effective_hooks(
    config: &Config,
    workspace_path: &std::path::Path,
) -> (Option<String>, Option<String>) {
    let repo = RepoConfig::load_from(workspace_path).unwrap_or_default();
    let pre = combined_script(
        config.hooks.pre_open.as_deref(),
        repo.hooks.pre_open.as_ref(),
    );
    let post = combined_script(
        config.hooks.post_open.as_deref(),
        repo.hooks.post_open.as_ref(),
    );
    (pre, post)
}

pub(super) fn launch_editor(
    workspace: &std::path::Path,
    cmd: Option<&str>,
    post_hook: Option<&str>,
    background: bool,
    ctx: &HookContext,
) -> Result<()> {
    match (cmd, post_hook) {
        (Some(c), Some(s)) => {
            let rendered = ctx.render(s);
            if !opener::open_with_hook(workspace, c, &rendered, background)? {
                eprintln!("Running post:open hook…");
                run_hook(s, ctx)?;
            }
        }
        (Some(c), None) => {
            opener::open_editor_or_terminal(workspace, c, background)?;
        }
        (None, Some(s)) => {
            eprintln!("Running post:open hook…");
            run_hook(s, ctx)?;
        }
        (None, None) => {}
    }
    Ok(())
}
