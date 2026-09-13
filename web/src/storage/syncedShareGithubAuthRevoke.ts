import { syncedShareWorkerOrigin } from '../syncedShare/workerUrl'

/**
 * Fire-and-forget call to the Worker's `POST /auth/github/revoke`
 * (`crates/live-share-worker/src/oauth.rs`), run from `disconnectGithub`
 * (`useSyncedShareOwner.ts`) at logout time. GitHub's real `/authorize`
 * endpoint has no "force fresh consent" parameter -- revoking this app's
 * authorization grant now is the only genuine way to make the *next*
 * sign-in re-show GitHub's consent screen instead of silently reusing the
 * old grant.
 *
 * Never throws, and the caller never awaits it: a failed revocation just
 * means the next sign-in might silently reuse the old grant, not a broken
 * app state -- the local token being revoked is already being discarded by
 * the caller regardless of whether this call succeeds.
 */
export async function revokeSyncedShareGithubGrant(options: {
  host: string
  identityToken: string
}): Promise<void> {
  try {
    await fetch(`${syncedShareWorkerOrigin(options.host)}/auth/github/revoke`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ identityToken: options.identityToken }),
    })
  } catch {
    // Best-effort: see the doc comment above.
  }
}
