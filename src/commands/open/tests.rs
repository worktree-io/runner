use super::*;

#[test]
fn test_build_hook_ctx_linear() {
    let issue = IssueRef::Linear {
        owner: "a".into(),
        repo: "b".into(),
        id: "X-1".into(),
    };
    let ctx = build_hook_context(&issue, std::path::Path::new("/tmp"));
    assert_eq!(ctx.issue, "X-1");
    assert_eq!(ctx.branch, "linear-X-1");
}
