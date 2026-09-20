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
// Resolves the bearer token sent to `GET /user` to one of a small set of
// known fixed test identities (see `mockGithubIdentity.mjs`) -- needed so
// idempotent-share e2e scenarios can prove two different GitHub accounts
// don't collide, not just "is a verified identity present or not".
import { createServer } from 'node:http'
import {
  DEFAULT_MOCK_GITHUB_LOGIN,
  DEFAULT_MOCK_GITHUB_USER_ID,
  identityForSyncedShareToken,
} from './mockGithubIdentity.mjs'

const PORT = Number(process.env.MOCK_GITHUB_PORT ?? 8788)
export const MOCK_GITHUB_LOGIN = DEFAULT_MOCK_GITHUB_LOGIN
export const MOCK_GITHUB_USER_ID = DEFAULT_MOCK_GITHUB_USER_ID

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

// Authorization codes are single-use on real GitHub -- exchanging an
// already-consumed code responds `200 OK` with an `error`/`error_description`
// pair instead of `access_token` (see `oauth.rs`'s `GithubTokenResponse` doc
// comment). Enforcing that here lets e2e reproduce the double-token-exchange
// regression (`SyncedShareGithubCallbackPage`'s `useEffect` firing the
// exchange twice under `StrictMode`) instead of masking it by accepting any
// code repeatedly.
const usedCodes = new Set()

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
    const raw = await readBody(req)
    let code
    try {
      code = JSON.parse(raw)?.code
    } catch {
      code = undefined
    }
    if (typeof code === 'string' && usedCodes.has(code)) {
      sendJson(res, 200, {
        error: 'bad_verification_code',
        error_description: 'The code passed is incorrect or expired.',
      })
      return
    }
    if (typeof code === 'string') {
      usedCodes.add(code)
    }
    sendJson(res, 200, { access_token: 'mock-github-access-token' })
    return
  }

  if (
    req.method === 'DELETE' &&
    url.pathname === '/applications/e2e-test-client-id/grant'
  ) {
    // Real GitHub responds `204 No Content` on a successful grant
    // revocation; this mock doesn't bother validating the Basic-auth header
    // or body, matching the scope of what these e2e scenarios need to
    // exercise (see `oauth.rs`'s `revoke_grant`).
    res.writeHead(204)
    res.end()
    return
  }

  if (req.method === 'GET' && url.pathname === '/user') {
    // A real `GET /user` call requires an `Authorization` header; this mock
    // doesn't bother validating the *scheme* (e.g. `Bearer `-prefix), only
    // resolving the token value to a known identity, matching the scope of
    // what these e2e scenarios need to exercise.
    const authorization = req.headers.authorization
    if (!authorization) {
      sendJson(res, 401, { message: 'Requires authentication' })
      return
    }
    const token = authorization.replace(/^Bearer\s+/i, '')
    const { id, login } = identityForSyncedShareToken(token)
    sendJson(res, 200, { id, login })
    return
  }

  sendJson(res, 404, { message: 'Not Found' })
})

server.listen(PORT, () => {
  console.log(`Mock GitHub OAuth server listening on http://localhost:${PORT}`)
})
