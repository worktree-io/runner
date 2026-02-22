use super::*;

fn ctx() -> HookContext {
    HookContext {
        owner: "acme".into(),
        repo: "api".into(),
        issue: "42".into(),
        branch: "issue-42".into(),
        worktree_path: "/tmp/wt".into(),
    }
}

#[test]
fn test_render_all_placeholders() {
    let out = ctx().render("{{owner}}/{{repo}}#{{issue}} {{branch}} {{worktree_path}}");
    assert_eq!(out, "acme/api#42 issue-42 /tmp/wt");
}

#[test]
fn test_render_no_placeholders() {
    assert_eq!(ctx().render("hello"), "hello");
}

#[test]
fn test_run_hook_success() {
    run_hook("true", &ctx()).unwrap();
}

#[test]
fn test_run_hook_nonzero_exit() {
    run_hook("exit 1", &ctx()).unwrap();
}
