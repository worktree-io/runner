use anyhow::Result;
use worktree_io::{config::Config, scheme};

pub fn cmd_setup() -> Result<()> {
    let config_path = Config::path()?;
    let already_existed = config_path.exists();
    let mut config = Config::load()?;

    let detected = detect_all_editors();
    match prompt_editor(&detected) {
        Ok(Some(cmd)) => config.editor.command = Some(cmd),
        Ok(None) => {}
        Err(e) => eprintln!("Warning: could not read editor choice: {e}"),
    }

    if config.hooks.pre_open.is_none() {
        config.hooks.pre_open = Some(
            "#!/usr/bin/env bash\necho \"pre:open: {{owner}}/{{repo}}#{{issue}} ({{branch}}) at {{worktree_path}}\"\n"
                .to_string(),
        );
    }
    if config.hooks.post_open.is_none() {
        config.hooks.post_open = Some(
            "#!/usr/bin/env bash\necho \"post:open: {{owner}}/{{repo}}#{{issue}} ({{branch}}) at {{worktree_path}}\"\n"
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

/// Probe PATH (and, on macOS, /Applications) for all known editors and terminals.
/// Returns (display name, config command) for each found.
fn detect_all_editors() -> Vec<(&'static str, &'static str)> {
    let path_candidates: &[(&str, &str)] = &[
        ("Cursor",       "cursor ."),
        ("VS Code",      "code ."),
        ("Zed",          "zed ."),
        ("Sublime Text", "subl ."),
        ("Neovim",       "nvim ."),
        ("Vim",          "vim ."),
    ];
    let mut found: Vec<(&str, &str)> = path_candidates.iter()
        .filter(|&&(_, cmd)| which(cmd.split_whitespace().next().unwrap()))
        .copied()
        .collect();

    let terminal_path_candidates: &[(&str, &str)] = &[
        ("Alacritty", "alacritty --working-directory ."),
        ("Kitty",     "kitty --directory ."),
        ("WezTerm",   "wezterm start --cwd ."),
    ];
    for &(name, cmd) in terminal_path_candidates {
        if which(cmd.split_whitespace().next().unwrap()) {
            found.push((name, cmd));
        }
    }

    #[cfg(target_os = "macos")]
    {
        found.push(("Terminal", "open -a Terminal ."));

        let app_candidates: &[(&str, &str, &str)] = &[
            ("iTerm2",  "open -a iTerm .",   "iTerm"),
            ("Warp",    "open -a Warp .",    "Warp"),
            ("Ghostty", "open -a Ghostty .", "Ghostty"),
        ];
        for &(name, cmd, app) in app_candidates {
            if macos_app_exists(app) {
                found.push((name, cmd));
            }
        }
    }

    #[cfg(target_os = "windows")]
    if which("wt") {
        found.push(("Windows Terminal", "wt -d ."));
    }

    found
}

#[cfg(target_os = "macos")]
fn macos_app_exists(app_name: &str) -> bool {
    let system = std::path::Path::new("/Applications").join(format!("{app_name}.app"));
    let user = dirs::home_dir()
        .map(|h| h.join("Applications").join(format!("{app_name}.app")));
    system.exists() || user.map_or(false, |p| p.exists())
}

fn prompt_editor(detected: &[(&str, &str)]) -> Result<Option<String>> {
    use std::io::{BufRead, Write};

    eprintln!("\nSelect your default editor or terminal:");
    for (i, (name, _)) in detected.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, name);
    }
    let custom_idx = detected.len() + 1;
    eprintln!("  {custom_idx}. Enter a custom command");
    eprintln!("  0. Skip (no editor configured)");
    eprint!("Choice [{}]: ", if detected.is_empty() { 0 } else { 1 });
    std::io::stderr().flush().ok();

    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();

    let choice: usize = if trimmed.is_empty() {
        if detected.is_empty() { 0 } else { 1 }
    } else {
        trimmed.parse().unwrap_or(usize::MAX)
    };

    if choice == 0 {
        return Ok(None);
    }
    if choice <= detected.len() {
        return Ok(Some(detected[choice - 1].1.to_string()));
    }
    if choice == custom_idx {
        eprint!("Enter editor command (e.g. \"hx .\"): ");
        std::io::stderr().flush().ok();
        let mut custom = String::new();
        stdin.lock().read_line(&mut custom)?;
        let cmd = custom.trim().to_string();
        return Ok(if cmd.is_empty() { None } else { Some(cmd) });
    }

    eprintln!("Invalid choice, skipping editor configuration.");
    Ok(None)
}

fn which(binary: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path).any(|dir| {
                let candidate = dir.join(binary);
                candidate.is_file() || {
                    #[cfg(target_os = "windows")]
                    { dir.join(format!("{binary}.exe")).is_file() }
                    #[cfg(not(target_os = "windows"))]
                    { false }
                }
            })
        })
        .unwrap_or(false)
}
