//! Handler for the `open-multi` subcommand.
use anyhow::{bail, Result};
use worktree_io::{
    config::Config,
    hooks::{run_hook, HookContext},
    issue::IssueRef,
    multi_workspace::create_multi_workspace,
    opener,
};

/// Open multiple repos as a single unified workspace.
///
/// `refs` is a list of issue references such as `"acme/backend#7"` or a full
/// GitHub issue URL.  Each reference is parsed independently; all worktrees
/// are created under `~/workspaces/<random-name>/`.
///
/// # Errors
///
/// Returns an error if any reference cannot be parsed, any repo cannot be
/// cloned, or the workspace root cannot be created.
// LLVM_COV_EXCL_START
pub fn cmd_open_multi(refs: &[String]) -> Result<()> {
    if refs.len() < 2 {
        bail!("open-multi requires at least two issue references");
    }
    let specs = refs
        .iter()
        .map(|r| IssueRef::parse_with_options(r).map(|(issue, _)| issue))
        .collect::<Result<Vec<_>>>()?;

    let workspaces_root = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?
        .join("workspaces");

    let root = create_multi_workspace(&specs, &workspaces_root)?;
    eprintln!("Created unified workspace at {}", root.display());

    let config = Config::load()?;

    let hook_ctx = HookContext {
        owner: String::new(),
        repo: String::new(),
        issue: String::new(),
        branch: String::new(),
        worktree_path: root.to_string_lossy().into_owned(),
    };

    if let Some(script) = &config.hooks.pre_open {
        eprintln!("Running pre:open hook…");
        run_hook(script, &hook_ctx)?;
    }

    let editor_cmd = if config.open.editor {
        config.editor.command
    } else {
        None
    };

    match (editor_cmd.as_deref(), config.hooks.post_open.as_deref()) {
        (Some(cmd), Some(script)) => {
            let rendered = hook_ctx.render(script);
            if !opener::open_with_hook(&root, cmd, &rendered)? {
                eprintln!("Running post:open hook…");
                run_hook(script, &hook_ctx)?;
            }
        }
        (Some(cmd), None) => opener::open_in_editor(&root, cmd)?,
        (None, Some(script)) => {
            eprintln!("Running post:open hook…");
            run_hook(script, &hook_ctx)?;
        }
        (None, None) => {}
    }

    Ok(())
}
// LLVM_COV_EXCL_STOP
