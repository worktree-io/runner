mod azure;
mod github;
mod shorthand;
mod worktree_url;

use anyhow::{bail, Result};

use super::{DeepLinkOptions, IssueRef};

impl IssueRef {
    /// Parse any of the supported input formats:
    /// - `https://github.com/owner/repo/issues/42`
    /// - `worktree://open?owner=X&repo=Y&issue=42`
    /// - `worktree://open?url=<encoded-github-url>`
    /// - `worktree://open?owner=X&repo=Y&linear_id=<uuid>`
    /// - `owner/repo#42`
    /// - `owner/repo@<linear-uuid>`
    ///
    /// # Errors
    ///
    /// Returns an error if `s` does not match any supported format or if the
    /// extracted values (e.g. issue number) are invalid.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        if s.starts_with("worktree://") {
            let (issue, _opts) = worktree_url::parse_worktree_url(s)?;
            return Ok(issue);
        }

        if s.starts_with("https://github.com") || s.starts_with("http://github.com") {
            return github::parse_github_url(s);
        }

        if s.starts_with("https://dev.azure.com") || s.starts_with("http://dev.azure.com") {
            return azure::parse_azure_devops_url(s);
        }

        if let Some(result) = shorthand::try_parse_shorthand(s) {
            return result;
        }

        bail!(
            "Could not parse issue reference: {s:?}\n\
             Supported formats:\n\
             - https://github.com/owner/repo/issues/42\n\
             - https://dev.azure.com/org/project/_workitems/edit/42\n\
             - worktree://open?owner=owner&repo=repo&issue=42\n\
             - worktree://open?owner=owner&repo=repo&linear_id=<uuid>\n\
             - worktree://open?org=org&project=project&repo=repo&work_item_id=42\n\
             - owner/repo#42\n\
             - owner/repo@<linear-uuid>\n\
             - org/project/repo!42"
        )
    }

    /// Like [`parse`] but also returns any [`DeepLinkOptions`] embedded in a
    /// `worktree://` URL (e.g. the `editor` query param).
    ///
    /// # Errors
    ///
    /// Returns an error if `s` cannot be parsed as a valid issue reference.
    pub fn parse_with_options(s: &str) -> Result<(Self, DeepLinkOptions)> {
        let s = s.trim();
        if s.starts_with("worktree://") {
            return worktree_url::parse_worktree_url(s);
        }
        Ok((Self::parse(s)?, DeepLinkOptions::default()))
    }
}
