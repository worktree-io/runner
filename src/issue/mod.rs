use std::path::PathBuf;

mod parse;
mod paths;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod linear_tests;

#[cfg(test)]
mod azure_tests;

#[cfg(test)]
mod uuid_tests;

#[cfg(test)]
mod parse_tests;

#[cfg(test)]
mod local_tests;

/// Options extracted from a `worktree://` deep link.
#[derive(Debug, Clone, Default)]
pub struct DeepLinkOptions {
    /// Editor override from the `editor` query param. May be a symbolic name
    /// (`cursor`, `code`, `zed`, `nvim`, etc.) or a raw percent-decoded command.
    pub editor: Option<String>,
}

/// A reference to an issue that identifies a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueRef {
    /// A GitHub issue identified by owner, repo, and number.
    GitHub {
        /// GitHub organization or user name.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        number: u64,
    },
    /// A Linear issue identified by its UUID, paired with the GitHub repo that
    /// hosts the code for that project.
    Linear {
        /// GitHub organization or user name that hosts the code.
        owner: String,
        /// Repository name.
        repo: String,
        /// Linear issue UUID.
        id: String,
    },
    /// An Azure DevOps work item paired with an Azure Repos git repository.
    AzureDevOps {
        /// Azure DevOps organization name.
        org: String,
        /// Azure DevOps project name.
        project: String,
        /// Azure Repos git repository name.
        repo: String,
        /// Work item ID.
        id: u64,
    },
    /// A local Centy issue — the repository itself is the source, no remote clone needed.
    Local {
        /// Absolute path to the local project repository.
        project_path: PathBuf,
        /// Human-readable issue number shown in the branch name.
        display_number: u32,
    },
}

impl IssueRef {
    /// Directory name used inside the bare clone for this worktree.
    #[must_use]
    pub fn workspace_dir_name(&self) -> String {
        match self {
            Self::GitHub { number, .. } => format!("issue-{number}"),
            Self::Linear { id, .. } => format!("linear-{id}"),
            Self::AzureDevOps { id, .. } => format!("workitem-{id}"),
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
            Self::GitHub { owner, repo, .. } | Self::Linear { owner, repo, .. } => {
                format!("https://github.com/{owner}/{repo}.git")
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
