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

#[test]
fn test_render_all_placeholders() {
    let out = ctx().render("{{owner}}/{{repo}}#{{issue}} {{branch}} {{worktree_path}}");
    assert_eq!(out, "acme/api#42 issue-42 /tmp/wt");
}

#[test]
fn test_render_no_placeholders() {
    assert_eq!(ctx().render("hello"), "hello");
}

#[cfg(not(windows))]
#[test]
fn test_run_hook_success() {
    run_hook("true", &ctx()).unwrap();
}

#[cfg(windows)]
#[test]
fn test_run_hook_success_windows() {
    run_hook("@echo off\r\necho hello", &ctx()).unwrap();
}

#[cfg(not(windows))]
#[test]
fn test_run_hook_nonzero_exit() {
    run_hook("exit 1", &ctx()).unwrap();
}

#[cfg(windows)]
#[test]
fn test_run_hook_nonzero_exit_windows() {
    run_hook("@echo off\r\nexit /b 1", &ctx()).unwrap();
}

/// Regression test: two hooks running concurrently (same process, different
/// threads — identical PID) must not overwrite each other's temp script.
/// With the old PID-based filename both threads wrote to the same file, so
/// one hook silently executed the other's rendered script.
#[cfg(not(windows))]
#[test]
fn test_concurrent_hooks_use_distinct_contexts() {
    let out_a = std::env::temp_dir().join("worktree-test-concurrent-a.txt");
    let out_b = std::env::temp_dir().join("worktree-test-concurrent-b.txt");
    let _ = std::fs::remove_file(&out_a);
    let _ = std::fs::remove_file(&out_b);

    let path_a = out_a.to_str().unwrap().to_string();
    let path_b = out_b.to_str().unwrap().to_string();

    // {{issue}} is rendered to the actual number before the script runs.
    let script_a = format!("printf '%s' '{{{{issue}}}}' > '{path_a}'");
    let script_b = format!("printf '%s' '{{{{issue}}}}' > '{path_b}'");

    let ctx_a = HookContext {
        issue: "159".into(),
        ..ctx()
    };
    let ctx_b = HookContext {
        issue: "129".into(),
        ..ctx()
    };

    let h1 = std::thread::spawn(move || run_hook(&script_a, &ctx_a).unwrap());
    let h2 = std::thread::spawn(move || run_hook(&script_b, &ctx_b).unwrap());
    h1.join().unwrap();
    h2.join().unwrap();

    let val_a = std::fs::read_to_string(&out_a).unwrap_or_default();
    let val_b = std::fs::read_to_string(&out_b).unwrap_or_default();
    let _ = std::fs::remove_file(&out_a);
    let _ = std::fs::remove_file(&out_b);

    assert_eq!(val_a, "159", "hook A ran with wrong issue context");
    assert_eq!(val_b, "129", "hook B ran with wrong issue context");
}

#[cfg(not(windows))]
#[test]
fn test_run_hook_multiline() {
    let f = std::env::temp_dir().join("worktree-test-multiline.txt");
    let p = f.to_str().unwrap().to_string();
    run_hook(&format!("printf a > '{p}'\nprintf b >> '{p}'\n"), &ctx()).unwrap();
    let got = std::fs::read_to_string(&f).unwrap_or_default();
    std::fs::remove_file(&f).ok();
    assert_eq!(got, "ab", "all lines of a multi-line hook must run");
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
