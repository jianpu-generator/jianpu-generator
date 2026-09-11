import { useCallback, useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import {
  openSyncedShareGithubSignInPopup,
  type SyncedShareGithubAuthResult,
  useSyncedShareGithubAuth,
} from '../storage/syncedShareGithubAuth'
import {
  buildSyncedShareNetworkFailure,
  buildSyncedShareResponseFailure,
  type SyncedShareFailure,
} from '../syncedShare/errors'
import type {
  CreateShareRequest,
  CreateShareResponse,
  SyncedStopRequest,
  SyncedUpdateRequest,
} from '../syncedShare/protocol'
import { buildSyncedShareUrl } from '../syncedShareUrl'
import { AUTOSAVE_DEBOUNCE_MS } from './useStorageBackend'

/** Public Synced Share GitHub OAuth App client id -- a dedicated app,
 * separate from `githubAuth.ts`'s storage-backend device flow. Originally
 * this reused that same app's client id (per §0), but doing so meant
 * GitHub's consent screen for this identity-only sign-in surfaced the
 * storage app's full existing `repo` grant as "existing access", defeating
 * the point of keeping this connection minimally-scoped -- so §0 was
 * revised to require full isolation (a second registered app). Not a
 * secret: it's visible in every authorization request the browser sends. */
const SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID =
  import.meta.env.VITE_SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID ?? ''

function activeFlagKey(fileId: string): string {
  return `jianpu:synced-share-active:v1:${fileId}`
}

function readActiveFlag(fileId: string): boolean {
  return localStorage.getItem(activeFlagKey(fileId)) === 'true'
}

/** Persists the server-generated `shareId` for a file (§1: "Client persists
 * the returned shareId locally (keyed by file) so stopping/resuming sync on
 * the same file reuses the same link"), keyed the same way as
 * `activeFlagKey` above. There is no more client-side derivation
 * (`deriveSyncedShareIdentity`, deleted task 11) -- the id only ever comes
 * from the worker's `POST /shares` response. */
function shareIdStorageKey(fileId: string): string {
  return `jianpu:synced-share-id:v1:${fileId}`
}

function readStoredShareId(fileId: string): string | null {
  return localStorage.getItem(shareIdStorageKey(fileId))
}

function writeStoredShareId(fileId: string, shareId: string): void {
  localStorage.setItem(shareIdStorageKey(fileId), shareId)
}

function syncedShareEndpointUrl(host: string, shareId: string): string {
  return `https://${host}/shares/${shareId}`
}

function createShareEndpointUrl(host: string): string {
  return `https://${host}/shares`
}

export interface UseSyncedShareOwnerResult {
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether the dedicated Synced Share GitHub sign-in connection
   * (`syncedShareGithubAuth.ts`, distinct from `githubAuth.ts`'s
   * storage-backend connection) currently has a stored token. */
  isGithubConnected: boolean
  /** Cached GitHub username from that connection, for the "Synced as
   * @username" identity chip -- `null` until connected. */
  githubLogin: string | null
  /** Starts (or resumes) syncing this file and returns its viewer link, or
   * `null` if it couldn't. Async because starting a share now requires a
   * `POST /shares` round trip to the worker the first time (no more
   * client-side derivation) -- a resumed share (persisted `shareId` already
   * on this device) still resolves immediately with no network call.
   *
   * Per §0's "no seamless OAuth-then-continue flow" decision, this is a
   * no-op (resolves `null`) while the Synced Share GitHub connection isn't
   * present -- it never itself starts the sign-in popup. Callers (e.g.
   * `SyncedShareButton`) are expected to check `isGithubConnected` first and
   * show the sign-in prompt (mockup Screen 1) instead of calling this. A
   * failed `POST /shares` (e.g. GitHub verification failure) resolves
   * `null` too, surfaced instead via `syncFailure` below. */
  startSync: () => Promise<string | null>
  stopSync: () => void
  broadcastContent: (content: string) => void
  /** Opens the popup "sign in with GitHub" flow for the dedicated Synced
   * Share connection. Resolving this promise never itself starts a share --
   * per §0, the user must click "start sync" again once connected. */
  signInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  /** Logs out of the dedicated Synced Share GitHub connection -- clears the
   * stored token/login (so the "Synced as @username" chip disappears and
   * `isGithubConnected` goes back to `false`), stopping any active sync
   * first (mirrors clicking "Stop Sync": a sync can't keep pushing updates
   * without a verified identity to sign them with). */
  disconnectGithub: () => void
  /** Set whenever a write (a "create", "update" push, or a "stop") fails --
   * a `401` GitHub-verification failure from the worker, a non-2xx
   * response, or a network-level error. Drives the full-screen error dialog
   * (task 9); `null` means no failure is currently being shown. There is no
   * automatic retry -- dismissing it (`dismissSyncFailure`) just returns to
   * idle. */
  syncFailure: SyncedShareFailure | null
  dismissSyncFailure: () => void
}

/**
 * Owns the owner side of a Synced Share session for a single file. `fileId`
 * (stable across renames, unlike `filename`) keys both the persisted
 * `shareId` (see `shareIdStorageKey`) and the "is this file synced" flag, so
 * a session survives a rename and reproduces the same link across
 * stop/start cycles.
 *
 * There is no persistent connection: `broadcastContent` just `POST`s the
 * current content to the share's `docs` row, debounced at the same
 * `AUTOSAVE_DEBOUNCE_MS` cadence as a regular save (not on every keystroke)
 * — a viewer only sees a push once they reload, so there is no benefit to
 * pushing more often than the content is actually persisted.
 *
 * Ownership is always a verified GitHub identity (§0) -- there is no more
 * anonymous/device-secret ownership path (task 11 deleted
 * `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity`).
 */
export function useSyncedShareOwner(
  filename: string,
  fileId: string,
  content: string,
): UseSyncedShareOwnerResult {
  const [isActive, setIsActive] = useState(() => readActiveFlag(fileId))
  const [syncFailure, setSyncFailure] = useState<SyncedShareFailure | null>(
    null,
  )
  const [githubAuth, setGithubAuth] = useSyncedShareGithubAuth()
  const isGithubConnected = githubAuth !== null
  const githubLogin = githubAuth?.login ?? null
  const [shareId, setShareId] = useState<string | null>(() =>
    readStoredShareId(fileId),
  )
  // Mirrors `shareId` for the click handler below, which needs to read it
  // synchronously (state updates aren't visible until the next render).
  const shareIdRef = useRef<string | null>(shareId)
  const revisionRef = useRef(0)
  // Mirrors the latest `content` prop so the "just started syncing" effect
  // below (which can't see React props at the time it fires) can push the
  // share's very first doc without waiting for the owner to make an edit.
  const contentRef = useRef(content)
  contentRef.current = content

  useEffect(() => {
    const stored = readStoredShareId(fileId)
    setIsActive(readActiveFlag(fileId))
    shareIdRef.current = stored
    setShareId(stored)
  }, [fileId])

  const session = isActive ? shareId : null

  const pushUpdate = useCallback(
    (content: string) => {
      const host = import.meta.env.VITE_SYNCED_SHARE_HOST
      if (!session || !host || !githubAuth) return
      revisionRef.current += 1
      const request: SyncedUpdateRequest = {
        type: 'update',
        identityToken: githubAuth.token,
        filename,
        content,
        revision: revisionRef.current,
      }
      void fetch(syncedShareEndpointUrl(host, session), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      })
        .then(async (response) => {
          if (response.ok) return
          setSyncFailure(
            await buildSyncedShareResponseFailure('update', response),
          )
        })
        .catch((error: unknown) => {
          setSyncFailure(buildSyncedShareNetworkFailure('update', error))
        })
    },
    [filename, session, githubAuth],
  )

  // Pushes the initial doc the moment a session starts syncing (including a
  // page reload while already syncing), so a viewer opening the link right
  // away doesn't find an empty share from before the owner's first edit.
  useEffect(() => {
    if (!session) return
    pushUpdate(contentRef.current)
    // `pushUpdate` itself only changes identity when `filename`/`session` do
    // (see its own `useCallback` deps), so listing it here doesn't cause
    // this to re-run on every content change — it still only fires on
    // session identity change, deliberately not on every edit. See
    // `broadcastContent` below for the debounced path edits actually take.
  }, [session, pushUpdate])

  const broadcastContent = useDebouncedCallback(
    pushUpdate,
    AUTOSAVE_DEBOUNCE_MS,
  )

  /** Calls `POST /shares` to mint a brand-new server-generated share id
   * (§1). Failures are surfaced via `syncFailure`, same as a write. */
  const createShare = useCallback(
    async (host: string, identityToken: string): Promise<string | null> => {
      const request: CreateShareRequest = { identityToken }
      try {
        const response = await fetch(createShareEndpointUrl(host), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(request),
        })
        if (!response.ok) {
          setSyncFailure(
            await buildSyncedShareResponseFailure('create', response),
          )
          return null
        }
        const body = (await response.json()) as CreateShareResponse
        return body.shareId
      } catch (error) {
        setSyncFailure(buildSyncedShareNetworkFailure('create', error))
        return null
      }
    },
    [],
  )

  const startSync = useCallback(async (): Promise<string | null> => {
    // Per §0's "no seamless OAuth-then-continue flow" decision: this never
    // triggers the sign-in popup itself, it just declines to start a share.
    // `SyncedShareButton` checks `isGithubConnected` up front and shows the
    // sign-in prompt (mockup Screen 1) instead of calling this in that case
    // -- this check is a defensive backstop, not the primary gate.
    if (!isGithubConnected || !githubAuth) return null
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    if (!host) return null

    let currentShareId = shareIdRef.current
    if (!currentShareId) {
      currentShareId = await createShare(host, githubAuth.token)
      if (!currentShareId) return null
      shareIdRef.current = currentShareId
      setShareId(currentShareId)
      writeStoredShareId(fileId, currentShareId)
    }

    localStorage.setItem(activeFlagKey(fileId), 'true')
    revisionRef.current = 0
    setIsActive(true)
    return buildSyncedShareUrl(currentShareId, filename)
  }, [fileId, filename, isGithubConnected, githubAuth, createShare])

  const stopSync = useCallback(() => {
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    const current = session
    localStorage.setItem(activeFlagKey(fileId), 'false')
    setIsActive(false)
    if (!current || !host || !githubAuth) return
    const stop: SyncedStopRequest = {
      type: 'stop',
      identityToken: githubAuth.token,
    }
    void fetch(syncedShareEndpointUrl(host, current), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(stop),
    })
      .then(async (response) => {
        if (response.ok) return
        setSyncFailure(await buildSyncedShareResponseFailure('stop', response))
      })
      .catch((error: unknown) => {
        setSyncFailure(buildSyncedShareNetworkFailure('stop', error))
      })
  }, [fileId, session, githubAuth])

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
    if (session) stopSync()
    setGithubAuth(null)
  }, [session, stopSync, setGithubAuth])

  return {
    isSynced: session !== null,
    syncedShareLink: session ? buildSyncedShareUrl(session, filename) : null,
    isGithubConnected,
    githubLogin,
    startSync,
    stopSync,
    broadcastContent,
    signInWithGithub,
    disconnectGithub,
    syncFailure,
    dismissSyncFailure,
  }
}
