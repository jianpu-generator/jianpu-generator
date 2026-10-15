import { useEffect, useRef, useState } from 'react'
import { displayFileName, isReadOnlyFile } from '../fileStore'
import { SpinnerLabel } from './SpinnerLabel'

export function FileTabName({
  name,
  active,
  onSelect,
  onRename,
  renameError,
  renaming = false,
  label,
}: {
  name: string
  active: boolean
  onSelect: (name: string) => void
  onRename: (from: string, to: string) => void
  /** Rust's reason renaming `from` to `to` would be rejected, if it would. */
  renameError?: (from: string, to: string) => string | undefined
  renaming?: boolean
  /** Text shown for the row; defaults to the extension-less `name`. A tree
   * row passes just the file's own name, not its folder path. */
  label?: string
}) {
  const readOnly = isReadOnlyFile(name)
  const [draft, setDraft] = useState(displayFileName(name))
  const [editing, setEditing] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    setDraft(displayFileName(name))
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
    const trimmed = draft.trim()
    const error = trimmed ? renameError?.(name, trimmed) : undefined
    return (
      <div className="file-tab-rename">
        <input
          ref={inputRef}
          type="text"
          className="file-tab-name"
          value={draft}
          aria-current="true"
          aria-invalid={error !== undefined}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={() => {
            if (!error && trimmed && trimmed !== displayFileName(name)) {
              onRename(name, trimmed)
            } else {
              setDraft(displayFileName(name))
            }
            setEditing(false)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              // A rejected name stays open so its message can be read.
              if (!error) e.currentTarget.blur()
            } else if (e.key === 'Escape') {
              setDraft(displayFileName(name))
              setEditing(false)
              e.currentTarget.blur()
            }
          }}
        />
        {error ? (
          <p className="file-tab-rename-error" role="alert">
            {error}
          </p>
        ) : null}
      </div>
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
      <SpinnerLabel pending={renaming} label={label ?? displayFileName(name)} />
    </button>
  )
}
