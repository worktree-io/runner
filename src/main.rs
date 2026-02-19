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
#[command(name = "runner", about = "Open GitHub issues as git worktree workspaces")]
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

        /// Force open in file explorer
        #[arg(long)]
        explorer: bool,

        /// Force open in terminal
        #[arg(long)]
        terminal: bool,

        /// Print the workspace path and exit without opening anything
        #[arg(long)]
        print_path: bool,
    },

    /// Manage runner configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage the worktree:// URL scheme handler
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
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
        Commands::Open {
            issue_ref,
            editor,
            explorer,
            terminal,
            print_path,
        } => cmd_open(&issue_ref, editor, explorer, terminal, print_path)?,

        Commands::Config { action } => cmd_config(action)?,

        Commands::Daemon { action } => cmd_daemon(action)?,
    }

    Ok(())
}

fn cmd_open(
    issue_ref: &str,
    force_editor: bool,
    force_explorer: bool,
    force_terminal: bool,
    print_path: bool,
) -> Result<()> {
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

    // Determine what to open: explicit flags override config
    let open_editor = force_editor || (!force_explorer && !force_terminal && config.open.editor);
    let open_explorer = force_explorer || (!force_editor && !force_terminal && config.open.explorer);
    let open_terminal = force_terminal || (!force_editor && !force_explorer && config.open.terminal);

    // If nothing at all is selected, fall back to terminal
    let open_terminal = open_terminal || (!open_editor && !open_explorer);

    if open_editor {
        if let Some(cmd) = &config.editor.command {
            opener::open_in_editor(&workspace.path, cmd)?;
        } else {
            eprintln!("No editor configured. Set editor.command in your config.");
        }
    }

    if open_explorer {
        opener::open_in_explorer(&workspace.path)?;
    }

    if open_terminal {
        opener::open_in_terminal(&workspace.path, config.terminal.command.as_deref())?;
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
