import { useCallback, useEffect, useRef, useState } from 'react'
import { useAccountAuth } from '../storage/accountAuth'
import {
  openSyncedShareGithubSignInPopup,
  type SyncedShareGithubAuthResult,
} from '../storage/accountAuthPopup'
import { revokeSyncedShareGithubGrant } from '../storage/accountAuthRevoke'
import {
  buildSyncedShareFailure,
  type SyncedShareFailure,
  type SyncedShareOperation,
} from '../syncedShare/errors'
import { callWorker, createWorkerClient } from '../syncedShare/workerClient'
import { buildSyncedShareUrl } from '../syncedShareUrl'

/** Public Synced Share GitHub OAuth App client id -- a dedicated app,
 * separate from the deleted storage-backend device flow. Originally
 * this reused that same app's client id (per §0), but doing so meant
 * GitHub's consent screen for this identity-only sign-in surfaced the
 * storage app's full existing `repo` grant as "existing access", defeating
 * the point of keeping this connection minimally-scoped -- so §0 was
 * revised to require full isolation (a second registered app). Not a
 * secret: it's visible in every authorization request the browser sends. */
const SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID =
  import.meta.env.VITE_SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID ?? ''

/** A file's share state as last reported by the worker. Tagged with the
 * `fileId` it belongs to, so a response for a file the owner has since
 * switched away from can never be read as the current file's state. */
interface FileShareStatus {
  fileId: string
  shareId: string
  ended: boolean
}

export interface UseSyncedShareOwnerResult {
  /** Whether a live link is possible for the current file at all: only a
   * cloud-stored (account-owned) file can be shared live, since the link
   * points at its `files` row. Local and demo files get the static
   * `#share=` link only. */
  canSync: boolean
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether the dedicated Synced Share GitHub sign-in connection
   * (`accountAuth.ts`) currently has a stored token. */
  isGithubConnected: boolean
  /** Cached GitHub username from that connection, for the "Synced as
   * @username" identity chip -- `null` until connected. */
  githubLogin: string | null
  /** Starts (or resumes) sharing this file and returns its viewer link, or
   * `null` if it couldn't. The worker always hands back the file's one
   * share id, so re-sharing -- from any device -- reproduces the same link.
   *
   * Per §0's "no seamless OAuth-then-continue flow" decision, this is a
   * no-op (resolves `null`) while the Synced Share GitHub connection isn't
   * present -- it never itself starts the sign-in popup. Callers (e.g.
   * `ShareModal`) are expected to check `isGithubConnected` first and
   * show its sign-in state instead of calling this. A failed request (e.g.
   * GitHub verification failure) resolves `null` too, surfaced instead via
   * `syncFailure` below. */
  startSync: () => Promise<string | null>
  stopSync: () => void
  /** Opens the popup "sign in with GitHub" flow for the dedicated Synced
   * Share connection. Resolving this promise never itself starts a share --
   * per §0, the user must click "start sync" again once connected. */
  signInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  /** Logs out of the dedicated Synced Share GitHub connection -- clears the
   * stored token/login (so the "Synced as @username" chip disappears and
   * `isGithubConnected` goes back to `false`), stopping the current file's
   * live share first (mirrors clicking "Stop Sync"). */
  disconnectGithub: () => void
  /** Set whenever a request (start, stop, or the status lookup) fails --
   * the worker's `unauthorized` GitHub-verification failure, any other
   * failure response, or a network-level error. Drives the full-screen error dialog
   * (task 9); `null` means no failure is currently being shown. There is no
   * automatic retry -- dismissing it (`dismissSyncFailure`) just returns to
   * idle. An auth rejection additionally clears the stored GitHub connection (see
   * `recordSyncFailure`), so the next "Sync" click re-prompts sign-in
   * instead of resending the same now-invalid token forever. */
  syncFailure: SyncedShareFailure | null
  dismissSyncFailure: () => void
}

/**
 * Owns the owner side of a Synced Share for the current file. A share is a
 * pointer to the file's cloud `files` row (see
 * `crates/live-share-worker/src/share.rs`): viewers read the file's saved
 * content directly, so this hook never sends content anywhere -- the
 * ordinary cloud autosave is the only write path.
 *
 * Whether the file is shared lives on the server, fetched from `POST
 * /files/:id/share/status` whenever the file or the signed-in identity
 * changes, so every device the owner signs in on sees the same live/stopped
 * state and link. `isSynced` is derived from that status (and only counts
 * when it's tagged with the current file), never mirrored into separate
 * state, so a file switch can't briefly pair one file with another's share.
 *
 * Ownership is always a verified GitHub identity (§0) -- there is no
 * anonymous/device-secret ownership path.
 */
export function useSyncedShareOwner(
  filename: string,
  /** The current file's cloud `files.id` (see `useScoreSource.ts`'s
   * `cloudFileIdFor`), `null` for a local or demo file, which can't be
   * shared live. */
  cloudFileId: string | null,
): UseSyncedShareOwnerResult {
  const [status, setStatus] = useState<FileShareStatus | null>(null)
  const [syncFailure, setSyncFailure] = useState<SyncedShareFailure | null>(
    null,
  )
  const [githubAuth, setGithubAuth] = useAccountAuth()
  const isGithubConnected = githubAuth !== null
  const githubLogin = githubAuth?.login ?? null
  const identityToken = githubAuth?.token ?? null

  // The file currently on screen, read by async completions below so a
  // response that lands after the owner switched files is dropped instead
  // of overwriting the new file's status.
  const cloudFileIdRef = useRef(cloudFileId)
  cloudFileIdRef.current = cloudFileId

  const applyStatusFor = useCallback(
    (fileId: string, next: FileShareStatus | null) => {
      if (cloudFileIdRef.current === fileId) setStatus(next)
    },
    [],
  )

  /** Records a request failure and, if it's an auth rejection (the worker's
   * `unauthorized` `ApiError` -- GitHub itself rejected the stored identity
   * token as revoked/invalid, not merely a transient network blip), also
   * clears the stored Synced Share GitHub auth: without this, a token
   * that's gone stale (e.g. its GitHub grant was revoked) keeps being
   * resent forever, and every retry just reproduces the same 401 and
   * re-shows this dialog instead of prompting a fresh sign-in. */
  const recordSyncFailure = useCallback(
    (failure: SyncedShareFailure) => {
      setSyncFailure(failure)
      if (failure.authRejected) setGithubAuth(null)
    },
    [setGithubAuth],
  )

  /** Sends one owner-side request (`send`, given the worker client).
   * Resolves `{ data }` with its success body, or `null` after recording the
   * failure. */
  const runShareRequest = useCallback(
    async <Data>(
      operation: SyncedShareOperation,
      send: (client: ReturnType<typeof createWorkerClient>) => Promise<Data>,
    ): Promise<{ data: Data } | null> => {
      const host = import.meta.env.VITE_SYNCED_SHARE_HOST
      if (!host) return null
      try {
        return { data: await send(createWorkerClient(host)) }
      } catch (error) {
        recordSyncFailure(buildSyncedShareFailure(operation, error))
        return null
      }
    },
    [recordSyncFailure],
  )

  useEffect(() => {
    if (!cloudFileId || !identityToken) return
    void runShareRequest('status', (client) =>
      callWorker(
        client.POST('/files/{id}/share/status', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    ).then((result) => {
      if (!result) return
      const { share } = result.data
      applyStatusFor(
        cloudFileId,
        share ? { fileId: cloudFileId, ...share } : null,
      )
    })
  }, [cloudFileId, identityToken, runShareRequest, applyStatusFor])

  const startSync = useCallback(async (): Promise<string | null> => {
    // Per §0's "no seamless OAuth-then-continue flow" decision: this never
    // triggers the sign-in popup itself, it just declines to start a share.
    // `ShareModal` checks `isGithubConnected` up front and shows its
    // sign-in state instead of calling this in that case -- this check is
    // a defensive backstop, not the primary gate.
    if (!cloudFileId || !identityToken) return null
    const result = await runShareRequest('start', (client) =>
      callWorker(
        client.POST('/files/{id}/share', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    )
    if (!result) return null
    const { shareId } = result.data
    applyStatusFor(cloudFileId, {
      fileId: cloudFileId,
      shareId,
      ended: false,
    })
    return buildSyncedShareUrl(shareId, filename)
  }, [cloudFileId, identityToken, filename, runShareRequest, applyStatusFor])

  const isSynced =
    cloudFileId !== null &&
    identityToken !== null &&
    status?.fileId === cloudFileId &&
    !status.ended

  const stopSync = useCallback(() => {
    if (!isSynced || !cloudFileId || !identityToken || !status) return
    const stopped = { ...status, ended: true }
    void runShareRequest('stop', (client) =>
      callWorker(
        client.POST('/files/{id}/share/stop', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    ).then((result) => {
      if (result) applyStatusFor(cloudFileId, stopped)
    })
  }, [
    isSynced,
    cloudFileId,
    identityToken,
    status,
    runShareRequest,
    applyStatusFor,
  ])

  const dismissSyncFailure = useCallback(() => {
    setSyncFailure(null)
  }, [])

  const signInWithGithub =
    useCallback((): Promise<SyncedShareGithubAuthResult> => {
      return openSyncedShareGithubSignInPopup({
        clientId: SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID,
      })
    }, [])

  const disconnectGithub = useCallback(() => {
    stopSync()
    // GitHub's real `/authorize` endpoint has no "force fresh consent"
    // parameter -- revoking this app's authorization grant now is the only
    // genuine way to make the *next* sign-in re-show GitHub's consent
    // screen instead of silently reusing this one. Fire-and-forget: local
    // state always wins, a failed revocation never blocks logging out.
    if (githubAuth) {
      const host = import.meta.env.VITE_SYNCED_SHARE_HOST ?? ''
      void revokeSyncedShareGithubGrant({
        host,
        identityToken: githubAuth.token,
      })
    }
    setStatus(null)
    setGithubAuth(null)
  }, [stopSync, githubAuth, setGithubAuth])

  return {
    canSync: cloudFileId !== null,
    isSynced,
    syncedShareLink:
      isSynced && status ? buildSyncedShareUrl(status.shareId, filename) : null,
    isGithubConnected,
    githubLogin,
    startSync,
    stopSync,
    signInWithGithub,
    disconnectGithub,
    syncFailure,
    dismissSyncFailure,
  }
}
