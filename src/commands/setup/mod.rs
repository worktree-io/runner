mod detect;
mod prompt;

use anyhow::Result;
use worktree_io::{config::Config, scheme};

pub fn cmd_setup() -> Result<()> {
    let config_path = Config::path()?;
    let already_existed = config_path.exists();
    let mut config = Config::load()?;

    let detected = detect::detect_all_editors();
    match prompt::prompt_editor(&detected) {
        Ok(Some(cmd)) => config.editor.command = Some(cmd),
        Ok(None) => {}
        Err(e) => eprintln!("Warning: could not read editor choice: {e}"),
    }

    if config.hooks.pre_open.is_none() {
        config.hooks.pre_open = Some(
            "#!/usr/bin/env bash\necho \"Opening worktree for {{owner}}/{{repo}}#{{issue}}…\"\n"
                .to_string(),
        );
    }
    if config.hooks.post_open.is_none() {
        config.hooks.post_open = Some(
            "#!/usr/bin/env bash\necho \"Worktree ready: {{owner}}/{{repo}}#{{issue}} ({{branch}})\"\n"
                .to_string(),
        );
    }

    config.save()?;
    if already_existed {
        eprintln!("Updated config at {}", config_path.display());
    } else {
        eprintln!("Created config at {}", config_path.display());
    }

    match scheme::install() {
        Ok(()) => {}
        Err(e) => eprintln!("Warning: could not register URL scheme handler: {e}"),
    }

    eprintln!("\nSetup complete! Run: worktree open <github-issue-url>");
    Ok(())
}
