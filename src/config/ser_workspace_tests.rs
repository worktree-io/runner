use super::*;

#[test]
fn test_workspace_section_present_in_default() {
    let s = Config::default().to_toml_with_comments();
    assert!(s.contains("[workspace]"));
    assert!(s.contains("# Workspace lifecycle configuration."));
    assert!(!s.contains("ttl ="));
}

#[test]
fn test_workspace_section_with_ttl() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    let s = c.to_toml_with_comments();
    assert!(s.contains("[workspace]"));
    assert!(s.contains("ttl ="));
    assert!(s.contains("# Maximum age of a workspace before it is considered expired."));
}

#[test]
fn test_workspace_ttl_round_trips() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.ttl.is_some());
}
