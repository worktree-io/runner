use std::path::PathBuf;

use super::IssueRef;

impl IssueRef {
    /// Path to the worktree checkout.
    ///
    /// For `Local`: `~/worktrees/local/{project_name}/issue-{display_number}`
    /// For others:  `~/worktrees/github/{owner}/{repo}/issue-N`
    #[must_use]
    pub fn temp_path(&self) -> PathBuf {
        self.bare_clone_path().join(self.workspace_dir_name())
    }

    /// Path to the bare clone (or the local repo itself for `Local`).
    ///
    /// # Panics
    ///
    /// Panics if the home directory cannot be determined.
    #[must_use]
    pub fn bare_clone_path(&self) -> PathBuf {
        match self {
            Self::GitHub { owner, repo, .. } | Self::Linear { owner, repo, .. } => dirs::home_dir()
                .expect("could not determine home directory")
                .join("worktrees")
                .join("github")
                .join(owner)
                .join(repo),
            Self::Local { project_path, .. } => {
                let project_name = project_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                dirs::home_dir()
                    .expect("could not determine home directory")
                    .join("worktrees")
                    .join("local")
                    .join(project_name.as_ref())
            }
        }
    }
}
