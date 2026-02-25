---
displayNumber: 28
status: in-progress
priority: 2
createdAt: 2026-02-25T22:59:54.369267+00:00
updatedAt: 2026-02-25T23:01:07.717865+00:00
---

# setup command does not configure workspace TTL

The `setup` command (`cmd_setup`) only configures the editor, default hooks, and URL scheme handler. It never prompts the user to set `workspace.ttl`, leaving TTL unconfigured after running setup.\n\nUsers must manually run `config set workspace.ttl <value>` to enable workspace expiry. The setup flow is the natural place to introduce and configure TTL.
