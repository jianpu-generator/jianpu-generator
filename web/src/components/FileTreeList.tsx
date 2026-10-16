import {
  ChevronDownIcon,
  ChevronRightIcon,
  Link2Icon,
} from '@radix-ui/react-icons'
import type { CSSProperties } from 'react'
import { displayFileName } from '../fileStore'
import type { FileTreeViewRow } from '../jianpuWasm'
import { FileTabName } from './FileTabName'
import { type FolderActions, FolderRowMenu } from './FolderRowMenu'

function FolderGlyph() {
  return (
    <svg
      className="file-tree-folder-icon"
      viewBox="0 0 15 15"
      fill="none"
      stroke="currentColor"
      strokeWidth="1"
      aria-hidden="true"
    >
      <path d="M1.5 3.5h4l1.5 1.5h6.5v6.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1z" />
    </svg>
  )
}

/** The muted "via <Folder>" tag of a row inside a shared folder. */
function ViaTag({ name }: { name: string }) {
  return (
    <span
      className="file-tree-via-tag"
      aria-hidden="true"
    >{`via ${name}`}</span>
  )
}

export interface FileTreeListProps {
  /** Already-visible rows, computed in Rust (`fileTreeView`). */
  rows: FileTreeViewRow[]
  renamingName: string | null
  onToggleFolder: (key: string) => void
  onSelect: (name: string) => void
  onRename: (from: string, to: string) => void
  /** Rust's reason a rename/move would be rejected, if it would. */
  renameError: (from: string, to: string) => string | undefined
  /** The folder rows' `···` menu; the read-only `Demo/` row has none. */
  folderActions: FolderActions
  creating: boolean
  creatingFolder: boolean
  importing: boolean
}

export function FileTreeList({
  rows,
  renamingName,
  onToggleFolder,
  onSelect,
  onRename,
  renameError,
  folderActions,
  creating,
  creatingFolder,
  importing,
}: FileTreeListProps) {
  return (
    <ul className="file-tabs" aria-label="Files">
      {rows.map((row) => {
        const rowStyle = { '--tree-depth': row.depth } as CSSProperties
        switch (row.kind) {
          case 'folder':
          case 'demo-folder':
            return (
              <li
                key={`${row.kind}:${row.key}`}
                className={`file-tree-row file-tree-folder-row${row.kind === 'demo-folder' ? ' file-tree-demo-row' : ''}`}
                style={rowStyle}
              >
                <button
                  type="button"
                  className="file-tree-folder"
                  aria-expanded={row.expanded}
                  onClick={() => onToggleFolder(row.key)}
                >
                  {row.expanded ? (
                    <ChevronDownIcon aria-hidden="true" />
                  ) : (
                    <ChevronRightIcon aria-hidden="true" />
                  )}
                  <FolderGlyph />
                  <span className="file-tree-name">{row.name}</span>
                  {row.shareId ? (
                    <Link2Icon
                      className="file-tree-share-marker"
                      aria-hidden="true"
                    />
                  ) : null}
                  {!row.shareId && row.sharedVia ? (
                    <ViaTag name={row.sharedVia.name} />
                  ) : null}
                  <span className="file-tree-count" aria-hidden="true">
                    {row.fileCount}
                  </span>
                </button>
                {row.kind === 'folder' ? (
                  <FolderRowMenu
                    folder={row.key}
                    name={row.name}
                    actions={folderActions}
                    creating={creating}
                    creatingFolder={creatingFolder}
                    importing={importing}
                    shareId={row.shareId}
                    sharedVia={row.sharedVia}
                  />
                ) : null}
              </li>
            )
          case 'empty-folder-hint':
            return (
              <li
                key={`hint:${row.depth}:${row.name}:${row.key}`}
                className="file-tree-row file-tree-empty-hint"
                style={rowStyle}
              >
                {row.name}
              </li>
            )
          default:
            return (
              <li
                key={`${row.kind}:${row.key}`}
                className={`file-tab file-tree-row${row.isActive ? ' file-tab--active' : ''}`}
                style={rowStyle}
              >
                <span className="file-tree-spacer" aria-hidden="true" />
                <FileTabName
                  name={row.key}
                  label={displayFileName(row.name)}
                  active={row.isActive}
                  onSelect={onSelect}
                  onRename={onRename}
                  renameError={renameError}
                  renaming={renamingName === row.key}
                />
                {row.sharedVia ? <ViaTag name={row.sharedVia.name} /> : null}
              </li>
            )
        }
      })}
    </ul>
  )
}
