# worktree-agent-runner

A lightweight Node.js HTTP server (zero npm dependencies) that receives
Ameliso test-run delegation webhooks and provisions local Claude Code
agent sessions via `worktree open`.

## How it works

1. Ameliso server POSTs `{repo_id, run_id, case_ids, owner?, repo?}` to
   `POST /delegate`.
2. The server validates the HMAC-SHA256 signature (header:
   `x-ameliso-signature: sha256=<hex>`), responds **202 Accepted**
   immediately, then shells out to:
   ```
   worktree open <owner>/<repo> --headless --json \
     --env AMELISO_RUN_ID=<run_id> \
     --env AMELISO_CASE_IDS=<comma-joined case_ids>
   ```
3. The worktree agent session picks up `AMELISO_RUN_ID` and
   `AMELISO_CASE_IDS` from the environment and uses the Ameliso MCP
   to execute cases and record results.

## Setup

```bash
cp .env.example .env
# edit .env — set AMELISO_WEBHOOK_SECRET to the same value configured
# in the Ameliso server's WORKTREE_IO_WEBHOOK_SECRET env var
node src/index.js
```

## Required environment variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3001` | HTTP port to listen on |
| `AMELISO_WEBHOOK_SECRET` | _(none)_ | Shared HMAC secret. If unset, signature validation is skipped (dev mode). |
| `AMELISO_GITHUB_OWNER` | _(none)_ | Fallback GitHub org/owner when not in the webhook payload |
| `AMELISO_GITHUB_REPO` | _(none)_ | Fallback GitHub repo name when not in the webhook payload |

## Configuring Ameliso

Set the following in your Ameliso server environment:

```
WORKTREE_IO_WEBHOOK_URL=http://localhost:3001/delegate
WORKTREE_IO_WEBHOOK_SECRET=<same-value-as-AMELISO_WEBHOOK_SECRET>
```

## Manual test

```bash
# Start the server in one terminal (dev mode, no secret):
node src/index.js

# In another terminal, send a test webhook:
curl -s -X POST http://localhost:3001/delegate \
  -H 'Content-Type: application/json' \
  -d '{"run_id":"run-123","case_ids":["cases/login/basic.md"],"owner":"my-org","repo":"my-tests"}' | jq .
# Should respond: {"accepted":true,"run_id":"run-123","repo_id":null}
```

To test with HMAC validation:

```bash
SECRET=my-secret
BODY='{"run_id":"run-123","case_ids":["cases/login/basic.md"],"owner":"my-org","repo":"my-tests"}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$SECRET" | awk '{print $2}')
curl -s -X POST http://localhost:3001/delegate \
  -H 'Content-Type: application/json' \
  -H "x-ameliso-signature: sha256=$SIG" \
  -d "$BODY" | jq .
```
