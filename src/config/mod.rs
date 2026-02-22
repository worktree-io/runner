mod ops;

#[cfg(test)]
#[path = "ops_tests.rs"]
mod ops_tests;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub editor: EditorConfig,
    pub open: OpenConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    #[serde(rename = "pre:open", skip_serializing_if = "Option::is_none", default)]
    pub pre_open: Option<String>,
    #[serde(rename = "post:open", skip_serializing_if = "Option::is_none", default)]
    pub post_open: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EditorConfig {
    /// Command to launch the editor, e.g. "code ." or "nvim ."
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenConfig {
    pub editor: bool,
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self { editor: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_default() {
        let c = Config::default();
        assert!(c.editor.command.is_none());
        assert!(c.open.editor);
        assert!(c.hooks.pre_open.is_none());
    }
    #[test]
    fn test_editor_config_default() {
        assert!(EditorConfig::default().command.is_none());
    }
    #[test]
    fn test_open_config_default() {
        assert!(OpenConfig::default().editor);
    }
}
