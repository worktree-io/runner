---
createdAt: 2026-02-26T23:39:21.909294+00:00
updatedAt: 2026-02-26T23:39:21.909294+00:00
customFields:
  persona: developer-mixed-repos-three-with-issues-one-context-only
  acceptance-criteria: Given a mix of repo#issue pairs and bare repo slugs (no issue), when I run `worktree open-multi <repo-a>#7 <repo-b>#12 <repo-c>#3 <repo-d>`, then repos with an issue get an issue-derived branch and repos without an issue are checked out on their default branch, all four appear under one workspace root, the editor opens there, and TTL covers the full workspace.
---

# Open a workspace with three issue-linked repos and one context-only repo

## User Story

As a **developer** whose task has GitHub issues in three repos but also requires a fourth context-only repo (e.g., a shared types package) with no associated issue, I want to include that extra repo in the same workspace checkout — checked out on its default branch — without having to create a fake issue or run a separate `git worktree add` outside the tool.

## Example

```
worktree open-multi acme/backend#7 acme/frontend#12 acme/gateway#3 acme/shared-types
```

Creates:

```
~/workspaces/calm_babbage/
  backend-7/      ← branch: issue-7
  frontend-12/    ← branch: issue-12
  gateway-3/      ← branch: issue-3
  shared-types/   ← checked out on default branch (main)
```

- `shared-types` is included in TTL tracking alongside the others
- No issue is created or required for `shared-types`
