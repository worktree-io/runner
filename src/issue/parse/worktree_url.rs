use anyhow::{bail, Context, Result};
use url::Url;

use crate::issue::{DeepLinkOptions, IssueRef};

use super::params::UrlParams;

pub(super) fn parse_worktree_url(s: &str) -> Result<(IssueRef, DeepLinkOptions)> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;
    let mut p = UrlParams::default();

    for (key, val) in url.query_pairs() {
        match key.as_ref() {
            "owner" => p.owner = Some(val.into_owned()),
            "repo" => p.repo = Some(val.into_owned()),
            "issue" => {
                p.issue_num = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid issue number: {val}"))?,
                );
            }
            "linear_id" => {
                let id = val.into_owned();
                if uuid::Uuid::parse_str(&id).is_err() {
                    bail!("Invalid Linear issue UUID: {id}");
                }
                p.linear_id = Some(id);
            }
            "url" => p.url_param = Some(val.into_owned()),
            "editor" => p.editor = Some(val.into_owned()),
            "org" => p.ado_org = Some(val.into_owned()),
            "project" => p.ado_project = Some(val.into_owned()),
            "ado_repo" => p.ado_repo = Some(val.into_owned()),
            "work_item_id" => {
                p.ado_work_item_id = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid work item ID: {val}"))?,
                );
            }
            "jira_host" => p.jira_host = Some(val.into_owned()),
            "jira_issue_key" => p.jira_issue_key = Some(val.into_owned()),
            "gitlab_host" => p.gitlab_host = Some(val.into_owned()),
            _ => {}
        }
    }

    let opts = DeepLinkOptions { editor: p.editor };

    if let Some(url_str) = p.url_param {
        return Ok((super::github::parse_github_url(&url_str)?, opts));
    }

    if let Some(id) = p.linear_id {
        return Ok((
            IssueRef::Linear {
                owner: p.owner.context("Missing 'owner' query param")?,
                repo: p.repo.context("Missing 'repo' query param")?,
                id,
            },
            opts,
        ));
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

    Ok((
        IssueRef::GitHub {
            owner: p.owner.context("Missing 'owner' query param")?,
            repo: p.repo.context("Missing 'repo' query param")?,
            number: p.issue_num.context("Missing 'issue' query param")?,
        },
        opts,
    ))
}
