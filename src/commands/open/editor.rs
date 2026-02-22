pub(super) fn resolve_editor_command(name: &str) -> String {
    let candidates: &[(&str, &str)] = &[
        ("cursor", "cursor ."),
        ("code", "code ."),
        ("zed", "zed ."),
        ("subl", "subl ."),
        ("nvim", "nvim ."),
        ("vim", "vim ."),
        ("iterm", "open -a iTerm ."),
        ("iterm2", "open -a iTerm ."),
        ("warp", "open -a Warp ."),
        ("ghostty", "open -a Ghostty ."),
        ("alacritty", "alacritty --working-directory ."),
        ("kitty", "kitty --directory ."),
        ("wezterm", "wezterm start --cwd ."),
        ("wt", "wt -d ."),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_known_editors() {
        assert_eq!(resolve_editor_command("cursor"), "cursor .");
        assert_eq!(resolve_editor_command("CODE"), "code .");
        assert_eq!(resolve_editor_command("nvim"), "nvim .");
        assert_eq!(resolve_editor_command("iterm2"), "open -a iTerm .");
        assert_eq!(resolve_editor_command("warp"), "open -a Warp .");
        assert_eq!(resolve_editor_command("ghostty"), "open -a Ghostty .");
        assert_eq!(
            resolve_editor_command("alacritty"),
            "alacritty --working-directory ."
        );
        assert_eq!(resolve_editor_command("kitty"), "kitty --directory .");
        assert_eq!(resolve_editor_command("wezterm"), "wezterm start --cwd .");
        assert_eq!(resolve_editor_command("wt"), "wt -d .");
        assert_eq!(resolve_editor_command("windowsterminal"), "wt -d .");
    }
    #[test]
    fn test_terminal_macos() {
        let cmd = resolve_editor_command("terminal");
        assert!(!cmd.is_empty());
    }
    #[test]
    fn test_unknown_passthrough() {
        assert_eq!(resolve_editor_command("hx ."), "hx .");
    }
}
