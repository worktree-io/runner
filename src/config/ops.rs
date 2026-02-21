use anyhow::{Context, Result};
use std::path::PathBuf;

use super::Config;

impl Config {
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join(".config").join("worktree").join("config.toml"))
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
            "open.editor" => Ok(self.open.editor.to_string()),
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
    }

    /// Set a config value by dot-separated key path
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "editor.command" => {
                self.editor.command = if value.is_empty() { None } else { Some(value.to_string()) };
            }
            "open.editor" => {
                self.open.editor = value.parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
        Ok(())
    }
}
