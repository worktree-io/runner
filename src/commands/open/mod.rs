mod editor;

use anyhow::Result;
use worktree_io::{
    config::Config,
    hooks::{run_hook, HookContext},
    issue::IssueRef,
    opener,
    workspace::Workspace,
};

pub fn cmd_open(issue_ref: &str, force_editor: bool, print_path: bool) -> Result<()> {
    let (issue, deep_link_opts) = IssueRef::parse_with_options(issue_ref)?;
    let workspace = Workspace::open_or_create(issue.clone())?;

    if workspace.created {
        eprintln!("Created workspace at {}", workspace.path.display());
    } else {
        eprintln!("Workspace already exists at {}", workspace.path.display());
    }

    if print_path {
        println!("{}", workspace.path.display());
        return Ok(());
    }

    let config = Config::load()?;
    let hook_ctx = build_hook_context(&issue, &workspace.path);

    if let Some(script) = &config.hooks.pre_open {
        eprintln!("Running pre:open hook…");
        run_hook(script, &hook_ctx)?;
    }

    let editor_cmd: Option<String> = if let Some(editor_name) = deep_link_opts.editor {
        Some(editor::resolve_editor_command(&editor_name))
    } else if force_editor || config.open.editor {
        if config.editor.command.is_none() {
            eprintln!("No editor configured. Run: worktree setup");
        }
        config.editor.command.clone()
    } else {
        None
    };

    match (editor_cmd.as_deref(), config.hooks.post_open.as_deref()) {
        (Some(cmd), Some(script)) => {
            let rendered = hook_ctx.render(script);
            if !opener::open_with_hook(&workspace.path, cmd, &rendered)? {
                eprintln!("Running post:open hook…");
                run_hook(script, &hook_ctx)?;
            }
        }
        (Some(cmd), None) => {
            opener::open_in_editor(&workspace.path, cmd)?;
        }
        (None, Some(script)) => {
            eprintln!("Running post:open hook…");
            run_hook(script, &hook_ctx)?;
        }
        (None, None) => {}
    }

    Ok(())
}

fn build_hook_context(issue: &IssueRef, worktree_path: &std::path::Path) -> HookContext {
    let (owner, repo, issue_str) = match issue {
        IssueRef::GitHub { owner, repo, number } => (owner.clone(), repo.clone(), number.to_string()),
        IssueRef::Linear { owner, repo, id } => (owner.clone(), repo.clone(), id.clone()),
    };
    HookContext {
        owner,
        repo,
        issue: issue_str,
        branch: issue.branch_name(),
        worktree_path: worktree_path.to_string_lossy().into_owned(),
    }
}
