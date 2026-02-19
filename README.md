# worktree

A CLI tool that opens GitHub issues as git worktree workspaces. Paste a GitHub issue URL (or use a `worktree://` deep link) and `worktree` clones the repo as a bare clone, creates a dedicated worktree branch, and opens it in your editor, terminal, or file explorer.

## Install

```sh
cargo install --path .
```

Then run first-time setup:

```sh
worktree setup
```

This detects your editor, writes the default config, and registers the `worktree://` URL scheme handler — all in one step.

## Usage

### Open a workspace

```sh
# GitHub issue URL
worktree open https://github.com/owner/repo/issues/42

# Shorthand
worktree open owner/repo#42

# worktree:// deep link (used by browser extensions / integrations)
worktree open "worktree://open?owner=owner&repo=repo&issue=42"
```

Flags to control what gets opened (override config):

| Flag | Description |
|------|-------------|
| `--editor` | Open in the configured editor |
| `--explorer` | Open in the file explorer |
| `--terminal` | Open in a terminal |
| `--print-path` | Print the workspace path and exit |

### Configuration

```sh
worktree config init          # write default config to disk
worktree config show          # print current config
worktree config path          # print config file path
worktree config set <key> <value>
worktree config get <key>
```

Config keys:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `editor.command` | string | — | Command to open the editor, e.g. `code .` |
| `terminal.command` | string | — | Command to open a terminal; uses platform default when unset |
| `open.editor` | bool | `false` | Open editor by default |
| `open.explorer` | bool | `false` | Open file explorer by default |
| `open.terminal` | bool | `true` | Open terminal by default |

The config file lives at:

- **macOS/Linux:** `~/.config/worktree/config.toml`
- **Windows:** `%APPDATA%\worktree\config.toml`

Example `config.toml`:

```toml
[editor]
command = "code ."

[terminal]
command = ""

[open]
editor = true
explorer = false
terminal = false
```

### worktree:// URL scheme

Register `worktree` as the system handler for `worktree://` links so they open automatically from the browser:

```sh
worktree daemon install    # register the URL scheme handler
worktree daemon uninstall  # unregister it
worktree daemon status     # check whether it's registered
```

Platform details:

- **macOS** — installs a minimal `.app` bundle in `~/Applications/WorktreeRunner.app` and registers it with Launch Services.
- **Linux** — installs a `.desktop` file in `~/.local/share/applications/` and registers it with `xdg-mime`.
- **Windows** — writes the handler to `HKCU\Software\Classes\worktree` in the registry.

## How it works

1. Parses the issue reference into `owner`, `repo`, and `number`.
2. Bare-clones the repository to `$TMPDIR/worktree-io/github/<owner>/<repo>` (re-uses the clone on subsequent runs and fetches latest).
3. Creates a git worktree at `$TMPDIR/worktree-io/github/<owner>/<repo>/issue-<N>` on a branch named `issue-<N>`.
   - If the branch already exists on the remote it is checked out and tracked locally.
   - Otherwise a new branch is created from the repo's default branch (`main`, `master`, etc.).
4. Opens the workspace directory according to your config or the flags you passed.

## License

MIT
