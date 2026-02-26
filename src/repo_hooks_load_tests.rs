use super::*;

#[test]
fn test_scaffold_creates_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let created = scaffold_if_missing(dir.path()).unwrap();
    assert!(created);
    let contents = std::fs::read_to_string(dir.path().join(".worktree.toml")).unwrap();
    assert_eq!(contents, SCAFFOLD);
}

#[test]
fn test_scaffold_skips_when_file_present() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".worktree.toml");
    std::fs::write(&path, b"existing").unwrap();
    let created = scaffold_if_missing(dir.path()).unwrap();
    assert!(!created);
    // existing content must be untouched
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "existing");
}

#[test]
fn test_scaffold_is_all_comments_or_blank() {
    for line in SCAFFOLD.lines() {
        let trimmed = line.trim();
        assert!(
            trimmed.is_empty() || trimmed.starts_with('#'),
            "unexpected non-comment line in scaffold: {line:?}"
        );
    }
}

#[test]
fn test_scaffold_parses_as_empty_config() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_if_missing(dir.path()).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert!(cfg.hooks.pre_open.is_none());
    assert!(cfg.hooks.post_open.is_none());
}

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
fn test_load_from_with_hooks() {
    let dir = tempfile::tempdir().unwrap();
    let toml = r#"
[hooks."pre:open"]
script = "npm install"
order = "before"

[hooks."post:open"]
script = "echo done"
order = "after"
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
fn test_load_from_order_defaults_to_before() {
    let dir = tempfile::tempdir().unwrap();
    let toml = "[hooks.\"pre:open\"]\nscript = \"echo hi\"\n";
    std::fs::write(dir.path().join(".worktree.toml"), toml.as_bytes()).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().order, HookOrder::Before);
}

#[test]
fn test_load_from_replace_order() {
    let dir = tempfile::tempdir().unwrap();
    let toml = "[hooks.\"pre:open\"]\nscript = \"cargo build\"\norder = \"replace\"\n";
    std::fs::write(dir.path().join(".worktree.toml"), toml.as_bytes()).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().order, HookOrder::Replace);
}
