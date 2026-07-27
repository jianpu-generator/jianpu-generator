import PartySocket from 'partysocket'
import { useCallback, useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { LiveServerMessage } from '../live/protocol'
import { parseLiveShareFromHash } from '../liveShareUrl'
import type { SharePayload } from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import { useImportToStorage } from './useImportToStorage'

export type LiveViewerStatus = 'connecting' | 'live' | 'disconnected' | 'ended'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Parses a `#live=` URL hash (if present) on mount and, for its lifetime,
 * keeps a read-only WebSocket connection open to the PartyKit room it names.
 * Produces the same `{filename, content}` shape `useSharedPreview` does, so
 * it plugs into the identical `source`/`readOnly` derivation in `App.tsx`.
 * Also collapses the editor pane, mirroring `useSharedPreview`.
 */
export function useLiveViewer(
  setEditorCollapsed: (collapsed: boolean) => void,
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
) {
  const [liveViewerPreview, setLiveViewerPreview] =
    useState<SharePayload | null>(null)
  const [liveViewerStatus, setLiveViewerStatus] =
    useState<LiveViewerStatus>('connecting')
  const importToStorage = useImportToStorage(
    store,
    backend,
    setStore,
    setFileOpError,
  )
  const handleImportLive = useCallback(async () => {
    if (!liveViewerPreview) return
    try {
      await importToStorage(
        liveViewerPreview.filename,
        liveViewerPreview.content,
      )
    } catch {
      // handled by importToStorage via setFileOpError
    }
  }, [liveViewerPreview, importToStorage])

  useEffect(() => {
    const parsed = parseLiveShareFromHash()
    if (!parsed) return
    const host = import.meta.env.VITE_PARTYKIT_HOST
    if (!host) return

    setEditorCollapsed(true)

    const socket = new PartySocket({ host, room: parsed.roomId })

    const handleOpen = () =>
      setLiveViewerStatus((prev) => (prev === 'ended' ? prev : 'live'))
    const handleClose = () =>
      setLiveViewerStatus((prev) => (prev === 'ended' ? prev : 'disconnected'))
    const handleMessage = (event: MessageEvent<string>) => {
      const message: LiveServerMessage = JSON.parse(event.data)
      if (message.type === 'sync') {
        if (message.ended) {
          setLiveViewerPreview(null)
          setLiveViewerStatus('ended')
          return
        }
        setLiveViewerStatus('live')
        setLiveViewerPreview({
          filename: message.filename,
          content: message.content,
        })
      } else if (message.type === 'update') {
        setLiveViewerPreview({
          filename: message.filename,
          content: message.content,
        })
      } else if (message.type === 'ended') {
        setLiveViewerPreview(null)
        setLiveViewerStatus('ended')
        socket.close()
      }
    }

    socket.addEventListener('open', handleOpen)
    socket.addEventListener('close', handleClose)
    socket.addEventListener('error', handleClose)
    socket.addEventListener('message', handleMessage)

    return () => {
      socket.removeEventListener('open', handleOpen)
      socket.removeEventListener('close', handleClose)
      socket.removeEventListener('error', handleClose)
      socket.removeEventListener('message', handleMessage)
      socket.close()
    }
  }, [setEditorCollapsed])

  return { liveViewerPreview, liveViewerStatus, handleImportLive }
}
