use anyhow::Result;
use std::path::PathBuf;

use crate::git::{
    bare_clone, branch_exists_remote, create_worktree, detect_default_branch, git_fetch,
};
use crate::issue::IssueRef;

pub struct Workspace {
    pub path: PathBuf,
    pub issue: IssueRef,
    /// true if this call actually created the worktree; false if it already existed
    pub created: bool,
}

impl Workspace {
    /// Open an existing worktree or create a fresh one.
    pub fn open_or_create(issue: IssueRef) -> Result<Self> {
        let worktree_path = issue.temp_path();
        let bare_path = issue.bare_clone_path();

        // Fast path: worktree already exists
        if worktree_path.exists() {
            return Ok(Self {
                path: worktree_path,
                issue,
                created: false,
            });
        }

        // LLVM_COV_EXCL_START
        if !bare_path.exists() {
            eprintln!(
                "Cloning {} (bare) into {}…",
                issue.clone_url(),
                bare_path.display()
            );
            bare_clone(&issue.clone_url(), &bare_path)?;
        } else {
            eprintln!("Fetching origin…");
            git_fetch(&bare_path)?;
        }

        let base_branch = detect_default_branch(&bare_path)?;
        eprintln!("Default branch: {base_branch}");

        let branch = issue.branch_name();
        let branch_exists = branch_exists_remote(&bare_path, &branch);

        eprintln!(
            "Creating worktree {} at {}…",
            branch,
            worktree_path.display()
        );
        create_worktree(
            &bare_path,
            &worktree_path,
            &branch,
            &base_branch,
            branch_exists,
        )?;

        Ok(Self {
            path: worktree_path,
            issue,
            created: true,
        })
        // LLVM_COV_EXCL_STOP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::IssueRef;

    #[test]
    fn test_open_or_create_existing() {
        let issue = IssueRef::GitHub {
            owner: "__test_wt__".into(),
            repo: "__test_wt__".into(),
            number: 9999,
        };
        let path = issue.temp_path();
        std::fs::create_dir_all(&path).unwrap();
        let ws = Workspace::open_or_create(issue).unwrap();
        assert!(!ws.created);
        std::fs::remove_dir_all(&path).ok();
    }
}
