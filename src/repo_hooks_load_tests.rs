use super::*;

#[test]
fn test_load_from_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    assert!(RepoConfig::load_from(dir.path()).is_none());
}

#[test]
fn test_load_from_invalid_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".worktree.toml"), b"not valid toml [[[").unwrap();
    assert!(RepoConfig::load_from(dir.path()).is_none());
}

#[test]
fn test_load_from_empty_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".worktree.toml"), b"").unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert!(cfg.hooks.pre_open.is_none());
    assert!(cfg.hooks.post_open.is_none());
}

#[test]
fn test_load_from_with_hooks_under_table() {
    let dir = tempfile::tempdir().unwrap();
    let toml = r#"
[hooks]
"pre:open" = "npm install"
"pre:open:order" = "before"
"post:open" = "echo done"
"post:open:order" = "after"
"#;
    std::fs::write(dir.path().join(".worktree.toml"), toml.as_bytes()).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    let pre = cfg.hooks.pre_open.unwrap();
    assert_eq!(pre.script, "npm install");
    assert_eq!(pre.order, HookOrder::Before);
    let post = cfg.hooks.post_open.unwrap();
    assert_eq!(post.script, "echo done");
    assert_eq!(post.order, HookOrder::After);
}

#[test]
fn test_load_from_user_actual_file_format() {
    // Mirrors the real .worktree.toml a user produces by uncommenting the
    // hook lines from the scaffold without uncommenting the `[hooks]`
    // header. Regression test for the silent-no-op bug.
    let dir = tempfile::tempdir().unwrap();
    let toml = "\"post:open\" = \"cargo build\"\n\"post:open:order\" = \"before\"\n";
    std::fs::write(dir.path().join(".worktree.toml"), toml).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    let post = cfg.hooks.post_open.expect("post:open should parse");
    assert_eq!(post.script, "cargo build");
    assert_eq!(post.order, HookOrder::Before);
}

#[test]
fn test_load_from_order_defaults_to_before() {
    let dir = tempfile::tempdir().unwrap();
    let toml = "\"pre:open\" = \"echo hi\"\n";
    std::fs::write(dir.path().join(".worktree.toml"), toml).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().order, HookOrder::Before);
}

#[test]
fn test_load_from_replace_order() {
    let dir = tempfile::tempdir().unwrap();
    let toml = "\"pre:open\" = \"cargo build\"\n\"pre:open:order\" = \"replace\"\n";
    std::fs::write(dir.path().join(".worktree.toml"), toml).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().order, HookOrder::Replace);
}
