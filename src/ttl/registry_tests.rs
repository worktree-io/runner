use std::path::PathBuf;

use super::*;

#[test]
fn test_registry_path_ends_with_workspaces_toml() {
    let p = WorkspaceRegistry::path().unwrap();
    assert!(p.ends_with(".config/worktree/workspaces.toml"));
}

#[test]
fn test_registry_default_is_empty() {
    assert!(WorkspaceRegistry::default().workspace.is_empty());
}

#[test]
fn test_registry_register_adds_entry() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/some/path"));
    assert_eq!(r.workspace.len(), 1);
    assert_eq!(r.workspace[0].path, PathBuf::from("/some/path"));
}

#[test]
fn test_registry_register_idempotent() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/some/path"));
    r.register(PathBuf::from("/some/path"));
    assert_eq!(r.workspace.len(), 1);
}

#[test]
fn test_registry_register_different_paths() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/path/a"));
    r.register(PathBuf::from("/path/b"));
    assert_eq!(r.workspace.len(), 2);
}
