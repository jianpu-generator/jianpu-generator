import { useCallback, useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
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
  /** Marks the file as synced and returns its viewer link — returned
   * synchronously so the caller can copy it to the clipboard in the same
   * click handler that starts the session. The link is deterministic (see
   * `deriveSyncedShareIdentity`), so this reproduces the same link every time
   * rather than minting a new one. */
  startSync: () => string
  stopSync: () => void
  broadcastContent: (content: string) => void
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
      }
      void fetch(syncedShareEndpointUrl(host, session.shareId), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      })
    },
    [filename, session],
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

  const startSync = useCallback((): string => {
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
  }, [fileId, filename])

  const stopSync = useCallback(() => {
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    const current = session
    localStorage.setItem(activeFlagKey(fileId), 'false')
    setIsActive(false)
    if (!current || !host) return
    const stop: SyncedStopRequest = {
      type: 'stop',
      ownerToken: current.ownerToken,
    }
    void fetch(syncedShareEndpointUrl(host, current.shareId), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(stop),
    })
  }, [fileId, session])

  return {
    isSynced: session !== null,
    syncedShareLink: session
      ? buildSyncedShareUrl(session.shareId, filename)
      : null,
    startSync,
    stopSync,
    broadcastContent,
  }
}
