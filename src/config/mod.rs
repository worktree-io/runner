mod ops;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            open: OpenConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { command: None }
    }
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self { editor: true }
    }
}
