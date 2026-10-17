import { ChevronDownIcon } from '@radix-ui/react-icons'
import { useState } from 'react'
import { displayFileName } from '../fileStore'
import type { SharedFolderViewer } from '../hooks/useSharedFolderViewer'
import { sharedFolderActiveLabel } from '../sharedFolder'
import { FileTreeList } from './FileTreeList'
import { ResponsiveMenu } from './ResponsiveMenu'

/**
 * The file picker of a viewer of a shared folder: the same trigger button
 * and tree list as the owner's `FileSwitcher`, but rooted at the shared
 * folder (its rows come from Rust) and read-only, with none of the file
 * actions, Demo files or storage menus.
 */
export function SharedFolderSwitcher({
  viewer,
}: {
  viewer: SharedFolderViewer
}) {
  const [open, setOpen] = useState(false)
  const label = viewer.activePath
    ? sharedFolderActiveLabel(viewer.activePath)
    : null
  return (
    <div className="file-tab-bar" data-testid="shared-folder-switcher">
      <div className="export-menu">
        <ResponsiveMenu.Root open={open} onOpenChange={setOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button type="button" className="preview-export-btn">
              {label && label.folderNames.length > 0 ? (
                <>
                  <span className="file-switcher-folder-prefix">
                    {`${label.folderNames.join(' / ')} /`}
                  </span>{' '}
                </>
              ) : null}
              {label ? displayFileName(label.fileName) : viewer.folderName}
              <ChevronDownIcon
                className="export-menu-caret"
                aria-hidden="true"
              />
            </button>
          </ResponsiveMenu.Trigger>
          <ResponsiveMenu.Content
            className="export-menu-list file-tab-bar-files-list"
            align="end"
            sideOffset={4}
            title={viewer.folderName}
          >
            {viewer.rows.length === 0 ? (
              <p className="file-tab-bar-hint">This folder is empty.</p>
            ) : null}
            <FileTreeList
              rows={viewer.rows}
              readOnly
              onToggleFolder={viewer.onToggleFolder}
              onSelect={(path) => {
                viewer.onSelect(path)
                setOpen(false)
              }}
            />
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      </div>
    </div>
  )
}
