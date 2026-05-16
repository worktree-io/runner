/**
 * worktree-agent-runner
 *
 * Receives Ameliso delegation webhooks and provisions local agent sessions via
 * `worktree open`. Zero external dependencies — uses only Node.js built-ins.
 */

import http from "node:http";
import crypto from "node:crypto";
import { spawn } from "node:child_process";

const PORT = parseInt(process.env.PORT ?? "3001", 10);
const WEBHOOK_SECRET = process.env.AMELISO_WEBHOOK_SECRET;
const DEFAULT_OWNER = process.env.AMELISO_GITHUB_OWNER;
const DEFAULT_REPO = process.env.AMELISO_GITHUB_REPO;

if (!WEBHOOK_SECRET) {
  console.warn(
    "[agent-runner] WARNING: AMELISO_WEBHOOK_SECRET is not set — HMAC validation skipped (dev mode)"
  );
}

/**
 * Validate the HMAC-SHA256 signature on the raw request body.
 * The header is expected to be: x-ameliso-signature: sha256=<hex>
 *
 * @param {Buffer} rawBody
 * @param {string|undefined} signatureHeader
 * @returns {boolean}
 */
function isSignatureValid(rawBody, signatureHeader) {
  if (!WEBHOOK_SECRET) {
    return true; // dev mode: skip validation
  }
  if (!signatureHeader || !signatureHeader.startsWith("sha256=")) {
    return false;
  }
  const provided = signatureHeader.slice("sha256=".length);
  const expected = crypto
    .createHmac("sha256", WEBHOOK_SECRET)
    .update(rawBody)
    .digest("hex");
  // Constant-time comparison to prevent timing attacks
  try {
    return crypto.timingSafeEqual(
      Buffer.from(provided, "hex"),
      Buffer.from(expected, "hex")
    );
  } catch {
    return false;
  }
}

/**
 * Collect the full request body as a Buffer.
 *
 * @param {http.IncomingMessage} req
 * @returns {Promise<Buffer>}
 */
function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => resolve(Buffer.concat(chunks)));
    req.on("error", reject);
  });
}

/**
 * Spawn `worktree open <owner>/<repo> --headless --json
 *   --env AMELISO_RUN_ID=<run_id>
 *   --env AMELISO_CASE_IDS=<comma-joined case_ids>`
 * in the background and log the outcome.
 *
 * @param {string} owner
 * @param {string} repo
 * @param {string} runId
 * @param {string[]} caseIds
 */
function spawnWorktreeSession(owner, repo, runId, caseIds) {
  const ref = `${owner}/${repo}`;
  const caseIdsCsv = caseIds.join(",");
  const args = [
    "open",
    ref,
    "--headless",
    "--json",
    "--env",
    `AMELISO_RUN_ID=${runId}`,
    "--env",
    `AMELISO_CASE_IDS=${caseIdsCsv}`,
  ];

  // Forward MCP endpoint env vars so the per-repo hook can override .mcp.json
  // for local dev (agent-runner sets AMELISO_GRPC_ADDR / AMELISO_API_URL in .env).
  if (process.env.AMELISO_GRPC_ADDR) {
    args.push("--env", `AMELISO_GRPC_ADDR=${process.env.AMELISO_GRPC_ADDR}`);
  }
  if (process.env.AMELISO_API_URL) {
    args.push("--env", `AMELISO_API_URL=${process.env.AMELISO_API_URL}`);
  }
  if (process.env.AMELISO_MCP_BIN) {
    args.push("--env", `AMELISO_MCP_BIN=${process.env.AMELISO_MCP_BIN}`);
  }

  console.log(
    `[agent-runner] Spawning: worktree ${args.join(" ")}`
  );

  const child = spawn("worktree", args, {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env },
  });

  let stdout = "";
  let stderr = "";

  child.stdout.on("data", (d) => {
    stdout += d.toString();
  });
  child.stderr.on("data", (d) => {
    stderr += d.toString();
  });

  child.on("close", (code) => {
    if (code === 0) {
      console.log(
        `[agent-runner] worktree session started for run=${runId} repo=${ref}. output=${stdout.trim()}`
      );
    } else {
      console.error(
        `[agent-runner] worktree exited with code ${code} for run=${runId} repo=${ref}. stderr=${stderr.trim()}`
      );
    }
  });

  child.on("error", (err) => {
    console.error(
      `[agent-runner] Failed to spawn worktree for run=${runId}: ${err.message}`
    );
  });
}

const server = http.createServer(async (req, res) => {
  if (req.method === "POST" && req.url === "/delegate") {
    let rawBody;
    try {
      rawBody = await readBody(req);
    } catch (err) {
      console.error("[agent-runner] Error reading request body:", err);
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Failed to read request body" }));
      return;
    }

    const signature = req.headers["x-ameliso-signature"];
    if (!isSignatureValid(rawBody, signature)) {
      console.warn("[agent-runner] Invalid or missing HMAC signature — rejecting request");
      res.writeHead(401, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Invalid signature" }));
      return;
    }

    let payload;
    try {
      payload = JSON.parse(rawBody.toString("utf8"));
    } catch {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Invalid JSON body" }));
      return;
    }

    const { repo_id, run_id, case_ids } = payload;
    // owner/repo can come from the payload directly, or fall back to env vars
    const owner = payload.owner ?? DEFAULT_OWNER;
    const repo = payload.repo ?? DEFAULT_REPO;

    if (!run_id || !Array.isArray(case_ids)) {
      res.writeHead(422, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({ error: "Missing required fields: run_id, case_ids" })
      );
      return;
    }

    if (!owner || !repo) {
      res.writeHead(422, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          error:
            "Cannot determine target repo — set owner/repo in payload or AMELISO_GITHUB_OWNER/AMELISO_GITHUB_REPO env vars",
        })
      );
      return;
    }

    // Respond 202 immediately, then do work in the background
    res.writeHead(202, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ accepted: true, run_id, repo_id: repo_id ?? null }));

    console.log(
      `[agent-runner] Accepted delegation for run_id=${run_id} repo=${owner}/${repo} case_ids=[${case_ids.join(", ")}]`
    );

    spawnWorktreeSession(owner, repo, String(run_id), case_ids.map(String));
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

server.listen(PORT, () => {
  console.log(`[agent-runner] Listening on port ${PORT}`);
});

export { isSignatureValid, spawnWorktreeSession };
