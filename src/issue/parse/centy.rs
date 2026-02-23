use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::issue::IssueRef;

/// Walk up from `start` looking for a directory that contains a `.centy/` subdirectory.
///
/// Returns the first ancestor (inclusive) that has `.centy/`, or `None` if none is found.
pub(super) fn find_centy_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".centy").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None; // LLVM_COV_EXCL_LINE
        }
    }
}

/// Parse a `centy:<number>` shorthand into an [`IssueRef::Local`].
///
/// Walks up from the current directory to find the nearest `.centy/` ancestor.
///
/// # Errors
///
/// Returns an error if the number is not a valid positive integer, the current directory
/// cannot be determined, or no `.centy/` directory is found in the current directory or
/// any of its ancestors.
pub(super) fn parse_centy(s: &str) -> Result<IssueRef> {
    let id_str = s
        .strip_prefix("centy:")
        .expect("caller checked starts_with(\"centy:\")");
    let Ok(display_number) = id_str.parse::<u32>() else {
        return Err(anyhow::anyhow!(
            "Invalid Centy issue number: {id_str:?} — expected a positive integer"
        ));
    };
    // LLVM_COV_EXCL_START
    let cwd = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Could not determine current directory: {e}"))?;
    let Some(project_path) = find_centy_root(&cwd) else {
        return Err(anyhow::anyhow!(
            "No Centy project found: could not find a .centy/ directory in {} or any parent",
            cwd.display()
        ));
    };
    Ok(IssueRef::Local {
        project_path,
        display_number,
    })
    // LLVM_COV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_centy_root_in_start_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".centy")).unwrap();
        assert_eq!(find_centy_root(dir.path()), Some(dir.path().to_path_buf()));
    }

    #[test]
    fn find_centy_root_in_parent() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(".centy")).unwrap();
        let child = root.path().join("sub").join("dir");
        fs::create_dir_all(&child).unwrap();
        assert_eq!(find_centy_root(&child), Some(root.path().to_path_buf()));
    }

    #[test]
    fn parse_centy_invalid_number() {
        let err = parse_centy("centy:abc").unwrap_err();
        assert!(err.to_string().contains("Invalid Centy issue number"));
    }
}
