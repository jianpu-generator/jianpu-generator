import { useCallback, useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { SharePayload } from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import type { SyncedDoc } from '../syncedShare/protocol'
import {
  clearSyncedShareHash,
  parseSyncedShareFromHash,
} from '../syncedShareUrl'
import { useImportToStorage } from './useImportToStorage'

export type SyncedShareViewerStatus =
  | 'loading'
  | 'synced'
  | 'unreachable'
  | 'ended'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Parses a `#synced=` URL hash (if present) on mount and fetches the
 * KV-backed share's current doc once. Produces the same `{filename,
 * content}` shape `useSharedPreview` does, so it plugs into the identical
 * `source`/`readOnly` derivation in `App.tsx`. Also collapses the editor
 * pane, mirroring `useSharedPreview`.
 *
 * There is no push from the server: an owner's later edits only reach a
 * viewer that reloads the page (see
 * `useSyncedShareOwner.broadcastContent`), by design — the sharer is
 * expected to tell viewers to refresh when they want an update seen.
 */
export function useSyncedShareViewer(
  setEditorCollapsed: (collapsed: boolean) => void,
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
) {
  const [syncedShareViewerPreview, setSyncedShareViewerPreview] =
    useState<SharePayload | null>(null)
  const [syncedShareViewerStatus, setSyncedShareViewerStatus] =
    useState<SyncedShareViewerStatus>('loading')
  const importToStorage = useImportToStorage(
    store,
    backend,
    setStore,
    setFileOpError,
  )
  const handleImportSyncedShare = useCallback(async () => {
    if (!syncedShareViewerPreview) return
    try {
      await importToStorage(
        syncedShareViewerPreview.filename,
        syncedShareViewerPreview.content,
      )
      clearSyncedShareHash()
      setSyncedShareViewerPreview(null)
    } catch {
      // handled by importToStorage via setFileOpError
    }
  }, [syncedShareViewerPreview, importToStorage])

  useEffect(() => {
    const parsed = parseSyncedShareFromHash()
    if (!parsed) return
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    if (!host) return

    setEditorCollapsed(true)

    let cancelled = false
    void fetch(`https://${host}/shares/${parsed.shareId}`)
      .then((response) => {
        if (!response.ok)
          throw new Error(`Unexpected status ${response.status}`)
        return response.json() as Promise<SyncedDoc>
      })
      .then((doc) => {
        if (cancelled) return
        if (doc.ended) {
          setSyncedShareViewerPreview(null)
          setSyncedShareViewerStatus('ended')
          return
        }
        setSyncedShareViewerPreview({
          filename: doc.filename,
          content: doc.content,
        })
        setSyncedShareViewerStatus('synced')
      })
      .catch(() => {
        if (!cancelled) setSyncedShareViewerStatus('unreachable')
      })

    return () => {
      cancelled = true
    }
  }, [setEditorCollapsed])

  return {
    syncedShareViewerPreview,
    syncedShareViewerStatus,
    handleImportSyncedShare,
  }
}
