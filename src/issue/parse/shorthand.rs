use anyhow::Result;

use crate::issue::IssueRef;

pub(super) fn try_parse_shorthand(s: &str) -> Option<Result<IssueRef>> {
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
        return Some(Ok(IssueRef::Linear {
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
        Err(_) => {
            return Some(Err(anyhow::anyhow!(
                "Invalid issue number in shorthand: {num_str}"
            )))
        }
    };

    Some(Ok(IssueRef::GitHub {
        owner: owner.to_string(),
        repo: repo.to_string(),
        number,
    }))
}
