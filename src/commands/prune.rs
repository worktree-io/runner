use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{bail, Result};
use worktree_io::ttl::{Ttl, WorkspaceRecord, WorkspaceRegistry};
use worktree_io::{config::Config, ttl};

pub fn cmd_prune(json: bool) -> Result<()> {
    let config = Config::load()?;
    let Some(ttl) = config.workspace.ttl else {
        bail!("No workspace TTL configured. Set workspace.ttl in your config (e.g. \"7days\").");
    };
    let now = SystemTime::now();
    let mut registry = WorkspaceRegistry::load()?;
    let checked = registry.workspace.len();
    let expired = ttl::prune(&registry.workspace, &ttl, now);
    if !json && expired.is_empty() {
        eprintln!("Checked {checked} workspace(s), none expired (TTL: {ttl}).");
        return Ok(());
    }
    let json_data = json.then(|| entries_json(&expired, &ttl));
    let expired_paths: Vec<PathBuf> = expired.iter().map(|r| r.path.clone()).collect();
    for record in &expired {
        if !json {
            let age = now.duration_since(record.created_at).unwrap_or_default();
            eprintln!(
                "Removing {} (age: {}, TTL: {ttl})",
                record.path.display(),
                humantime::format_duration(age)
            );
        }
        if let Err(e) = std::fs::remove_dir_all(&record.path) {
            eprintln!("Warning: failed to remove {}: {e}", record.path.display());
        }
    }
    registry
        .workspace
        .retain(|r| !expired_paths.contains(&r.path));
    registry.save()?;
    if let Some(entries) = json_data {
        println!("{{\"checked\":{checked},\"pruned\":[{entries}],\"ttl\":\"{ttl}\"}}");
    } else {
        eprintln!("Pruned {} expired workspace(s).", expired_paths.len());
    }
    Ok(())
}

fn entries_json(expired: &[&WorkspaceRecord], ttl: &Ttl) -> String {
    expired
        .iter()
        .map(|r| {
            let path_str = r.path.display().to_string();
            let path_esc = path_str.replace('\\', "\\\\").replace('"', "\\\"");
            let ea = r
                .created_at
                .checked_add(ttl.duration())
                .unwrap_or(r.created_at);
            format!(
                "{{\"path\":\"{path_esc}\",\"expired_at\":\"{}\"}}",
                humantime::format_rfc3339(ea)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
