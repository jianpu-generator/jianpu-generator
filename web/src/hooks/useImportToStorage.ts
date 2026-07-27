import { useCallback } from 'react'
import { type FileStoreState, mergeBackendResult } from '../fileStore'
import type { StorageBackend } from '../storage/types'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Imports a filename/content pair into the active file store via the
 * backend, surfacing failures through `setFileOpError`. Shared by
 * `useSharedPreview` (`#share=` links) and the live-view import action
 * (`#live=` links), which both hand it the same `{filename, content}` shape.
 */
export function useImportToStorage(
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
) {
  return useCallback(
    async (filename: string, content: string) => {
      const base = store
      try {
        const next = await backend.importFile(base, filename, content)
        setStore((prev) => mergeBackendResult(prev, base, next))
      } catch (error) {
        setFileOpError({
          title: 'Could not import shared score',
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
        throw error
      }
    },
    [store, backend, setStore, setFileOpError],
  )
}
