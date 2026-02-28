# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.14.0] - 2026-02-28

### Added
- `worktree restore` command to recover manually deleted worktree folders: scans the workspace registry for orphaned entries and recreates the working directory by pruning the stale git reference and re-adding the worktree at its original path ([#62](https://github.com/worktree-io/runner/pull/62))
- "1 day" preset option for workspace TTL configuration during `worktree setup`

## [0.13.0] - 2026-02-27

### Added
- Scaffold `.worktree.toml` automatically when missing on `worktree open`: the file is created from the embedded template so repos get per-repo hook config on first open ([#52](https://github.com/worktree-io/runner/pull/52))
- Centralized TOML templates as compile-time assets: extracted `.worktree.toml`, `config.toml`, and `workspaces.toml` scaffolds into `assets/` and embedded them via `include_str!` in a new `templates` module; added a test ensuring `assets/config.toml` stays in sync with `Config::default().to_toml_with_comments()` ([#57](https://github.com/worktree-io/runner/pull/57))
- Docker-based snapshot tests for generated files: verifies that the TOML templates written to disk match their embedded `assets/` sources across a clean environment ([#56](https://github.com/worktree-io/runner/pull/56))

### Fixed
- Trap `SIGINT` in the bootstrap script so that pressing `^C` inside a worktree shell returns to the worktree directory instead of the original cwd ([#55](https://github.com/worktree-io/runner/pull/55))

## [0.12.1] - 2026-02-26

### Added
- Per-repo hooks via `.worktree.toml`: repositories can now define `pre:open` and `post:open` hook scripts scoped to the repo, with an `order` field (`before`, `after`, `replace`) controlling how they compose with global hooks ([#50](https://github.com/worktree-io/runner/pull/50))

## [0.12.0] - 2026-02-26

### Added
- Jira issue support: `jira:<key>` shorthand and `https://your-domain.atlassian.net/browse/<KEY>` URL parsing ([#45](https://github.com/worktree-io/runner/pull/45))
- GitLab issue support: `gl:<number>` shorthand, `https://gitlab.com/<owner>/<repo>/-/issues/<N>` URL parsing, and `gitlab_host` deep link parameter ([#45](https://github.com/worktree-io/runner/pull/45))
- `worktree prune` command to remove expired workspaces from disk and the registry ([#48](https://github.com/worktree-io/runner/pull/48))
- `workspace.auto_prune` config option: when enabled, expired worktrees are pruned automatically on each `worktree open` invocation ([#47](https://github.com/worktree-io/runner/pull/47))
- `worktree setup` now prompts for workspace TTL with preset options (7 days, 30 days, 90 days) or a custom duration ([#46](https://github.com/worktree-io/runner/pull/46))

## [0.11.0] - 2026-02-25

### Added
- Workspace TTL management and registry: `Ttl` type, `WorkspaceRecord`, and `WorkspaceRegistry` persisted to `~/.config/worktree/workspaces.toml`; new workspaces are auto-registered on `worktree open` ([#39](https://github.com/worktree-io/runner/pull/39))
- `centy:<number>` shorthand: resolves a Centy issue by walking up to the nearest `.centy/` ancestor directory
- `gh:<number>` shorthand: resolves a GitHub issue against the `origin` remote of the current git repository
- Linux: bundle PNG icon via `include_bytes!` and install to `~/.local/share/icons/hicolor/256x256/apps/worktree-runner.png` during `worktree scheme install` ([#37](https://github.com/worktree-io/runner/pull/37))
- Linux: add `Icon=worktree-runner` field to the `worktree-runner.desktop` entry ([#37](https://github.com/worktree-io/runner/pull/37))

### Fixed
- Linux: `worktree scheme uninstall` now also removes the PNG icon file ([#37](https://github.com/worktree-io/runner/pull/37))
- macOS: call `launchctl bootout` then `launchctl bootstrap gui/<uid>` immediately after writing the `LaunchAgent` plist so the `worktree://` URL scheme is active in the current session without requiring a logout ([#41](https://github.com/worktree-io/runner/pull/41), fixes [#27](https://github.com/worktree-io/runner/issues/27))
- macOS: add `StartInterval 3600` to the `LaunchAgent` plist so `lsregister` re-runs every hour and recovers from macOS Launch Services database resets mid-session ([#41](https://github.com/worktree-io/runner/pull/41), fixes [#27](https://github.com/worktree-io/runner/issues/27))

## [0.9.0] - 2026-02-23

### Fixed
- Use UUIDs for temporary hook script filenames to prevent PID collision when hooks run concurrently

### Changed
- Extracted `pre:open` and `post:open` default hook strings to named variables in setup

## [0.8.0] - 2026-02-22

### Added
- `config edit` subcommand to open `config.toml` in the configured editor ([#16](https://github.com/worktree-io/runner/pull/16))
- Inline TOML comments describing each config field when saving ([#17](https://github.com/worktree-io/runner/pull/17))
- Website comment prepended to `config.toml` on save ([#15](https://github.com/worktree-io/runner/pull/15))

### Changed
- Enabled pedantic and strict Clippy linting across the codebase ([#14](https://github.com/worktree-io/runner/pull/14))

## [0.7.2] - 2026-02-22

### Changed
- Exposed `resolve_editor_command` as a public API for library consumers

## [0.7.1] - 2026-02-22

### Added
- Pre-commit hook: `cargo fmt` formatting check ([#9](https://github.com/worktree-io/runner/pull/9))
- Pre-commit hook: `cargo clippy` lint check ([#11](https://github.com/worktree-io/runner/pull/11))
- Pre-commit hook: `cspell` spell check ([#13](https://github.com/worktree-io/runner/pull/13))
- Pre-push hook: `cargo build` check ([#10](https://github.com/worktree-io/runner/pull/10))
- Pre-push hook: 100% test coverage enforcement ([#12](https://github.com/worktree-io/runner/pull/12))

## [0.7.0] - 2026-02-21

### Added
- `post:open` hook now runs inside the terminal window when opening a worktree in a terminal emulator (iTerm2, Terminal.app, Alacritty, Kitty, WezTerm); for IDE editors a separate terminal is found automatically

### Changed
- Config file moved to `~/.config/worktree/config.toml` for a consistent cross-platform path
- Default hook messages generated by `setup` are now human-readable (e.g. "Opening worktree for ..." / "Worktree ready: ...")
- Replaced custom UUID validation with the `uuid` crate ([#8](https://github.com/worktree-io/runner/pull/8))

## [0.6.3] - 2026-02-21

### Added
- Pre-push hook via cargo-husky to enforce the 200-line file limit ([#7](https://github.com/worktree-io/runner/pull/7))
- Hook execution is printed to stderr before each hook runs ("Running pre:open hook..." / "Running post:open hook...")

### Fixed
- Hooks now inherit an augmented PATH (adds `/opt/homebrew/bin`, `/usr/local/bin`, etc.) so tools like `pnpm` are found when triggered via the `worktree://` URL scheme

## [0.6.2] - 2026-02-21

### Added
- `version` subcommand and `--version` / `-V` flags ([#6](https://github.com/worktree-io/runner/pull/6))

## [0.6.1] - 2026-02-21

### Added
- Default `pre:open` and `post:open` hooks included in the generated configuration

### Changed
- Each subcommand split into its own file under `src/commands/`

## [0.6.0] - 2026-02-21

### Added
- `pre:open` and `post:open` bash script hooks with Mustache templating ([#5](https://github.com/worktree-io/runner/pull/5))

## [0.5.3] - 2026-02-19

### Added
- Native terminal emulator support in editor selection (iTerm2, Warp, Ghostty, Alacritty, Kitty, WezTerm, Terminal.app)

## [0.5.2] - 2026-02-19

### Added
- Linear issue UUID support in `worktree://` deep links ([#4](https://github.com/worktree-io/runner/pull/4))

## [0.5.1] - 2026-02-19

### Fixed
- Editor not opening when launched via URL scheme from a browser — AppleScript's `do shell script` runs with a stripped-down PATH that excludes `/usr/local/bin`; the runner now prepends common binary directories before spawning the editor

### Changed
- Renamed `daemon` subcommand to `scheme` and removed the `install` action ([#3](https://github.com/worktree-io/runner/pull/3))

## [0.5.0] - 2026-02-19

### Added
- `editor` query parameter support in `worktree://` deep links ([#2](https://github.com/worktree-io/runner/pull/2))

## [0.4.0] - 2026-02-19

### Changed
- `worktree setup` now interactively prompts the user to select an editor

## [0.3.0] - 2026-02-19

### Changed
- Simplified opener to editor-only; removed terminal and file explorer options

## [0.2.0] - 2026-02-19

### Added
- `worktree setup` command to generate initial configuration

### Changed
- Binary renamed from `runner` to `worktree`

## [0.1.0] - 2026-02-19

### Added
- Initial release with CI/CD pipeline publishing cross-platform release artifacts ([#1](https://github.com/worktree-io/runner/pull/1))

[Unreleased]: https://github.com/worktree-io/runner/compare/v0.12.1...HEAD
[0.12.1]: https://github.com/worktree-io/runner/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/worktree-io/runner/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/worktree-io/runner/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/worktree-io/runner/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/worktree-io/runner/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/worktree-io/runner/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/worktree-io/runner/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/worktree-io/runner/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/worktree-io/runner/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/worktree-io/runner/compare/v0.6.3...v0.7.0
[0.6.3]: https://github.com/worktree-io/runner/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/worktree-io/runner/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/worktree-io/runner/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/worktree-io/runner/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/worktree-io/runner/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/worktree-io/runner/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/worktree-io/runner/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/worktree-io/runner/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/worktree-io/runner/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/worktree-io/runner/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/worktree-io/runner/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/worktree-io/runner/commits/v0.1.0
