use super::*;

#[test]
fn parse_worktree_url_env_params() {
    let (_r, opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&env=FOO:bar&env=BAZ:qux")
            .unwrap();
    assert_eq!(opts.extra_env.len(), 2);
    assert_eq!(opts.extra_env[0], ("FOO".to_string(), "bar".to_string()));
    assert_eq!(opts.extra_env[1], ("BAZ".to_string(), "qux".to_string()));
}

#[test]
fn parse_worktree_url_env_with_adhoc() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?owner=acme&repo=api&adhoc=run-42&env=RUN_ID:abc123",
    )
    .unwrap();
    match r {
        IssueRef::Adhoc { name, .. } => assert_eq!(name, "run-42"),
        other => unreachable!("expected Adhoc, got {other:?}"),
    }
    assert_eq!(opts.extra_env.len(), 1);
    assert_eq!(
        opts.extra_env[0],
        ("RUN_ID".to_string(), "abc123".to_string())
    );
}

#[test]
fn parse_worktree_url_env_malformed_ignored() {
    // env param without colon separator should be silently ignored
    let (_r, opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&env=NOCOLON").unwrap();
    assert!(opts.extra_env.is_empty());
}
