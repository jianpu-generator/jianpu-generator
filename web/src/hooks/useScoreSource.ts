import {
  type FileStoreState,
  fileContent,
  fileIdForName,
  isReadOnlyFile,
} from '../fileStore'
import type { StorageBackend } from '../storage/types'
import { useSharedPreview } from './useSharedPreview'
import { useSyncedShareOwner } from './useSyncedShareOwner'
import { useSyncedShareViewer } from './useSyncedShareViewer'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Combines `useSharedPreview` (`#share=` links) and `useSyncedShareViewer`
 * (`#synced=` links) into the single `source`/`readOnly` derivation the editor
 * and preview panes consume. A static `#share=` link takes precedence over a
 * `#synced=` one if both are somehow present at once — a documented edge
 * case, not handled beyond this. `ended` also counts as
 * active (banner + hidden editor stay up) even though the owner stopping
 * clears the preview content along with it.
 */
export function useScoreSource(
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
  setEditorCollapsed: (collapsed: boolean) => void,
) {
  const { sharedPreview, handleDismissShared, handleImportShared } =
    useSharedPreview(
      store,
      backend,
      setStore,
      setFileOpError,
      setEditorCollapsed,
    )

  const syncedShareOwner = useSyncedShareOwner(
    store.active,
    fileIdForName(store, store.active),
    fileContent(store, store.active),
  )
  const {
    syncedShareViewerPreview,
    syncedShareViewerStatus,
    handleImportSyncedShare,
  } = useSyncedShareViewer(
    setEditorCollapsed,
    store,
    backend,
    setStore,
    setFileOpError,
  )

  const syncedShareViewerActive =
    sharedPreview === null &&
    (syncedShareViewerPreview !== null || syncedShareViewerStatus === 'ended')

  const source = sharedPreview
    ? sharedPreview.content
    : syncedShareViewerActive
      ? (syncedShareViewerPreview?.content ?? '')
      : fileContent(store, store.active)
  const readOnly =
    sharedPreview !== null ||
    syncedShareViewerActive ||
    isReadOnlyFile(store.active)

  return {
    sharedPreview,
    syncedShareOwner,
    syncedShareViewerActive,
    source,
    readOnly,
    syncedShare: {
      sharedPreview,
      onImportShared: handleImportShared,
      onDismissShared: handleDismissShared,
      viewerActive: syncedShareViewerActive,
      viewerStatus: syncedShareViewerStatus,
      viewerFilename: syncedShareViewerPreview?.filename ?? null,
      onImportSyncedShare: handleImportSyncedShare,
      isSynced: syncedShareOwner.isSynced,
      syncedShareLink: syncedShareOwner.syncedShareLink,
      onStartSync: syncedShareOwner.startSync,
      onStopSync: syncedShareOwner.stopSync,
    },
  }
}
