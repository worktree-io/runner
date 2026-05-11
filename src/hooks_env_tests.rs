use super::*;

fn ctx() -> HookContext {
    HookContext {
        owner: "acme".into(),
        repo: "api".into(),
        issue: "42".into(),
        branch: "issue-42".into(),
        worktree_path: "/tmp/wt".into(),
        extra_env: vec![],
    }
}

#[cfg(not(windows))]
#[test]
fn test_run_hook_extra_env_injected() {
    let f = std::env::temp_dir().join("worktree-test-extra-env.txt");
    let p = f.to_str().unwrap().to_string();
    let _ = std::fs::remove_file(&f);
    let ctx_with_env = HookContext {
        extra_env: vec![("MY_TEST_VAR".to_string(), "hello_from_env".to_string())],
        ..ctx()
    };
    run_hook(
        &format!("printf '%s' \"$MY_TEST_VAR\" > '{p}'"),
        &ctx_with_env,
    )
    .unwrap();
    let got = std::fs::read_to_string(&f).unwrap_or_default();
    std::fs::remove_file(&f).ok();
    assert_eq!(
        got, "hello_from_env",
        "extra_env must be injected into hook process"
    );
}

#[cfg(not(windows))]
#[test]
fn test_run_hook_multiple_extra_envs() {
    let f = std::env::temp_dir().join("worktree-test-multi-env.txt");
    let p = f.to_str().unwrap().to_string();
    let _ = std::fs::remove_file(&f);
    let ctx_with_env = HookContext {
        extra_env: vec![
            ("VAR_A".to_string(), "foo".to_string()),
            ("VAR_B".to_string(), "bar".to_string()),
        ],
        ..ctx()
    };
    run_hook(
        &format!("printf '%s:%s' \"$VAR_A\" \"$VAR_B\" > '{p}'"),
        &ctx_with_env,
    )
    .unwrap();
    let got = std::fs::read_to_string(&f).unwrap_or_default();
    std::fs::remove_file(&f).ok();
    assert_eq!(got, "foo:bar", "all extra_env vars must be injected");
}
