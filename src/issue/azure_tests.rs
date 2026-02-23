use super::*;

#[test]
fn test_parse_azure_devops_url() {
    let r = IssueRef::parse("https://dev.azure.com/myorg/myproject/_workitems/edit/42").unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myproject".into(),
            id: 42,
        }
    );
}

#[test]
fn test_parse_azure_devops_url_invalid_id() {
    let err =
        IssueRef::parse("https://dev.azure.com/myorg/myproject/_workitems/edit/abc").unwrap_err();
    assert!(err.to_string().contains("Invalid work item ID"));
}

#[test]
fn test_parse_azure_devops_url_wrong_format() {
    let err = IssueRef::parse("https://dev.azure.com/myorg/myproject/_boards/board").unwrap_err();
    assert!(err
        .to_string()
        .contains("Expected Azure DevOps work item URL"));
}

#[test]
fn test_parse_azure_devops_shorthand() {
    let r = IssueRef::parse("myorg/myproject/myrepo!42").unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myrepo".into(),
            id: 42,
        }
    );
}

#[test]
fn test_parse_azure_devops_shorthand_invalid_id() {
    let err = IssueRef::parse("myorg/myproject/myrepo!abc").unwrap_err();
    assert!(err.to_string().contains("Invalid work item ID"));
}

#[test]
fn test_parse_azure_devops_shorthand_missing_parts() {
    let err = IssueRef::parse("myorg/myproject!42").unwrap_err();
    assert!(err.to_string().contains("Invalid Azure DevOps shorthand"));
}

#[test]
fn test_parse_azure_devops_worktree_url() {
    let r = IssueRef::parse(
        "worktree://open?org=myorg&project=myproject&ado_repo=myrepo&work_item_id=42",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myrepo".into(),
            id: 42,
        }
    );
}

#[test]
fn test_parse_azure_devops_worktree_url_defaults_repo_to_project() {
    let r = IssueRef::parse("worktree://open?org=myorg&project=myproject&work_item_id=42").unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myproject".into(),
            id: 42,
        }
    );
}

#[test]
fn test_parse_azure_devops_worktree_url_with_editor() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?org=myorg&project=myproject&work_item_id=42&editor=cursor",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myproject".into(),
            id: 42,
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn test_azure_devops_workspace_dir_name() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert_eq!(r.workspace_dir_name(), "workitem-42");
    assert_eq!(r.branch_name(), "workitem-42");
}

#[test]
fn test_azure_devops_clone_url() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert_eq!(
        r.clone_url(),
        "https://dev.azure.com/myorg/myproject/_git/myrepo"
    );
}

#[test]
fn test_azure_devops_paths() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert!(r
        .bare_clone_path()
        .ends_with("worktrees/azuredevops/myorg/myproject/myrepo"));
    assert!(r
        .temp_path()
        .ends_with("worktrees/azuredevops/myorg/myproject/myrepo/workitem-42"));
}
