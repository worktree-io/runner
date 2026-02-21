use std::path::PathBuf;

mod parse;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod linear_tests;

#[cfg(test)]
mod uuid_tests;

/// Options extracted from a `worktree://` deep link.
#[derive(Debug, Clone, Default)]
pub struct DeepLinkOptions {
    /// Editor override from the `editor` query param. May be a symbolic name
    /// (`cursor`, `code`, `zed`, `nvim`, etc.) or a raw percent-decoded command.
    pub editor: Option<String>,
}

/// A reference to an issue that identifies a workspace.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueRef {
    GitHub {
        owner: String,
        repo: String,
        number: u64,
    },
    /// A Linear issue identified by its UUID, paired with the GitHub repo that
    /// hosts the code for that project.
    Linear {
        owner: String,
        repo: String,
        id: String,
    },
}

impl IssueRef {
    /// Directory name used inside the bare clone for this worktree.
    pub fn workspace_dir_name(&self) -> String {
        match self {
            Self::GitHub { number, .. } => format!("issue-{number}"),
            Self::Linear { id, .. } => format!("linear-{id}"),
        }
    }

    /// Git branch name for this issue worktree.
    pub fn branch_name(&self) -> String {
        self.workspace_dir_name()
    }

    /// HTTPS clone URL for the repository.
    pub fn clone_url(&self) -> String {
        match self {
            Self::GitHub { owner, repo, .. } | Self::Linear { owner, repo, .. } => {
                format!("https://github.com/{owner}/{repo}.git")
            }
        }
    }

    /// Path to the worktree checkout: `~/worktrees/github/owner/repo/issue-N`
    pub fn temp_path(&self) -> PathBuf {
        self.bare_clone_path().join(self.workspace_dir_name())
    }

    /// Path to the bare clone: `~/worktrees/github/owner/repo`
    pub fn bare_clone_path(&self) -> PathBuf {
        match self {
            Self::GitHub { owner, repo, .. } | Self::Linear { owner, repo, .. } => {
                dirs::home_dir()
                    .expect("could not determine home directory")
                    .join("worktrees")
                    .join("github")
                    .join(owner)
                    .join(repo)
            }
        }
    }
}

