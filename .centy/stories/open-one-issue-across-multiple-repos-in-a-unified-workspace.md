---
createdAt: 2026-02-26T23:32:21.336637+00:00
updatedAt: 2026-02-26T23:32:21.336637+00:00
customFields:
  persona: full-stack-developer
  acceptance-criteria: Given a single issue URL/ref and two or more repo slugs, when I run `worktree open-multi <issue> <repo-a> <repo-b>`, then a unified workspace folder is created with one subdirectory per repo each checked out on a branch derived from the issue; the editor opens at the workspace root; TTL is tracked for the whole workspace.
---

# Open one issue across multiple repos in a unified workspace

## User Story

As a **full-stack developer** working on a feature that touches both the backend (`acme/api`) and frontend (`acme/web`) repos, I want to run a single command that opens the same issue as a worktree in every repo and launches my editor at the shared workspace root, so that I can navigate all related code without juggling separate editor windows.

## Example

```
worktree open-multi acme/api#42 acme/web acme/design-system
```

Creates:

```
~/workspaces/bold_turing/
  api-42/          ← worktree for acme/api at issue-42
  web-42/          ← worktree for acme/web at issue-42
  design-system-42/
```

Editor opens at `~/workspaces/bold_turing/`.
