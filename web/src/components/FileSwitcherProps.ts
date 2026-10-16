import type { FileStoreState } from '../fileStore'
import type { FolderShares } from '../hooks/useFolderShares'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SyncedShareGithubAuthResult } from '../storage/accountAuthPopup'

export interface FileSwitcherProps {
  store: FileStoreState
  /** Label shown on the trigger button — the currently active file's name,
   * whether it's a user file or one of the read-only demo files (which live
   * in the "Demo" submenu nested inside this same dropdown). */
  triggerLabel: string
  onSelect: (name: string) => void
  /** Resolves once the file exists; `name` is vetted by Rust beforehand. */
  onCreate: (folder: string | undefined, name: string) => void
  /** Resolves to the resulting store, `undefined` if it failed. */
  onCreateFolder: (
    parent: string | undefined,
    name: string,
  ) => Promise<FileStoreState | undefined>
  onDuplicate: () => void
  onRename: (from: string, to: string) => void
  onDelete: (name: string) => void
  /** Renames/moves a whole folder (Rust has already vetted the path). */
  onMoveFolder: (from: string, to: string) => void
  onDeleteFolder: (folder: string) => void
  onOpenStorageSettings: () => void
  saveStatus: DisplaySaveStatus
  saveLabel: string | null
  /** `Date.now()`-comparable deadline for the pending autosave, used to
   * render a countdown while `saveStatus === 'unsaved'`. */
  autosaveDeadline: number | null
  /** Whether a `createFile` call is in flight — disables "New" and shows a
   * spinner on it. */
  creating?: boolean
  creatingFolder?: boolean
  /** Name of the file currently being deleted, if any — disables and spins
   * the Delete menu item rather than the whole tab bar. */
  deletingName?: string | null
  /** Whether a `duplicateFile` call is in flight — disables "Duplicate" and
   * shows a spinner on it. */
  duplicating?: boolean
  /** Name of the file currently being renamed, if any — disables and spins
   * just that file's tab name rather than the whole tab bar. */
  renamingName?: string | null
  /** Whether the cloud backend is still fetching its file list — shows a
   * spinner on the trigger and a loading hint instead of the demo hint. */
  isLoadingCloud?: boolean
  importing?: boolean
  /** `folder` is the folder to import into (the top level when omitted). */
  onImportFile?: (file: File, folder?: string) => void
  /** Names of files currently in the trash — used for the "Bin (N)" menu
   * item's count and to decide whether it renders at all. */
  binNames: string[]
  /** Opens the Bin modal, which lists `binNames` and handles restoring. */
  onOpenBin: () => void
  onOpenSyncPanel?: () => void
  /** See `ShareModalProps.canSync`. */
  canSync: boolean
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether the dedicated Synced Share GitHub sign-in connection is
   * present -- see `useSyncedShareOwner.ts`. */
  isGithubConnected: boolean
  /** Cached GitHub username for the "Synced as @username" identity row;
   * `null` until connected. */
  githubLogin: string | null
  onStartSync: () => Promise<string | null>
  onStopSync: () => void
  onSignInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  /** Live links of whole folders and the overview behind the share markers. */
  folderShares: FolderShares
}
