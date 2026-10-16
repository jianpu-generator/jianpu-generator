import type { FolderDialogState } from '../components/FolderDialogs'
import type { FolderActions } from '../components/FolderRowMenu'
import type { FolderShareTarget } from '../components/FolderShareModal'
import type { FolderShares } from './useFolderShares'

interface FolderRowActionsInput {
  folderShares: FolderShares
  onCreate: (folder?: string) => void
  onImportFile?: (file: File, folder?: string) => void
  /** Closes the file list drawer/menu. */
  closeList: () => void
  openFolderDialog: (dialog: FolderDialogState) => void
  openFolderShare: (target: FolderShareTarget) => void
  /** Opens the name prompt for a new folder inside `parent`. */
  openNewFolder: (parent: string) => void
}

/** What each item of a folder row's `···` menu does. Pure wiring: the new
 * folder's name, the share state and the link all come from Rust. */
export function useFolderRowActions({
  folderShares,
  onCreate,
  onImportFile,
  closeList,
  openFolderDialog,
  openFolderShare,
  openNewFolder,
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
    onNewFolderHere: (folder) => {
      closeList()
      openNewFolder(folder)
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
