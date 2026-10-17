import type { FolderDialogState } from '../components/FolderDialogs'
import type { FolderActions } from '../components/FolderRowMenu'
import type { FolderShareTarget } from '../components/FolderShareModal'
import type { FolderShares } from './useFolderShares'

interface FolderRowActionsInput {
  folderShares: FolderShares
  onImportFile?: (file: File, folder?: string) => void
  /** Closes the file browser, for actions that finish rather than prompt. */
  closeList: () => void
  openFolderDialog: (dialog: FolderDialogState) => void
  openFolderShare: (target: FolderShareTarget) => void
  /** Opens the name prompt for a new file inside `folder`. */
  openNewFile: (folder: string) => void
  /** Opens the name prompt for a new folder inside `parent`. */
  openNewFolder: (parent: string) => void
}

/** What each item of a folder row's `···` menu does. Pure wiring: the new
 * entry's name, the share state and the link all come from Rust. */
export function useFolderRowActions({
  folderShares,
  onImportFile,
  closeList,
  openFolderDialog,
  openFolderShare,
  openNewFile,
  openNewFolder,
}: FolderRowActionsInput): FolderActions {
  // Prompts keep the file browser open underneath, so the user lands back in
  // the tree afterwards.
  const dialog = (kind: FolderDialogState['kind']) => (folder: string) =>
    openFolderDialog({ kind, folder })
  const share = (tab: FolderShareTarget['tab']) => (folder: string) =>
    openFolderShare({ folder, tab })
  return {
    onNewFileHere: openNewFile,
    onNewFolderHere: openNewFolder,
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
