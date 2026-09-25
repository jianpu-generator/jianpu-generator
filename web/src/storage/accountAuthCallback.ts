import {
  callWorker,
  createWorkerClient,
  NetworkFailure,
  UnreadableResponse,
  WorkerRequestError,
  type WorkerSchemas,
} from '../syncedShare/workerClient'
import { writeStoredSyncedShareGithubAuth } from './accountAuth'
import {
  clearPendingSyncedShareGithubSignIn,
  readPendingSyncedShareGithubSignIn,
  SYNCED_SHARE_GITHUB_AUTH_MESSAGE_TYPE,
  SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY,
  type SyncedShareGithubAuthResult,
} from './accountAuthPopup'

/**
 * Completes the popup half of Synced Share's "sign in with GitHub" flow --
 * see `accountAuthPopup.ts`'s doc comment for the flow this
 * finishes.
 */

export interface CompleteSyncedShareGithubSignInFromCallbackOptions {
  /** Host (no scheme) of the Synced Share worker's `/auth/github/callback`
   * route -- the same `VITE_SYNCED_SHARE_HOST` env var
   * `useSyncedShareOwner.ts` uses for the `/files/:id/share` routes. */
  host: string
  /** Defaults to `window.location.search`; overridable for tests. */
  search?: string
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
  relayResultViaStorage(result)
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

  let body: WorkerSchemas['GithubOauthCallbackResponse']
  try {
    body = await callWorker(
      createWorkerClient(options.host).POST('/auth/github/callback', {
        body: {
          code,
          codeVerifier: pending.codeVerifier,
          redirectUri: pending.redirectUri,
        },
      }),
    )
  } catch (error) {
    // Every failure, including a `2xx` whose body isn't valid JSON (e.g. a
    // truncated response from a worker restarting mid-request), must
    // resolve rather than throw out of this function -- see
    // `completeSyncedShareGithubSignInFromCallback`'s caller
    // (`SyncedShareGithubCallbackPage`), which has no `.catch` on this
    // promise and would otherwise leave the popup stuck on "Signing in with
    // GitHub…" forever (an unhandled rejection, not a resolved failure the
    // UI can render).
    return {
      ok: false,
      reason: 'error',
      error: describeTokenExchangeFailure(error),
    }
  }
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

/** COOP-safe companion to `relayResultToOpener` above -- see
 * `SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY`'s doc comment for why this
 * is needed at all. Deliberately *not* gated on `window.opener` (unlike
 * `relayResultToOpener`): `window.opener` is exactly the thing COOP
 * severance nulls out, so gating this the same way would defeat its whole
 * purpose. Writing when this page was opened directly (no popup, no
 * opener listening) is harmless -- just an unread `localStorage` entry. */
function relayResultViaStorage(result: SyncedShareGithubAuthResult): void {
  try {
    // `nonce` guarantees the write always changes the stored value, and
    // therefore always fires a `storage` event in the opener, even when
    // two consecutive attempts resolve to the exact same result --
    // per spec, a `setItem` call that doesn't actually change the value
    // fires no event at all.
    localStorage.setItem(
      SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY,
      JSON.stringify({
        ...result,
        nonce: Math.random().toString(36).slice(2),
      }),
    )
  } catch {
    // Best-effort only (e.g. private-browsing storage quotas) --
    // `relayResultToOpener` above or the opener's `.closed` poll are the
    // remaining fallbacks if this write fails.
  }
}

function describeTokenExchangeFailure(error: unknown): string {
  if (error instanceof WorkerRequestError) {
    const detail =
      error.apiError?.code === 'upstream_failed'
        ? error.apiError.message
        : JSON.stringify(error.rawBody)
    return `GitHub token exchange failed: status=${error.response.status}, body=${detail}`
  }
  if (error instanceof UnreadableResponse) {
    return `GitHub token exchange returned an unreadable response: ${error.message}`
  }
  const message =
    error instanceof NetworkFailure || error instanceof Error
      ? error.message
      : String(error)
  return `Could not reach the Synced Share worker: ${message}`
}
