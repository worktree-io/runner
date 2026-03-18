use worktree_io::opener::entries::{all_entries, DetectMethod};

pub(super) fn detect_all_editors() -> Vec<(&'static str, &'static str)> {
    all_entries()
        .into_iter()
        .filter(|e| match &e.detect {
            DetectMethod::Path(bin) => which(bin),
            #[cfg(target_os = "macos")]
            DetectMethod::MacosApp(app) => macos_app_exists(app),
            DetectMethod::Always => true,
        })
        .map(|e| (e.display, e.command))
        .collect()
}

#[cfg(target_os = "macos")]
fn macos_app_exists(app_name: &str) -> bool {
    let system = std::path::Path::new("/Applications").join(format!("{app_name}.app"));
    let user = dirs::home_dir().map(|h| h.join("Applications").join(format!("{app_name}.app")));
    system.exists() || user.is_some_and(|p| p.exists())
}

pub(super) fn which(binary: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|dir| {
            let candidate = dir.join(binary);
            candidate.is_file() || {
                #[cfg(target_os = "windows")]
                {
                    dir.join(format!("{binary}.exe")).is_file()
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_which_sh_exists() {
        assert!(which("sh"));
    }
    #[test]
    fn test_which_nonexistent() {
        assert!(!which("__no_such_binary_xyz__"));
    }
    #[test]
    fn test_detect_all_editors_returns_vec() {
        let editors = detect_all_editors();
        assert!(editors.iter().any(|&(name, _)| name == "Terminal"));
    }
}
