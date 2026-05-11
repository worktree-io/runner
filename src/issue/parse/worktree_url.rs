use crate::issue::{DeepLinkOptions, IssueRef};
use anyhow::{Context, Result};
use url::Url;

use super::worktree_url_params::parse_query_params;

pub(super) fn parse_worktree_url(s: &str) -> Result<(IssueRef, DeepLinkOptions)> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;
    let p = parse_query_params(&url)?;
    let opts = DeepLinkOptions {
        editor: p.editor,
        no_hooks: p.no_hooks,
        extra_env: p.extra_env,
    };
    if let Some(url_str) = p.url_param {
        return Ok((super::github::parse_github_url(&url_str)?, opts));
    }
    if let Some(id) = p.linear_id {
        let owner = p.owner.context("Missing 'owner' query param")?;
        let repo = p.repo.context("Missing 'repo' query param")?;
        return Ok((IssueRef::Linear { owner, repo, id }, opts));
    }
    if let Some(id) = p.ado_work_item_id {
        return Ok((
            super::azure::resolve_worktree_params(p.ado_org, p.ado_project, p.ado_repo, id)?,
            opts,
        ));
    }
    if let Some(issue_key) = p.jira_issue_key {
        return Ok((
            super::jira::resolve_worktree_params(p.jira_host, issue_key, p.owner, p.repo)?,
            opts,
        ));
    }
    if p.gitlab_host.is_some() {
        return Ok((
            IssueRef::GitLab {
                owner: p.owner.context("Missing 'owner' query param")?,
                repo: p.repo.context("Missing 'repo' query param")?,
                number: p.issue_num.context("Missing 'issue' query param")?,
            },
            opts,
        ));
    }
    let owner = p.owner.context("Missing 'owner' query param")?;
    let repo = p.repo.context("Missing 'repo' query param")?;
    let issue = if let Some(number) = p.issue_num {
        IssueRef::GitHub {
            owner,
            repo,
            number,
        }
    } else if let Some(name) = p.adhoc_name {
        IssueRef::Adhoc { owner, repo, name }
    } else {
        let name = crate::name_gen::generate_name();
        IssueRef::Adhoc { owner, repo, name }
    };
    Ok((issue, opts))
}
