import {
  DotsHorizontalIcon,
  FilePlusIcon,
  Link2Icon,
  MoveIcon,
  Pencil1Icon,
  PlusIcon,
  TrashIcon,
  UpdateIcon,
} from '@radix-ui/react-icons'
import { useState } from 'react'
import { ImportButton } from './ImportButton'
import { ResponsiveMenu } from './ResponsiveMenu'

/** What a folder row's `···` menu can do. Every callback gets the folder's
 * full path; none of them interprets it. */
export interface FolderActions {
  onNewFileHere: (folder: string) => void
  onNewFolderHere: (folder: string) => void
  onImportHere: (folder: string, file: File) => void
  onRename: (folder: string) => void
  onMove: (folder: string) => void
  onDelete: (folder: string) => void
  /** Opens the Share dialog of `folder` (its static tab). */
  onShare: (folder: string) => void
  /** Opens the Share dialog of a shared `folder` on its synced tab. */
  onShareSettings: (folder: string) => void
  /** Copies the synced link of a shared folder. */
  onCopyShareLink: (shareId: string, name: string) => void
}

export interface FolderRowMenuProps {
  /** Full path of the folder. */
  folder: string
  /** The folder's own name, for the menu's accessible name and drawer title. */
  name: string
  actions: FolderActions
  creating: boolean
  creatingFolder: boolean
  importing: boolean
  /** The folder's own live link, when it has one. */
  shareId?: string
  /** The shared folder this one lies inside, when it does. */
  sharedVia?: { folder: string; name: string }
}

/** The `···` menu of a folder row (hover on desktop; always visible on
 * touch screens, see `.folder-row-menu-trigger` in `file-switcher.css`). */
export function FolderRowMenu({
  folder,
  name,
  actions,
  creating,
  creatingFolder,
  importing,
  shareId,
  sharedVia,
}: FolderRowMenuProps) {
  const [open, setOpen] = useState(false)

  return (
    <ResponsiveMenu.Root open={open} onOpenChange={setOpen}>
      <ResponsiveMenu.Trigger asChild>
        <button
          type="button"
          className="folder-row-menu-trigger"
          aria-label={`Actions for folder ${name}`}
        >
          <DotsHorizontalIcon aria-hidden="true" />
        </button>
      </ResponsiveMenu.Trigger>
      <ResponsiveMenu.Content
        className="export-menu-list"
        align="end"
        sideOffset={4}
        title={name}
      >
        {shareId ? (
          <>
            <ResponsiveMenu.Item
              className="export-menu-item"
              onSelect={() => actions.onCopyShareLink(shareId, name)}
            >
              <Link2Icon aria-hidden="true" />
              Copy share link
            </ResponsiveMenu.Item>
            <ResponsiveMenu.Item
              className="export-menu-item"
              onSelect={() => actions.onShareSettings(folder)}
            >
              <UpdateIcon aria-hidden="true" />
              Sharing settings
            </ResponsiveMenu.Item>
            <hr className="folder-menu-separator" />
          </>
        ) : null}
        <ResponsiveMenu.Item
          className="export-menu-item"
          disabled={creating}
          onSelect={() => actions.onNewFileHere(folder)}
        >
          <PlusIcon aria-hidden="true" />
          New file here
        </ResponsiveMenu.Item>
        <ResponsiveMenu.Item
          className="export-menu-item"
          disabled={creatingFolder}
          onSelect={() => actions.onNewFolderHere(folder)}
        >
          <FilePlusIcon aria-hidden="true" />
          New folder here
        </ResponsiveMenu.Item>
        <ImportButton
          label="Import here"
          importing={importing}
          onImportFile={(file) => {
            setOpen(false)
            actions.onImportHere(folder, file)
          }}
        />
        <hr className="folder-menu-separator" />
        <ResponsiveMenu.Item
          className="export-menu-item"
          onSelect={() => actions.onRename(folder)}
        >
          <Pencil1Icon aria-hidden="true" />
          Rename
        </ResponsiveMenu.Item>
        <ResponsiveMenu.Item
          className="export-menu-item"
          onSelect={() => actions.onMove(folder)}
        >
          <MoveIcon aria-hidden="true" />
          Move to…
        </ResponsiveMenu.Item>
        {shareId ? null : sharedVia ? (
          <ResponsiveMenu.Item
            className="export-menu-item"
            onSelect={() => actions.onShareSettings(sharedVia.folder)}
          >
            <Link2Icon aria-hidden="true" />
            {`Shared via ${sharedVia.name}`}
          </ResponsiveMenu.Item>
        ) : (
          <ResponsiveMenu.Item
            className="export-menu-item"
            onSelect={() => actions.onShare(folder)}
          >
            <Link2Icon aria-hidden="true" />
            Share folder
          </ResponsiveMenu.Item>
        )}
        <hr className="folder-menu-separator" />
        <ResponsiveMenu.Item
          className="export-menu-item export-menu-item--danger"
          onSelect={() => actions.onDelete(folder)}
        >
          <TrashIcon aria-hidden="true" />
          Delete folder
        </ResponsiveMenu.Item>
      </ResponsiveMenu.Content>
    </ResponsiveMenu.Root>
  )
}
