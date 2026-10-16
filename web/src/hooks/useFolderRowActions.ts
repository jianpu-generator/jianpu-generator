import type { FolderDialogState } from '../components/FolderDialogs'
import type { FolderActions } from '../components/FolderRowMenu'
import type { FolderShareTarget } from '../components/FolderShareModal'
import type { FileStoreState } from '../fileStore'
import { foldersToReveal } from '../fileStoreQueries'
import type { FolderShares } from './useFolderShares'

interface FolderRowActionsInput {
  store: FileStoreState
  folderShares: FolderShares
  onCreate: (folder?: string) => void
  onCreateFolder: (parent?: string) => Promise<FileStoreState | undefined>
  onImportFile?: (file: File, folder?: string) => void
  expandFolders: (folders: string[]) => void
  /** Closes the file list drawer/menu. */
  closeList: () => void
  openFolderDialog: (dialog: FolderDialogState) => void
  openFolderShare: (target: FolderShareTarget) => void
}

/** What each item of a folder row's `···` menu does. Pure wiring: the new
 * folder's name, the share state and the link all come from Rust. */
export function useFolderRowActions({
  store,
  folderShares,
  onCreate,
  onCreateFolder,
  onImportFile,
  expandFolders,
  closeList,
  openFolderDialog,
  openFolderShare,
}: FolderRowActionsInput): FolderActions {
  const dialog = (kind: FolderDialogState['kind']) => (folder: string) => {
    closeList()
    openFolderDialog({ kind, folder })
  }
  const share = (tab: FolderShareTarget['tab']) => (folder: string) => {
    closeList()
    openFolderShare({ folder, tab })
  }
  return {
    onNewFileHere: async (folder) => {
      await onCreate(folder)
      closeList()
    },
    onNewFolderHere: async (folder) => {
      const next = await onCreateFolder(folder)
      const known = new Set(store.folders)
      // Open the folder and the one Rust just named inside it.
      expandFolders(
        (next?.folders ?? [])
          .filter((created) => !known.has(created))
          .flatMap(foldersToReveal),
      )
    },
    onImportHere: (folder, file) => {
      onImportFile?.(file, folder)
      closeList()
    },
    onRename: dialog('rename'),
    onMove: dialog('move'),
    onDelete: dialog('delete'),
    onShare: share('static'),
    onShareSettings: share('synced'),
    onCopyShareLink: (shareId, name) => {
      closeList()
      void navigator.clipboard
        .writeText(folderShares.linkFor(shareId, name))
        .catch(() => undefined)
    },
  }
}
