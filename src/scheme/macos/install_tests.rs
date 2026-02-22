use super::*;
#[test]
fn test_applescript_quoted_no_special() {
    assert_eq!(
        applescript_quoted("/usr/bin/worktree"),
        "\"/usr/bin/worktree\""
    );
}
#[test]
fn test_applescript_quoted_backslash() {
    assert_eq!(applescript_quoted("C:\\foo"), "\"C:\\\\foo\"");
}
#[test]
fn test_applescript_quoted_double_quote() {
    assert_eq!(applescript_quoted("say \"hi\""), "\"say \\\"hi\\\"\"");
}
