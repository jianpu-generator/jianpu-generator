import * as oauth from 'oauth4webapi'
import { useLocalStorage } from 'usehooks-ts'
import { syncedShareWorkerOrigin } from '../syncedShare/workerUrl'

/**
 * Dedicated, minimally-scoped "sign in with GitHub" connection used only to
 * verify a Synced Share owner's identity (`GET /user`, no other scope) --
 * see `TODO-synced-share-rust-d1-migration.md` §0/§6. This is deliberately
 * a separate module from `./githubAuth.ts`, which is the broad-scope,
 * opt-in device-flow connection used for the storage backend's Contents API
 * access. The two connections store their tokens under different
 * `localStorage`/`sessionStorage` keys and are never conflated -- sending
 * this connection's token on a write must never imply Contents API access,
 * and vice versa.
 *
 * Per that same decision, this can reuse the *same registered GitHub OAuth
 * App* (`client_id`) as `githubAuth.ts` -- only the requested `scope` (none,
 * here) and the flow (redirect + PKCE, not device flow) differ.
 *
 * This module now covers the whole flow (task 8): opening the authorization
 * request as a **popup** (not a full-page redirect, per the mockup's "OAuth
 * popup" screen), and completing it once GitHub redirects the popup back to
 * `SYNCED_SHARE_GITHUB_REDIRECT_PATH` -- `SyncedShareGithubCallbackPage`
 * (rendered at that path, see `main.tsx`) calls
 * `completeSyncedShareGithubSignInFromCallback` below, which exchanges the
 * code, persists the resulting token, and relays the outcome back to the
 * opener window via `postMessage` before closing the popup.
 * `openSyncedShareGithubSignInPopup` is what `useSyncedShareOwner.ts` calls
 * to drive the whole round trip from the opener's side. Per §0's "no
 * seamless OAuth-then-continue" decision, resolving this promise never
 * itself starts a share -- the caller must still act on the result (e.g.
 * letting the user click "start sync" again).
 */

/** `sessionStorage` key holding the in-flight PKCE `code_verifier` + `state`
 * pair, read back by whatever completes the flow. Kept in `sessionStorage`,
 * not `localStorage`: this value is only ever needed within the single
 * popup round-trip a sign-in attempt makes, and should not outlive or leak
 * across unrelated tabs/sessions. A popup opened via `window.open` from a
 * same-origin page starts with a *copy* of the opener's `sessionStorage`
 * (per the HTML spec), taken at the moment it's opened -- since this value
 * is written before the popup opens (see `openSyncedShareGithubSignInPopup`),
 * the popup's own read of this same key sees it too. */
export const SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY =
  'jianpu:synced-share-github-pkce:v1'

/** Path (relative to the app's own *base path*, not its origin -- see
 * `syncedShareGithubCallbackPathname` below) GitHub redirects back to once
 * the user approves or denies the request. `SyncedShareGithubCallbackPage`
 * (rendered at this path -- see `main.tsx`) is responsible for completing
 * the flow via `completeSyncedShareGithubSignInFromCallback` below -- this
 * module does not render anything itself. */
export const SYNCED_SHARE_GITHUB_REDIRECT_PATH = '/synced-share/github-callback'

/** Full pathname (including the app's configured base path) that GitHub
 * should redirect back to. `import.meta.env.BASE_URL` is Vite's resolved
 * `base` config (`VITE_BASE_PATH` in `vite.config.ts`) -- `/` on Cloudflare
 * Pages (served from the domain root) but `/jianpu-generator/` on GitHub
 * Pages (served under a repo-name subpath, the currently-primary
 * deployment). Building the redirect URI from `window.location.origin`
 * alone, as this used to, silently drops that subpath and produces a
 * `redirect_uri` GitHub Pages has no content at -- this repo has no
 * SPA-fallback `404.html`, so that would be a genuine 404, not a routing
 * quirk the app could recover from client-side. */
export function syncedShareGithubCallbackPathname(): string {
  const base = import.meta.env.BASE_URL.endsWith('/')
    ? import.meta.env.BASE_URL
    : `${import.meta.env.BASE_URL}/`
  return `${base}${SYNCED_SHARE_GITHUB_REDIRECT_PATH.replace(/^\//, '')}`
}

/** `localStorage` key holding the persisted Synced Share GitHub sign-in
 * token -- deliberately a different key from `githubAuth.ts`'s
 * `GITHUB_AUTH_STORAGE_KEY`, per this module's doc comment: the two
 * connections' tokens are never conflated. */
export const SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY =
  'jianpu:synced-share-github-auth:v1'

/** `postMessage` type tag the callback popup uses to relay its result back
 * to the opener window. Scoped to this app's own origin on both send and
 * receive (see `openSyncedShareGithubSignInPopup`). */
const SYNCED_SHARE_GITHUB_AUTH_MESSAGE_TYPE =
  'jianpu:synced-share-github-auth-result'

const GITHUB_AUTHORIZATION_ENDPOINT = 'https://github.com/login/oauth/authorize'

/** Persisted result of a completed Synced Share GitHub sign-in. `login` is
 * cached alongside the token purely for the "Synced as @username" identity
 * chip -- it is never treated as verified on its own; every write is still
 * verified server-side from the token (task 7). */
export interface StoredSyncedShareGithubAuth {
  token: string
  login: string
}

function readStoredSyncedShareGithubAuthRaw(): StoredSyncedShareGithubAuth | null {
  try {
    const raw = localStorage.getItem(SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<StoredSyncedShareGithubAuth>
    if (typeof parsed.token !== 'string' || typeof parsed.login !== 'string') {
      return null
    }
    return { token: parsed.token, login: parsed.login }
  } catch {
    return null
  }
}

/**
 * Reads the persisted Synced Share GitHub sign-in token directly from
 * `localStorage`, bypassing React. Used by `useSyncedShareOwner.ts`'s
 * imperative write path (`fetch` calls run outside any component's render
 * cycle), mirroring `githubAuth.ts`'s `readStoredGithubAuth`.
 */
export function readStoredSyncedShareGithubAuth(): StoredSyncedShareGithubAuth | null {
  return readStoredSyncedShareGithubAuthRaw()
}

function writeStoredSyncedShareGithubAuth(
  value: StoredSyncedShareGithubAuth | null,
): void {
  try {
    if (value) {
      localStorage.setItem(
        SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY,
        JSON.stringify(value),
      )
    } else {
      localStorage.removeItem(SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY)
    }
  } catch {
    // Ignore write failures (e.g. private-browsing storage quotas); the
    // token simply won't survive a reload, which is a safe degradation --
    // the user just has to sign in again.
  }
}

export function clearStoredSyncedShareGithubAuth(): void {
  writeStoredSyncedShareGithubAuth(null)
}

/**
 * Reactive accessor for the stored Synced Share GitHub auth, so the
 * "Synced as @username" chip and the "start sync" click handler re-render
 * the moment a popup sign-in completes. Mirrors `githubAuth.ts`'s
 * `useGithubAuthToken`.
 */
export function useSyncedShareGithubAuth() {
  return useLocalStorage<StoredSyncedShareGithubAuth | null>(
    SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY,
    null,
  )
}

/** Outcome of a popup sign-in attempt, resolved by
 * `openSyncedShareGithubSignInPopup` and relayed by the callback page over
 * `postMessage`. `reason` on failure distinguishes an abandoned/expired
 * attempt (§0: "the UI just returns to idle") from an actual error worth
 * surfacing. */
export type SyncedShareGithubAuthResult =
  | { ok: true; login: string }
  | { ok: false; reason: 'cancelled' | 'blocked' | 'error'; error: string }

/** In-flight PKCE state persisted across the redirect round-trip. */
export interface SyncedShareGithubPkceState {
  codeVerifier: string
  state: string
  /** The exact `redirect_uri` sent with the authorization request -- GitHub
   * requires the token-exchange request to repeat it verbatim. */
  redirectUri: string
}

function readSyncedShareGithubPkceState(): SyncedShareGithubPkceState | null {
  try {
    const raw = sessionStorage.getItem(SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY)
    if (!raw) return null
    return JSON.parse(raw) as SyncedShareGithubPkceState
  } catch {
    return null
  }
}

function writeSyncedShareGithubPkceState(
  value: SyncedShareGithubPkceState,
): void {
  try {
    sessionStorage.setItem(
      SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY,
      JSON.stringify(value),
    )
  } catch {
    // Ignore write failures (e.g. private-browsing storage quotas); the
    // in-flight sign-in attempt simply won't be completable after the
    // redirect, which the completion step (task 8) must handle as a
    // "returned to idle, no share created" case per §0.
  }
}

/** Reads back (without clearing) whatever PKCE state this module last
 * persisted, for the completion step to consume. */
export function readPendingSyncedShareGithubSignIn(): SyncedShareGithubPkceState | null {
  return readSyncedShareGithubPkceState()
}

/** Clears the in-flight PKCE state, e.g. once the completion step has
 * consumed it, or the sign-in attempt is abandoned. */
export function clearPendingSyncedShareGithubSignIn(): void {
  try {
    sessionStorage.removeItem(SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY)
  } catch {
    // Nothing to clean up if storage isn't available in the first place.
  }
}

async function buildSyncedShareGithubAuthorizationUrl(
  clientId: string,
): Promise<string> {
  const codeVerifier = oauth.generateRandomCodeVerifier()
  const codeChallenge = await oauth.calculatePKCECodeChallenge(codeVerifier)
  const state = oauth.generateRandomState()
  const redirectUri = new URL(
    syncedShareGithubCallbackPathname(),
    window.location.origin,
  ).toString()

  writeSyncedShareGithubPkceState({ codeVerifier, state, redirectUri })

  const authorizationUrl = new URL(GITHUB_AUTHORIZATION_ENDPOINT)
  authorizationUrl.searchParams.set('client_id', clientId)
  authorizationUrl.searchParams.set('redirect_uri', redirectUri)
  authorizationUrl.searchParams.set('response_type', 'code')
  authorizationUrl.searchParams.set('state', state)
  authorizationUrl.searchParams.set('code_challenge', codeChallenge)
  authorizationUrl.searchParams.set('code_challenge_method', 'S256')
  // Forces GitHub to re-show its login/consent screen on every attempt,
  // even when the browser still has a live github.com session and a prior
  // grant for this app's client_id -- without it, GitHub silently
  // redirects straight back with a code (no prompt at all), which is
  // surprising right after "logging out" of Synced Share (that only clears
  // this app's own local token, never the github.com session or the app's
  // authorization grant -- see `disconnectGithub` in
  // `useSyncedShareOwner.ts`).
  authorizationUrl.searchParams.set('prompt', 'login')
  // No `scope` parameter is sent at all -- GitHub's `GET /user` returns
  // public profile fields (including the numeric id this connection
  // actually needs) to an unscoped token, which is the minimal possible ask
  // per §0.

  return authorizationUrl.toString()
}

export interface OpenSyncedShareGithubSignInPopupOptions {
  /** The Synced Share GitHub OAuth App's client id (public, not secret --
   * reused from the same registered app as `githubAuth.ts`'s device flow,
   * per §0). */
  clientId: string
}

/**
 * Drives the whole popup "sign in with GitHub" round trip from the
 * opener's side: opens a popup at GitHub's authorization endpoint (PKCE
 * challenge attached), waits for `SyncedShareGithubCallbackPage` to relay
 * the outcome via `postMessage` once GitHub redirects the popup back, and
 * resolves with that outcome. Also resolves (as a `'cancelled'` failure) if
 * the popup is closed before completing, so the caller never hangs waiting
 * on an abandoned sign-in -- per §0, the UI must simply return to idle in
 * that case.
 *
 * Never throws; a popup blocked by the browser resolves as a `'blocked'`
 * failure instead.
 */
export async function openSyncedShareGithubSignInPopup(
  options: OpenSyncedShareGithubSignInPopupOptions,
): Promise<SyncedShareGithubAuthResult> {
  const authorizationUrl = await buildSyncedShareGithubAuthorizationUrl(
    options.clientId,
  )

  const popup = window.open(
    authorizationUrl,
    'jianpu-synced-share-github-auth',
    'width=600,height=720,noopener=no',
  )

  if (!popup) {
    clearPendingSyncedShareGithubSignIn()
    return {
      ok: false,
      reason: 'blocked',
      error: 'The GitHub sign-in popup was blocked by the browser.',
    }
  }

  return new Promise<SyncedShareGithubAuthResult>((resolve) => {
    let settled = false

    function finish(result: SyncedShareGithubAuthResult) {
      if (settled) return
      settled = true
      window.clearInterval(closedPoll)
      window.removeEventListener('message', onMessage)
      resolve(result)
    }

    function onMessage(event: MessageEvent) {
      if (event.origin !== window.location.origin) return
      const data = event.data as Record<string, unknown> | null
      if (
        typeof data !== 'object' ||
        data === null ||
        data.type !== SYNCED_SHARE_GITHUB_AUTH_MESSAGE_TYPE
      ) {
        return
      }
      const { type: _type, ...result } = data
      finish(result as unknown as SyncedShareGithubAuthResult)
    }

    window.addEventListener('message', onMessage)

    // The popup navigates within GitHub's own origin, so this window can't
    // observe its location -- polling `.closed` is the only cross-origin-
    // safe way to notice the user closed it without completing.
    const closedPoll = window.setInterval(() => {
      if (popup.closed) {
        clearPendingSyncedShareGithubSignIn()
        finish({
          ok: false,
          reason: 'cancelled',
          error: 'The GitHub sign-in popup was closed before completing.',
        })
      }
    }, 500)
  })
}

export interface CompleteSyncedShareGithubSignInFromCallbackOptions {
  /** Host (no scheme) of the Synced Share worker's `/auth/github/callback`
   * route -- the same `VITE_SYNCED_SHARE_HOST` env var
   * `useSyncedShareOwner.ts` uses for the `/shares/...` routes. */
  host: string
  /** Defaults to `window.location.search`; overridable for tests. */
  search?: string
}

interface GithubOauthCallbackResponse {
  accessToken: string
  login?: string
}

/**
 * Completes the popup "sign in with GitHub" flow, run from
 * `SyncedShareGithubCallbackPage` once GitHub redirects the popup back to
 * `SYNCED_SHARE_GITHUB_REDIRECT_PATH`: validates the returned `state`
 * against the pending PKCE state, exchanges the authorization `code` for a
 * token via the Worker's `POST /auth/github/callback` (task 6), persists
 * the resulting token (and cached `login`, for the identity chip), clears
 * the pending PKCE state, and relays the outcome to the opener window via
 * `postMessage` (scoped to this app's own origin) so
 * `openSyncedShareGithubSignInPopup` can resolve. Never throws -- every
 * failure path resolves/relays a `SyncedShareGithubAuthResult` instead.
 */
export async function completeSyncedShareGithubSignInFromCallback(
  options: CompleteSyncedShareGithubSignInFromCallbackOptions,
): Promise<SyncedShareGithubAuthResult> {
  const result = await runSyncedShareGithubCallback(options)
  clearPendingSyncedShareGithubSignIn()
  relayResultToOpener(result)
  return result
}

async function runSyncedShareGithubCallback(
  options: CompleteSyncedShareGithubSignInFromCallbackOptions,
): Promise<SyncedShareGithubAuthResult> {
  const params = new URLSearchParams(options.search ?? window.location.search)
  const errorParam = params.get('error')
  if (errorParam) {
    return {
      ok: false,
      reason: 'error',
      error: params.get('error_description') ?? errorParam,
    }
  }

  const code = params.get('code')
  const returnedState = params.get('state')
  if (!code || !returnedState) {
    return {
      ok: false,
      reason: 'error',
      error: 'GitHub redirected back without a code/state pair.',
    }
  }

  const pending = readPendingSyncedShareGithubSignIn()
  if (!pending || pending.state !== returnedState) {
    return {
      ok: false,
      reason: 'error',
      error:
        'The sign-in state did not match -- possible CSRF or a stale/expired attempt.',
    }
  }

  let response: Response
  try {
    response = await fetch(
      `${syncedShareWorkerOrigin(options.host)}/auth/github/callback`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code,
          codeVerifier: pending.codeVerifier,
          redirectUri: pending.redirectUri,
        }),
      },
    )
  } catch (error) {
    return {
      ok: false,
      reason: 'error',
      error: `Could not reach the Synced Share worker: ${error instanceof Error ? error.message : String(error)}`,
    }
  }

  if (!response.ok) {
    const body = await response.text().catch(() => '')
    return {
      ok: false,
      reason: 'error',
      error: `GitHub token exchange failed: status=${response.status}, body=${body}`,
    }
  }

  const body = (await response.json()) as GithubOauthCallbackResponse
  const login = body.login ?? ''
  writeStoredSyncedShareGithubAuth({ token: body.accessToken, login })
  return { ok: true, login }
}

function relayResultToOpener(result: SyncedShareGithubAuthResult): void {
  if (!window.opener) return
  try {
    window.opener.postMessage(
      { type: SYNCED_SHARE_GITHUB_AUTH_MESSAGE_TYPE, ...result },
      window.location.origin,
    )
  } catch {
    // No opener to relay to (e.g. this page was opened directly, not as a
    // popup) -- `SyncedShareGithubCallbackPage` falls back to rendering the
    // outcome itself in that case.
  }
}
