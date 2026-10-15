import {
  ArchiveIcon,
  ChevronDownIcon,
  CopyIcon,
  DotsHorizontalIcon,
  FilePlusIcon,
  GearIcon,
  Link2Icon,
  MoveIcon,
  Pencil1Icon,
  PlusIcon,
  TrashIcon,
} from '@radix-ui/react-icons'
import { useState } from 'react'
import {
  displayFileName,
  fileContent,
  isReadOnlyFile,
  sortedUserFileNames,
} from '../fileStore'
import {
  fileMoveTargets,
  foldersToReveal,
  moveDestination,
  renameError,
} from '../fileStoreQueries'
import { useExpandedFolders } from '../hooks/useExpandedFolders'
import { useFileTreeView } from '../hooks/useFileTreeView'
import type { FileSwitcherProps } from './FileSwitcherProps'
import { FileTreeList } from './FileTreeList'
import { type FolderDialogState, FolderDialogs } from './FolderDialogs'
import { ImportButton } from './ImportButton'
import { MoveToDialog } from './MoveToDialog'
import { RenameFileDialog } from './RenameFileDialog'
import { ResponsiveMenu } from './ResponsiveMenu'
import { SaveStatusBadge } from './SaveStatusBadge'
import { ShareModal } from './ShareModal'
import { SpinnerLabel } from './SpinnerLabel'

export function FileSwitcher({
  store,
  triggerLabel,
  onSelect,
  onCreate,
  onCreateFolder,
  onDuplicate,
  onRename,
  onDelete,
  onMoveFolder,
  onDeleteFolder,
  onOpenStorageSettings,
  saveStatus,
  saveLabel,
  autosaveDeadline,
  creating = false,
  creatingFolder = false,
  deletingName = null,
  duplicating = false,
  renamingName = null,
  isLoadingCloud = false,
  importing = false,
  onImportFile,
  binNames,
  onOpenBin,
  onOpenSyncPanel,
  canSync,
  isSynced,
  syncedShareLink,
  isGithubConnected,
  githubLogin,
  onStartSync,
  onStopSync,
  onSignInWithGithub,
}: FileSwitcherProps) {
  const names = sortedUserFileNames(store)
  const showEmptyHint = !isLoadingCloud && names.length === 0

  const [filesOpen, setFilesOpen] = useState(false)
  const {
    expanded,
    toggle: toggleFolder,
    expand: expandFolders,
  } = useExpandedFolders()
  const { rows, activeLabel } = useFileTreeView(store, expanded, triggerLabel)

  const listRenameError = (from: string, to: string) =>
    renameError(store, from, to)
  const [actionsOpen, setActionsOpen] = useState(false)
  const [shareModalOpen, setShareModalOpen] = useState(false)
  const [renameOpen, setRenameOpen] = useState(false)
  const [moveOpen, setMoveOpen] = useState(false)
  const [folderDialog, setFolderDialog] = useState<FolderDialogState | null>(
    null,
  )
  const folderAction =
    (kind: 'rename' | 'move' | 'delete') => (folder: string) => {
      setFilesOpen(false)
      setFolderDialog({ kind, folder })
    }

  return (
    <div className="file-tab-bar">
      <SaveStatusBadge
        status={saveStatus}
        label={saveLabel}
        onOpen={onOpenSyncPanel}
        autosaveDeadline={autosaveDeadline}
      />
      <div className="export-menu">
        <ResponsiveMenu.Root open={filesOpen} onOpenChange={setFilesOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button type="button" className="preview-export-btn">
              {activeLabel.folderNames.length > 0 ? (
                <>
                  <span className="file-switcher-folder-prefix">
                    {`${activeLabel.folderNames.join(' / ')} /`}
                  </span>{' '}
                </>
              ) : null}
              {displayFileName(activeLabel.fileName)}
              {isLoadingCloud ? (
                <span className="file-tab-bar-spinner" aria-hidden="true" />
              ) : (
                <ChevronDownIcon
                  className="export-menu-caret"
                  aria-hidden="true"
                />
              )}
            </button>
          </ResponsiveMenu.Trigger>
          <ResponsiveMenu.Content
            className="export-menu-list file-tab-bar-files-list"
            align="end"
            sideOffset={4}
            title="Files"
          >
            {isLoadingCloud ? (
              <p className="file-tab-bar-hint">Loading files from the cloud…</p>
            ) : showEmptyHint ? (
              <p className="file-tab-bar-hint">
                No files yet — click New to create one.
              </p>
            ) : null}
            <FileTreeList
              rows={rows}
              renamingName={renamingName}
              onToggleFolder={toggleFolder}
              onSelect={(selected) => {
                onSelect(selected)
                setFilesOpen(false)
              }}
              onRename={onRename}
              renameError={listRenameError}
              creating={creating}
              creatingFolder={creatingFolder}
              importing={importing}
              folderActions={{
                onNewFileHere: async (folder) => {
                  await onCreate(folder)
                  setFilesOpen(false)
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
                  setFilesOpen(false)
                },
                onRename: folderAction('rename'),
                onMove: folderAction('move'),
                onDelete: folderAction('delete'),
              }}
            />
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      </div>
      <div className="export-menu">
        <ResponsiveMenu.Root open={actionsOpen} onOpenChange={setActionsOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button
              type="button"
              className="preview-export-btn"
              aria-label="File actions"
            >
              <DotsHorizontalIcon aria-hidden="true" />
            </button>
          </ResponsiveMenu.Trigger>
          <ResponsiveMenu.Content
            className="export-menu-list"
            align="end"
            sideOffset={4}
            title="File actions"
          >
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={creating}
              onClick={async () => {
                await onCreate(activeLabel.folderPath)
                setActionsOpen(false)
              }}
            >
              <PlusIcon aria-hidden="true" />
              <SpinnerLabel pending={creating} label="New" />
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={creatingFolder}
              onClick={async () => {
                const next = await onCreateFolder(activeLabel.folderPath)
                const known = new Set(store.folders)
                // Open the folder Rust just named.
                expandFolders(next?.folders?.filter((f) => !known.has(f)) ?? [])
                setActionsOpen(false)
              }}
            >
              <FilePlusIcon aria-hidden="true" />
              <SpinnerLabel pending={creatingFolder} label="New folder" />
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={duplicating}
              onClick={async () => {
                await onDuplicate()
                setActionsOpen(false)
              }}
            >
              <CopyIcon aria-hidden="true" />
              <SpinnerLabel pending={duplicating} label="Duplicate" />
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={
                isReadOnlyFile(store.active) || renamingName === store.active
              }
              onClick={() => {
                setActionsOpen(false)
                setRenameOpen(true)
              }}
            >
              <Pencil1Icon aria-hidden="true" />
              <SpinnerLabel
                pending={renamingName === store.active}
                label="Rename"
              />
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={isReadOnlyFile(store.active)}
              onClick={() => {
                setActionsOpen(false)
                setMoveOpen(true)
              }}
            >
              <MoveIcon aria-hidden="true" />
              Move to…
            </button>
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              data-testid="share-button"
              onClick={() => {
                setActionsOpen(false)
                setShareModalOpen(true)
              }}
            >
              <Link2Icon aria-hidden="true" />
              Share
            </button>
            {onImportFile ? (
              <ImportButton
                disabled={isLoadingCloud}
                importing={importing}
                onImportFile={(file) => {
                  onImportFile(file)
                  setActionsOpen(false)
                }}
              />
            ) : null}
            <button
              type="button"
              role="menuitem"
              className="export-menu-item export-menu-item--danger"
              disabled={
                isReadOnlyFile(store.active) || deletingName === store.active
              }
              onClick={async () => {
                await onDelete(store.active)
                setActionsOpen(false)
              }}
            >
              <TrashIcon aria-hidden="true" />
              <SpinnerLabel
                pending={deletingName === store.active}
                label="Delete"
              />
            </button>
            {binNames.length > 0 ? (
              <button
                type="button"
                role="menuitem"
                className="export-menu-item file-tab-bar-bin-trigger"
                aria-haspopup="dialog"
                onClick={() => {
                  setActionsOpen(false)
                  onOpenBin()
                }}
              >
                <ArchiveIcon aria-hidden="true" />
                {`Bin (${binNames.length})`}
              </button>
            ) : null}
            <button
              type="button"
              role="menuitem"
              className="export-menu-item"
              onClick={() => {
                setActionsOpen(false)
                onOpenStorageSettings()
              }}
            >
              <GearIcon aria-hidden="true" />
              Storage…
            </button>
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      </div>
      <RenameFileDialog
        open={renameOpen}
        onOpenChange={setRenameOpen}
        path={store.active}
        onRename={onRename}
        renameError={listRenameError}
      />
      <MoveToDialog
        open={moveOpen}
        onOpenChange={setMoveOpen}
        path={store.active}
        currentFolder={activeLabel.folderPath}
        targets={moveOpen ? fileMoveTargets(store) : []}
        destination={(folder) => moveDestination(store.active, folder)}
        moveError={(to) => renameError(store, store.active, to)}
        onMove={onRename}
      />
      <FolderDialogs
        store={store}
        dialog={folderDialog}
        onClose={() => setFolderDialog(null)}
        onMoveFolder={onMoveFolder}
        onDeleteFolder={onDeleteFolder}
        expandFolders={expandFolders}
      />
      <ShareModal
        open={shareModalOpen}
        onOpenChange={setShareModalOpen}
        filename={store.active}
        content={fileContent(store, store.active)}
        canSync={canSync}
        isSynced={isSynced}
        syncedShareLink={syncedShareLink}
        isGithubConnected={isGithubConnected}
        githubLogin={githubLogin}
        onStartSync={onStartSync}
        onStopSync={onStopSync}
        onSignInWithGithub={onSignInWithGithub}
      />
    </div>
  )
}
