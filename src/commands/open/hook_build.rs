use std::time::SystemTime;
use worktree_io::{
    config::Config,
    hooks::HookContext,
    issue::IssueRef,
    ttl::{self, WorkspaceRegistry},
};

pub(super) fn build_hook_context(issue: &IssueRef, worktree_path: &std::path::Path) -> HookContext {
    let (owner, repo, issue_str) = match issue {
        IssueRef::GitHub {
            owner,
            repo,
            number,
        }
        | IssueRef::GitLab {
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
        IssueRef::Adhoc {
            owner, repo, name, ..
        } => (owner.clone(), repo.clone(), name.clone()),
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
        extra_env: vec![],
    }
}

pub(super) fn run_auto_prune(config: &Config) {
    if config.workspace.auto_prune {
        if let Some(ttl_val) = &config.workspace.ttl {
            if let Ok(mut registry) = WorkspaceRegistry::load() {
                let now = SystemTime::now();
                let expired = ttl::prune(&registry.workspace, ttl_val, now);
                let expired_paths: Vec<_> = expired.iter().map(|r| r.path.clone()).collect();
                for path in &expired_paths {
                    eprintln!("Pruning expired workspace at {}…", path.display());
                    let _ = std::fs::remove_dir_all(path);
                }
                registry
                    .workspace
                    .retain(|r| !expired_paths.contains(&r.path));
                let _ = registry.save();
            }
        }
    }
}
