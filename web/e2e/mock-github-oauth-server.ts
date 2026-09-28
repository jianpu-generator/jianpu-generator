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
// known fixed test identities (see `mockGithubIdentity.ts`) -- needed so
// idempotent-share e2e scenarios can prove two different GitHub accounts
// don't collide, not just "is a verified identity present or not".
//
// Also a stateful fake of GitHub's token revocation, for tokens it minted
// itself: an exchange of a login-bearing code (`authorizationCodeFor(login)`)
// mints a distinct token per sign-in, like real GitHub does per device.
// Revoking the app's *grant* (`DELETE /applications/{client_id}/grant`)
// revokes every token minted for that login; revoking one *token*
// (`DELETE /applications/{client_id}/token`) revokes just that one. A
// revoked token then gets GitHub's real `401 Bad credentials` from
// `GET /user`. Seeded tokens (`syncedShareIdentityTokenFor`) and the legacy
// shared `mock-github-access-token` stay outside this tracking entirely --
// many scenarios share them across parallel workers, so one scenario's
// sign-out must never revoke them for everyone else.
import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
} from 'node:http'
import {
  DEFAULT_MOCK_GITHUB_LOGIN,
  DEFAULT_MOCK_GITHUB_USER_ID,
  identityForLogin,
  identityForSyncedShareToken,
  loginFromAuthorizationCode,
} from './mockGithubIdentity.ts'

const PORT = Number(process.env.MOCK_GITHUB_PORT ?? 8788)
export const MOCK_GITHUB_LOGIN = DEFAULT_MOCK_GITHUB_LOGIN
export const MOCK_GITHUB_USER_ID = DEFAULT_MOCK_GITHUB_USER_ID

function readBody(req: IncomingMessage): Promise<string> {
  return new Promise((resolve) => {
    let data = ''
    req.on('data', (chunk) => {
      data += chunk
    })
    req.on('end', () => resolve(data))
  })
}

function sendJson(res: ServerResponse, status: number, body: unknown): void {
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
const usedCodes = new Set<string>()

// Tokens minted by a login-bearing exchange, and which login each belongs to.
const mintedTokenLogins = new Map<string, string>()
const revokedTokens = new Set<string>()

function revokeMintedTokensFor(login: string): void {
  for (const [token, tokenLogin] of mintedTokenLogins) {
    if (tokenLogin === login) revokedTokens.add(token)
  }
}

async function readAccessToken(
  req: IncomingMessage,
): Promise<string | undefined> {
  try {
    const accessToken: unknown = JSON.parse(await readBody(req))?.access_token
    return typeof accessToken === 'string' ? accessToken : undefined
  } catch {
    return undefined
  }
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
    const raw = await readBody(req)
    let code: unknown
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
    const login =
      typeof code === 'string' ? loginFromAuthorizationCode(code) : null
    if (login) {
      const token = `mock-github-access-token:${login}:${crypto.randomUUID()}`
      mintedTokenLogins.set(token, login)
      sendJson(res, 200, { access_token: token })
      return
    }
    sendJson(res, 200, { access_token: 'mock-github-access-token' })
    return
  }

  // Real GitHub responds `204 No Content` on a successful revocation; this
  // mock doesn't bother validating the Basic-auth header, matching the scope
  // of what these e2e scenarios need to exercise (see `oauth.rs`).
  if (
    req.method === 'DELETE' &&
    url.pathname === '/applications/e2e-test-client-id/grant'
  ) {
    const token = await readAccessToken(req)
    const login = token && mintedTokenLogins.get(token)
    if (login) revokeMintedTokensFor(login)
    res.writeHead(204)
    res.end()
    return
  }

  if (
    req.method === 'DELETE' &&
    url.pathname === '/applications/e2e-test-client-id/token'
  ) {
    const token = await readAccessToken(req)
    if (token && mintedTokenLogins.has(token)) revokedTokens.add(token)
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
    if (revokedTokens.has(token)) {
      sendJson(res, 401, {
        message: 'Bad credentials',
        documentation_url: 'https://docs.github.com/rest',
        status: '401',
      })
      return
    }
    const mintedLogin = mintedTokenLogins.get(token)
    const { id, login } = mintedLogin
      ? identityForLogin(mintedLogin)
      : identityForSyncedShareToken(token)
    sendJson(res, 200, { id, login })
    return
  }

  sendJson(res, 404, { message: 'Not Found' })
})

server.listen(PORT, () => {
  console.log(`Mock GitHub OAuth server listening on http://localhost:${PORT}`)
})
