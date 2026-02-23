mod shorthand;
mod worktree_url;

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
            return parse_github_url(s);
        }

        if s.starts_with("https://dev.azure.com") || s.starts_with("http://dev.azure.com") {
            return parse_azure_devops_url(s);
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

/// Parse an Azure DevOps work item URL.
///
/// Expected format: `https://dev.azure.com/{org}/{project}/_workitems/edit/{id}`
///
/// Since the URL does not include the git repository name, the project name is
/// used as the repository name by default.
pub(super) fn parse_azure_devops_url(s: &str) -> Result<IssueRef> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL has no path")?
        .filter(|s| !s.is_empty())
        .collect();

    // Expected: [org, project, "_workitems", "edit", id]
    if segments.len() < 5 || segments[2] != "_workitems" || segments[3] != "edit" {
        bail!(
            "Expected Azure DevOps work item URL like \
             https://dev.azure.com/org/project/_workitems/edit/42, got: {s}"
        );
    }

    let org = segments[0].to_string();
    let project = segments[1].to_string();
    let id = segments[4]
        .parse::<u64>()
        .with_context(|| format!("Invalid work item ID in URL: {}", segments[4]))?;

    Ok(IssueRef::AzureDevOps {
        repo: project.clone(),
        org,
        project,
        id,
    })
}

pub(super) fn parse_github_url(s: &str) -> Result<IssueRef> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL has no path")?
        .filter(|s| !s.is_empty())
        .collect();

    if segments.len() < 4 || segments[2] != "issues" {
        bail!("Expected GitHub issue URL like https://github.com/owner/repo/issues/42, got: {s}");
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let number = segments[3]
        .parse::<u64>()
        .with_context(|| format!("Invalid issue number in URL: {}", segments[3]))?;

    Ok(IssueRef::GitHub {
        owner,
        repo,
        number,
    })
}
