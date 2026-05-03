use super::*;
use std::fs;
use tempfile::TempDir;
use worktree_io::ttl::Ttl;

fn make_script(dir: &TempDir, name: &str, content: &str) {
    let scripts = dir.path().join(".worktree-io");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join(name), content).unwrap();
}

#[test]
fn test_load_worktree_io_script_ok() {
    let dir = TempDir::new().unwrap();
    make_script(&dir, "setup", "echo hello");
    let content = load_worktree_io_script(dir.path(), "setup").unwrap();
    assert_eq!(content, "echo hello");
}

#[test]
fn test_load_worktree_io_script_missing() {
    let dir = TempDir::new().unwrap();
    let err = load_worktree_io_script(dir.path(), "missing").unwrap_err();
    assert!(err.to_string().contains("script not found"));
}

#[test]
fn test_load_worktree_io_script_rejects_traversal() {
    let dir = TempDir::new().unwrap();
    for bad in ["../evil", "foo/bar", ".", ".."] {
        let err = load_worktree_io_script(dir.path(), bad).unwrap_err();
        assert!(
            err.to_string().contains("invalid script name"),
            "expected rejection for {bad:?}, got: {err}"
        );
    }
}

#[test]
fn test_build_hook_ctx_linear() {
    let issue = IssueRef::Linear {
        owner: "a".into(),
        repo: "b".into(),
        id: "X-1".into(),
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.issue, "X-1");
    assert_eq!(ctx.branch, "linear-X-1");
}

#[test]
fn test_build_hook_ctx_azure_devops() {
    let issue = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.owner, "myorg/myproject");
    assert_eq!(ctx.repo, "myrepo");
    assert_eq!(ctx.issue, "42");
    assert_eq!(ctx.branch, "workitem-42");
}

#[test]
fn test_build_hook_ctx_gitlab() {
    let issue = IssueRef::GitLab {
        owner: "myorg".into(),
        repo: "myrepo".into(),
        number: 7,
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.owner, "myorg");
    assert_eq!(ctx.repo, "myrepo");
    assert_eq!(ctx.issue, "7");
}

#[test]
fn test_build_hook_ctx_jira() {
    let issue = IssueRef::Jira {
        host: "acme.atlassian.net".into(),
        issue_key: "PROJ-42".into(),
        owner: "acme".into(),
        repo: "backend".into(),
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.owner, "acme");
    assert_eq!(ctx.repo, "backend");
    assert_eq!(ctx.issue, "PROJ-42");
}

#[test]
fn test_build_hook_ctx_adhoc() {
    let issue = IssueRef::Adhoc {
        owner: "myorg".into(),
        repo: "myrepo".into(),
        name: "bold_turing".into(),
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.owner, "myorg");
    assert_eq!(ctx.repo, "myrepo");
    assert_eq!(ctx.issue, "bold_turing");
}

#[test]
fn test_build_hook_ctx_local() {
    let issue = IssueRef::Local {
        project_path: std::path::PathBuf::from("/home/user/myproject"),
        display_number: 3,
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.owner, "myproject");
    assert_eq!(ctx.repo, "");
    assert_eq!(ctx.issue, "3");
}

#[test]
fn test_run_auto_prune_disabled() {
    let config = Config::default();
    run_auto_prune(&config);
}

#[test]
fn test_run_auto_prune_no_ttl() {
    let mut config = Config::default();
    config.workspace.auto_prune = true;
    run_auto_prune(&config);
}

#[test]
fn test_run_auto_prune_with_ttl_no_expired() {
    let mut config = Config::default();
    config.workspace.auto_prune = true;
    // 1000-year TTL: no real registry entry can be this old, so nothing is pruned
    config.workspace.ttl = Some(Ttl::new(std::time::Duration::from_secs(
        1_000 * 365 * 24 * 3600,
    )));
    run_auto_prune(&config);
}
