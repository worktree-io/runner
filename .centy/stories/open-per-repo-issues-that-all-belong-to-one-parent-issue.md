---
createdAt: 2026-02-26T23:39:08.496913+00:00
updatedAt: 2026-02-26T23:39:08.496913+00:00
customFields:
  persona: github-developer-per-repo-issues-under-a-parent-issue
  acceptance-criteria: Given repo#issue pairs for three or more repos (each with a distinct issue number), when I run `worktree open-multi <repo-a>#7 <repo-b>#12 <repo-c>#3`, then each repo is checked out on a branch derived from its own issue number, all repos share one workspace folder, and the editor opens at the workspace root.
---

# Open per-repo issues that all belong to one parent issue

## User Story

As a **GitHub developer** whose team files a separate issue per repo for each cross-cutting task (with all issues linked to a parent epic), I want to supply each repo+issue pair so every repo gets a branch that matches its own issue number — keeping PR titles and changelog entries accurate while still giving me a single editor workspace.

## Example

```
worktree open-multi acme/backend#7 acme/frontend#12 acme/gateway#3
```

Creates:

```
~/workspaces/brave_hopper/
  backend-7/    ← branch: issue-7
  frontend-12/  ← branch: issue-12
  gateway-3/    ← branch: issue-3
```

Editor opens at `~/workspaces/brave_hopper/`.
