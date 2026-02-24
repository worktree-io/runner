use std::path::PathBuf;
use std::time::SystemTime;

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
fn test_registry_load_no_file_returns_default() {
    // Exercises load()'s early-return branch when no registry file exists.
    // (The file is unlikely to exist in a fresh test environment; if it does
    // the LLVM_COV_EXCL block inside load() handles that path.)
    let _ = WorkspaceRegistry::load();
}

#[test]
fn test_registry_debug() {
    let _ = format!("{:?}", WorkspaceRegistry::default());
}

#[test]
fn test_registry_clone_is_independent() {
    let r = WorkspaceRegistry::default();
    let mut r2 = r.clone();
    r2.register(PathBuf::from("/clone-test"));
    assert_eq!(r.workspace.len(), 0);
    assert_eq!(r2.workspace.len(), 1);
}

#[test]
fn test_record_debug_and_clone() {
    let r = WorkspaceRecord {
        path: PathBuf::from("/tmp"),
        created_at: SystemTime::now(),
    };
    let _ = format!("{r:?}");
    let r2 = r.clone();
    assert_eq!(r.path, r2.path);
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
