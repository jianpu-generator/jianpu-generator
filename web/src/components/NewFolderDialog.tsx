import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useRef, useState } from 'react'
import './FileDialogs.css'

export interface NewFolderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** The folder the new one goes inside; `undefined` for the top level. */
  parent: string | undefined
  onCreate: (parent: string | undefined, name: string) => void
  /** Rust's reason creating this folder would be rejected, if it would. */
  createError: (parent: string | undefined, name: string) => string | undefined
}

/** Asks for the new folder's name; nothing is created until it is confirmed.
 * Rust explains live why a name is rejected. */
export function NewFolderDialog({
  open,
  onOpenChange,
  parent,
  onCreate,
  createError,
}: NewFolderDialogProps) {
  const [draft, setDraft] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (!open) setDraft('')
  }, [open])

  const trimmed = draft.trim()
  const error = open && trimmed ? createError(parent, trimmed) : undefined
  const canConfirm = trimmed !== '' && error === undefined

  const confirm = () => {
    if (!canConfirm) return
    onOpenChange(false)
    onCreate(parent, trimmed)
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content
          className="file-dialog"
          aria-describedby={undefined}
          onOpenAutoFocus={(event) => {
            event.preventDefault()
            inputRef.current?.focus()
          }}
        >
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              New folder
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <form
            onSubmit={(event) => {
              event.preventDefault()
              confirm()
            }}
          >
            <div className="file-dialog-body">
              <p className="file-dialog-hint">
                {parent === undefined
                  ? 'Name the new folder.'
                  : `Name the new folder inside ${parent}.`}
              </p>
              <input
                ref={inputRef}
                type="text"
                className="file-dialog-input"
                aria-label="Folder name"
                aria-invalid={error !== undefined}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
              />
              {error ? (
                <p className="file-dialog-warn" role="alert">
                  {error}
                </p>
              ) : null}
            </div>
            <div className="file-dialog-footer">
              <button
                type="button"
                className="file-dialog-button"
                onClick={() => onOpenChange(false)}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="file-dialog-button file-dialog-button--primary"
                disabled={!canConfirm}
              >
                Create
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
