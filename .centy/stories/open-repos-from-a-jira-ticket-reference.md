---
createdAt: 2026-02-26T23:39:00.041455+00:00
updatedAt: 2026-02-26T23:39:00.041455+00:00
customFields:
  persona: jira-developer-one-ticket-several-repos
  acceptance-criteria: Given a Jira ticket identifier (e.g., PROJ-456) and two or more repo slugs, when I run `worktree open-multi PROJ-456 <repo-a> <repo-b>`, then worktrees are created with branches named after the ticket (e.g., PROJ-456) in each repo under a unified workspace, and the editor opens at the workspace root. No GitHub issue lookup is performed.
---

# Open repos from a Jira ticket reference

## User Story

As a **Jira developer** whose team tracks work in Jira rather than GitHub issues, I want to pass a Jira ticket reference directly to `open-multi` so that consistently-named branches are created across all affected repos in one workspace — without needing a corresponding GitHub issue in every repo.

## Example

```
worktree open-multi PROJ-456 acme/backend acme/frontend acme/infra
```

Creates:

```
~/workspaces/eager_turing/
  backend-PROJ-456/
  frontend-PROJ-456/
  infra-PROJ-456/
```

Branch name in each repo: `PROJ-456` (no issue number lookup attempted).
