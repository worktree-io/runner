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
    /// Parse any of the supported input formats:
    /// - `https://github.com/owner/repo/issues/42`
    /// - `worktree://open?owner=X&repo=Y&issue=42`
    /// - `worktree://open?url=<encoded-github-url>`
    /// - `worktree://open?owner=X&repo=Y&linear_id=<uuid>`
    /// - `owner/repo#42`
    /// - `owner/repo@<linear-uuid>`
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

        // Try owner/repo#N shorthand or owner/repo@<uuid>
        if let Some(result) = Self::try_parse_shorthand(s) {
            return result;
        }

        bail!(
            "Could not parse issue reference: {s:?}\n\
             Supported formats:\n\
             - https://github.com/owner/repo/issues/42\n\
             - worktree://open?owner=owner&repo=repo&issue=42\n\
             - worktree://open?owner=owner&repo=repo&linear_id=<uuid>\n\
             - owner/repo#42\n\
             - owner/repo@<linear-uuid>"
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
        let mut linear_id = None;
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
                "linear_id" => {
                    let id = val.into_owned();
                    if !is_uuid(&id) {
                        bail!("Invalid Linear issue UUID: {id}");
                    }
                    linear_id = Some(id);
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

        if let Some(id) = linear_id {
            return Ok((
                Self::Linear {
                    owner: owner.context("Missing 'owner' query param")?,
                    repo: repo.context("Missing 'repo' query param")?,
                    id,
                },
                opts,
            ));
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
        // Format: owner/repo#42  or  owner/repo@<linear-uuid>
        if let Some((repo_part, id)) = s.split_once('@') {
            let (owner, repo) = repo_part.split_once('/')?;
            if owner.is_empty() || repo.is_empty() {
                return Some(Err(anyhow::anyhow!("Invalid shorthand format: {s}")));
            }
            if !is_uuid(id) {
                return Some(Err(anyhow::anyhow!(
                    "Invalid Linear issue UUID in shorthand: {id}"
                )));
            }
            return Some(Ok(Self::Linear {
                owner: owner.to_string(),
                repo: repo.to_string(),
                id: id.to_string(),
            }));
        }

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

/// Returns `true` if `s` matches the standard UUID format
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (all hex, case-insensitive).
fn is_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let expected_lengths = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(expected_lengths.iter())
        .all(|(part, &len)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
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
        assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
        assert!(r.temp_path().ends_with("worktrees/github/acme/api/issue-7"));
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

    // --- Linear tests ---

    #[test]
    fn test_parse_linear_shorthand() {
        let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
        let r = IssueRef::parse(&format!("acme/api@{uuid}")).unwrap();
        assert_eq!(
            r,
            IssueRef::Linear {
                owner: "acme".into(),
                repo: "api".into(),
                id: uuid.into(),
            }
        );
    }

    #[test]
    fn test_parse_linear_shorthand_invalid_uuid() {
        let err = IssueRef::parse("acme/api@not-a-uuid").unwrap_err();
        assert!(err.to_string().contains("Invalid Linear issue UUID"));
    }

    #[test]
    fn test_parse_linear_worktree_url() {
        let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
        let url = format!("worktree://open?owner=acme&repo=api&linear_id={uuid}");
        let r = IssueRef::parse(&url).unwrap();
        assert_eq!(
            r,
            IssueRef::Linear {
                owner: "acme".into(),
                repo: "api".into(),
                id: uuid.into(),
            }
        );
    }

    #[test]
    fn test_parse_linear_worktree_url_with_editor() {
        let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
        let url = format!("worktree://open?owner=acme&repo=api&linear_id={uuid}&editor=cursor");
        let (r, opts) = IssueRef::parse_with_options(&url).unwrap();
        assert_eq!(
            r,
            IssueRef::Linear {
                owner: "acme".into(),
                repo: "api".into(),
                id: uuid.into(),
            }
        );
        assert_eq!(opts.editor.as_deref(), Some("cursor"));
    }

    #[test]
    fn test_linear_workspace_dir_name() {
        let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
        let r = IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: uuid.into(),
        };
        assert_eq!(r.workspace_dir_name(), format!("linear-{uuid}"));
        assert_eq!(r.branch_name(), format!("linear-{uuid}"));
    }

    #[test]
    fn test_linear_clone_url() {
        let r = IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: "9cad7a4b-9426-4788-9dbc-e784df999053".into(),
        };
        assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
    }

    #[test]
    fn test_linear_paths() {
        let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
        let r = IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: uuid.into(),
        };
        assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
        assert!(r
            .temp_path()
            .ends_with(format!("worktrees/github/acme/api/linear-{uuid}")));
    }

    #[test]
    fn test_is_uuid_valid() {
        assert!(is_uuid("9cad7a4b-9426-4788-9dbc-e784df999053"));
        assert!(is_uuid("00000000-0000-0000-0000-000000000000"));
        assert!(is_uuid("FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF"));
    }

    #[test]
    fn test_is_uuid_invalid() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid("9cad7a4b-9426-4788-9dbc"));
        assert!(!is_uuid("9cad7a4b94264788-9dbc-e784df999053"));
        assert!(!is_uuid("9cad7a4b-9426-4788-9dbc-e784df99905z")); // 'z' invalid
    }
}
