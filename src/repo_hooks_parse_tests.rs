use super::parse;
use crate::repo_hooks::HookOrder;

#[test]
fn parse_invalid_toml_returns_error() {
    let err = parse("not [valid").unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn parse_empty_toml_yields_no_hooks() {
    let cfg = parse("").unwrap();
    assert!(cfg.hooks.pre_open.is_none());
    assert!(cfg.hooks.post_open.is_none());
}

#[test]
fn parse_root_level_keys() {
    let cfg = parse("\"pre:open\" = \"a\"\n\"post:open\" = \"b\"\n").unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().script, "a");
    assert_eq!(cfg.hooks.post_open.unwrap().script, "b");
}

#[test]
fn parse_only_pre_at_root() {
    let cfg = parse("\"pre:open\" = \"a\"\n").unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().script, "a");
    assert!(cfg.hooks.post_open.is_none());
}

#[test]
fn parse_only_post_at_root() {
    let cfg = parse("\"post:open\" = \"b\"\n").unwrap();
    assert!(cfg.hooks.pre_open.is_none());
    assert_eq!(cfg.hooks.post_open.unwrap().script, "b");
}

#[test]
fn parse_hooks_table_takes_precedence_over_root() {
    let toml = "\"pre:open\" = \"root\"\n[hooks]\n\"pre:open\" = \"nested\"\n";
    let cfg = parse(toml).unwrap();
    assert_eq!(cfg.hooks.pre_open.unwrap().script, "nested");
}

#[test]
fn parse_rejects_hooks_not_a_table() {
    let err = parse("hooks = \"oops\"\n").unwrap_err();
    assert!(err.contains("`hooks` must be a TOML table"));
}

#[test]
fn parse_rejects_non_string_script() {
    let err = parse("\"pre:open\" = 42\n").unwrap_err();
    assert!(err.contains("`pre:open` must be a string"));
}

#[test]
fn parse_rejects_non_string_order() {
    let err = parse("\"pre:open\" = \"x\"\n\"pre:open:order\" = 1\n").unwrap_err();
    assert!(err.contains("`pre:open:order` must be a string"));
}

#[test]
fn parse_rejects_unknown_order_value() {
    let err = parse("\"pre:open\" = \"x\"\n\"pre:open:order\" = \"sometime\"\n").unwrap_err();
    assert!(err.contains("invalid `pre:open:order` value `sometime`"));
}

#[test]
fn parse_order_after() {
    let toml = "\"pre:open\" = \"x\"\n\"pre:open:order\" = \"after\"\n";
    assert_eq!(
        parse(toml).unwrap().hooks.pre_open.unwrap().order,
        HookOrder::After
    );
}

#[test]
fn parse_order_replace() {
    let toml = "\"pre:open\" = \"x\"\n\"pre:open:order\" = \"replace\"\n";
    assert_eq!(
        parse(toml).unwrap().hooks.pre_open.unwrap().order,
        HookOrder::Replace
    );
}

#[test]
fn parse_order_before() {
    let toml = "\"pre:open\" = \"x\"\n\"pre:open:order\" = \"before\"\n";
    assert_eq!(
        parse(toml).unwrap().hooks.pre_open.unwrap().order,
        HookOrder::Before
    );
}
