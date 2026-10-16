import {
  ArchiveIcon,
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
import { fileContent, isReadOnlyFile, sortedUserFileNames } from '../fileStore'
import {
  fileMoveTargets,
  moveDestination,
  renameError,
} from '../fileStoreQueries'
import { useExpandedFolders } from '../hooks/useExpandedFolders'
import { useFileTreeView } from '../hooks/useFileTreeView'
import { useFolderRowActions } from '../hooks/useFolderRowActions'
import { useNewFolderPrompt } from '../hooks/useNewFolderPrompt'
import { useFileStaticShareLink } from '../hooks/useStaticShareLink'
import { FileBrowserModal } from './FileBrowserModal'
import type { FileSwitcherProps } from './FileSwitcherProps'
import { FileSwitcherTrigger } from './FileSwitcherTrigger'
import { type FolderDialogState, FolderDialogs } from './FolderDialogs'
import { FolderShareModal, type FolderShareTarget } from './FolderShareModal'
import { ImportButton } from './ImportButton'
import { MoveToDialog } from './MoveToDialog'
import { NewFolderDialog } from './NewFolderDialog'
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
  folderShares,
}: FileSwitcherProps) {
  const names = sortedUserFileNames(store)
  const showEmptyHint = !isLoadingCloud && names.length === 0
  const [filesOpen, setFilesOpen] = useState(false)
  const [query, setQuery] = useState('')
  const {
    expanded,
    toggle: toggleFolder,
    expand: expandFolders,
  } = useExpandedFolders()
  const { rows, activeLabel } = useFileTreeView(
    store,
    expanded,
    triggerLabel,
    folderShares.overview,
    query,
  )
  const listRenameError = (from: string, to: string) =>
    renameError(store, from, to)
  const [actionsOpen, setActionsOpen] = useState(false)
  const [shareModalOpen, setShareModalOpen] = useState(false)
  const [renameOpen, setRenameOpen] = useState(false)
  const [moveOpen, setMoveOpen] = useState(false)
  const [folderDialog, setFolderDialog] = useState<FolderDialogState | null>(
    null,
  )
  const newFolderPrompt = useNewFolderPrompt({
    store,
    onCreateFolder,
    expandFolders,
  })
  const [shareFolder, setShareFolder] = useState<FolderShareTarget | null>(null)
  const fileStaticLink = useFileStaticShareLink(
    shareModalOpen,
    store.active,
    fileContent(store, store.active),
  )
  const folderActions = useFolderRowActions({
    folderShares,
    onCreate,
    onImportFile,
    closeList: () => setFilesOpen(false),
    openFolderDialog: setFolderDialog,
    openFolderShare: setShareFolder,
    openNewFolder: newFolderPrompt.open,
  })

  return (
    <div className="file-tab-bar">
      <SaveStatusBadge
        status={saveStatus}
        label={saveLabel}
        onOpen={onOpenSyncPanel}
        autosaveDeadline={autosaveDeadline}
      />
      <div className="export-menu">
        <FileBrowserModal
          open={filesOpen}
          onOpenChange={(open) => {
            setFilesOpen(open)
            if (open) setQuery('')
          }}
          trigger={
            <FileSwitcherTrigger
              activeLabel={activeLabel}
              isLoadingCloud={isLoadingCloud}
            />
          }
          activePath={store.active}
          query={query}
          onQueryChange={setQuery}
          rows={rows}
          isLoadingCloud={isLoadingCloud}
          storeIsEmpty={showEmptyHint}
          creating={creating}
          creatingFolder={creatingFolder}
          importing={importing}
          onNewFile={async () => {
            await onCreate(activeLabel.folderPath)
            setFilesOpen(false)
          }}
          onNewFolder={() => {
            setFilesOpen(false)
            newFolderPrompt.open(activeLabel.folderPath)
          }}
          onImportFile={
            onImportFile
              ? (file) => {
                  onImportFile(file, activeLabel.folderPath)
                  setFilesOpen(false)
                }
              : undefined
          }
          treeProps={{
            renamingName,
            onToggleFolder: toggleFolder,
            onSelect: (selected) => {
              onSelect(selected)
              setFilesOpen(false)
            },
            onRename,
            renameError: listRenameError,
            creating,
            creatingFolder,
            importing,
            folderActions,
          }}
        />
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
              onClick={() => {
                setActionsOpen(false)
                newFolderPrompt.open(activeLabel.folderPath)
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
      <NewFolderDialog {...newFolderPrompt.dialogProps} />
      <FolderDialogs
        store={store}
        dialog={folderDialog}
        shares={folderShares.overview}
        onClose={() => setFolderDialog(null)}
        onMoveFolder={onMoveFolder}
        onDeleteFolder={onDeleteFolder}
        expandFolders={expandFolders}
      />
      <ShareModal
        open={shareModalOpen}
        onOpenChange={setShareModalOpen}
        kind="file"
        name={store.active}
        staticLink={fileStaticLink}
        canSync={canSync}
        isSynced={isSynced}
        syncedShareLink={syncedShareLink}
        isGithubConnected={isGithubConnected}
        githubLogin={githubLogin}
        onStartSync={onStartSync}
        onStopSync={onStopSync}
        onSignInWithGithub={onSignInWithGithub}
      />
      <FolderShareModal
        store={store}
        target={shareFolder}
        onClose={() => setShareFolder(null)}
        folderShares={folderShares}
        isGithubConnected={isGithubConnected}
        githubLogin={githubLogin}
        onSignInWithGithub={onSignInWithGithub}
      />
    </div>
  )
}
