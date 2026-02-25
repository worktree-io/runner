/// Query parameters accumulated while parsing a `worktree://` URL.
#[derive(Default)]
pub(super) struct UrlParams {
    pub(super) owner: Option<String>,
    pub(super) repo: Option<String>,
    pub(super) issue_num: Option<u64>,
    pub(super) linear_id: Option<String>,
    pub(super) url_param: Option<String>,
    pub(super) editor: Option<String>,
    pub(super) ado_org: Option<String>,
    pub(super) ado_project: Option<String>,
    pub(super) ado_repo: Option<String>,
    pub(super) ado_work_item_id: Option<u64>,
    pub(super) jira_host: Option<String>,
    pub(super) jira_issue_key: Option<String>,
    pub(super) gitlab_host: Option<String>,
}
