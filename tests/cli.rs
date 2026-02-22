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
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {:?} failed", args);
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
