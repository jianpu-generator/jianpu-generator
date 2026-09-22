import { useLocalStorage } from 'usehooks-ts'

/**
 * Dedicated, minimally-scoped "sign in with GitHub" connection used to
 * verify an account's identity (`GET /user`, no other scope) for both
 * Synced Share ownership and cloud file storage -- see
 * `TODO-synced-share-rust-d1-migration.md` §0/§6. This was originally
 * deliberately kept separate from the storage backend's now-deleted
 * broad-scope, opt-in device-flow connection (which was used for direct
 * GitHub Contents API access before storage moved to D1); the two
 * connections stored their tokens under different
 * `localStorage`/`sessionStorage` keys and were never conflated -- sending
 * this connection's token on a write must never imply Contents API access,
 * and vice versa.
 *
 * Per that same decision, this originally reused the *same registered
 * GitHub OAuth App* (`client_id`) as that device-flow connection, differing
 * only in the requested `scope` (none, here) and the flow (redirect + PKCE,
 * not device flow). That was revised: GitHub grants OAuth App scope per app
 * per user,
 * not per individual authorization request, so sharing a `client_id` meant
 * this identity-only sign-in's consent (and later reauthorization) screen
 * surfaced the storage app's full existing `repo` grant as "existing
 * access" once a user had ever connected the storage backend -- exactly the
 * broad-scope over-ask this dedicated connection exists to avoid. This now
 * uses its own separate registered OAuth App and `client_id`
 * (`VITE_SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID`), giving it full isolation.
 *
 * This module holds just the persisted-auth-token storage (the bit every
 * other piece of the flow reads/writes). The popup "open sign-in" half
 * (opening the authorization request as a popup, PKCE state, the
 * authorization URL) lives in `./accountAuthPopup.ts`; completing
 * the flow once GitHub redirects back (`SyncedShareGithubCallbackPage`
 * calls `completeSyncedShareGithubSignInFromCallback`) lives in
 * `./accountAuthCallback.ts`. Both were split out of this file to
 * stay under the project's per-file line limit -- see
 * `accountAuthPopup.ts`'s doc comment for the rest of the flow
 * description.
 */

/** `localStorage` key holding the persisted Synced Share GitHub sign-in
 * token -- deliberately a different key from the deleted device-flow
 * storage backend's key, per this module's doc comment: the two
 * connections' tokens are never conflated. */
export const SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY =
  'jianpu:synced-share-github-auth:v1'

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
 * cycle).
 */
export function readStoredSyncedShareGithubAuth(): StoredSyncedShareGithubAuth | null {
  return readStoredSyncedShareGithubAuthRaw()
}

/** Persists (or, given `null`, clears) the Synced Share GitHub sign-in
 * token. Exported so `accountAuthCallback.ts` can call it once the
 * token exchange succeeds. */
export function writeStoredSyncedShareGithubAuth(
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
 * the moment a popup sign-in completes.
 */
export function useAccountAuth() {
  return useLocalStorage<StoredSyncedShareGithubAuth | null>(
    SYNCED_SHARE_GITHUB_AUTH_STORAGE_KEY,
    null,
  )
}
