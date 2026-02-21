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
        run_hook(script, &hook_ctx)?;
    }

    if let Some(editor_name) = deep_link_opts.editor {
        let cmd = resolve_editor_command(&editor_name);
        opener::open_in_editor(&workspace.path, &cmd)?;
    } else if force_editor || config.open.editor {
        if let Some(cmd) = &config.editor.command {
            opener::open_in_editor(&workspace.path, cmd)?;
        } else {
            eprintln!("No editor configured. Run: worktree setup");
        }
    }

    if let Some(script) = &config.hooks.post_open {
        run_hook(script, &hook_ctx)?;
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

fn resolve_editor_command(name: &str) -> String {
    let candidates: &[(&str, &str)] = &[
        ("cursor",          "cursor ."),
        ("code",            "code ."),
        ("zed",             "zed ."),
        ("subl",            "subl ."),
        ("nvim",            "nvim ."),
        ("vim",             "vim ."),
        ("iterm",           "open -a iTerm ."),
        ("iterm2",          "open -a iTerm ."),
        ("warp",            "open -a Warp ."),
        ("ghostty",         "open -a Ghostty ."),
        ("alacritty",       "alacritty --working-directory ."),
        ("kitty",           "kitty --directory ."),
        ("wezterm",         "wezterm start --cwd ."),
        ("wt",              "wt -d ."),
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
