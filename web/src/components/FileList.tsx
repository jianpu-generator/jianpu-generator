import { useEffect, useRef, useState } from 'react'
import {
  DEMO_FILE_NAME,
  type FileStoreState,
  isReadOnlyFile,
  sortedBinNames,
  sortedFileNames,
} from '../fileStore'

export interface FileListProps {
  store: FileStoreState
  onSelect: (name: string) => void
  onCreate: () => void
  onDuplicate: () => void
  onRename: (from: string, to: string) => void
  onDelete: (name: string) => void
  onRestore: (name: string) => void
}

function FileTabName({
  name,
  active,
  onSelect,
  onRename,
}: {
  name: string
  active: boolean
  onSelect: (name: string) => void
  onRename: (from: string, to: string) => void
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
    >
      {name}
    </button>
  )
}

export function FileTabBar({
  store,
  onSelect,
  onCreate,
  onDuplicate,
  onRename,
  onDelete,
  onRestore,
}: FileListProps) {
  const names = sortedFileNames(store)
  const binNames = sortedBinNames(store)
  const showHint = names.length === 1 && names[0] === DEMO_FILE_NAME

  return (
    <div className="file-tab-bar">
      <div className="file-tab-bar-actions">
        <button type="button" className="file-tab-bar-btn" onClick={onCreate}>
          New
        </button>
        <button
          type="button"
          className="file-tab-bar-btn"
          onClick={onDuplicate}
        >
          Duplicate
        </button>
      </div>
      {showHint ? (
        <p className="file-tab-bar-hint">
          Demo is read-only — duplicate to edit.
        </p>
      ) : null}
      <div className="file-tab-bar-tabs-scroll">
        <ul className="file-tabs" aria-label="Files">
          {names.map((name) => {
            const active = name === store.active
            const readOnly = isReadOnlyFile(name)

            return (
              <li
                key={name}
                className={`file-tab${active ? ' file-tab--active' : ''}`}
              >
                <FileTabName
                  name={name}
                  active={active}
                  onSelect={onSelect}
                  onRename={onRename}
                />
                {!readOnly ? (
                  <button
                    type="button"
                    className="file-tab-close"
                    aria-label={`Move ${name} to bin`}
                    onClick={() => onDelete(name)}
                  >
                    ×
                  </button>
                ) : null}
              </li>
            )
          })}
        </ul>
      </div>
      {binNames.length > 0 ? (
        <details className="file-tab-bar-bin">
          <summary className="file-tab-bar-bin-summary">
            Bin ({binNames.length})
          </summary>
          <ul className="file-tab-bar-bin-items">
            {binNames.map((name) => (
              <li key={name} className="file-tab-bar-bin-item">
                <span className="file-tab-bar-bin-name">{name}</span>
                <button
                  type="button"
                  className="file-tab-bar-restore"
                  aria-label={`Restore ${name}`}
                  onClick={() => onRestore(name)}
                >
                  ↩
                </button>
              </li>
            ))}
          </ul>
        </details>
      ) : null}
    </div>
  )
}
