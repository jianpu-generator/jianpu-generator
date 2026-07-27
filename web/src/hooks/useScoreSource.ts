import {
  type FileStoreState,
  fileContent,
  fileIdForName,
  isReadOnlyFile,
} from '../fileStore'
import type { StorageBackend } from '../storage/types'
import { useLiveOwner } from './useLiveOwner'
import { useLiveViewer } from './useLiveViewer'
import { useSharedPreview } from './useSharedPreview'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Combines `useSharedPreview` (`#share=` links) and `useLiveViewer`
 * (`#live=` links) into the single `source`/`readOnly` derivation the editor
 * and preview panes consume. A static `#share=` link takes precedence over a
 * `#live=` one if both are somehow present at once — documented edge case in
 * the Live Share plan, not handled beyond this. `ended` also counts as
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

  const liveOwner = useLiveOwner(
    store.active,
    fileIdForName(store, store.active),
    fileContent(store, store.active),
  )
  const { liveViewerPreview, liveViewerStatus, handleImportLive } =
    useLiveViewer(setEditorCollapsed, store, backend, setStore, setFileOpError)

  const liveViewerActive =
    sharedPreview === null &&
    (liveViewerPreview !== null || liveViewerStatus === 'ended')

  const source = sharedPreview
    ? sharedPreview.content
    : liveViewerActive
      ? (liveViewerPreview?.content ?? '')
      : fileContent(store, store.active)
  const readOnly =
    sharedPreview !== null || liveViewerActive || isReadOnlyFile(store.active)

  return {
    sharedPreview,
    liveOwner,
    liveViewerActive,
    source,
    readOnly,
    liveShare: {
      sharedPreview,
      onImportShared: handleImportShared,
      onDismissShared: handleDismissShared,
      viewerActive: liveViewerActive,
      viewerStatus: liveViewerStatus,
      viewerFilename: liveViewerPreview?.filename ?? null,
      onImportLive: handleImportLive,
      isLive: liveOwner.isLive,
      liveUrl: liveOwner.liveUrl,
      onStartLive: liveOwner.startLive,
      onStopLive: liveOwner.stopLive,
    },
  }
}
