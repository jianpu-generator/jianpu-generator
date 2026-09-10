import * as oauth from 'oauth4webapi'

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
 * Scope of this module (task 6 only): starting the authorization request
 * with a PKCE challenge and redirecting the browser to GitHub. Completing
 * the flow -- reading `?code=`/`?state=` off the redirect-back URL, POSTing
 * to the Worker's token-exchange route, and persisting the resulting token
 * -- is wired into `useSyncedShareOwner.ts` in a later task (task 8), per
 * §0's "no seamless OAuth-then-continue" decision: this module only ever
 * gets the user to GitHub and back, it does not itself decide what happens
 * next.
 */

/** `sessionStorage` key holding the in-flight PKCE `code_verifier` + `state`
 * pair, read back by whatever completes the flow (task 8). Kept in
 * `sessionStorage`, not `localStorage`: this value is only ever needed
 * within the single redirect round-trip a sign-in attempt makes, and
 * should not outlive or leak across unrelated tabs/sessions. */
export const SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY = 'jianpu:synced-share-github-pkce:v1'

/** Path (relative to the app's own origin) GitHub redirects back to once
 * the user approves or denies the request. Whatever page renders here is
 * responsible for completing the flow (task 8) -- this module does not
 * render anything itself. */
export const SYNCED_SHARE_GITHUB_REDIRECT_PATH = '/synced-share/github-callback'

const GITHUB_AUTHORIZATION_ENDPOINT = 'https://github.com/login/oauth/authorize'

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

function writeSyncedShareGithubPkceState(value: SyncedShareGithubPkceState): void {
  try {
    sessionStorage.setItem(SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY, JSON.stringify(value))
  } catch {
    // Ignore write failures (e.g. private-browsing storage quotas); the
    // in-flight sign-in attempt simply won't be completable after the
    // redirect, which the completion step (task 8) must handle as a
    // "returned to idle, no share created" case per §0.
  }
}

/** Reads back (without clearing) whatever PKCE state this module last
 * persisted, for the completion step (task 8) to consume. */
export function readPendingSyncedShareGithubSignIn(): SyncedShareGithubPkceState | null {
  return readSyncedShareGithubPkceState()
}

/** Clears the in-flight PKCE state, e.g. once the completion step (task 8)
 * has consumed it, or the sign-in attempt is abandoned. */
export function clearPendingSyncedShareGithubSignIn(): void {
  try {
    sessionStorage.removeItem(SYNCED_SHARE_GITHUB_PKCE_STORAGE_KEY)
  } catch {
    // Nothing to clean up if storage isn't available in the first place.
  }
}

export interface StartSyncedShareGithubSignInOptions {
  /** The Synced Share GitHub OAuth App's client id (public, not secret --
   * reused from the same registered app as `githubAuth.ts`'s device flow,
   * per §0). */
  clientId: string
}

/**
 * Starts the redirect + PKCE "sign in with GitHub" flow: generates a fresh
 * PKCE `code_verifier`/`code_challenge` and `state`, persists the verifier
 * and state so the completion step (task 8) can consume them, then
 * navigates the browser to GitHub's authorization endpoint.
 *
 * No `scope` parameter is sent at all -- GitHub's `GET /user` returns public
 * profile fields (including the numeric id this connection actually needs)
 * to an unscoped token, which is the minimal possible ask per §0.
 *
 * This function navigates away and never resolves under normal operation;
 * it returns only if `window.location.assign` itself throws.
 */
export async function startSyncedShareGithubSignIn(
  options: StartSyncedShareGithubSignInOptions,
): Promise<void> {
  const codeVerifier = oauth.generateRandomCodeVerifier()
  const codeChallenge = await oauth.calculatePKCECodeChallenge(codeVerifier)
  const state = oauth.generateRandomState()
  const redirectUri = new URL(SYNCED_SHARE_GITHUB_REDIRECT_PATH, window.location.origin).toString()

  writeSyncedShareGithubPkceState({ codeVerifier, state, redirectUri })

  const authorizationUrl = new URL(GITHUB_AUTHORIZATION_ENDPOINT)
  authorizationUrl.searchParams.set('client_id', options.clientId)
  authorizationUrl.searchParams.set('redirect_uri', redirectUri)
  authorizationUrl.searchParams.set('response_type', 'code')
  authorizationUrl.searchParams.set('state', state)
  authorizationUrl.searchParams.set('code_challenge', codeChallenge)
  authorizationUrl.searchParams.set('code_challenge_method', 'S256')

  window.location.assign(authorizationUrl.toString())
}
