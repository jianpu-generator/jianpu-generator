import type { FileStoreState } from '../fileStore'
import type { FileOpError } from '../hooks/useFileOperations'
import type {
  StorageBackendPreference,
  StorageBackendTarget,
} from '../hooks/useStorageBackend'
import type { SharePayload } from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import { ErrorModal } from './ErrorModal'
import { SharedPreviewBanner } from './SharedPreviewBanner'
import { StorageSettingsModal } from './StorageSettingsModal'

interface AppOverlaysProps {
  fileOpError: FileOpError | null
  setFileOpError: (error: FileOpError | null) => void
  storageSettingsOpen: boolean
  setStorageSettingsOpen: (open: boolean) => void
  backend: StorageBackend
  isLoadingGithub: boolean
  preference: StorageBackendPreference
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  selectedMeasureRange: { start: number; end: number } | null
  sharedPreview: SharePayload | null
  handleImportShared: () => void
  handleDismissShared: () => void
}

/** Modals, the shared-preview banner, and the hidden test-probe span that sit
 * above the main editor/preview workspace. Grouped here purely to keep
 * App.tsx's JSX under the project's per-file line limit. */
export function AppOverlays({
  fileOpError,
  setFileOpError,
  storageSettingsOpen,
  setStorageSettingsOpen,
  backend,
  isLoadingGithub,
  preference,
  switchBackend,
  store,
  setStore,
  selectedMeasureRange,
  sharedPreview,
  handleImportShared,
  handleDismissShared,
}: AppOverlaysProps) {
  return (
    <>
      <ErrorModal
        open={fileOpError !== null}
        onOpenChange={(open) => {
          if (!open) setFileOpError(null)
        }}
        title={fileOpError?.title ?? ''}
        message={fileOpError?.message ?? ''}
        stack={fileOpError?.stack}
      />
      <StorageSettingsModal
        open={storageSettingsOpen}
        onOpenChange={setStorageSettingsOpen}
        backend={backend}
        isLoadingGithub={isLoadingGithub}
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
      />
      <span
        data-testid="selected-measure-range"
        aria-hidden="true"
        style={{ display: 'none' }}
      >
        {selectedMeasureRange
          ? `${selectedMeasureRange.start}-${selectedMeasureRange.end}`
          : ''}
      </span>
      {sharedPreview ? (
        <SharedPreviewBanner
          filename={sharedPreview.filename}
          onImport={handleImportShared}
          onDiscard={handleDismissShared}
        />
      ) : null}
    </>
  )
}
