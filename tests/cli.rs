#![allow(missing_docs)]
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_worktree");

fn temp_home(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wt-cli-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn pre_create_workspace(home: &Path, owner: &str, repo: &str, issue: u64) -> PathBuf {
    let wt = home
        .join("worktrees")
        .join("github")
        .join(owner)
        .join(repo)
        .join(format!("issue-{issue}"));
    std::fs::create_dir_all(&wt).unwrap();
    wt
}

fn write_config(home: &Path, toml: &str) {
    let cfg = home.join(".config").join("worktree").join("config.toml");
    std::fs::create_dir_all(cfg.parent().unwrap()).unwrap();
    std::fs::write(cfg, toml).unwrap();
}

fn git_in(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-C", dir.to_str().unwrap()])
        .args(args)
        // Unset inherited git env vars so `-C dir` is honoured even inside
        // a git worktree hook, where GIT_DIR would otherwise override it.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} failed");
}

fn setup_bare_clone(home: &Path, owner: &str, repo: &str) {
    let src = home.join("_src_");
    std::fs::create_dir_all(&src).unwrap();
    git_in(&src, &["init", "-b", "main"]);
    git_in(&src, &["config", "user.email", "t@t.com"]);
    git_in(&src, &["config", "user.name", "T"]);
    std::fs::write(src.join("f"), "x").unwrap();
    git_in(&src, &["add", "."]);
    git_in(&src, &["commit", "-m", "init"]);
    let bare = home.join("worktrees").join("github").join(owner).join(repo);
    std::fs::create_dir_all(&bare).unwrap();
    Command::new("git")
        .args([
            "clone",
            "--bare",
            src.to_str().unwrap(),
            bare.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    git_in(
        &bare,
        &[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    );
    git_in(&bare, &["fetch", "origin"]);
}

#[test]
fn test_config_path() {
    let h = temp_home("cfg_path");
    let out = run(&h, &["config", "path"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("config.toml"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_config_show_default() {
    let h = temp_home("cfg_show");
    let out = run(&h, &["config", "show"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_config_init() {
    let h = temp_home("cfg_init");
    let out = run(&h, &["config", "init"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_config_set_and_get() {
    let h = temp_home("cfg_setget");
    run(&h, &["config", "set", "editor.command", "nvim ."]);
    let out = run(&h, &["config", "get", "editor.command"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "nvim .");
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_config_edit_with_editor() {
    let h = temp_home("cfg_edit_ed");
    write_config(&h, "[editor]\ncommand = \"echo .\"\n");
    let out = run(&h, &["config", "edit"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_config_edit_no_editor() {
    let h = temp_home("cfg_edit_no_ed");
    let out = Command::new(BIN)
        .env("HOME", &h)
        .env_remove("EDITOR")
        .args(["config", "edit"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("No editor configured"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_print_path() {
    let h = temp_home("op_pp");
    pre_create_workspace(&h, "__t__", "__t__", 1);
    let out = run(&h, &["open", "--print-path", "__t__/__t__#1"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout)
        .trim()
        .ends_with("issue-1"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_no_editor() {
    let h = temp_home("op_no_ed");
    pre_create_workspace(&h, "__t__", "__t__", 2);
    let out = run(&h, &["open", "__t__/__t__#2"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_created() {
    let h = temp_home("op_created");
    setup_bare_clone(&h, "__tnew__", "__tnew__");
    let out = run(&h, &["open", "__tnew__/__tnew__#1"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_scaffold_created() {
    let h = temp_home("op_scaffold");
    setup_bare_clone(&h, "__sc__", "__sc__");
    let out = run(&h, &["open", "__sc__/__sc__#1"]);
    assert!(out.status.success());
    let wt = h
        .join("worktrees")
        .join("github")
        .join("__sc__")
        .join("__sc__")
        .join("issue-1");
    let scaffold = wt.join(".worktree.toml");
    assert!(scaffold.exists(), ".worktree.toml should have been created");
    let contents = std::fs::read_to_string(&scaffold).unwrap();
    assert!(
        contents.contains("[hooks]"),
        "scaffold should mention [hooks]"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("created .worktree.toml"),
        "expected scaffold log line in stderr: {stderr}"
    );
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_scaffold_skipped_when_present() {
    let h = temp_home("op_scaffold_skip");
    let wt = pre_create_workspace(&h, "__sk__", "__sk__", 5);
    std::fs::write(wt.join(".worktree.toml"), b"# custom").unwrap();
    let out = run(&h, &["open", "__sk__/__sk__#5"]);
    assert!(out.status.success());
    let contents = std::fs::read_to_string(wt.join(".worktree.toml")).unwrap();
    assert_eq!(
        contents, "# custom",
        "existing .worktree.toml must not be overwritten"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("created .worktree.toml"),
        "should not log scaffold when file already exists"
    );
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_deep_link_editor() {
    let h = temp_home("op_dl_ed");
    pre_create_workspace(&h, "__t__", "__t__", 3);
    let url = "worktree://open?owner=__t__&repo=__t__&issue=3&editor=echo";
    let out = run(&h, &["open", url]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_editor_off() {
    let h = temp_home("op_ed_off");
    pre_create_workspace(&h, "__t__", "__t__", 4);
    write_config(&h, "[open]\neditor = false\n");
    let out = run(&h, &["open", "__t__/__t__#4"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_pre_hook() {
    let h = temp_home("op_pre");
    pre_create_workspace(&h, "__t__", "__t__", 5);
    write_config(&h, "[hooks]\n\"pre:open\" = \"#!/bin/sh\\ntrue\\n\"\n");
    let out = run(&h, &["open", "__t__/__t__#5"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_post_hook_no_editor() {
    let h = temp_home("op_post");
    pre_create_workspace(&h, "__t__", "__t__", 6);
    write_config(
        &h,
        "[open]\neditor = false\n[hooks]\n\"post:open\" = \"#!/bin/sh\\ntrue\\n\"\n",
    );
    let out = run(&h, &["open", "__t__/__t__#6"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_open_editor_and_post_hook() {
    let h = temp_home("op_ed_hook");
    pre_create_workspace(&h, "__t__", "__t__", 7);
    write_config(
        &h,
        "[editor]\ncommand = \"echo .\"\n[hooks]\n\"post:open\" = \"#!/bin/sh\\ntrue\\n\"\n",
    );
    let out = run(&h, &["open", "__t__/__t__#7"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

fn write_registry(home: &Path, entries: &[(&Path, &str)]) {
    let reg = home
        .join(".config")
        .join("worktree")
        .join("workspaces.toml");
    std::fs::create_dir_all(reg.parent().unwrap()).unwrap();
    let content = entries
        .iter()
        .map(|(path, ts)| {
            let escaped = path
                .to_str()
                .unwrap()
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            format!("[[workspace]]\npath = \"{escaped}\"\ncreated_at = \"{ts}\"\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(reg, content).unwrap();
}

#[test]
fn test_prune_no_ttl() {
    let h = temp_home("prune_no_ttl");
    let out = run(&h, &["prune"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("No workspace TTL configured"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_prune_no_expired() {
    let h = temp_home("prune_no_exp");
    write_config(&h, "[workspace]\nttl = \"7days\"\n");
    let out = run(&h, &["prune"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("No expired workspaces found"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_prune_removes_expired() {
    let h = temp_home("prune_exp");
    write_config(&h, "[workspace]\nttl = \"1s\"\n");
    let ws = h.join("old-workspace");
    std::fs::create_dir_all(&ws).unwrap();
    write_registry(&h, &[(&ws, "2000-01-01T00:00:00Z")]);
    let out = run(&h, &["prune"]);
    assert!(out.status.success());
    assert!(!ws.exists());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Pruned 1 expired workspace(s)"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_prune_warns_on_remove_failure() {
    let h = temp_home("prune_fail");
    write_config(&h, "[workspace]\nttl = \"1s\"\n");
    let ws = h.join("not-a-dir");
    std::fs::write(&ws, "file").unwrap();
    write_registry(&h, &[(&ws, "2000-01-01T00:00:00Z")]);
    let out = run(&h, &["prune"]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Warning: failed to remove"));
    assert!(stderr.contains("Pruned 1 expired workspace(s)"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_no_orphans() {
    let h = temp_home("restore_none");
    // Registry is empty — nothing to restore.
    let out = run(&h, &["restore"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("No orphaned worktrees found"));
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_skips_local_worktrees() {
    let h = temp_home("restore_local");
    // Create a registry entry that looks like a local worktree (path doesn't exist).
    let local_path = h
        .join("worktrees")
        .join("local")
        .join("myproject")
        .join("issue-5");
    write_registry(&h, &[(&local_path, "2025-01-01T00:00:00Z")]);
    let out = run(&h, &["restore"]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Skipping local worktree"),
        "expected skip message for local worktree, got: {stderr}"
    );
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_skips_missing_bare_clone() {
    let h = temp_home("restore_no_bare");
    // Orphaned remote worktree whose bare clone no longer exists.
    let wt_path = h
        .join("worktrees")
        .join("github")
        .join("__rb__")
        .join("__rb__")
        .join("issue-1");
    write_registry(&h, &[(&wt_path, "2025-01-01T00:00:00Z")]);
    let out = run(&h, &["restore"]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("bare clone no longer exists"),
        "expected bare-clone-missing message, got: {stderr}"
    );
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_skips_path_with_no_file_name() {
    let h = temp_home("restore_no_fname");
    // A path ending in ".." has no file_name(); the restore command should
    // silently skip it rather than panic.
    let wt_path = std::path::PathBuf::from("/nonexistent_worktree_test_dir/repo/..");
    write_registry(&h, &[(&wt_path, "2025-01-01T00:00:00Z")]);
    let out = run(&h, &["restore"]);
    assert!(out.status.success());
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_non_git_bare_warns_and_fails() {
    let h = temp_home("restore_non_git");
    // A regular directory (not a git repo) acts as the "bare clone".
    let fake_bare = h
        .join("worktrees")
        .join("github")
        .join("__ng__")
        .join("__ng__");
    std::fs::create_dir_all(&fake_bare).unwrap();
    // Orphaned registry entry pointing into the non-git directory.
    let wt_path = fake_bare.join("issue-1");
    write_registry(&h, &[(&wt_path, "2025-01-01T00:00:00Z")]);
    let out = run(&h, &["restore"]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Warning: worktree prune failed"),
        "expected prune warning, got: {stderr}"
    );
    assert!(
        stderr.contains("Failed to restore"),
        "expected failed-restore message, got: {stderr}"
    );
    std::fs::remove_dir_all(&h).ok();
}

#[test]
fn test_restore_recreates_deleted_worktree() {
    let h = temp_home("restore_ok");
    setup_bare_clone(&h, "__rr__", "__rr__");

    // First open to create the worktree.
    let out = run(&h, &["open", "__rr__/__rr__#10"]);
    assert!(out.status.success(), "open failed: {:?}", out.stderr);

    let wt_path = h
        .join("worktrees")
        .join("github")
        .join("__rr__")
        .join("__rr__")
        .join("issue-10");
    assert!(wt_path.exists(), "worktree should exist after open");

    // Simulate manual deletion.
    std::fs::remove_dir_all(&wt_path).unwrap();
    assert!(
        !wt_path.exists(),
        "worktree should be gone after manual delete"
    );

    // Restore should recreate it.
    let out = run(&h, &["restore"]);
    assert!(out.status.success(), "restore failed: {:?}", out.stderr);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Restored"),
        "expected 'Restored' in output, got: {stderr}"
    );
    assert!(wt_path.exists(), "worktree should exist after restore");

    std::fs::remove_dir_all(&h).ok();
}
