use anyhow::{bail, Context, Result};
use url::Url;

pub struct QueryParams {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub issue_num: Option<u64>,
    pub linear_id: Option<String>,
    pub url_param: Option<String>,
    pub editor: Option<String>,
    pub no_hooks: bool,
    pub ado_org: Option<String>,
    pub ado_project: Option<String>,
    pub ado_repo: Option<String>,
    pub ado_work_item_id: Option<u64>,
    pub jira_host: Option<String>,
    pub jira_issue_key: Option<String>,
    pub gitlab_host: Option<String>,
    pub extra_env: Vec<(String, String)>,
    pub adhoc_name: Option<String>,
}

pub fn parse_query_params(url: &Url) -> Result<QueryParams> {
    let mut p = QueryParams {
        owner: None,
        repo: None,
        issue_num: None,
        linear_id: None,
        url_param: None,
        editor: None,
        no_hooks: false,
        ado_org: None,
        ado_project: None,
        ado_repo: None,
        ado_work_item_id: None,
        jira_host: None,
        jira_issue_key: None,
        gitlab_host: None,
        extra_env: Vec::new(),
        adhoc_name: None,
    };
    for (key, val) in url.query_pairs() {
        match key.as_ref() {
            "owner" => p.owner = Some(val.into_owned()),
            "repo" => p.repo = Some(val.into_owned()),
            "issue" => {
                p.issue_num = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid issue number: {val}"))?,
                );
            }
            "linear_id" => {
                let id = val.into_owned();
                if uuid::Uuid::parse_str(&id).is_err() {
                    bail!("Invalid Linear issue UUID: {id}");
                }
                p.linear_id = Some(id);
            }
            "url" => p.url_param = Some(val.into_owned()),
            "editor" => p.editor = Some(val.into_owned()),
            "no_hooks" => p.no_hooks = val == "1",
            "org" => p.ado_org = Some(val.into_owned()),
            "project" => p.ado_project = Some(val.into_owned()),
            "ado_repo" => p.ado_repo = Some(val.into_owned()),
            "work_item_id" => {
                p.ado_work_item_id = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid work item ID: {val}"))?,
                );
            }
            "jira_host" => p.jira_host = Some(val.into_owned()),
            "jira_issue_key" => p.jira_issue_key = Some(val.into_owned()),
            "gitlab_host" => p.gitlab_host = Some(val.into_owned()),
            "env" => {
                if let Some((k, v)) = val.as_ref().split_once(':') {
                    p.extra_env.push((k.to_string(), v.to_string()));
                }
            }
            "adhoc" => p.adhoc_name = Some(val.into_owned()),
            _ => {}
        }
    }
    Ok(p)
}
