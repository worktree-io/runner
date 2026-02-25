use super::IssueRef;

impl IssueRef {
    /// Directory name used inside the bare clone for this worktree.
    #[must_use]
    pub fn workspace_dir_name(&self) -> String {
        match self {
            Self::GitHub { number, .. } | Self::GitLab { number, .. } => {
                format!("issue-{number}")
            }
            Self::Linear { id, .. } => format!("linear-{id}"),
            Self::AzureDevOps { id, .. } => format!("workitem-{id}"),
            Self::Jira { issue_key, .. } => format!("jira-{}", issue_key.to_lowercase()),
            Self::Local { display_number, .. } => format!("issue-{display_number}"),
        }
    }

    /// Git branch name for this issue worktree.
    #[must_use]
    pub fn branch_name(&self) -> String {
        self.workspace_dir_name()
    }

    /// HTTPS clone URL for the repository.
    ///
    /// # Panics
    ///
    /// Always panics for `IssueRef::Local` — local repos are never cloned.
    #[must_use]
    pub fn clone_url(&self) -> String {
        match self {
            Self::GitHub { owner, repo, .. }
            | Self::Linear { owner, repo, .. }
            | Self::Jira { owner, repo, .. } => {
                format!("https://github.com/{owner}/{repo}.git")
            }
            Self::GitLab { owner, repo, .. } => {
                format!("https://gitlab.com/{owner}/{repo}.git")
            }
            Self::AzureDevOps {
                org, project, repo, ..
            } => {
                format!("https://dev.azure.com/{org}/{project}/_git/{repo}")
            }
            Self::Local { .. } => {
                unreachable!("clone_url is never called for IssueRef::Local")
            }
        }
    }
}
