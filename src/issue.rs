use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use url::Url;

/// Options extracted from a `worktree://` deep link.
#[derive(Debug, Clone, Default)]
pub struct DeepLinkOptions {
    /// Editor override from the `editor` query param. May be a symbolic name
    /// (`cursor`, `code`, `zed`, `nvim`, etc.) or a raw percent-decoded command.
    pub editor: Option<String>,
}

/// A reference to a GitHub issue that identifies a workspace.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueRef {
    GitHub {
        owner: String,
        repo: String,
        number: u64,
    },
}

impl IssueRef {
    /// Parse any of the supported input formats:
    /// - `https://github.com/owner/repo/issues/42`
    /// - `worktree://open?owner=X&repo=Y&issue=42`
    /// - `worktree://open?url=<encoded-github-url>`
    /// - `owner/repo#42`
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // Try worktree:// scheme first
        if s.starts_with("worktree://") {
            let (issue, _opts) = Self::parse_worktree_url(s)?;
            return Ok(issue);
        }

        // Try https://github.com URL
        if s.starts_with("https://github.com") || s.starts_with("http://github.com") {
            return Self::parse_github_url(s);
        }

        // Try owner/repo#N shorthand
        if let Some(result) = Self::try_parse_shorthand(s) {
            return result;
        }

        bail!(
            "Could not parse issue reference: {s:?}\n\
             Supported formats:\n\
             - https://github.com/owner/repo/issues/42\n\
             - worktree://open?owner=owner&repo=repo&issue=42\n\
             - owner/repo#42"
        )
    }

    /// Like [`parse`] but also returns any [`DeepLinkOptions`] embedded in a
    /// `worktree://` URL (e.g. the `editor` query param).
    pub fn parse_with_options(s: &str) -> Result<(Self, DeepLinkOptions)> {
        let s = s.trim();
        if s.starts_with("worktree://") {
            return Self::parse_worktree_url(s);
        }
        Ok((Self::parse(s)?, DeepLinkOptions::default()))
    }

    fn parse_worktree_url(s: &str) -> Result<(Self, DeepLinkOptions)> {
        let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;
        let mut owner = None;
        let mut repo = None;
        let mut issue_num = None;
        let mut url_param = None;
        let mut editor = None;

        for (key, val) in url.query_pairs() {
            match key.as_ref() {
                "owner" => owner = Some(val.into_owned()),
                "repo" => repo = Some(val.into_owned()),
                "issue" => {
                    issue_num = Some(
                        val.parse::<u64>()
                            .with_context(|| format!("Invalid issue number: {val}"))?,
                    );
                }
                "url" => {
                    // query_pairs() already percent-decodes the value for us
                    url_param = Some(val.into_owned());
                }
                "editor" => editor = Some(val.into_owned()),
                _ => {}
            }
        }

        let opts = DeepLinkOptions { editor };

        if let Some(url_str) = url_param {
            return Ok((Self::parse_github_url(&url_str)?, opts));
        }

        Ok((
            Self::GitHub {
                owner: owner.context("Missing 'owner' query param")?,
                repo: repo.context("Missing 'repo' query param")?,
                number: issue_num.context("Missing 'issue' query param")?,
            },
            opts,
        ))
    }

    fn parse_github_url(s: &str) -> Result<Self> {
        let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

        let segments: Vec<&str> = url
            .path_segments()
            .context("URL has no path")?
            .filter(|s| !s.is_empty())
            .collect();

        // Expect: owner / repo / "issues" / number
        if segments.len() < 4 || segments[2] != "issues" {
            bail!(
                "Expected GitHub issue URL like https://github.com/owner/repo/issues/42, got: {s}"
            );
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();
        let number = segments[3]
            .parse::<u64>()
            .with_context(|| format!("Invalid issue number in URL: {}", segments[3]))?;

        Ok(Self::GitHub { owner, repo, number })
    }

    fn try_parse_shorthand(s: &str) -> Option<Result<Self>> {
        // Format: owner/repo#42
        let (repo_part, num_str) = s.split_once('#')?;
        let (owner, repo) = repo_part.split_once('/')?;

        if owner.is_empty() || repo.is_empty() {
            return Some(Err(anyhow::anyhow!("Invalid shorthand format: {s}")));
        }

        let number = match num_str.parse::<u64>() {
            Ok(n) => n,
            Err(_) => return Some(Err(anyhow::anyhow!("Invalid issue number in shorthand: {num_str}"))),
        };

        Some(Ok(Self::GitHub {
            owner: owner.to_string(),
            repo: repo.to_string(),
            number,
        }))
    }

    /// Directory name used inside the bare clone for this worktree.
    pub fn workspace_dir_name(&self) -> String {
        match self {
            Self::GitHub { number, .. } => format!("issue-{number}"),
        }
    }

    /// Git branch name for this issue worktree.
    pub fn branch_name(&self) -> String {
        self.workspace_dir_name()
    }

    /// HTTPS clone URL for the repository.
    pub fn clone_url(&self) -> String {
        match self {
            Self::GitHub { owner, repo, .. } => {
                format!("https://github.com/{owner}/{repo}.git")
            }
        }
    }

    /// Path to the worktree checkout: `$TMPDIR/worktree-io/github/owner/repo/issue-N`
    pub fn temp_path(&self) -> PathBuf {
        self.bare_clone_path().join(self.workspace_dir_name())
    }

    /// Path to the bare clone: `$TMPDIR/worktree-io/github/owner/repo`
    pub fn bare_clone_path(&self) -> PathBuf {
        match self {
            Self::GitHub { owner, repo, .. } => std::env::temp_dir()
                .join("worktree-io")
                .join("github")
                .join(owner)
                .join(repo),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorthand() {
        let r = IssueRef::parse("owner/repo#42").unwrap();
        assert_eq!(
            r,
            IssueRef::GitHub {
                owner: "owner".into(),
                repo: "repo".into(),
                number: 42
            }
        );
    }

    #[test]
    fn test_parse_github_url() {
        let r = IssueRef::parse("https://github.com/microsoft/vscode/issues/12345").unwrap();
        assert_eq!(
            r,
            IssueRef::GitHub {
                owner: "microsoft".into(),
                repo: "vscode".into(),
                number: 12345
            }
        );
    }

    #[test]
    fn test_parse_worktree_url() {
        let r = IssueRef::parse("worktree://open?owner=acme&repo=api&issue=7").unwrap();
        assert_eq!(
            r,
            IssueRef::GitHub {
                owner: "acme".into(),
                repo: "api".into(),
                number: 7
            }
        );
    }

    #[test]
    fn test_parse_worktree_url_with_editor_symbolic() {
        let (r, opts) =
            IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&issue=42&editor=cursor")
                .unwrap();
        assert_eq!(
            r,
            IssueRef::GitHub {
                owner: "acme".into(),
                repo: "api".into(),
                number: 42,
            }
        );
        assert_eq!(opts.editor.as_deref(), Some("cursor"));
    }

    #[test]
    fn test_parse_worktree_url_with_editor_raw_command() {
        let (r, opts) =
            IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&issue=42&editor=my-editor%20.")
                .unwrap();
        assert_eq!(r, IssueRef::GitHub { owner: "acme".into(), repo: "api".into(), number: 42 });
        assert_eq!(opts.editor.as_deref(), Some("my-editor ."));
    }

    #[test]
    fn test_parse_with_options_no_editor() {
        let (_r, opts) =
            IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&issue=42").unwrap();
        assert!(opts.editor.is_none());
    }

    #[test]
    fn test_parse_with_options_non_deep_link() {
        let (_r, opts) = IssueRef::parse_with_options("acme/api#42").unwrap();
        assert!(opts.editor.is_none());
    }

    #[test]
    fn test_paths() {
        let r = IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 7,
        };
        assert!(r.bare_clone_path().ends_with("worktree-io/github/acme/api"));
        assert!(r.temp_path().ends_with("worktree-io/github/acme/api/issue-7"));
    }

    #[test]
    fn test_clone_url() {
        let r = IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 7,
        };
        assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
    }
}
