import { useMemo } from 'react'
import { useLocalStorage } from 'usehooks-ts'
import { FILE_STORE_KEY, type FileStoreState } from '../fileStore'
import {
  deserializeStoreSync,
  localBackend,
  readInitialStoreSync,
} from '../storage/localBackend'
import type { SaveStatus, StorageBackend } from '../storage/types'

export interface UseStorageBackendResult {
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  backend: StorageBackend
  saveStatus: SaveStatus
}

/**
 * Holds the file store's in-memory state and wires it to a `StorageBackend`.
 * In this step the backend is always `localBackend` — there is no
 * `switchBackend()` yet, since there is nothing else to switch to.
 *
 * `store`/`setStore` keep the same ergonomics `useFileStore` used to expose,
 * so callers still perform sync updates (e.g. selecting a file, or applying
 * `backend.updateActiveContent`) via `setStore`. Structural operations
 * (create/duplicate/rename/delete/restore) are modeled as async on
 * `StorageBackend`, so callers `await backend.xxxFile(store)` and then
 * `setStore` the result.
 */
export function useStorageBackend(): UseStorageBackendResult {
  const [store, setStore] = useLocalStorage<FileStoreState>(
    FILE_STORE_KEY,
    readInitialStoreSync,
    { deserializer: deserializeStoreSync },
  )

  const saveStatus = useMemo(() => localBackend.status(), [])

  return { store, setStore, backend: localBackend, saveStatus }
}
