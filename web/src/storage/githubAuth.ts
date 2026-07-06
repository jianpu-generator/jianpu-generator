import { createOAuthDeviceAuth } from '@octokit/auth-oauth-device'
import { Octokit } from '@octokit/rest'
import { useLocalStorage } from 'usehooks-ts'

/**
 * `localStorage` key holding the persisted GitHub OAuth token. Shared
 * between the imperative reads/writes in this module and
 * `useGithubAuthToken` (exposed for a future settings UI that needs to
 * re-render when the token changes).
 */
export const GITHUB_AUTH_STORAGE_KEY = 'jianpu:github-auth:v1'

/** Verification payload GitHub returns to start the device flow. Renders as
 * a `user_code`/`verification_uri` prompt in the UI wired in a later step. */
export interface GithubDeviceVerification {
  device_code: string
  user_code: string
  verification_uri: string
  expires_in: number
  interval: number
}

export interface StoredGithubAuth {
  token: string
  scopes: string[]
}

/** Connection status reported after validating a stored token against `GET
 * /user`. `'needs-reconnect'` means the token was cleared because GitHub
 * rejected it (401); the caller must run `connectWithDeviceFlow` again. */
export type GithubAuthStatus =
  | { state: 'disconnected' }
  | { state: 'connected'; username: string }
  | { state: 'needs-reconnect' }

/**
 * Reads the persisted token directly from `localStorage`, bypassing React.
 * Used by the auth/backend logic in this module (and, eventually,
 * `githubBackend.ts`), which runs outside any component's render cycle and
 * so cannot call the `useLocalStorage` hook below.
 */
export function readStoredGithubAuth(): StoredGithubAuth | null {
  try {
    const raw = localStorage.getItem(GITHUB_AUTH_STORAGE_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<StoredGithubAuth>
    if (typeof parsed.token !== 'string') return null
    return { token: parsed.token, scopes: parsed.scopes ?? [] }
  } catch {
    return null
  }
}

function writeStoredGithubAuth(value: StoredGithubAuth | null): void {
  try {
    if (value) {
      localStorage.setItem(GITHUB_AUTH_STORAGE_KEY, JSON.stringify(value))
    } else {
      localStorage.removeItem(GITHUB_AUTH_STORAGE_KEY)
    }
  } catch {
    // Ignore write failures (e.g. private-browsing storage quotas); the
    // token simply won't survive a reload, which is a safe degradation.
  }
}

export function clearStoredGithubAuth(): void {
  writeStoredGithubAuth(null)
}

/**
 * Reactive accessor for the stored token, for a future settings UI that
 * needs to re-render on connect/disconnect. Not used by this module's own
 * imperative logic — see `readStoredGithubAuth`/`writeStoredGithubAuth`.
 */
export function useGithubAuthToken() {
  return useLocalStorage<StoredGithubAuth | null>(
    GITHUB_AUTH_STORAGE_KEY,
    null,
  )
}

/**
 * Route map for the two device-flow calls that GitHub itself doesn't send
 * CORS headers for. Everything else Octokit does (Contents API, `/user`,
 * ...) is called directly against `api.github.com` and never touches this
 * proxy. Paths must match `cf-oauth-proxy/functions/` (step 1): `POST
 * /device/code` and `POST /oauth/token`.
 */
const PROXIED_ROUTES: Record<string, string> = {
  'POST /login/device/code': '/device/code',
  'POST /login/oauth/access_token': '/oauth/token',
}

interface ProxyResponse {
  status: number
  url: string
  headers: Record<string, string>
  data: unknown
}

/**
 * A minimal, duck-typed stand-in for `@octokit/request`'s `RequestInterface`
 * — `@octokit/auth-oauth-device` only ever calls `request(route, params)`
 * and reads `request.endpoint.DEFAULTS.baseUrl` / `request.endpoint.merge`
 * (the latter only to build error context on a rejected device/token
 * response). Building a real `@octokit/endpoint` instance would pull in a
 * dependency this project doesn't otherwise install; this stub supplies
 * just enough surface for those two call sites.
 */
function createProxyRequest(proxyBaseUrl: string) {
  async function request(
    route: string,
    parameters?: Record<string, unknown>,
  ): Promise<ProxyResponse> {
    const path = PROXIED_ROUTES[route]
    if (!path) {
      throw new Error(`githubAuth: unsupported proxied route "${route}"`)
    }
    const { baseUrl: _baseUrl, headers: _headers, ...body } = parameters ?? {}
    const response = await fetch(`${proxyBaseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
      body: JSON.stringify(body),
    })
    const data = (await response.json()) as unknown
    const headers: Record<string, string> = {}
    response.headers.forEach((value, key) => {
      headers[key] = value
    })
    return {
      status: response.status,
      url: response.url,
      headers,
      data,
    }
  }

  request.endpoint = {
    DEFAULTS: { baseUrl: proxyBaseUrl },
    merge: (route: string, parameters?: Record<string, unknown>) => ({
      method: 'POST',
      url: `${proxyBaseUrl}${PROXIED_ROUTES[route] ?? ''}`,
      headers: {},
      ...parameters,
    }),
  }

  return request
}

export interface DeviceFlowOptions {
  clientId: string
  proxyBaseUrl: string
  scopes?: string[]
  /** Renders `verification.user_code` / `verification.verification_uri` to
   * the user; UI for this is wired in a later step. */
  onVerification: (verification: GithubDeviceVerification) => void | Promise<void>
}

/**
 * Runs the OAuth device flow to completion and persists the resulting token.
 * Resolves once the user has approved the request on GitHub's device page
 * (i.e. after `onVerification` fires and polling succeeds).
 */
export async function connectWithDeviceFlow(
  options: DeviceFlowOptions,
): Promise<StoredGithubAuth> {
  const auth = createOAuthDeviceAuth({
    clientType: 'oauth-app',
    clientId: options.clientId,
    scopes: options.scopes ?? [],
    // `RequestInterface` expects a fuller shape than this proxy-only stub
    // provides; see `createProxyRequest`'s doc comment for why a full
    // `@octokit/request` instance isn't used here.
    request: createProxyRequest(
      options.proxyBaseUrl,
    ) as unknown as Parameters<typeof createOAuthDeviceAuth>[0]['request'],
    onVerification: options.onVerification,
  })

  const authentication = await auth({ type: 'oauth' })
  const stored: StoredGithubAuth = {
    token: authentication.token,
    scopes: authentication.scopes,
  }
  writeStoredGithubAuth(stored)
  return stored
}

/**
 * Validates the stored token via `GET /user` (CORS-permitted directly
 * against `api.github.com`, no proxy needed). Clears the token on `401` so
 * the caller can prompt to reconnect.
 */
export async function checkGithubAuthStatus(): Promise<GithubAuthStatus> {
  const stored = readStoredGithubAuth()
  if (!stored) return { state: 'disconnected' }

  const octokit = new Octokit({ auth: stored.token })
  try {
    const { data } = await octokit.rest.users.getAuthenticated()
    return { state: 'connected', username: data.login }
  } catch (error) {
    if (isUnauthorized(error)) {
      clearStoredGithubAuth()
      return { state: 'needs-reconnect' }
    }
    throw error
  }
}

function isUnauthorized(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'status' in error &&
    (error as { status: unknown }).status === 401
  )
}

/**
 * Clears the local token only. There is no revoke-proxy endpoint in v1 (see
 * `cf-oauth-proxy`'s README) — the token remains valid on GitHub's side
 * until the user revokes it from their GitHub authorized-apps settings.
 */
export function disconnectGithub(): void {
  clearStoredGithubAuth()
}
