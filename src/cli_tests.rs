use super::*;
use clap::Parser;

fn parse(args: &[&str]) -> Commands {
    Cli::parse_from(std::iter::once("worktree").chain(args.iter().copied())).command
}

#[test]
fn test_open_env_single() {
    let cmd = parse(&["open", "acme/repo#1", "--env", "FOO=bar"]);
    match cmd {
        Commands::Open { env, .. } => {
            assert_eq!(env, vec!["FOO=bar"]);
        }
        _other => unreachable!("unexpected command variant"),
    }
}

#[test]
fn test_open_env_multiple() {
    let cmd = parse(&[
        "open",
        "acme/repo#1",
        "--env",
        "FOO=bar",
        "--env",
        "BAZ=qux",
    ]);
    match cmd {
        Commands::Open { env, .. } => {
            assert_eq!(env, vec!["FOO=bar", "BAZ=qux"]);
        }
        _other => unreachable!("unexpected command variant"),
    }
}

#[test]
fn test_open_env_empty_by_default() {
    let cmd = parse(&["open", "acme/repo#1"]);
    match cmd {
        Commands::Open { env, .. } => {
            assert!(env.is_empty());
        }
        _other => unreachable!("unexpected command variant"),
    }
}

#[test]
fn test_open_json_flag() {
    let cmd = parse(&["open", "acme/repo#1", "--json"]);
    match cmd {
        Commands::Open { json, .. } => {
            assert!(json);
        }
        _other => unreachable!("unexpected command variant"),
    }
}

#[test]
fn test_open_json_false_by_default() {
    let cmd = parse(&["open", "acme/repo#1"]);
    match cmd {
        Commands::Open { json, .. } => {
            assert!(!json);
        }
        _other => unreachable!("unexpected command variant"),
    }
}

#[test]
fn test_open_env_and_json_together() {
    let cmd = parse(&[
        "open",
        "acme/repo#1",
        "--env",
        "KEY=val",
        "--json",
        "--headless",
    ]);
    match cmd {
        Commands::Open {
            env,
            json,
            headless,
            ..
        } => {
            assert_eq!(env, vec!["KEY=val"]);
            assert!(json);
            assert!(headless);
        }
        _other => unreachable!("unexpected command variant"),
    }
}
