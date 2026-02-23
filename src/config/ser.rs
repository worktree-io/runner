use std::fmt::Write as _;

use super::Config;

/// Wrap a string value in TOML basic-string quotes, escaping special characters.
fn toml_quoted(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

impl Config {
    /// Serialize the config to a TOML string with inline documentation comments.
    ///
    /// Each section header and field is preceded by a `#` comment that matches
    /// the doc-comment on the corresponding struct field.  The resulting string
    /// round-trips cleanly through [`toml::from_str`].
    #[must_use]
    pub fn to_toml_with_comments(&self) -> String {
        let mut out = String::new();

        // Website header comment
        out.push_str("# runner \u{2014} https://worktree.io\n\n");

        // [editor] ------------------------------------------------------------
        out.push_str("# Editor configuration.\n");
        out.push_str("[editor]\n");
        if let Some(cmd) = &self.editor.command {
            out.push_str("# Command to launch the editor, e.g. \"code .\" or \"nvim .\"\n");
            writeln!(out, "command = {}", toml_quoted(cmd)).unwrap();
        }
        out.push('\n');

        // [open] --------------------------------------------------------------
        out.push_str("# Workspace open behavior.\n");
        out.push_str("[open]\n");
        out.push_str("# Whether to launch the configured editor when opening a workspace.\n");
        writeln!(out, "editor = {}", self.open.editor).unwrap();
        out.push('\n');

        // [hooks] -------------------------------------------------------------
        out.push_str("# Hook scripts run around the open command.\n");
        out.push_str("[hooks]\n");
        if let Some(pre) = &self.hooks.pre_open {
            out.push_str("# Script run before opening the workspace.\n");
            writeln!(out, "\"pre:open\" = {}", toml_quoted(pre)).unwrap();
        }
        if let Some(post) = &self.hooks.post_open {
            out.push_str("# Script run after opening the workspace.\n");
            writeln!(out, "\"post:open\" = {}", toml_quoted(post)).unwrap();
        }

        out
    }
}

#[cfg(test)]
#[path = "ser_tests.rs"]
mod ser_tests;
