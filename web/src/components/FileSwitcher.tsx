import {
  ArchiveIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  CopyIcon,
  DotsHorizontalIcon,
  GearIcon,
  Link2Icon,
  Pencil1Icon,
  PlusIcon,
  TrashIcon,
} from '@radix-ui/react-icons'
import { useEffect, useState } from 'react'
import {
  DEMO_FILE_NAMES,
  displayFileName,
  type FileStoreState,
  fileContent,
  isReadOnlyFile,
  sortedUserFileNames,
} from '../fileStore'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SyncedShareGithubAuthResult } from '../storage/accountAuthPopup'
import { FileTabName } from './FileTabName'
import { ImportButton } from './ImportButton'
import { ResponsiveMenu } from './ResponsiveMenu'
import { SaveStatusBadge } from './SaveStatusBadge'
import { ShareModal } from './ShareModal'
import { SpinnerLabel } from './SpinnerLabel'

export interface FileSwitcherProps {
  store: FileStoreState
  /** Label shown on the trigger button — the currently active file's name,
   * whether it's a user file or one of the read-only demo files (which live
   * in the "Demo" submenu nested inside this same dropdown). */
  triggerLabel: string
  onSelect: (name: string) => void
  onCreate: () => void
  onDuplicate: () => void
  onRename: (from: string, to: string) => void
  onDelete: (name: string) => void
  onOpenStorageSettings: () => void
  saveStatus: DisplaySaveStatus
  /** `Date.now()`-comparable deadline for the pending autosave, used to
   * render a countdown while `saveStatus === 'unsaved'`. */
  autosaveDeadline: number | null
  /** Whether a `createFile` call is in flight — disables "New" and shows a
   * spinner on it. */
  creating?: boolean
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
  onImportFile?: (file: File) => void
  /** Names of files currently in the trash — used for the "Bin (N)" menu
   * item's count and to decide whether it renders at all. */
  binNames: string[]
  /** Opens the Bin modal, which lists `binNames` and handles restoring. */
  onOpenBin: () => void
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
}

export function FileSwitcher({
  store,
  triggerLabel,
  onSelect,
  onCreate,
  onDuplicate,
  onRename,
  onDelete,
  onOpenStorageSettings,
  saveStatus,
  autosaveDeadline,
  creating = false,
  deletingName = null,
  duplicating = false,
  renamingName = null,
  isLoadingCloud = false,
  importing = false,
  onImportFile,
  binNames,
  onOpenBin,
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
  const [demoOpen, setDemoOpen] = useState(false)
  useEffect(() => {
    if (!filesOpen) setDemoOpen(false)
  }, [filesOpen])

  const [actionsOpen, setActionsOpen] = useState(false)
  const [shareModalOpen, setShareModalOpen] = useState(false)

  return (
    <div className="file-tab-bar">
      <SaveStatusBadge
        status={saveStatus}
        autosaveDeadline={autosaveDeadline}
      />
      <div className="export-menu">
        <ResponsiveMenu.Root open={filesOpen} onOpenChange={setFilesOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button type="button" className="preview-export-btn">
              {displayFileName(triggerLabel)}
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
            <ul className="file-tabs" aria-label="Files">
              {names.map((name) => {
                const active = name === store.active

                return (
                  <li
                    key={name}
                    className={`file-tab${active ? ' file-tab--active' : ''}`}
                  >
                    <FileTabName
                      name={name}
                      active={active}
                      onSelect={(selected) => {
                        onSelect(selected)
                        setFilesOpen(false)
                      }}
                      onRename={onRename}
                      renaming={renamingName === name}
                    />
                  </li>
                )
              })}
            </ul>
            <div className="file-tab-bar-demo-section">
              <button
                type="button"
                className="file-tab-bar-demo-toggle"
                aria-haspopup="menu"
                aria-expanded={demoOpen}
                onClick={() => setDemoOpen((prev) => !prev)}
              >
                {demoOpen ? (
                  <ChevronDownIcon aria-hidden="true" />
                ) : (
                  <ChevronRightIcon aria-hidden="true" />
                )}
                Demo
              </button>
              {demoOpen ? (
                <ul
                  className="file-tabs file-tab-bar-demo-list"
                  aria-label="Demo files"
                >
                  {DEMO_FILE_NAMES.map((name) => {
                    const active = name === store.active

                    return (
                      <li
                        key={name}
                        className={`file-tab${active ? ' file-tab--active' : ''}`}
                      >
                        <FileTabName
                          name={name}
                          active={active}
                          onSelect={(selected) => {
                            onSelect(selected)
                            setFilesOpen(false)
                          }}
                          onRename={onRename}
                        />
                      </li>
                    )
                  })}
                </ul>
              ) : null}
            </div>
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
                await onCreate()
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
              onClick={async () => {
                const next = window.prompt(
                  'Rename file',
                  displayFileName(store.active),
                )
                const trimmed = next?.trim()
                if (!trimmed || trimmed === store.active) return
                await onRename(store.active, trimmed)
                setActionsOpen(false)
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
      <ShareModal
        open={shareModalOpen}
        onOpenChange={setShareModalOpen}
        filename={store.active}
        content={fileContent(store, store.active)}
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
