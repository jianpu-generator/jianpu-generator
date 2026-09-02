import type { FileStoreState } from '../fileStore'
import { sortedBinNames } from '../fileStore'
import type { FileOpError } from '../hooks/useFileOperations'
import type { PendingDownload } from '../hooks/useJianpuWorkerTypes'
import type {
  StorageBackendPreference,
  StorageBackendTarget,
} from '../hooks/useStorageBackend'
import type { StorageBackend } from '../storage/types'
import { BinModal } from './BinModal'
import { DownloadRenameModal } from './DownloadRenameModal'
import { ErrorModal } from './ErrorModal'
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
  refreshSaveStatus: (syncedStore?: FileStoreState) => void
  selectedMeasureRange: { start: number; end: number } | null
  binOpen: boolean
  setBinOpen: (open: boolean) => void
  onRestore: (name: string) => void
  restoringFileName?: string | null
  pendingDownload: PendingDownload | null
  onConfirmDownload: (filename: string) => void
  onCancelDownload: () => void
}

/** Modals and the hidden test-probe span that sit above the main
 * editor/preview workspace. Grouped here purely to keep App.tsx's JSX under
 * the project's per-file line limit. */
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
  refreshSaveStatus,
  selectedMeasureRange,
  binOpen,
  setBinOpen,
  onRestore,
  restoringFileName,
  pendingDownload,
  onConfirmDownload,
  onCancelDownload,
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
        refreshSaveStatus={refreshSaveStatus}
      />
      <BinModal
        open={binOpen}
        onOpenChange={setBinOpen}
        binNames={sortedBinNames(store)}
        onRestore={onRestore}
        restoringName={restoringFileName}
      />
      <DownloadRenameModal
        pending={pendingDownload}
        onConfirm={onConfirmDownload}
        onCancel={onCancelDownload}
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
    </>
  )
}
