use super::entries::{all_entries, DetectMethod, EditorEntry};

/// Return only the editors/terminals available on the current system.
#[must_use]
pub fn available_entries() -> Vec<EditorEntry> {
    all_entries()
        .into_iter()
        .filter(|e| is_available(&e.detect))
        .collect()
}

fn is_available(method: &DetectMethod) -> bool {
    match method {
        DetectMethod::Path(bin) => which::which(bin).is_ok(),
        #[cfg(target_os = "macos")]
        DetectMethod::MacosApp(app) => macos_app_exists(app),
        DetectMethod::Always => true,
    }
}

#[cfg(target_os = "macos")]
fn macos_app_exists(app_name: &str) -> bool {
    let system = std::path::Path::new("/Applications").join(format!("{app_name}.app"));
    let user = dirs::home_dir().map(|h| h.join("Applications").join(format!("{app_name}.app")));
    system.exists() || user.is_some_and(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_which_sh_exists() {
        assert!(which::which("sh").is_ok());
    }
    #[test]
    fn test_which_nonexistent() {
        assert!(which::which("__no_such_binary_xyz__").is_err());
    }
    #[test]
    fn test_available_entries_returns_vec() {
        let editors = available_entries();
        assert!(editors.iter().any(|e| e.display == "Terminal"));
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_app_exists_system_path() {
        // Safari is always present in /Applications on macOS
        assert!(macos_app_exists("Safari"));
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_app_not_exists_covers_user_closure() {
        // Forces execution of the is_some_and closure (system path absent)
        assert!(!macos_app_exists("__NonExistentApp__xyz123__"));
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_available_macos_app() {
        assert!(is_available(&DetectMethod::MacosApp("Safari")));
        assert!(!is_available(&DetectMethod::MacosApp(
            "__NonExistentApp__xyz123__"
        )));
    }
}
