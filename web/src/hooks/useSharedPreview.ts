import { useCallback, useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import {
  clearShareHash,
  parseShareFromHash,
  type SharePayload,
} from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import { useImportToStorage } from './useImportToStorage'

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
  const importToStorage = useImportToStorage(
    store,
    backend,
    setStore,
    setFileOpError,
  )

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
    try {
      await importToStorage(sharedPreview.filename, sharedPreview.content)
      clearShareHash()
      setSharedPreview(null)
    } catch {
      // handled by importToStorage via setFileOpError
    }
  }, [sharedPreview, importToStorage])

  return {
    sharedPreview,
    handleDismissShared,
    handleImportShared,
  }
}
