mod editor;
mod hook_build;
mod hook_ctx;

use anyhow::Result;
use hook_build::{build_hook_context, run_auto_prune};
use hook_ctx::{effective_hooks, launch_editor, load_worktree_io_script};
use worktree_io::{
    config::Config,
    hooks::run_hook,
    issue::{DeepLinkOptions, IssueRef},
    repo_hooks_scaffold::scaffold_if_missing,
    workspace::Workspace,
};

#[allow(clippy::fn_params_excessive_bools)]
pub fn cmd_open(
    issue_ref: Option<&str>,
    force_editor: bool,
    no_hooks: bool,
    headless: bool,
    script: Option<&str>,
    extra_env: Vec<(String, String)>,
    json: bool,
) -> Result<()> {
    let (issue, deep_link_opts) = match issue_ref {
        Some(r) => IssueRef::parse_with_options(r)?,
        None => (IssueRef::from_current_repo()?, DeepLinkOptions::default()),
    };
    // Merge extra_env from CLI with any env pairs from deep link options.
    let mut merged_env = deep_link_opts.extra_env.clone();
    merged_env.extend(extra_env);

    let workspace = Workspace::open_or_create(issue.clone())?;

    if json {
        println!(
            "{{\"path\":{:?},\"created\":{}}}",
            workspace.path.display().to_string(),
            workspace.created
        );
    } else if workspace.created {
        eprintln!("Created workspace at {}", workspace.path.display());
    } else {
        eprintln!("Workspace already exists at {}", workspace.path.display());
    }

    if matches!(scaffold_if_missing(&workspace.path), Ok(true)) && !json {
        eprintln!("created .worktree.toml (no active config — edit to enable hooks)");
    }

    let config = Config::load()?;

    run_auto_prune(&config);
    let mut hook_ctx = build_hook_context(&issue, &workspace.path);
    hook_ctx.extra_env = merged_env;
    let (effective_pre, effective_post) = if let Some(name) = script {
        (None, Some(load_worktree_io_script(&workspace.path, name)?))
    } else if no_hooks || deep_link_opts.no_hooks {
        (None, None)
    } else {
        effective_hooks(&config, &workspace.path)
    };

    if let Some(script) = &effective_pre {
        if !json {
            eprintln!("Running pre:open hook…");
        }
        run_hook(script, &hook_ctx)?;
    }

    if headless {
        if let Some(script) = &effective_post {
            if !json {
                eprintln!("Running post:open hook…");
            }
            run_hook(script, &hook_ctx)?;
        }
        return Ok(());
    }
    let editor_cmd: Option<String> = if let Some(editor_name) = deep_link_opts.editor {
        Some(editor::resolve_editor_command(&editor_name))
    } else if force_editor || config.open.editor {
        if config.editor.command.is_none() {
            eprintln!("No editor configured. Run: worktree setup");
        }
        config.editor.command
    } else {
        None
    };
    launch_editor(
        &workspace.path,
        editor_cmd.as_deref(),
        effective_post.as_deref(),
        config.editor.background,
        &hook_ctx,
    )?;
    Ok(())
}
#[cfg(test)]
#[path = "hook_build_tests.rs"]
mod hook_build_tests;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
