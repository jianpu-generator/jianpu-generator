import { useCallback, useEffect, useState } from 'react'
import { type FileStoreState, mergeBackendResult } from '../fileStore'
import {
  clearShareHash,
  parseShareFromHash,
  type SharePayload,
} from '../shareUrl'
import type { StorageBackend } from '../storage/types'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Parses a `#share=` URL hash (if present) on mount, exposing the decoded
 * payload plus handlers to import it into the active file store or discard
 * it. Also collapses the editor pane when a shared preview is present.
 */
export function useSharedPreview(
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
  setEditorCollapsed: (collapsed: boolean) => void,
) {
  const [sharedPreview, setSharedPreview] = useState<SharePayload | null>(null)

  useEffect(() => {
    let cancelled = false
    void parseShareFromHash().then((parsed) => {
      if (!cancelled) {
        setSharedPreview(parsed)
        if (parsed) setEditorCollapsed(true)
      }
    })
    return () => {
      cancelled = true
    }
  }, [setEditorCollapsed])

  const handleDismissShared = useCallback(() => {
    clearShareHash()
    setSharedPreview(null)
  }, [])

  const handleImportShared = useCallback(async () => {
    if (!sharedPreview) return
    const base = store
    try {
      const next = await backend.importFile(
        base,
        sharedPreview.filename,
        sharedPreview.content,
      )
      setStore((prev) => mergeBackendResult(prev, base, next))
      clearShareHash()
      setSharedPreview(null)
    } catch (error) {
      setFileOpError({
        title: 'Could not import shared score',
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      })
    }
  }, [sharedPreview, store, backend, setStore, setFileOpError])

  return {
    sharedPreview,
    handleDismissShared,
    handleImportShared,
  }
}
