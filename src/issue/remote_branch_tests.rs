use super::*;

fn github_ref() -> IssueRef {
    IssueRef::RemoteBranch {
        host: "github".into(),
        owner: "acme".into(),
        repo: "api".into(),
        branch: "feature-x".into(),
    }
}

fn gitlab_ref() -> IssueRef {
    IssueRef::RemoteBranch {
        host: "gitlab".into(),
        owner: "acme".into(),
        repo: "web".into(),
        branch: "fix-login".into(),
    }
}

#[test]
fn workspace_dir_name_is_branch() {
    assert_eq!(github_ref().workspace_dir_name(), "feature-x");
    assert_eq!(gitlab_ref().workspace_dir_name(), "fix-login");
}

#[test]
fn branch_name_matches_workspace_dir_name() {
    assert_eq!(github_ref().branch_name(), "feature-x");
}

#[test]
fn clone_url_github() {
    assert_eq!(github_ref().clone_url(), "https://github.com/acme/api.git");
}

#[test]
fn clone_url_gitlab() {
    assert_eq!(gitlab_ref().clone_url(), "https://gitlab.com/acme/web.git");
}

#[test]
fn bare_clone_path_github() {
    let r = github_ref();
    assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
}

#[test]
fn bare_clone_path_gitlab() {
    let r = gitlab_ref();
    assert!(r.bare_clone_path().ends_with("worktrees/gitlab/acme/web"));
}

#[test]
fn temp_path_includes_branch() {
    let r = github_ref();
    assert!(r
        .temp_path()
        .ends_with("worktrees/github/acme/api/feature-x"));
}

#[test]
#[should_panic(expected = "multi_dir_name is not supported for RemoteBranch")]
fn multi_dir_name_panics() {
    let _ = github_ref().multi_dir_name();
}
