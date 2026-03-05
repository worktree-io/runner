<p align="center">
  <img src="assets/logo.svg" alt="worktree" width="96">
</p>

# worktree

A CLI tool that opens GitHub issues as git worktree workspaces. Paste a GitHub issue URL (or use a `worktree://` deep link) and `worktree` clones the repo as a bare clone, creates a dedicated worktree branch, and opens it in your editor.

## Install

Download a prebuilt binary from the [latest release](https://github.com/worktree-io/runner/releases/latest):

| Platform | Download |
| -------- | -------- |
| macOS (Apple Silicon) | `worktree-macos-aarch64.tar.gz` |
| macOS (Intel) | `worktree-macos-x86_64.tar.gz` |
| Linux x86_64 | `worktree-linux-x86_64.tar.gz` |
| Linux ARM64 | `worktree-linux-aarch64.tar.gz` |
| Windows | `worktree-windows-x86_64.zip` |

Or install from [crates.io](https://crates.io/crates/worktree-io):

```sh
cargo install worktree-io
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

# worktree:// deep link (used by browser / integrations)
worktree open "worktree://open?owner=owner&repo=repo&issue=42"
```

Flags:

| Flag           | Description                       |
| -------------- | --------------------------------- |
| `--editor`     | Force open in editor              |
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

| Key              | Type   | Default | Description                                            |
| ---------------- | ------ | ------- | ------------------------------------------------------ |
| `editor.command` | string | —       | Command to open the editor, e.g. `code .` or `nvim .` |
| `open.editor`    | bool   | `true`  | Open editor automatically                              |

The config file lives at:

- **macOS/Linux:** `~/.config/worktree/config.toml`
- **Windows:** `%USERPROFILE%\.config\worktree\config.toml`

Example `config.toml`:

```toml
[editor]
command = "code ."

[open]
editor = true
```

### Hooks

Run shell scripts automatically when a workspace is opened.

| Layer | Configured in | Scope |
| ----- | ------------- | ----- |
| **Global** | `config.toml` under `[hooks]` | All repos |
| **Per-repo** | `.worktree.toml` in the repo root under `[hooks]` | That repo only |

Both layers use the same hook names and template variables. For `open-multi`, each hook fires once per repo with that repo's context.

| Hook | When it runs |
| ---- | ------------ |
| `pre:open` | After the worktree is created, before the editor launches |
| `post:open` | After the editor launches |

| Template variable | Description |
| ----------------- | ----------- |
| `{{owner}}` | GitHub owner / org |
| `{{repo}}` | Repository name |
| `{{issue}}` | Issue number |
| `{{branch}}` | Branch name (e.g. `issue-42`) |
| `{{worktree_path}}` | Absolute path to the worktree directory |

Per-repo hooks compose with the global hook via an `order` field:

| `order` | Behavior |
| ------- | -------- |
| `before` *(default)* | Per-repo script runs first, then global |
| `after` | Global script runs first, then per-repo |
| `replace` | Only the per-repo script runs; global is suppressed |

A non-zero exit code prints a warning but does not abort the open.

### worktree:// URL scheme

Register `worktree` as the system handler for `worktree://` links so they open automatically from the browser:

```sh
worktree scheme install    # register the URL scheme handler
worktree scheme uninstall  # unregister it
worktree scheme status     # check whether it's registered
```

Platform details:

- **macOS** — installs a minimal `.app` bundle in `~/Applications/WorktreeRunner.app` and registers it with Launch Services.
- **Linux** — installs a `.desktop` file in `~/.local/share/applications/` and registers it with `xdg-mime`.
- **Windows** — writes the handler to `HKCU\Software\Classes\worktree` in the registry.

## How it works

1. Parses the issue reference into `owner`, `repo`, and `number`.
2. Bare-clones the repository to `~/worktrees/github/<owner>/<repo>` (re-uses the clone on subsequent runs and fetches latest).
3. Creates a git worktree at `~/worktrees/github/<owner>/<repo>/issue-<N>` on a branch named `issue-<N>`.
   - If the branch already exists on the remote it is checked out and tracked locally.
   - Otherwise a new branch is created from the repo's default branch (`main`, `master`, etc.).
4. Opens the workspace directory in the configured editor.

## License

MIT
