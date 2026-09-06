import { useCallback, useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import type { LiveStopRequest, LiveUpdateRequest } from '../live/protocol'
import {
  buildLiveShareUrl,
  deriveLiveIdentity,
  getOrCreateDeviceSecret,
  type LiveIdentity,
} from '../liveShareUrl'
import { AUTOSAVE_DEBOUNCE_MS } from './useStorageBackend'

function activeFlagKey(fileId: string): string {
  return `jianpu:live-active:v1:${fileId}`
}

function readActiveFlag(fileId: string): boolean {
  return localStorage.getItem(activeFlagKey(fileId)) === 'true'
}

function roomUrl(host: string, roomId: string): string {
  return `https://${host}/rooms/${roomId}`
}

export interface UseLiveOwnerResult {
  isLive: boolean
  liveUrl: string | null
  /** Marks the file as live and returns its viewer link — returned
   * synchronously so the caller can copy it to the clipboard in the same
   * click handler that starts the session. The link is deterministic (see
   * `deriveLiveIdentity`), so this reproduces the same link every time
   * rather than minting a new one. */
  startLive: () => string
  stopLive: () => void
  broadcastContent: (content: string) => void
}

/**
 * Owns the owner side of a Live Share session for a single file. `fileId`
 * (stable across renames, unlike `filename`) keys both the room identity
 * derivation and the persisted "is this file live" flag, so a session
 * survives a rename and reproduces the same link across stop/start cycles.
 *
 * There is no persistent connection: `broadcastContent` just `PUT`s the
 * current content to the room's KV entry, debounced at the same
 * `AUTOSAVE_DEBOUNCE_MS` cadence as a regular save (not on every keystroke)
 * — a viewer only sees a push once they reload, so there is no benefit to
 * pushing more often than the content is actually persisted.
 */
export function useLiveOwner(
  filename: string,
  fileId: string,
  content: string,
): UseLiveOwnerResult {
  const [isActive, setIsActive] = useState(() => readActiveFlag(fileId))
  const [identity, setIdentity] = useState<LiveIdentity | null>(null)
  // Mirrors `identity` for the click handler below, which needs to read it
  // synchronously (state updates aren't visible until the next render).
  const identityRef = useRef<LiveIdentity | null>(null)
  const revisionRef = useRef(0)
  // Mirrors the latest `content` prop so the "just went live" effect below
  // (which can't see React props at the time it fires) can push the room's
  // very first doc without waiting for the owner to make an edit.
  const contentRef = useRef(content)
  contentRef.current = content

  useEffect(() => {
    setIsActive(readActiveFlag(fileId))
    setIdentity(null)
    identityRef.current = null
    let cancelled = false
    void deriveLiveIdentity(getOrCreateDeviceSecret(), fileId).then((next) => {
      if (cancelled) return
      identityRef.current = next
      setIdentity(next)
    })
    return () => {
      cancelled = true
    }
  }, [fileId])

  const session = isActive ? identity : null

  const pushUpdate = useCallback(
    (content: string) => {
      const host = import.meta.env.VITE_PARTYKIT_HOST
      if (!session || !host) return
      revisionRef.current += 1
      const request: LiveUpdateRequest = {
        type: 'update',
        ownerToken: session.ownerToken,
        filename,
        content,
        revision: revisionRef.current,
      }
      void fetch(roomUrl(host, session.roomId), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      })
    },
    [filename, session],
  )

  // Pushes the initial doc the moment a session becomes live (including a
  // page reload while already live), so a viewer opening the link right
  // away doesn't find an empty room from before the owner's first edit.
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

  const startLive = useCallback((): string => {
    // In practice always populated by the time a user can click: derivation
    // starts on mount and resolves in well under a millisecond.
    const current = identityRef.current
    if (!current) {
      throw new Error('Live identity not ready yet — try again')
    }
    localStorage.setItem(activeFlagKey(fileId), 'true')
    revisionRef.current = 0
    setIsActive(true)
    return buildLiveShareUrl(current.roomId, filename)
  }, [fileId, filename])

  const stopLive = useCallback(() => {
    const host = import.meta.env.VITE_PARTYKIT_HOST
    const current = session
    localStorage.setItem(activeFlagKey(fileId), 'false')
    setIsActive(false)
    if (!current || !host) return
    const stop: LiveStopRequest = {
      type: 'stop',
      ownerToken: current.ownerToken,
    }
    void fetch(roomUrl(host, current.roomId), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(stop),
    })
  }, [fileId, session])

  return {
    isLive: session !== null,
    liveUrl: session ? buildLiveShareUrl(session.roomId, filename) : null,
    startLive,
    stopLive,
    broadcastContent,
  }
}
