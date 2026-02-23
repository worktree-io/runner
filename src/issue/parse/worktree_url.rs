use anyhow::{bail, Context, Result};
use url::Url;

use crate::issue::{DeepLinkOptions, IssueRef};

pub(super) fn parse_worktree_url(s: &str) -> Result<(IssueRef, DeepLinkOptions)> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;
    let mut owner = None;
    let mut repo = None;
    let mut issue_num = None;
    let mut linear_id = None;
    let mut url_param = None;
    let mut editor = None;
    let mut ado_org = None;
    let mut ado_project = None;
    let mut ado_repo = None;
    let mut ado_work_item_id = None;

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
                url_param = Some(val.into_owned());
            }
            "editor" => editor = Some(val.into_owned()),
            "org" => ado_org = Some(val.into_owned()),
            "project" => ado_project = Some(val.into_owned()),
            "ado_repo" => ado_repo = Some(val.into_owned()),
            "work_item_id" => {
                ado_work_item_id = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid work item ID: {val}"))?,
                );
            }
            _ => {}
        }
    }

    let opts = DeepLinkOptions { editor };

    if let Some(url_str) = url_param {
        return Ok((super::parse_github_url(&url_str)?, opts));
    }

    if let Some(id) = linear_id {
        return Ok((
            IssueRef::Linear {
                owner: owner.context("Missing 'owner' query param")?,
                repo: repo.context("Missing 'repo' query param")?,
                id,
            },
            opts,
        ));
    }

    if let Some(id) = ado_work_item_id {
        let org = ado_org.context("Missing 'org' query param")?;
        let project = ado_project.context("Missing 'project' query param")?;
        let repo = ado_repo.unwrap_or_else(|| project.clone());
        return Ok((
            IssueRef::AzureDevOps {
                org,
                project,
                repo,
                id,
            },
            opts,
        ));
    }

    Ok((
        IssueRef::GitHub {
            owner: owner.context("Missing 'owner' query param")?,
            repo: repo.context("Missing 'repo' query param")?,
            number: issue_num.context("Missing 'issue' query param")?,
        },
        opts,
    ))
}
