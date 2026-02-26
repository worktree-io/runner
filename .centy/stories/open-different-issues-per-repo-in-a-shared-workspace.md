---
createdAt: 2026-02-26T23:32:32.094533+00:00
updatedAt: 2026-02-26T23:32:32.094533+00:00
customFields:
  acceptance-criteria: Given repo#issue pairs for two or more repos, when I run `worktree open-multi <repo-a>#<issue-a> <repo-b>#<issue-b>`, then a unified workspace is created with each repo checked out on its own branch, the editor opens at the workspace root, and per-repo hooks run for each repo while group-level hooks run once.
  persona: platform-microservices-engineer
---

# Open different issues per repo in a shared workspace

## User Story

As a **platform / microservices engineer** coordinating a cross-cutting change where each service has its own issue number (backend #7, frontend #12, infra #3), I want to open all of them in a single unified workspace with one command, so that branches are created consistently in every service and I can edit across all of them from a single editor root.

## Example

```
worktree open-multi acme/backend#7 acme/frontend#12 acme/infra#3
```

Creates:

```
~/workspaces/sharp_hopper/
  backend-7/    ← worktree for acme/backend at issue-7
  frontend-12/  ← worktree for acme/frontend at issue-12
  infra-3/      ← worktree for acme/infra at issue-3
```

Per-repo `.worktree.toml` hooks run for each repo; global hooks run once at workspace level.
