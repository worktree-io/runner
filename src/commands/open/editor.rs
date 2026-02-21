pub(super) fn resolve_editor_command(name: &str) -> String {
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
