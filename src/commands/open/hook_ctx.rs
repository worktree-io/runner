use worktree_io::{hooks::HookContext, issue::IssueRef};

pub(super) fn build_hook_context(issue: &IssueRef, worktree_path: &std::path::Path) -> HookContext {
    let (owner, repo, issue_str) = match issue {
        IssueRef::GitHub {
            owner,
            repo,
            number,
        } => (owner.clone(), repo.clone(), number.to_string()),
        IssueRef::Linear { owner, repo, id } => (owner.clone(), repo.clone(), id.clone()),
        IssueRef::AzureDevOps {
            org,
            project,
            repo,
            id,
        } => (format!("{org}/{project}"), repo.clone(), id.to_string()),
        IssueRef::Jira {
            owner,
            repo,
            issue_key,
            ..
        } => (owner.clone(), repo.clone(), issue_key.clone()),
        IssueRef::Local {
            project_path,
            display_number,
        } => {
            let project_name = project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            (project_name, String::new(), display_number.to_string())
        }
    };
    HookContext {
        owner,
        repo,
        issue: issue_str,
        branch: issue.branch_name(),
        worktree_path: worktree_path.to_string_lossy().into_owned(),
    }
}
