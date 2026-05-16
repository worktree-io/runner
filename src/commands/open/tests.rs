use super::*;
use std::fs;
use tempfile::TempDir;

fn make_script(dir: &TempDir, name: &str, content: &str) {
    let scripts = dir.path().join(".worktree");
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
