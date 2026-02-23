use super::*;
#[test]
fn test_ide_command_returns_false() {
    let p = std::path::Path::new("/tmp");
    assert!(!try_terminal_with_init(p, "code .", "echo hi").unwrap());
}
#[test]
fn test_unknown_command_returns_false() {
    let p = std::path::Path::new("/tmp");
    assert!(!try_terminal_with_init(p, "hx .", "echo hi").unwrap());
}
