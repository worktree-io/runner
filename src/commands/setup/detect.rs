pub(super) fn detect_all_editors() -> Vec<(&'static str, &'static str)> {
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

pub(super) fn which(binary: &str) -> bool {
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
