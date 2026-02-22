use anyhow::Result;
use std::process::Command;

use crate::opener::augmented_path;

pub struct HookContext {
    pub owner: String,
    pub repo: String,
    pub issue: String,
    pub branch: String,
    pub worktree_path: String,
}

impl HookContext {
    pub fn render(&self, template: &str) -> String {
        template
            .replace("{{owner}}", &self.owner)
            .replace("{{repo}}", &self.repo)
            .replace("{{issue}}", &self.issue)
            .replace("{{branch}}", &self.branch)
            .replace("{{worktree_path}}", &self.worktree_path)
    }
}

/// Render `script` with `ctx`, write to a temp file, and execute it.
/// Stdout and stderr are forwarded to the caller's terminal.
/// A non-zero exit code prints a warning but does not return an error.
pub fn run_hook(script: &str, ctx: &HookContext) -> Result<()> {
    let rendered = ctx.render(script);

    let tmp_path = std::env::temp_dir().join(format!("worktree-hook-{}.sh", std::process::id()));
    std::fs::write(&tmp_path, rendered.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let result = Command::new("sh")
        .arg(&tmp_path)
        .env("PATH", augmented_path())
        .status();
    let _ = std::fs::remove_file(&tmp_path);

    match result {
        Ok(status) if !status.success() => {
            eprintln!("Warning: hook exited with status {:?}", status.code());
        }
        // LLVM_COV_EXCL_START
        Err(e) => {
            eprintln!("Warning: failed to run hook: {e}");
        }
        // LLVM_COV_EXCL_STOP
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx() -> HookContext {
        HookContext {
            owner: "acme".into(),
            repo: "api".into(),
            issue: "42".into(),
            branch: "issue-42".into(),
            worktree_path: "/tmp/wt".into(),
        }
    }
    #[test]
    fn test_render_all_placeholders() {
        let out = ctx().render("{{owner}}/{{repo}}#{{issue}} {{branch}} {{worktree_path}}");
        assert_eq!(out, "acme/api#42 issue-42 /tmp/wt");
    }
    #[test]
    fn test_render_no_placeholders() {
        assert_eq!(ctx().render("hello"), "hello");
    }
    #[test]
    fn test_run_hook_success() {
        run_hook("true", &ctx()).unwrap();
    }
    #[test]
    fn test_run_hook_nonzero_exit() {
        run_hook("exit 1", &ctx()).unwrap();
    }
}
