use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub editor: EditorConfig,
    pub terminal: TerminalConfig,
    pub open: OpenConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    /// Command to launch the editor, e.g. "code ." or "nvim ."
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Command to launch a terminal in the workspace dir; None uses platform default
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenConfig {
    pub editor: bool,
    pub explorer: bool,
    pub terminal: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            open: OpenConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { command: None }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { command: None }
    }
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self {
            editor: false,
            explorer: false,
            terminal: true,
        }
    }
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("runner").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    /// Get a config value by dot-separated key path
    pub fn get_value(&self, key: &str) -> Result<String> {
        match key {
            "editor.command" => Ok(self.editor.command.clone().unwrap_or_default()),
            "terminal.command" => Ok(self.terminal.command.clone().unwrap_or_default()),
            "open.editor" => Ok(self.open.editor.to_string()),
            "open.explorer" => Ok(self.open.explorer.to_string()),
            "open.terminal" => Ok(self.open.terminal.to_string()),
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
    }

    /// Set a config value by dot-separated key path
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "editor.command" => {
                self.editor.command = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            "terminal.command" => {
                self.terminal.command = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            "open.editor" => {
                self.open.editor = value.parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            "open.explorer" => {
                self.open.explorer = value.parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            "open.terminal" => {
                self.open.terminal = value.parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
        Ok(())
    }
}
