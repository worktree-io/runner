use anyhow::Result;
use clap::{Parser, Subcommand};

use worktree_io::{
    config::Config,
    daemon,
    issue::IssueRef,
    opener,
    workspace::Workspace,
};

#[derive(Parser)]
#[command(name = "worktree", about = "Open GitHub issues as git worktree workspaces")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an issue reference, create a worktree, and open it
    Open {
        /// Issue reference: GitHub URL, worktree:// deep link, or owner/repo#N
        #[arg(value_name = "REF")]
        issue_ref: String,

        /// Force open in editor
        #[arg(long)]
        editor: bool,

        /// Print the workspace path and exit without opening anything
        #[arg(long)]
        print_path: bool,
    },

    /// Manage worktree configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage the worktree:// URL scheme handler
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Run first-time setup: detect editor, write config, register URL scheme
    Setup,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Print the current configuration
    Show,
    /// Print the configuration file path
    Path,
    /// Write the default configuration to disk
    Init,
    /// Set a configuration value (e.g. editor.command "code .")
    Set {
        key: String,
        value: String,
    },
    /// Get a configuration value
    Get {
        key: String,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Register the worktree:// URL scheme handler
    Install,
    /// Unregister the worktree:// URL scheme handler
    Uninstall,
    /// Check whether the URL scheme handler is registered
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Open { issue_ref, editor, print_path } => {
            cmd_open(&issue_ref, editor, print_path)?
        }

        Commands::Config { action } => cmd_config(action)?,

        Commands::Daemon { action } => cmd_daemon(action)?,

        Commands::Setup => cmd_setup()?,
    }

    Ok(())
}

fn cmd_open(issue_ref: &str, force_editor: bool, print_path: bool) -> Result<()> {
    let issue = IssueRef::parse(issue_ref)?;
    let workspace = Workspace::open_or_create(issue)?;

    if workspace.created {
        eprintln!("Created workspace at {}", workspace.path.display());
    } else {
        eprintln!("Workspace already exists at {}", workspace.path.display());
    }

    if print_path {
        println!("{}", workspace.path.display());
        return Ok(());
    }

    let config = Config::load()?;

    if force_editor || config.open.editor {
        if let Some(cmd) = &config.editor.command {
            opener::open_in_editor(&workspace.path, cmd)?;
        } else {
            eprintln!("No editor configured. Run: worktree setup");
        }
    }

    Ok(())
}

fn cmd_config(action: ConfigAction) -> Result<()> {
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

fn cmd_daemon(action: DaemonAction) -> Result<()> {
    match action {
        DaemonAction::Install => daemon::install()?,
        DaemonAction::Uninstall => daemon::uninstall()?,
        DaemonAction::Status => println!("{}", daemon::status()?),
    }
    Ok(())
}

fn cmd_setup() -> Result<()> {
    let config_path = Config::path()?;
    let already_existed = config_path.exists();
    let mut config = Config::load()?;

    // Auto-detect editor if not already configured
    if config.editor.command.is_none() {
        if let Some((name, cmd)) = detect_editor() {
            eprintln!("Detected editor: {name}");
            config.editor.command = Some(cmd);
        }
    }

    config.save()?;
    if already_existed {
        eprintln!("Updated config at {}", config_path.display());
    } else {
        eprintln!("Created config at {}", config_path.display());
    }

    // Register URL scheme handler (warn but don't abort on failure)
    match daemon::install() {
        Ok(()) => {}
        Err(e) => eprintln!("Warning: could not register URL scheme handler: {e}"),
    }

    eprintln!("\nSetup complete! Run: worktree open <github-issue-url>");
    Ok(())
}

/// Probe PATH for known editors; return (display name, config command) for the first found.
fn detect_editor() -> Option<(&'static str, String)> {
    let candidates: &[(&str, &str)] = &[
        ("Cursor",       "cursor ."),
        ("VS Code",      "code ."),
        ("Zed",          "zed ."),
        ("Sublime Text", "subl ."),
        ("Neovim",       "nvim ."),
        ("Vim",          "vim ."),
    ];
    for &(name, cmd) in candidates {
        let binary = cmd.split_whitespace().next().unwrap();
        if which(binary) {
            return Some((name, cmd.to_string()));
        }
    }
    None
}

/// Return true if `binary` is found anywhere in PATH.
fn which(binary: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path).any(|dir| {
                let candidate = dir.join(binary);
                candidate.is_file() || {
                    #[cfg(target_os = "windows")]
                    { dir.join(format!("{binary}.exe")).is_file() }
                    #[cfg(not(target_os = "windows"))]
                    { false }
                }
            })
        })
        .unwrap_or(false)
}
