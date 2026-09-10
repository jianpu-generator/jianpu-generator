// Minimal mock of the two GitHub HTTP endpoints the Synced Share Rust
// worker calls server-side: `POST /login/oauth/access_token` (token
// exchange, `crates/live-share-worker/src/oauth.rs`) and `GET /user`
// (identity verification, `crates/live-share-worker/src/identity/github.rs`).
//
// Both calls happen inside the worker process (`wrangler dev`), not the
// browser -- Playwright's `page.route()` can only intercept requests the
// browser itself makes (used elsewhere in this suite for the popup's
// navigation to GitHub's authorization endpoint), so this is a real local
// HTTP server the worker is pointed at instead, via `SYNCED_SHARE_GITHUB_TOKEN_URL`/
// `SYNCED_SHARE_GITHUB_USER_URL` (see `crates/live-share-worker/wrangler.toml`'s
// e2e-only `.dev.vars.e2e` override loaded by `playwright.config.ts`'s
// webServer command). Ensures no Synced Share e2e run ever makes a real
// GitHub API call (task 11).
//
// Accepts any bearer token/code and always resolves to the same fixed test
// identity -- Synced Share e2e scenarios don't need multiple distinct
// GitHub identities, only "is a verified identity present or not".
import { createServer } from 'node:http'

const PORT = Number(process.env.MOCK_GITHUB_PORT ?? 8788)
export const MOCK_GITHUB_LOGIN = 'e2e-test-user'
export const MOCK_GITHUB_USER_ID = 987654321

function readBody(req) {
  return new Promise((resolve) => {
    let data = ''
    req.on('data', (chunk) => {
      data += chunk
    })
    req.on('end', () => resolve(data))
  })
}

function sendJson(res, status, body) {
  const text = JSON.stringify(body)
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(text),
  })
  res.end(text)
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url ?? '/', `http://localhost:${PORT}`)

  // Health-check target for Playwright's `webServer.url` (see
  // `playwright.config.ts`) -- distinct from the catch-all 404 below so the
  // config doesn't need to wait on `/user`, which 401s without a header.
  if (req.method === 'GET' && url.pathname === '/health') {
    sendJson(res, 200, { ok: true })
    return
  }

  if (req.method === 'POST' && url.pathname === '/login/oauth/access_token') {
    await readBody(req) // Body is ignored -- any code/verifier is accepted.
    sendJson(res, 200, { access_token: 'mock-github-access-token' })
    return
  }

  if (req.method === 'GET' && url.pathname === '/user') {
    // A real `GET /user` call requires an `Authorization` header; this mock
    // doesn't bother validating its value (any bearer token resolves to the
    // same fixed identity), matching the scope of what these e2e scenarios
    // need to exercise.
    if (!req.headers.authorization) {
      sendJson(res, 401, { message: 'Requires authentication' })
      return
    }
    sendJson(res, 200, { id: MOCK_GITHUB_USER_ID, login: MOCK_GITHUB_LOGIN })
    return
  }

  sendJson(res, 404, { message: 'Not Found' })
})

server.listen(PORT, () => {
  console.log(`Mock GitHub OAuth server listening on http://localhost:${PORT}`)
})
