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

/** The file a synced share would point at (see `useSyncedShareOwner`'s
 * `cloudFileId` param): the active file's own D1 `files.id` when it's
 * stored in the cloud backend, `null` otherwise. A local file has no
 * server-side row for a viewer to read, and a built-in demo file isn't the
 * user's to share live, so both only get the static `#share=` link. */
function cloudFileIdFor(
  backend: StorageBackend,
  state: FileStoreState,
  activeFilename: string,
): string | null {
  if (backend.kind !== 'cloud' || isReadOnlyFile(activeFilename)) return null
  return fileIdForName(state, activeFilename)
}

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
    cloudFileIdFor(backend, store, store.active),
  )
  const {
    syncedShareViewerPreview,
    syncedShareViewerStatus,
    syncedShareViewerOwnerLogin,
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
      viewerOwnerLogin: syncedShareViewerOwnerLogin,
      onImportSyncedShare: handleImportSyncedShare,
      canSync: syncedShareOwner.canSync,
      isSynced: syncedShareOwner.isSynced,
      syncedShareLink: syncedShareOwner.syncedShareLink,
      isGithubConnected: syncedShareOwner.isGithubConnected,
      githubLogin: syncedShareOwner.githubLogin,
      onStartSync: syncedShareOwner.startSync,
      onStopSync: syncedShareOwner.stopSync,
      onSignInWithGithub: syncedShareOwner.signInWithGithub,
      onDisconnectGithub: syncedShareOwner.disconnectGithub,
      syncFailure: syncedShareOwner.syncFailure,
      onDismissSyncFailure: syncedShareOwner.dismissSyncFailure,
    },
  }
}
