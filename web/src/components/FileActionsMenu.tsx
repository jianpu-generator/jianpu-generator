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
import { isReadOnlyFile } from '../fileStore'
import { DeleteFileDialog } from './DeleteFileDialog'
import { ImportButton } from './ImportButton'
import { ResponsiveMenu } from './ResponsiveMenu'
import { SpinnerLabel } from './SpinnerLabel'

export interface FileActionsMenuProps {
  /** Path of the file being edited: what the menu acts on. */
  activePath: string
  /** Folder holding it, where New and New folder put their entry. */
  folderPath: string | undefined
  creating: boolean
  creatingFolder: boolean
  duplicating: boolean
  importing: boolean
  isLoadingCloud: boolean
  renamingName: string | null
  deletingName: string | null
  binNames: string[]
  onImportFile?: (file: File) => void
  onNewFile: (folder: string | undefined) => void
  onNewFolder: (folder: string | undefined) => void
  onDuplicate: () => void
  onRename: () => void
  onMove: () => void
  onShare: () => void
  onDelete: (name: string) => void
  onOpenBin: () => void
  onOpenStorageSettings: () => void
}

/** The `···` menu beside the file browser's trigger: actions on the file
 * being edited, plus the bin and storage settings. */
export function FileActionsMenu({
  activePath,
  folderPath,
  creating,
  creatingFolder,
  duplicating,
  importing,
  isLoadingCloud,
  renamingName,
  deletingName,
  binNames,
  onImportFile,
  onNewFile,
  onNewFolder,
  onDuplicate,
  onRename,
  onMove,
  onShare,
  onDelete,
  onOpenBin,
  onOpenStorageSettings,
}: FileActionsMenuProps) {
  const [actionsOpen, setActionsOpen] = useState(false)
  const [confirmingDelete, setConfirmingDelete] = useState(false)
  return (
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
            onClick={() => {
              setActionsOpen(false)
              onNewFile(folderPath)
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
              onNewFolder(folderPath)
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
            disabled={isReadOnlyFile(activePath) || renamingName === activePath}
            onClick={() => {
              setActionsOpen(false)
              onRename()
            }}
          >
            <Pencil1Icon aria-hidden="true" />
            <SpinnerLabel
              pending={renamingName === activePath}
              label="Rename"
            />
          </button>
          <button
            type="button"
            role="menuitem"
            className="export-menu-item"
            disabled={isReadOnlyFile(activePath)}
            onClick={() => {
              setActionsOpen(false)
              onMove()
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
              onShare()
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
            disabled={isReadOnlyFile(activePath) || deletingName === activePath}
            onClick={() => {
              setActionsOpen(false)
              setConfirmingDelete(true)
            }}
          >
            <TrashIcon aria-hidden="true" />
            <SpinnerLabel
              pending={deletingName === activePath}
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
      <DeleteFileDialog
        open={confirmingDelete}
        onOpenChange={setConfirmingDelete}
        file={activePath}
        onConfirm={onDelete}
      />
    </div>
  )
}
