---
displayNumber: 3
status: in-progress
priority: 3
createdAt: 2026-02-19T12:43:03.849944+00:00
updatedAt: 2026-02-19T12:50:55.754957+00:00
---

# Rename `daemon` subcommand and remove `install` action

The `daemon` subcommand manages URL scheme handler registration, not an actual background daemon process — the name is misleading.

## Changes

* Rename `worktree daemon` to a more accurate name (e.g. `worktree scheme` or `worktree handler`)
* Remove the `install` action — `worktree setup` already handles registration
* Keep `uninstall` and `status`, as they have no equivalent elsewhere in the CLI
