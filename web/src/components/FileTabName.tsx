import { useEffect, useRef, useState } from 'react'
import { displayFileName, isReadOnlyFile } from '../fileStore'
import { SpinnerLabel } from './SpinnerLabel'

export function FileTabName({
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
          if (trimmed && trimmed !== displayFileName(name)) {
            onRename(name, trimmed)
          } else {
            setDraft(displayFileName(name))
          }
          setEditing(false)
        }}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.currentTarget.blur()
          } else if (e.key === 'Escape') {
            setDraft(displayFileName(name))
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
      <SpinnerLabel pending={renaming} label={displayFileName(name)} />
    </button>
  )
}
