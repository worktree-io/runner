use super::*;

#[test]
fn parse_bare_owner_repo() {
    let r = IssueRef::parse("acme/api").unwrap();
    match r {
        IssueRef::Adhoc { owner, repo, name } => {
            assert_eq!(owner, "acme");
            assert_eq!(repo, "api");
            assert!(name.contains('_'), "expected adjective_noun: {name}");
        }
        other => panic!("expected Adhoc, got {other:?}"),
    }
}

#[test]
fn parse_worktree_url_no_issue() {
    let r = IssueRef::parse("worktree://open?owner=acme&repo=api").unwrap();
    match r {
        IssueRef::Adhoc { owner, repo, name } => {
            assert_eq!(owner, "acme");
            assert_eq!(repo, "api");
            assert!(name.contains('_'));
        }
        other => panic!("expected Adhoc, got {other:?}"),
    }
}

#[test]
fn parse_worktree_url_no_issue_with_editor() {
    let (r, opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&editor=cursor").unwrap();
    match r {
        IssueRef::Adhoc { owner, repo, .. } => {
            assert_eq!(owner, "acme");
            assert_eq!(repo, "api");
        }
        other => panic!("expected Adhoc, got {other:?}"),
    }
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn workspace_dir_and_branch() {
    let r = IssueRef::Adhoc {
        owner: "acme".into(),
        repo: "api".into(),
        name: "bold_turing".into(),
    };
    assert_eq!(r.workspace_dir_name(), "bold_turing");
    assert_eq!(r.branch_name(), "bold_turing");
}

#[test]
fn clone_url() {
    let r = IssueRef::Adhoc {
        owner: "acme".into(),
        repo: "api".into(),
        name: "bold_turing".into(),
    };
    assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
}

#[test]
fn paths() {
    let r = IssueRef::Adhoc {
        owner: "acme".into(),
        repo: "api".into(),
        name: "bold_turing".into(),
    };
    assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
    assert!(r
        .temp_path()
        .ends_with("worktrees/github/acme/api/bold_turing"));
}

#[test]
fn multi_dir_name() {
    let r = IssueRef::Adhoc {
        owner: "acme".into(),
        repo: "api".into(),
        name: "bold_turing".into(),
    };
    assert_eq!(r.multi_dir_name(), "api-bold_turing");
}

#[test]
fn parse_worktree_url_adhoc_param() {
    let (r, _opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&adhoc=my-session")
            .unwrap();
    match r {
        IssueRef::Adhoc { owner, repo, name } => {
            assert_eq!(owner, "acme");
            assert_eq!(repo, "api");
            assert_eq!(name, "my-session");
        }
        other => panic!("expected Adhoc, got {other:?}"),
    }
}

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
        other => panic!("expected Adhoc, got {other:?}"),
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
