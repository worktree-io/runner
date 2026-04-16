//! Parser for `.worktree.toml` per-repo hook configuration.
//!
//! The user-facing schema uses flat keys named after each hook. Hook scripts
//! and their optional `order` siblings may sit either at the document root or
//! inside an explicit `[hooks]` table. Both layouts produce the same
//! [`RepoConfig`].
//!
//! ```toml
//! "pre:open" = "cargo build"
//! "pre:open:order" = "before"   # optional; defaults to "before"
//!
//! [hooks]
//! "post:open" = "npm install"
//! ```

use crate::repo_hooks::{HookOrder, RepoConfig, RepoHookEntry, RepoHooksConfig};

/// Parse a `.worktree.toml` body into a [`RepoConfig`].
///
/// # Errors
///
/// Returns an error string when the document is not valid TOML, when a hook
/// script is not a string, or when an `order` value is missing or unknown.
pub fn parse(contents: &str) -> Result<RepoConfig, String> {
    let table: toml::Table = toml::from_str(contents).map_err(|e| e.to_string())?;
    let mut hooks = RepoHooksConfig::default();
    take_hooks(&table, &mut hooks)?;
    if let Some(h) = table.get("hooks") {
        let h = h
            .as_table()
            .ok_or_else(|| "`hooks` must be a TOML table".to_owned())?;
        take_hooks(h, &mut hooks)?;
    }
    Ok(RepoConfig { hooks })
}

fn take_hooks(table: &toml::Table, out: &mut RepoHooksConfig) -> Result<(), String> {
    if let Some(e) = extract_hook(table, "pre:open")? {
        out.pre_open = Some(e);
    }
    if let Some(e) = extract_hook(table, "post:open")? {
        out.post_open = Some(e);
    }
    Ok(())
}

fn extract_hook(table: &toml::Table, name: &str) -> Result<Option<RepoHookEntry>, String> {
    let Some(script) = table.get(name) else {
        return Ok(None);
    };
    let script = script
        .as_str()
        .ok_or_else(|| format!("`{name}` must be a string"))?;
    let order_key = format!("{name}:order");
    let order = match table.get(&order_key) {
        None => HookOrder::default(),
        Some(v) => {
            let s = v
                .as_str()
                .ok_or_else(|| format!("`{order_key}` must be a string"))?;
            parse_order(s).ok_or_else(|| {
                format!("invalid `{order_key}` value `{s}` (expected before/after/replace)")
            })?
        }
    };
    Ok(Some(RepoHookEntry {
        script: script.to_owned(),
        order,
    }))
}

fn parse_order(s: &str) -> Option<HookOrder> {
    match s {
        "before" => Some(HookOrder::Before),
        "after" => Some(HookOrder::After),
        "replace" => Some(HookOrder::Replace),
        _ => None,
    }
}

#[cfg(test)]
#[path = "repo_hooks_parse_tests.rs"]
mod tests;
