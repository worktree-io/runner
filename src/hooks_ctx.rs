/// Template variables available to hook scripts.
pub struct HookContext {
    /// GitHub owner / organization name.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Issue number or Linear UUID as a string.
    pub issue: String,
    /// Git branch name for the worktree.
    pub branch: String,
    /// Absolute path to the worktree directory.
    pub worktree_path: String,
    /// Extra environment variables to inject into the hook process.
    pub extra_env: Vec<(String, String)>,
}

impl HookContext {
    /// Expand `{{owner}}`, `{{repo}}`, `{{issue}}`, `{{branch}}`, and
    /// `{{worktree_path}}` placeholders in `template`.
    #[must_use]
    pub fn render(&self, template: &str) -> String {
        template
            .replace("{{owner}}", &self.owner)
            .replace("{{repo}}", &self.repo)
            .replace("{{issue}}", &self.issue)
            .replace("{{branch}}", &self.branch)
            .replace("{{worktree_path}}", &self.worktree_path)
    }
}
