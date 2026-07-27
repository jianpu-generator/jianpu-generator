import PartySocket from 'partysocket'
import { useCallback, useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import type { LiveStopMessage, LiveUpdateMessage } from '../live/protocol'
import {
  buildLiveShareUrl,
  deriveLiveIdentity,
  getOrCreateDeviceSecret,
  type LiveIdentity,
} from '../liveShareUrl'

/** Distinct from (and much shorter than) `useStorageBackend.ts`'s 20s
 * `AUTOSAVE_DEBOUNCE_MS` — this only throttles WebSocket chatter to viewers,
 * not a persistence write. */
export const LIVE_BROADCAST_DEBOUNCE_MS = 400

function activeFlagKey(fileId: string): string {
  return `jianpu:live-active:v1:${fileId}`
}

function readActiveFlag(fileId: string): boolean {
  return localStorage.getItem(activeFlagKey(fileId)) === 'true'
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
  const socketRef = useRef<PartySocket | null>(null)
  const revisionRef = useRef(0)
  // Mirrors the latest `content` prop so the socket-open handler (which
  // can't see React props) can send the room's very first doc without
  // waiting for the owner to make an edit.
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

  const sendUpdate = useCallback(
    (content: string) => {
      const socket = socketRef.current
      if (!socket || !session) return
      revisionRef.current += 1
      const message: LiveUpdateMessage = {
        type: 'update',
        filename,
        content,
        revision: revisionRef.current,
      }
      socket.send(JSON.stringify(message))
    },
    [filename, session],
  )

  useEffect(() => {
    const host = import.meta.env.VITE_PARTYKIT_HOST
    if (!session || !host) return
    const socket = new PartySocket({
      host,
      room: session.roomId,
      query: { ownerToken: session.ownerToken },
    })
    socketRef.current = socket
    const handleOpen = () => sendUpdate(contentRef.current)
    socket.addEventListener('open', handleOpen)
    return () => {
      socket.removeEventListener('open', handleOpen)
      socket.close()
      socketRef.current = null
    }
  }, [session, sendUpdate])

  const broadcastContent = useDebouncedCallback(
    sendUpdate,
    LIVE_BROADCAST_DEBOUNCE_MS,
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
    return buildLiveShareUrl(current.roomId)
  }, [fileId])

  const stopLive = useCallback(() => {
    // Tell the server to mark the room ended *before* the socket closes
    // (which happens as a side effect of `setIsActive(false)` below) so
    // viewers actually lose the score instead of the link staying quietly
    // viewable forever. The room itself — and its token — survive, so a
    // later "Go Live" on this file reproduces the exact same link.
    const finish = () => {
      localStorage.setItem(activeFlagKey(fileId), 'false')
      setIsActive(false)
    }
    const socket = socketRef.current
    if (!socket) {
      finish()
      return
    }
    const stop: LiveStopMessage = { type: 'stop' }
    if (socket.readyState === socket.OPEN) {
      socket.send(JSON.stringify(stop))
      finish()
      return
    }
    // Still connecting (e.g. "Go Live" and "Stop Live" clicked back to
    // back) — sending now would only buffer the message, and the socket
    // teardown that `finish()` triggers closes the connection before it
    // ever opens, discarding that buffer instead of flushing it. Send once
    // the socket actually opens instead.
    socket.addEventListener(
      'open',
      () => {
        socket.send(JSON.stringify(stop))
        finish()
      },
      { once: true },
    )
  }, [fileId])

  return {
    isLive: session !== null,
    liveUrl: session ? buildLiveShareUrl(session.roomId) : null,
    startLive,
    stopLive,
    broadcastContent,
  }
}
