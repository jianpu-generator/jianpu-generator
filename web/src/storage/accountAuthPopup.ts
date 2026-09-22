import * as oauth from 'oauth4webapi'

/**
 * Opens and drives the popup half of Synced Share's "sign in with GitHub"
 * flow (task 8): opening the authorization request as a **popup** (not a
 * full-page redirect, per the mockup's "OAuth popup" screen), and waiting
 * for `SyncedShareGithubCallbackPage` (rendered at
 * `SYNCED_SHARE_GITHUB_REDIRECT_PATH` -- see `main.tsx`) to relay the
 * outcome back once GitHub redirects the popup back there.
 * `completeSyncedShareGithubSignInFromCallback` (which exchanges the code,
 * persists the resulting token, and relays the outcome back to the opener
 * window via both `postMessage` and a `localStorage` relay -- see
 * `SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY`'s doc comment, which is also
 * what actually closes the popup, from the opener's side, not the popup's
 * own `window.close()`) lives in `./accountAuthCallback.ts`.
 * `openSyncedShareGithubSignInPopup` below is what `useSyncedShareOwner.ts`
 * calls to drive the whole round trip from the opener's side. Per §0's "no
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
 * the flow via `completeSyncedShareGithubSignInFromCallback` -- this module
 * does not render anything itself. */
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

/** `postMessage` type tag the callback popup uses to relay its result back
 * to the opener window. Scoped to this app's own origin on both send and
 * receive (see `openSyncedShareGithubSignInPopup`). Exported so
 * `accountAuthCallback.ts` can send the matching message. */
export const SYNCED_SHARE_GITHUB_AUTH_MESSAGE_TYPE =
  'jianpu:synced-share-github-auth-result'

/** `localStorage` key used as a same-origin relay channel for the popup to
 * signal its result back to the opener, alongside (not instead of) the
 * `postMessage` above. `postMessage` (via `relayResultToOpener` in
 * `accountAuthCallback.ts`) requires `window.opener` to still be
 * set from the popup's *own* side -- but GitHub's authorization endpoint
 * responds with `Cross-Origin-Opener-Policy: same-origin`, which
 * permanently severs `window.opener` once the popup navigates there, even
 * after it later navigates back to this app's own origin. That severs the
 * popup's own `window.close()` too (browsers refuse to let a script close a
 * window that no longer reports an opener), which is exactly the "popup
 * doesn't close itself" bug this relay exists to work around. `localStorage`
 * is scoped per *origin*, not per browsing-context-group, so the `storage`
 * event a write here fires reaches every other same-origin window
 * (including the opener) regardless of that severance. The opener listens
 * for it in `openSyncedShareGithubSignInPopup` and closes the popup itself
 * once it has a result -- its own reference to the popup (from
 * `window.open()`'s return value) was never subject to the severance, so
 * closing from that side works even when the popup can no longer close
 * itself or `postMessage` it directly. */
export const SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY =
  'jianpu:synced-share-github-auth-relay:v1'

const GITHUB_AUTHORIZATION_ENDPOINT = 'https://github.com/login/oauth/authorize'

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
  // GitHub's real `/authorize` endpoint has no "force fresh consent"
  // parameter (only `client_id`, `redirect_uri`, `login`, `scope`, `state`,
  // `allow_signup`, plus PKCE fields are supported) -- so this request
  // alone can't force GitHub to re-show its login/consent screen when the
  // browser still has a live github.com session and a prior grant for this
  // app. Forced re-consent is instead achieved by revoking the grant at
  // logout time (`disconnectGithub` in `useSyncedShareOwner.ts`, via
  // `revokeSyncedShareGithubGrant`): the *next* call to this endpoint then
  // genuinely has no grant to silently reuse.
  // No `scope` parameter is sent at all -- GitHub's `GET /user` returns
  // public profile fields (including the numeric id this connection
  // actually needs) to an unscoped token, which is the minimal possible ask
  // per §0.

  return authorizationUrl.toString()
}

export interface OpenSyncedShareGithubSignInPopupOptions {
  /** The Synced Share GitHub OAuth App's client id (public, not secret --
   * its own dedicated, registered app, separate from the deleted
   * storage-backend device flow, per §0). */
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

    // A `const` arrow function, not a hoisted `function` declaration --
    // TypeScript only carries the `popup` non-null narrowing above into a
    // closure defined after it, not into a hoisted one.
    const finish = (result: SyncedShareGithubAuthResult) => {
      if (settled) return
      settled = true
      window.clearInterval(closedPoll)
      window.removeEventListener('message', onMessage)
      window.removeEventListener('storage', onStorage)
      // Close from here rather than trust the popup's own `window.close()`
      // (`SyncedShareGithubCallbackPage`) -- see the relay storage key's
      // doc comment above for why that can be silently refused once
      // GitHub's `Cross-Origin-Opener-Policy` header has severed the
      // popup's `window.opener`, while this reference (held since
      // `window.open()` returned it) is unaffected. A no-op if the popup
      // already closed itself or the user closed it by hand.
      if (!popup.closed) popup.close()
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

    // COOP-safe fallback for `onMessage` above -- see the relay storage
    // key's doc comment for why `postMessage` alone isn't reliable here.
    function onStorage(event: StorageEvent) {
      if (event.key !== SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY) return
      if (!event.newValue) return
      try {
        const { nonce: _nonce, ...result } = JSON.parse(
          event.newValue,
        ) as Record<string, unknown>
        finish(result as unknown as SyncedShareGithubAuthResult)
      } catch {
        // Malformed value -- ignore; the `.closed` poll below is still a
        // backstop if this was the only channel that was going to fire.
      }
    }

    window.addEventListener('message', onMessage)
    window.addEventListener('storage', onStorage)

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
