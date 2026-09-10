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
  SyncedStopRequest,
  SyncedUpdateRequest,
} from '../syncedShare/protocol'
import {
  buildSyncedShareUrl,
  deriveSyncedShareIdentity,
  getOrCreateDeviceSecret,
  type SyncedShareIdentity,
} from '../syncedShareUrl'
import { AUTOSAVE_DEBOUNCE_MS } from './useStorageBackend'

/** Public Synced Share GitHub OAuth App client id -- reused from the same
 * registered app as `githubAuth.ts`'s device flow (per §0), only the
 * requested scope and flow differ. Not a secret: it's visible in every
 * authorization request the browser sends. */
const SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID =
  import.meta.env.VITE_GITHUB_OAUTH_CLIENT_ID ?? ''

function activeFlagKey(fileId: string): string {
  return `jianpu:synced-share-active:v1:${fileId}`
}

function readActiveFlag(fileId: string): boolean {
  return localStorage.getItem(activeFlagKey(fileId)) === 'true'
}

function syncedShareEndpointUrl(host: string, shareId: string): string {
  return `https://${host}/shares/${shareId}`
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
  /** Marks the file as synced and returns its viewer link — returned
   * synchronously so the caller can copy it to the clipboard in the same
   * click handler that starts the session. The link is deterministic (see
   * `deriveSyncedShareIdentity`), so this reproduces the same link every time
   * rather than minting a new one.
   *
   * Per §0's "no seamless OAuth-then-continue flow" decision, this is a
   * no-op (returns `null`) while the Synced Share GitHub connection isn't
   * present -- it never itself starts the sign-in popup. Callers (e.g.
   * `SyncedShareButton`) are expected to check `isGithubConnected` first and
   * show the sign-in prompt (mockup Screen 1) instead of calling this. */
  startSync: () => string | null
  stopSync: () => void
  broadcastContent: (content: string) => void
  /** Opens the popup "sign in with GitHub" flow for the dedicated Synced
   * Share connection. Resolving this promise never itself starts a share --
   * per §0, the user must click "start sync" again once connected. */
  signInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  /** Set whenever a write (an "update" push or a "stop") fails -- a `401`
   * GitHub-verification failure from the worker, a non-2xx response, or a
   * network-level error. Drives the full-screen error dialog (task 9);
   * `null` means no failure is currently being shown. There is no automatic
   * retry -- dismissing it (`dismissSyncFailure`) just returns to idle. */
  syncFailure: SyncedShareFailure | null
  dismissSyncFailure: () => void
}

/**
 * Owns the owner side of a Synced Share session for a single file. `fileId`
 * (stable across renames, unlike `filename`) keys both the share identity
 * derivation and the persisted "is this file synced" flag, so a session
 * survives a rename and reproduces the same link across stop/start cycles.
 *
 * There is no persistent connection: `broadcastContent` just `PUT`s the
 * current content to the share's KV entry, debounced at the same
 * `AUTOSAVE_DEBOUNCE_MS` cadence as a regular save (not on every keystroke)
 * — a viewer only sees a push once they reload, so there is no benefit to
 * pushing more often than the content is actually persisted.
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
  const [githubAuth] = useSyncedShareGithubAuth()
  const isGithubConnected = githubAuth !== null
  const githubLogin = githubAuth?.login ?? null
  const [identity, setIdentity] = useState<SyncedShareIdentity | null>(null)
  // Mirrors `identity` for the click handler below, which needs to read it
  // synchronously (state updates aren't visible until the next render).
  const identityRef = useRef<SyncedShareIdentity | null>(null)
  const revisionRef = useRef(0)
  // Mirrors the latest `content` prop so the "just started syncing" effect
  // below (which can't see React props at the time it fires) can push the
  // share's very first doc without waiting for the owner to make an edit.
  const contentRef = useRef(content)
  contentRef.current = content

  useEffect(() => {
    setIsActive(readActiveFlag(fileId))
    setIdentity(null)
    identityRef.current = null
    let cancelled = false
    void deriveSyncedShareIdentity(getOrCreateDeviceSecret(), fileId).then(
      (next) => {
        if (cancelled) return
        identityRef.current = next
        setIdentity(next)
      },
    )
    return () => {
      cancelled = true
    }
  }, [fileId])

  const session = isActive ? identity : null

  const pushUpdate = useCallback(
    (content: string) => {
      const host = import.meta.env.VITE_SYNCED_SHARE_HOST
      if (!session || !host) return
      revisionRef.current += 1
      const request: SyncedUpdateRequest = {
        type: 'update',
        ownerToken: session.ownerToken,
        filename,
        content,
        revision: revisionRef.current,
        // Sent whenever the Synced Share GitHub connection is present, per
        // task 8 -- see `SyncedIdentityFields` in `syncedShare/protocol.ts`.
        // The old `ownerToken` field above is untouched (task 11's job).
        ...(githubAuth ? { identityToken: githubAuth.token } : {}),
      }
      void fetch(syncedShareEndpointUrl(host, session.shareId), {
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

  const startSync = useCallback((): string | null => {
    // Per §0's "no seamless OAuth-then-continue flow" decision: this never
    // triggers the sign-in popup itself, it just declines to start a share.
    // `SyncedShareButton` checks `isGithubConnected` up front and shows the
    // sign-in prompt (mockup Screen 1) instead of calling this in that case
    // -- this check is a defensive backstop, not the primary gate.
    if (!isGithubConnected) return null
    // In practice always populated by the time a user can click: derivation
    // starts on mount and resolves in well under a millisecond.
    const current = identityRef.current
    if (!current) {
      throw new Error('Synced share identity not ready yet — try again')
    }
    localStorage.setItem(activeFlagKey(fileId), 'true')
    revisionRef.current = 0
    setIsActive(true)
    return buildSyncedShareUrl(current.shareId, filename)
  }, [fileId, filename, isGithubConnected])

  const stopSync = useCallback(() => {
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    const current = session
    localStorage.setItem(activeFlagKey(fileId), 'false')
    setIsActive(false)
    if (!current || !host) return
    const stop: SyncedStopRequest = {
      type: 'stop',
      ownerToken: current.ownerToken,
      ...(githubAuth ? { identityToken: githubAuth.token } : {}),
    }
    void fetch(syncedShareEndpointUrl(host, current.shareId), {
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

  const signInWithGithub = useCallback((): Promise<SyncedShareGithubAuthResult> => {
    return openSyncedShareGithubSignInPopup({
      clientId: SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID,
    })
  }, [])

  return {
    isSynced: session !== null,
    syncedShareLink: session
      ? buildSyncedShareUrl(session.shareId, filename)
      : null,
    isGithubConnected,
    githubLogin,
    startSync,
    stopSync,
    broadcastContent,
    signInWithGithub,
    syncFailure,
    dismissSyncFailure,
  }
}
