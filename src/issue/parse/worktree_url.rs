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

    Ok((
        IssueRef::GitHub {
            owner: owner.context("Missing 'owner' query param")?,
            repo: repo.context("Missing 'repo' query param")?,
            number: issue_num.context("Missing 'issue' query param")?,
        },
        opts,
    ))
}
