use anyhow::Result;
use clap::Subcommand;
use worktree_io::config::Config;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the current configuration
    Show,
    /// Print the configuration file path
    Path,
    /// Write the default configuration to disk
    Init,
    /// Set a configuration value (e.g. editor.command "code .")
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
}

pub fn cmd_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = Config::load()?;
            let pretty = toml::to_string_pretty(&config)?;
            print!("{pretty}");
        }
        ConfigAction::Path => {
            let path = Config::path()?;
            println!("{}", path.display());
        }
        ConfigAction::Init => {
            let config = Config::default();
            config.save()?;
            println!("Wrote default config to {}", Config::path()?.display());
        }
        ConfigAction::Set { key, value } => {
            let mut config = Config::load()?;
            config.set_value(&key, &value)?;
            config.save()?;
            println!("Set {key} = {value}");
        }
        ConfigAction::Get { key } => {
            let config = Config::load()?;
            println!("{}", config.get_value(&key)?);
        }
    }
    Ok(())
}
