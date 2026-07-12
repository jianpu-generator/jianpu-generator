import {
  ChevronDownIcon,
  CopyIcon,
  DotsHorizontalIcon,
  GearIcon,
  Pencil1Icon,
  PlusIcon,
  TrashIcon,
} from '@radix-ui/react-icons'
import { useEffect, useRef, useState } from 'react'
import {
  DEMO_FILE_NAME,
  type FileStoreState,
  fileContent,
  isReadOnlyFile,
  sortedFileNames,
} from '../fileStore'
import { useDismissableOpen } from '../hooks/useDismissableOpen'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SaveStatus } from '../storage/types'
import { ShareButton } from './ShareButton'

export interface FileSwitcherProps {
  store: FileStoreState
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
  /** Whether the GitHub backend is still fetching its file list — shows a
   * spinner on the trigger and a loading hint instead of the demo hint. */
  isLoadingGithub?: boolean
}

const SAVE_STATUS_LABEL: Record<SaveStatus, string> = {
  idle: '',
  saving: 'Saving…',
  saved: 'Saved',
  error: 'Save failed',
  offline: 'Offline',
}

/** Ticks once a second while `deadline` is non-null, so a rendered countdown
 * stays in sync without the parent re-rendering on every store change. */
function useCountdownSeconds(deadline: number | null): number | null {
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (deadline === null) return
    const interval = setInterval(() => setNow(Date.now()), 1_000)
    return () => clearInterval(interval)
  }, [deadline])

  return deadline === null
    ? null
    : Math.max(0, Math.ceil((deadline - now) / 1_000))
}

/** Shows the shared spinner alongside a button's label while `pending` is true. */
function SpinnerLabel({ pending, label }: { pending: boolean; label: string }) {
  return (
    <>
      {pending ? (
        <span
          className="file-tab-bar-spinner file-tab-bar-spinner--inline"
          aria-hidden="true"
        />
      ) : null}
      {label}
    </>
  )
}

function SaveStatusBadge({
  status,
  autosaveDeadline,
}: {
  status: DisplaySaveStatus
  autosaveDeadline: number | null
}) {
  const remainingSeconds = useCountdownSeconds(
    status === 'unsaved' ? autosaveDeadline : null,
  )
  const label =
    status === 'unsaved'
      ? remainingSeconds !== null
        ? `Unsaved (autosaving in ${remainingSeconds}s)`
        : 'Unsaved'
      : SAVE_STATUS_LABEL[status]
  if (!label) return null
  return (
    <span
      className={`file-tab-bar-save-status file-tab-bar-save-status--${status}`}
      data-testid="save-status-badge"
    >
      {label}
    </span>
  )
}

function FileTabName({
  name,
  active,
  onSelect,
  onRename,
  renaming = false,
}: {
  name: string
  active: boolean
  onSelect: (name: string) => void
  onRename: (from: string, to: string) => void
  renaming?: boolean
}) {
  const readOnly = isReadOnlyFile(name)
  const [draft, setDraft] = useState(name)
  const [editing, setEditing] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    setDraft(name)
  }, [name])

  useEffect(() => {
    if (!active) setEditing(false)
  }, [active])

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus()
      inputRef.current?.select()
    }
  }, [editing])

  if (active && editing && !readOnly) {
    return (
      <input
        ref={inputRef}
        type="text"
        className="file-tab-name"
        value={draft}
        aria-current="true"
        onChange={(e) => setDraft(e.target.value)}
        onBlur={() => {
          const trimmed = draft.trim()
          if (trimmed && trimmed !== name) {
            onRename(name, trimmed)
          } else {
            setDraft(name)
          }
          setEditing(false)
        }}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.currentTarget.blur()
          } else if (e.key === 'Escape') {
            setDraft(name)
            setEditing(false)
            e.currentTarget.blur()
          }
        }}
      />
    )
  }

  return (
    <button
      type="button"
      className="file-tab-name"
      aria-current={active ? 'true' : undefined}
      onClick={() => {
        if (!active) onSelect(name)
      }}
      onDoubleClick={() => {
        if (active && !readOnly) setEditing(true)
      }}
      disabled={renaming}
    >
      <SpinnerLabel pending={renaming} label={name} />
    </button>
  )
}

export function FileSwitcher({
  store,
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
  isLoadingGithub = false,
}: FileSwitcherProps) {
  const names = sortedFileNames(store)
  const showHint = names.length === 1 && names[0] === DEMO_FILE_NAME

  const filesContainerRef = useRef<HTMLDivElement>(null)
  const [filesOpen, setFilesOpen] = useDismissableOpen(filesContainerRef)

  const actionsContainerRef = useRef<HTMLDivElement>(null)
  const [actionsOpen, setActionsOpen] = useDismissableOpen(actionsContainerRef)

  return (
    <div className="file-tab-bar">
      <SaveStatusBadge
        status={saveStatus}
        autosaveDeadline={autosaveDeadline}
      />
      <div className="export-menu" ref={filesContainerRef}>
        <button
          type="button"
          className="preview-export-btn"
          aria-haspopup="menu"
          aria-expanded={filesOpen}
          onClick={() => setFilesOpen((prev) => !prev)}
        >
          {store.active}
          {isLoadingGithub ? (
            <span className="file-tab-bar-spinner" aria-hidden="true" />
          ) : (
            <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
          )}
        </button>
        {filesOpen ? (
          <div className="export-menu-list file-tab-bar-files-list">
            {isLoadingGithub ? (
              <p className="file-tab-bar-hint">Loading files from GitHub…</p>
            ) : showHint ? (
              <p className="file-tab-bar-hint">
                Demo is read-only — duplicate to edit.
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
          </div>
        ) : null}
      </div>
      <div className="export-menu" ref={actionsContainerRef}>
        <button
          type="button"
          className="preview-export-btn"
          aria-haspopup="menu"
          aria-expanded={actionsOpen}
          aria-label="File actions"
          onClick={() => setActionsOpen((prev) => !prev)}
        >
          <DotsHorizontalIcon aria-hidden="true" />
        </button>
        {actionsOpen ? (
          <div className="export-menu-list" role="menu">
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
                const next = window.prompt('Rename file', store.active)
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
            <ShareButton
              filename={store.active}
              content={fileContent(store, store.active)}
              className="export-menu-item"
            />
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
          </div>
        ) : null}
      </div>
    </div>
  )
}
