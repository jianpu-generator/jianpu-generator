import { createWorkerClient } from '../syncedShare/workerClient'

/**
 * Fire-and-forget call to the Worker's `POST /auth/github/revoke`
 * (`crates/live-share-worker/src/oauth.rs`), run from `disconnectGithub`
 * (`useSyncedShareOwner.ts`) at logout time. Revokes just this device's
 * token -- never the app's whole GitHub grant, which would also sign the
 * owner out on every other device (see `github_revoke`'s doc comment).
 *
 * Never throws, and the caller never awaits it: a failed revocation just
 * leaves a discarded token alive on GitHub's side, not a broken app state
 * -- the local token is already being discarded by the caller regardless
 * of whether this call succeeds.
 */
export async function revokeSyncedShareGithubToken(options: {
  host: string
  identityToken: string
}): Promise<void> {
  try {
    await createWorkerClient(options.host).POST('/auth/github/revoke', {
      body: { identityToken: options.identityToken },
    })
  } catch {
    // Best-effort: see the doc comment above.
  }
}
