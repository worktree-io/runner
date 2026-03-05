---
createdAt: 2026-03-05T12:22:23.352806+00:00
updatedAt: 2026-03-05T12:22:23.352806+00:00
customFields:
  acceptance-criteria: Given `pre:open` and/or `post:open` hooks are configured, when I run `worktree open-multi <repo-a>#N <repo-b>#N ...`, then each hook fires once per repo — in the order repos are processed — with that repo's template variables (`{{owner}}`, `{{repo}}`, `{{issue}}`, `{{branch}}`, `{{worktree_path}}`) resolved; hooks for repo B do not start until repo A's hooks finish; a failing hook exit code prints a warning but does not abort the remaining repos.
  persona: platform-microservices-engineer
---

# Run per-repo setup hooks when opening a multi-repo workspace

## User Story

As a **platform / microservices engineer** who has `post:open` hooks configured to bootstrap each service (install dependencies, copy `.env` templates, start a local dev server), I want those same hooks to fire automatically for every repo when I run `open-multi`, so that the entire workspace is ready to use the moment my editor opens — without me having to write a separate wrapper script or remember to run setup manually in each service directory.

## Example

Config (`~/.config/worktree/config.toml`):

```toml
[hooks]
"post:open" = """
cd {{worktree_path}}
cp .env.example .env 2>/dev/null || true
pnpm install --silent
"""
```

Command:

```
worktree open-multi acme/api#42 acme/web#42 acme/gateway#42
```

Hook execution order:

```
post:open  →  repo=api,     branch=issue-42, worktree_path=~/workspaces/bold_turing/api-42
post:open  →  repo=web,     branch=issue-42, worktree_path=~/workspaces/bold_turing/web-42
post:open  →  repo=gateway, branch=issue-42, worktree_path=~/workspaces/bold_turing/gateway-42
```

Each invocation receives the correct per-repo values; the editor opens only after all hooks have completed.
