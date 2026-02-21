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

        // Ensure the bare clone exists
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
    }
}
