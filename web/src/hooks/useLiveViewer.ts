import { useCallback, useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { LiveDoc } from '../live/protocol'
import { clearLiveShareHash, parseLiveShareFromHash } from '../liveShareUrl'
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
 * Parses a `#live=` URL hash (if present) on mount and fetches the KV-backed
 * room's current doc once. Produces the same `{filename, content}` shape
 * `useSharedPreview` does, so it plugs into the identical `source`/
 * `readOnly` derivation in `App.tsx`. Also collapses the editor pane,
 * mirroring `useSharedPreview`.
 *
 * There is no live push: an owner's later edits only reach a viewer that
 * reloads the page (see `useLiveOwner.broadcastContent`), by design — the
 * sharer is expected to tell viewers to refresh when they want an update
 * seen.
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
      clearLiveShareHash()
      setLiveViewerPreview(null)
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

    let cancelled = false
    void fetch(`https://${host}/rooms/${parsed.roomId}`)
      .then((response) => {
        if (!response.ok)
          throw new Error(`Unexpected status ${response.status}`)
        return response.json() as Promise<LiveDoc>
      })
      .then((doc) => {
        if (cancelled) return
        if (doc.ended) {
          setLiveViewerPreview(null)
          setLiveViewerStatus('ended')
          return
        }
        setLiveViewerPreview({ filename: doc.filename, content: doc.content })
        setLiveViewerStatus('live')
      })
      .catch(() => {
        if (!cancelled) setLiveViewerStatus('disconnected')
      })

    return () => {
      cancelled = true
    }
  }, [setEditorCollapsed])

  return { liveViewerPreview, liveViewerStatus, handleImportLive }
}
