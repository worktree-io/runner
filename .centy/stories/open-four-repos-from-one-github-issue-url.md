---
createdAt: 2026-02-26T23:38:49.593844+00:00
updatedAt: 2026-02-26T23:38:49.593844+00:00
customFields:
  acceptance-criteria: Given one GitHub issue URL and four repo slugs, when I run `worktree open-multi <gh-issue-url> <repo-a> <repo-b> <repo-c> <repo-d>`, then four worktree subdirectories are created under a single workspace folder each on a branch derived from the issue, the editor opens at the workspace root, and TTL covers the entire workspace.
  persona: github-developer-one-issue-four-repos
---

# Open four repos from one GitHub issue URL

## User Story

As a **GitHub developer** whose feature spans four repos (API, web, mobile, SDK), all tracked under a single GitHub issue, I want to pass that one issue URL plus all four repo slugs to `open-multi` so every repo is checked out on the correct branch in one workspace — with no repeated commands and no missed repos.

## Example

```
worktree open-multi https://github.com/acme/api/issues/42 \
  acme/api acme/web acme/mobile acme/sdk
```

Creates:

```
~/workspaces/quiet_darwin/
  api-42/
  web-42/
  mobile-42/
  sdk-42/
```

Editor opens at `~/workspaces/quiet_darwin/`.
