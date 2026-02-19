use anyhow::Result;
use clap::{Parser, Subcommand};
#[cfg(target_os = "macos")]
use dirs;

use worktree_io::{
    config::Config,
    scheme,
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
    Scheme {
        #[command(subcommand)]
        action: SchemeAction,
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
enum SchemeAction {
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

        Commands::Scheme { action } => cmd_scheme(action)?,

        Commands::Setup => cmd_setup()?,
    }

    Ok(())
}

fn cmd_open(issue_ref: &str, force_editor: bool, print_path: bool) -> Result<()> {
    let (issue, deep_link_opts) = IssueRef::parse_with_options(issue_ref)?;
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

    if let Some(editor_name) = deep_link_opts.editor {
        // Deep link editor param takes precedence over config
        let cmd = resolve_editor_command(&editor_name);
        opener::open_in_editor(&workspace.path, &cmd)?;
    } else {
        let config = Config::load()?;
        if force_editor || config.open.editor {
            if let Some(cmd) = &config.editor.command {
                opener::open_in_editor(&workspace.path, cmd)?;
            } else {
                eprintln!("No editor configured. Run: worktree setup");
            }
        }
    }

    Ok(())
}

/// Map a symbolic editor/terminal name to a launch command, or return the value as-is
/// if it is not a known symbol (treating it as a raw command string).
fn resolve_editor_command(name: &str) -> String {
    let candidates: &[(&str, &str)] = &[
        ("cursor",   "cursor ."),
        ("code",     "code ."),
        ("zed",      "zed ."),
        ("subl",     "subl ."),
        ("nvim",     "nvim ."),
        ("vim",      "vim ."),
        ("iterm",           "open -a iTerm ."),
        ("iterm2",          "open -a iTerm ."),
        ("warp",            "open -a Warp ."),
        ("ghostty",         "open -a Ghostty ."),
        ("alacritty",       "alacritty --working-directory ."),
        ("kitty",           "kitty --directory ."),
        ("wezterm",         "wezterm start --cwd ."),
        ("wt",              "wt -d ."),
        ("windowsterminal", "wt -d ."),
    ];
    for &(sym, cmd) in candidates {
        if name.eq_ignore_ascii_case(sym) {
            return cmd.to_string();
        }
    }
    // "terminal" resolves to the native OS terminal
    if name.eq_ignore_ascii_case("terminal") {
        #[cfg(target_os = "macos")]
        return "open -a Terminal .".to_string();
        #[cfg(target_os = "windows")]
        return "wt -d .".to_string();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return "xterm".to_string();
    }
    // Not a recognised symbolic name — treat as a raw command string
    name.to_string()
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

fn cmd_scheme(action: SchemeAction) -> Result<()> {
    match action {
        SchemeAction::Uninstall => scheme::uninstall()?,
        SchemeAction::Status => println!("{}", scheme::status()?),
    }
    Ok(())
}

fn cmd_setup() -> Result<()> {
    let config_path = Config::path()?;
    let already_existed = config_path.exists();
    let mut config = Config::load()?;

    let detected = detect_all_editors();
    match prompt_editor(&detected) {
        Ok(Some(cmd)) => config.editor.command = Some(cmd),
        Ok(None) => {}
        Err(e) => eprintln!("Warning: could not read editor choice: {e}"),
    }

    config.save()?;
    if already_existed {
        eprintln!("Updated config at {}", config_path.display());
    } else {
        eprintln!("Created config at {}", config_path.display());
    }

    // Register URL scheme handler (warn but don't abort on failure)
    match scheme::install() {
        Ok(()) => {}
        Err(e) => eprintln!("Warning: could not register URL scheme handler: {e}"),
    }

    eprintln!("\nSetup complete! Run: worktree open <github-issue-url>");
    Ok(())
}

/// Probe PATH (and, on macOS, /Applications) for all known editors and terminals.
/// Returns (display name, config command) for each found.
fn detect_all_editors() -> Vec<(&'static str, &'static str)> {
    // Editors detected via PATH binary
    let path_candidates: &[(&str, &str)] = &[
        ("Cursor",       "cursor ."),
        ("VS Code",      "code ."),
        ("Zed",          "zed ."),
        ("Sublime Text", "subl ."),
        ("Neovim",       "nvim ."),
        ("Vim",          "vim ."),
    ];
    let mut found: Vec<(&str, &str)> = path_candidates.iter()
        .filter(|&&(_, cmd)| which(cmd.split_whitespace().next().unwrap()))
        .copied()
        .collect();

    // Terminals detected via PATH binary (cross-platform)
    let terminal_path_candidates: &[(&str, &str)] = &[
        ("Alacritty",    "alacritty --working-directory ."),
        ("Kitty",        "kitty --directory ."),
        ("WezTerm",      "wezterm start --cwd ."),
    ];
    for &(name, cmd) in terminal_path_candidates {
        if which(cmd.split_whitespace().next().unwrap()) {
            found.push((name, cmd));
        }
    }

    // macOS: terminals installed as .app bundles (not on PATH)
    #[cfg(target_os = "macos")]
    {
        // Terminal.app ships with every macOS install
        found.push(("Terminal", "open -a Terminal ."));

        let app_candidates: &[(&str, &str, &str)] = &[
            ("iTerm2",  "open -a iTerm .",   "iTerm"),
            ("Warp",    "open -a Warp .",    "Warp"),
            ("Ghostty", "open -a Ghostty .", "Ghostty"),
        ];
        for &(name, cmd, app) in app_candidates {
            if macos_app_exists(app) {
                found.push((name, cmd));
            }
        }
    }

    // Windows Terminal
    #[cfg(target_os = "windows")]
    if which("wt") {
        found.push(("Windows Terminal", "wt -d ."));
    }

    found
}

/// Check whether `AppName.app` is installed in /Applications or ~/Applications on macOS.
#[cfg(target_os = "macos")]
fn macos_app_exists(app_name: &str) -> bool {
    let system = std::path::Path::new("/Applications").join(format!("{app_name}.app"));
    let user = dirs::home_dir()
        .map(|h| h.join("Applications").join(format!("{app_name}.app")));
    system.exists() || user.map_or(false, |p| p.exists())
}

/// Present an interactive editor selection menu and return the chosen command.
fn prompt_editor(detected: &[(&str, &str)]) -> Result<Option<String>> {
    use std::io::{BufRead, Write};

    eprintln!("\nSelect your default editor or terminal:");
    for (i, (name, _)) in detected.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, name);
    }
    let custom_idx = detected.len() + 1;
    eprintln!("  {custom_idx}. Enter a custom command");
    eprintln!("  0. Skip (no editor configured)");
    eprint!("Choice [{}]: ", if detected.is_empty() { 0 } else { 1 });
    std::io::stderr().flush().ok();

    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();

    let choice: usize = if trimmed.is_empty() {
        if detected.is_empty() { 0 } else { 1 }
    } else {
        trimmed.parse().unwrap_or(usize::MAX)
    };

    if choice == 0 {
        return Ok(None);
    }
    if choice <= detected.len() {
        return Ok(Some(detected[choice - 1].1.to_string()));
    }
    if choice == custom_idx {
        eprint!("Enter editor command (e.g. \"hx .\"): ");
        std::io::stderr().flush().ok();
        let mut custom = String::new();
        stdin.lock().read_line(&mut custom)?;
        let cmd = custom.trim().to_string();
        return Ok(if cmd.is_empty() { None } else { Some(cmd) });
    }

    eprintln!("Invalid choice, skipping editor configuration.");
    Ok(None)
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
