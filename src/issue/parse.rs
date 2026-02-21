use anyhow::{bail, Context, Result};
use url::Url;

use super::{DeepLinkOptions, IssueRef};

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
                    if uuid::Uuid::parse_str(&id).is_err() {
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
            if uuid::Uuid::parse_str(id).is_err() {
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
}
